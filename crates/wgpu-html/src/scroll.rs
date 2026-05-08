//! Document- and element-level scroll utilities.
//!
//! These helpers are layout-pure (no winit, no rendering): given a
//! [`LayoutBox`] and a `scroll_y` value, they compute clamped
//! offsets, scrollbar geometry, hit-tests for thumb / track, and
//! paint scrollbar quads into a [`DisplayList`]. Hosts compose
//! them to drive viewport scroll and per-element `overflow: scroll`
//! containers.
//!
//! Used by [`crate::interactivity`] adjacent code and by the
//! `wgpu-html-winit` harness; they were extracted from the demo
//! once it became clear every host wants the same behaviour.

use std::collections::BTreeMap;

use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_tree::{ScrollOffset, Tree};

// ── Geometry ────────────────────────────────────────────────────────────────

/// Track + thumb rectangles plus the precomputed `max_scroll` and
/// `travel` (vertical pixels the thumb can move).
#[derive(Debug, Clone, Copy)]
pub struct ScrollbarGeometry {
  pub track: Rect,
  pub thumb: Rect,
  pub max_scroll: f32,
  pub travel: f32,
}

/// Returns `pos.0 ∈ [rect.x, rect.x + rect.w)` and similarly for `y`.
pub fn rect_contains(rect: Rect, pos: (f32, f32)) -> bool {
  pos.0 >= rect.x && pos.0 < rect.x + rect.w && pos.1 >= rect.y && pos.1 < rect.y + rect.h
}

// ── Document scroll ─────────────────────────────────────────────────────────

/// Convert a viewport-space pointer position to document-space by
/// adding the current scroll offsets.
pub fn viewport_to_document(pos: (f32, f32), scroll_x: f32, scroll_y: f32) -> (f32, f32) {
  (pos.0 + scroll_x, pos.1 + scroll_y)
}

/// Clamp `scroll_y` into `[0, max_scroll_y]`.
pub fn clamp_scroll_y(scroll_y: f32, layout: &LayoutBox, viewport_h: f32) -> f32 {
  scroll_y.clamp(0.0, max_scroll_y(layout, viewport_h))
}

/// Clamp `scroll_x` into `[0, max_scroll_x]`.
pub fn clamp_scroll_x(scroll_x: f32, layout: &LayoutBox, viewport_w: f32) -> f32 {
  scroll_x.clamp(0.0, max_scroll_x(layout, viewport_w))
}

/// Maximum vertical scroll for the document root. `0.0` if the
/// document fits in the viewport.
pub fn max_scroll_y(layout: &LayoutBox, viewport_h: f32) -> f32 {
  (document_bottom(layout) - viewport_h).max(0.0)
}

/// Maximum horizontal scroll for the document root. `0.0` if the
/// document fits in the viewport.
pub fn max_scroll_x(layout: &LayoutBox, viewport_w: f32) -> f32 {
  (document_right(layout) - viewport_w).max(0.0)
}

/// Bottom edge (in document space) of the deepest descendant.
/// Stops recursing into overflow-clipped children (hidden / auto /
/// scroll) since their visible extent is bounded by their own rect,
/// but always recurses through the top-level elements (html → body)
/// so the viewport scroll sees the true content height.
pub fn document_bottom(b: &LayoutBox) -> f32 {
  document_bottom_inner(b, 0)
}

fn document_bottom_inner(b: &LayoutBox, depth: usize) -> f32 {
  b.children
    .iter()
    .map(|child| {
      use crate::models::common::css_enums::Overflow;
      let is_clipped = matches!(child.overflow.y, Overflow::Hidden | Overflow::Auto | Overflow::Scroll);
      if is_clipped && depth > 1 {
        child.margin_rect.y + child.margin_rect.h
      } else {
        document_bottom_inner(child, depth + 1)
      }
    })
    .fold(b.margin_rect.y + b.margin_rect.h, f32::max)
}

/// Compute the viewport scrollbar's track + thumb. Returns `None`
/// if the document fits, the viewport is too small, or the body
/// already handles scrolling via `overflow: scroll/auto`.
pub fn scrollbar_geometry(
  layout: &LayoutBox,
  viewport_w: f32,
  viewport_h: f32,
  scroll_y: f32,
) -> Option<ScrollbarGeometry> {
  let doc_h = document_bottom(layout).max(viewport_h);
  if doc_h <= viewport_h + 0.5 || viewport_w < 12.0 || viewport_h <= 0.0 {
    return None;
  }

  let track_w = VIEWPORT_SCROLLBAR_WIDTH;
  let margin = 2.0;
  let track = Rect::new(
    viewport_w - track_w - margin,
    margin,
    track_w,
    viewport_h - margin * 2.0,
  );
  let thumb_h = (track.h * viewport_h / doc_h).clamp(24.0, track.h);
  let max_scroll = max_scroll_y(layout, viewport_h);
  let travel = (track.h - thumb_h).max(0.0);
  let thumb_y = track.y + travel * (scroll_y / max_scroll.max(1.0));
  let thumb = Rect::new(track.x + 1.0, thumb_y, track.w - 2.0, thumb_h);

  Some(ScrollbarGeometry {
    track,
    thumb,
    max_scroll,
    travel,
  })
}

/// Inverse of the thumb position calculation: given a desired
/// thumb-top in viewport space, return the corresponding `scroll_y`.
pub fn scroll_y_from_thumb_top(thumb_top: f32, layout: &LayoutBox, viewport_w: f32, viewport_h: f32) -> f32 {
  let Some(geom) = scrollbar_geometry(layout, viewport_w, viewport_h, 0.0) else {
    return 0.0;
  };
  if geom.travel <= 0.0 {
    return 0.0;
  }
  let t = ((thumb_top - geom.track.y) / geom.travel).clamp(0.0, 1.0);
  t * geom.max_scroll
}

/// Right edge (in document space) of the widest descendant.
pub fn document_right(b: &LayoutBox) -> f32 {
  document_right_inner(b, 0)
}

fn document_right_inner(b: &LayoutBox, depth: usize) -> f32 {
  b.children
    .iter()
    .map(|child| {
      use crate::models::common::css_enums::Overflow;
      let is_clipped = matches!(child.overflow.x, Overflow::Hidden | Overflow::Auto | Overflow::Scroll);
      if is_clipped && depth > 1 {
        child.margin_rect.x + child.margin_rect.w
      } else {
        document_right_inner(child, depth + 1)
      }
    })
    .fold(b.margin_rect.x + b.margin_rect.w, f32::max)
}

/// Translate every quad / image / glyph / clip rect in a display
/// list by `dx` on the X axis. Used to apply viewport scroll.
pub fn translate_display_list_x(list: &mut DisplayList, dx: f32) {
  for quad in &mut list.quads {
    quad.rect.x += dx;
  }
  for image in &mut list.images {
    image.rect.x += dx;
  }
  for glyph in &mut list.glyphs {
    glyph.rect.x += dx;
  }
  for clip in &mut list.clips {
    if let Some(rect) = clip.rect.as_mut() {
      rect.x += dx;
    }
  }
}

/// Translate every quad / image / glyph / clip rect in a display
/// list by `dy` on the Y axis. Used to apply viewport scroll
/// after painting.
pub fn translate_display_list_y(list: &mut DisplayList, dy: f32) {
  for quad in &mut list.quads {
    quad.rect.y += dy;
  }
  for image in &mut list.images {
    image.rect.y += dy;
  }
  for glyph in &mut list.glyphs {
    glyph.rect.y += dy;
  }
  for clip in &mut list.clips {
    if let Some(rect) = clip.rect.as_mut() {
      rect.y += dy;
    }
  }
}

/// Check if the root or any top-level descendant (html → body)
/// already has a scrollbar via `overflow-y: scroll|auto`. When true,
/// the viewport scrollbar is suppressed to avoid double-painting.
pub fn body_handles_scroll(layout: &LayoutBox) -> bool {
  use crate::models::common::css_enums::Overflow;
  if matches!(layout.overflow.y, Overflow::Scroll | Overflow::Auto) {
    return true;
  }
  for child in &layout.children {
    if matches!(child.overflow.y, Overflow::Scroll | Overflow::Auto) {
      return true;
    }
    for grandchild in &child.children {
      if matches!(grandchild.overflow.y, Overflow::Scroll | Overflow::Auto) {
        return true;
      }
    }
  }
  false
}

/// Max scroll for the body element when it handles scrolling.
/// Walks through root → body and computes scrollable content height.
pub fn body_max_scroll(layout: &LayoutBox, _viewport_h: f32) -> f32 {
  use crate::models::common::css_enums::Overflow;
  for child in &layout.children {
    if matches!(child.overflow.y, Overflow::Scroll | Overflow::Auto) {
      let pad = element_padding_box(child);
      let content_h = scrollable_content_height(child);
      return (content_h - pad.h).max(0.0);
    }
    for grandchild in &child.children {
      if matches!(grandchild.overflow.y, Overflow::Scroll | Overflow::Auto) {
        let pad = element_padding_box(grandchild);
        let content_h = scrollable_content_height(grandchild);
        return (content_h - pad.h).max(0.0);
      }
    }
  }
  0.0
}

/// Width of the viewport scrollbar track in CSS pixels.
pub const VIEWPORT_SCROLLBAR_WIDTH: f32 = 10.0;

/// Default scrollbar colors — used only when the layout box has no
/// `scrollbar-color` set (i.e. the UA stylesheet didn't cascade).
pub const DEFAULT_TRACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
pub const DEFAULT_THUMB: [f32; 4] = [1.0, 1.0, 1.0, 0.2];

/// Append the viewport scrollbar (track + thumb) to `list` as a
/// final clip-less section. Colors come from the root layout box's
/// `scrollbar-color` CSS property (set in the UA stylesheet by default).
pub fn paint_viewport_scrollbar(
  list: &mut DisplayList,
  layout: &LayoutBox,
  viewport_w: f32,
  viewport_h: f32,
  scroll_y: f32,
) {
  let Some(geom) = scrollbar_geometry(layout, viewport_w, viewport_h, scroll_y) else {
    return;
  };

  let track_color = layout.overflow.scrollbar_track.unwrap_or(DEFAULT_TRACK);
  let thumb_color = layout.overflow.scrollbar_thumb.unwrap_or(DEFAULT_THUMB);

  let radius = geom.thumb.w * 0.5;
  list.push_clip(None, [0.0; 4], [0.0; 4]);
  if track_color[3] > 0.0 {
    list.push_quad(geom.track, track_color);
  }
  if thumb_color[3] > 0.0 {
    list.push_quad_rounded(geom.thumb, thumb_color, [radius; 4]);
  }
  list.finalize();
}

// ── Element scroll (overflow: scroll / auto) ────────────────────────────────

/// Padding-box rect (border-rect minus border insets). Used as the
/// scrollable region for an `overflow:scroll` container.
pub fn element_padding_box(b: &LayoutBox) -> Rect {
  Rect::new(
    b.border_rect.x + b.border.left,
    b.border_rect.y + b.border.top,
    (b.border_rect.w - b.border.horizontal()).max(0.0),
    (b.border_rect.h - b.border.vertical()).max(0.0),
  )
}

/// Total height of an element's scrollable content (descendants'
/// bottoms minus its own padding-top).
pub fn scrollable_content_height(b: &LayoutBox) -> f32 {
  let pad = element_padding_box(b);
  let mut bottom = pad.y + pad.h;
  for child in &b.children {
    bottom = bottom.max(element_subtree_bottom(child));
  }
  (bottom - pad.y).max(0.0)
}

/// Total width of an element's scrollable content (descendants'
/// right edges minus its own padding-left).
pub fn scrollable_content_width(b: &LayoutBox) -> f32 {
  let pad = element_padding_box(b);
  let mut right = pad.x + pad.w;
  for child in &b.children {
    right = right.max(element_subtree_right(child));
  }
  (right - pad.x).max(0.0)
}

fn element_subtree_bottom(b: &LayoutBox) -> f32 {
  let mut bottom = b.margin_rect.y + b.margin_rect.h;
  for child in &b.children {
    bottom = bottom.max(element_subtree_bottom(child));
  }
  bottom
}

fn element_subtree_right(b: &LayoutBox) -> f32 {
  let mut right = b.margin_rect.x + b.margin_rect.w;
  for child in &b.children {
    right = right.max(element_subtree_right(child));
  }
  right
}

/// Maximum vertical scroll inside `b`'s padding box.
pub fn max_element_scroll_y(b: &LayoutBox) -> f32 {
  (scrollable_content_height(b) - element_padding_box(b).h).max(0.0)
}

/// Maximum horizontal scroll inside `b`'s padding box.
pub fn max_element_scroll_x(b: &LayoutBox) -> f32 {
  (scrollable_content_width(b) - element_padding_box(b).w).max(0.0)
}

/// Compute the element-level scrollbar for an `overflow:scroll`
/// container, parameterised by its current `scroll_y` offset.
/// Returns `None` if the element doesn't need a scrollbar.
pub fn element_scrollbar_geometry(b: &LayoutBox, scroll_y: f32) -> Option<ScrollbarGeometry> {
  if !matches!(
    b.overflow.y,
    crate::models::common::css_enums::Overflow::Scroll | crate::models::common::css_enums::Overflow::Auto
  ) {
    return None;
  }
  let pad = element_padding_box(b);
  if pad.w <= 0.0 || pad.h <= 0.0 {
    return None;
  }
  let scroll_h = scrollable_content_height(b).max(pad.h);
  let max_scroll = (scroll_h - pad.h).max(0.0);
  if max_scroll <= 0.0 {
    return None;
  }
  let track_w = b.overflow.scrollbar_width.min(pad.w);
  if track_w <= 0.0 {
    return None;
  }
  let track = Rect::new(pad.x + pad.w - track_w, pad.y, track_w, pad.h);
  let thumb_h = (pad.h * pad.h / scroll_h).clamp(18.0_f32.min(pad.h), pad.h);
  let travel = (pad.h - thumb_h).max(0.0);
  let thumb_y = track.y + travel * (scroll_y.clamp(0.0, max_scroll) / max_scroll.max(1.0));
  let thumb = Rect::new(
    track.x + 2.0,
    thumb_y + 2.0,
    (track.w - 4.0).max(1.0),
    (thumb_h - 4.0).max(1.0),
  );
  Some(ScrollbarGeometry {
    track,
    thumb,
    max_scroll,
    travel,
  })
}

/// Horizontal scrollbar geometry. Mirrors [`element_scrollbar_geometry`]
/// but for the X axis.
pub fn element_scrollbar_geometry_x(b: &LayoutBox, scroll_x: f32) -> Option<ScrollbarGeometry> {
  use crate::models::common::css_enums::Overflow;
  if !matches!(b.overflow.x, Overflow::Scroll | Overflow::Auto) {
    return None;
  }
  let pad = element_padding_box(b);
  if pad.w <= 0.0 || pad.h <= 0.0 {
    return None;
  }
  let scroll_w = scrollable_content_width(b).max(pad.w);
  let max_scroll = (scroll_w - pad.w).max(0.0);
  if max_scroll <= 0.0 {
    return None;
  }
  let track_h = b.overflow.scrollbar_width.min(pad.h);
  if track_h <= 0.0 {
    return None;
  }
  let has_vbar = matches!(b.overflow.y, Overflow::Scroll | Overflow::Auto)
    && scrollable_content_height(b) > pad.h + 0.5;
  let bar_w = if has_vbar { pad.w - track_h } else { pad.w };
  let track = Rect::new(pad.x, pad.y + pad.h - track_h, bar_w, track_h);
  let thumb_w = (bar_w * bar_w / scroll_w).clamp(18.0_f32.min(bar_w), bar_w);
  let travel = (bar_w - thumb_w).max(0.0);
  let thumb_x = track.x + travel * (scroll_x.clamp(0.0, max_scroll) / max_scroll.max(1.0));
  let thumb = Rect::new(
    thumb_x + 2.0,
    track.y + 2.0,
    (thumb_w - 4.0).max(1.0),
    (track.h - 4.0).max(1.0),
  );
  Some(ScrollbarGeometry {
    track,
    thumb,
    max_scroll,
    travel,
  })
}

/// Find the deepest `overflow:scroll` container under `pos` (in
/// document space, walking `b.children` recursively while folding
/// in the per-element scroll offsets).
///
/// Returns the path, or `None` if `pos` is over no scrollable
/// element.
pub fn deepest_scrollable_path_at(
  b: &LayoutBox,
  pos: (f32, f32),
  offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
  let own_scroll_x = offsets
    .get(path)
    .map(|s| s.x)
    .unwrap_or(0.0)
    .clamp(0.0, max_element_scroll_x(b));
  let own_scroll_y = offsets
    .get(path)
    .map(|s| s.y)
    .unwrap_or(0.0)
    .clamp(0.0, max_element_scroll_y(b));
  let child_pos = (pos.0 + own_scroll_x, pos.1 + own_scroll_y);
  for (i, child) in b.children.iter().enumerate().rev() {
    if !child.border_rect.contains(child_pos.0, child_pos.1) {
      continue;
    }
    path.push(i);
    if let Some(found) = deepest_scrollable_path_at(child, child_pos, offsets, path) {
      path.pop();
      return Some(found);
    }
    path.pop();
  }

  use crate::models::common::css_enums::Overflow;
  let pad = element_padding_box(b);
  let scrollable_y = matches!(b.overflow.y, Overflow::Scroll | Overflow::Auto) && max_element_scroll_y(b) > 0.0;
  let scrollable_x = matches!(b.overflow.x, Overflow::Scroll | Overflow::Auto) && max_element_scroll_x(b) > 0.0;
  if (scrollable_x || scrollable_y) && rect_contains(pad, pos) {
    return Some(path.clone());
  }
  None
}

/// Like [`deepest_scrollable_path_at`] but also returns the
/// element-level scrollbar geometry if `pos` is over a track or
/// thumb specifically.
pub fn deepest_element_scrollbar_at(
  b: &LayoutBox,
  pos: (f32, f32),
  offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  path: &mut Vec<usize>,
) -> Option<(Vec<usize>, ScrollbarAxis, ScrollbarGeometry)> {
  let own_scroll_x = offsets
    .get(path)
    .map(|s| s.x)
    .unwrap_or(0.0)
    .clamp(0.0, max_element_scroll_x(b));
  let own_scroll_y = offsets
    .get(path)
    .map(|s| s.y)
    .unwrap_or(0.0)
    .clamp(0.0, max_element_scroll_y(b));
  let child_pos = (pos.0 + own_scroll_x, pos.1 + own_scroll_y);
  for (i, child) in b.children.iter().enumerate().rev() {
    if !child.border_rect.contains(child_pos.0, child_pos.1) {
      continue;
    }
    path.push(i);
    if let Some(found) = deepest_element_scrollbar_at(child, child_pos, offsets, path) {
      path.pop();
      return Some(found);
    }
    path.pop();
  }

  if let Some(geom) = element_scrollbar_geometry(b, own_scroll_y) {
    if rect_contains(geom.track, pos) || rect_contains(geom.thumb, pos) {
      return Some((path.clone(), ScrollbarAxis::Vertical, geom));
    }
  }
  if let Some(geom) = element_scrollbar_geometry_x(b, own_scroll_x) {
    if rect_contains(geom.track, pos) || rect_contains(geom.thumb, pos) {
      return Some((path.clone(), ScrollbarAxis::Horizontal, geom));
    }
  }
  None
}

// ── Element scrollbar drag ───────────────────────────────────────────────────

/// Which scrollbar axis is being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
  Vertical,
  Horizontal,
}

/// Reusable state for an in-progress scrollbar thumb drag on an
/// `overflow:scroll` element. Hosts store this while the drag is
/// active and call [`ElementScrollbarDrag::update`] on cursor-move.
#[derive(Debug, Clone)]
pub struct ElementScrollbarDrag {
  /// Tree path to the scrollable element.
  pub path: Vec<usize>,
  pub axis: ScrollbarAxis,
  /// Offset from the thumb's leading edge to the initial grab point
  /// so the thumb doesn't jump on the first move.
  pub grab_offset: f32,
}

impl ElementScrollbarDrag {
  /// Hit-test element scrollbars at `pos` and start a drag if one
  /// was hit. Returns `Some(drag)` if a scrollbar was clicked
  /// (consuming the mouse-down), `None` otherwise.
  pub fn try_start(layout: &LayoutBox, pos: (f32, f32), tree: &mut Tree) -> Option<Self> {
    let hit = deepest_element_scrollbar_at(layout, pos, &tree.interaction.scroll_offsets, &mut Vec::new())?;
    let (path, axis, geom) = hit;

    let (grab_pos, thumb_start) = match axis {
      ScrollbarAxis::Vertical => (pos.1, geom.thumb.y),
      ScrollbarAxis::Horizontal => (pos.0, geom.thumb.x),
    };

    if rect_contains(geom.thumb, pos) {
      Some(Self {
        path,
        axis,
        grab_offset: grab_pos - thumb_start,
      })
    } else {
      match axis {
        ScrollbarAxis::Vertical => scroll_element_thumb_to_y(tree, layout, path.clone(), grab_pos),
        ScrollbarAxis::Horizontal => scroll_element_thumb_to_x(tree, layout, path.clone(), grab_pos),
      }
      Some(Self {
        path,
        axis,
        grab_offset: 0.0,
      })
    }
  }

  /// Continue the drag: move the thumb to track the cursor.
  pub fn update(&self, layout: &LayoutBox, tree: &mut Tree, cursor_x: f32, cursor_y: f32) {
    match self.axis {
      ScrollbarAxis::Vertical => {
        scroll_element_thumb_to_y(tree, layout, self.path.clone(), cursor_y - self.grab_offset);
      }
      ScrollbarAxis::Horizontal => {
        scroll_element_thumb_to_x(tree, layout, self.path.clone(), cursor_x - self.grab_offset);
      }
    }
  }
}

/// Apply `delta_x` / `delta_y` (positive = scroll right / down) to
/// the deepest scrollable container under `doc_pos`. Returns `true`
/// if any scroll offset actually changed.
pub fn scroll_element_at(tree: &mut Tree, layout: &LayoutBox, doc_pos: (f32, f32), delta_x: f32, delta_y: f32) -> bool {
  let Some(path) = deepest_scrollable_path_at(layout, doc_pos, &tree.interaction.scroll_offsets, &mut Vec::new())
  else {
    return false;
  };
  let Some(box_) = layout.box_at_path(&path) else {
    return false;
  };
  let max_scroll_x = max_element_scroll_x(box_);
  let max_scroll_y = max_element_scroll_y(box_);
  if max_scroll_x <= 0.0 && max_scroll_y <= 0.0 {
    return false;
  }

  let existing = tree.interaction.scroll_offsets.get(&path).copied().unwrap_or_default();

  let old_x = existing.x.clamp(0.0, max_scroll_x);
  let new_x = (old_x + delta_x).clamp(0.0, max_scroll_x);

  let old_y = existing.y.clamp(0.0, max_scroll_y);
  let new_y = (old_y + delta_y).clamp(0.0, max_scroll_y);

  let changed_x = (new_x - old_x).abs() > 0.5;
  let changed_y = (new_y - old_y).abs() > 0.5;
  if changed_x || changed_y {
    if new_x <= 0.0 && new_y <= 0.0 {
      tree.interaction.scroll_offsets.remove(&path);
    } else {
      tree
        .interaction
        .scroll_offsets
        .insert(path.clone(), ScrollOffset { x: new_x, y: new_y });
    }
    tree.scroll_event(&path);
  }
  // Always consume the event to prevent propagation to the viewport.
  true
}

/// Drag an element-level scrollbar's thumb to `thumb_top` (in
/// document space).
pub fn scroll_element_thumb_to_y(tree: &mut Tree, layout: &LayoutBox, path: Vec<usize>, thumb_top: f32) {
  let Some(box_) = layout.box_at_path(&path) else {
    return;
  };
  let Some(geom) = element_scrollbar_geometry(box_, 0.0) else {
    return;
  };
  if geom.travel <= 0.0 {
    return;
  }
  let t = ((thumb_top - geom.track.y) / geom.travel).clamp(0.0, 1.0);
  let scroll_y = t * geom.max_scroll;
  let existing_x = tree.interaction.scroll_offsets.get(&path).map(|s| s.x).unwrap_or(0.0);
  if scroll_y <= 0.0 && existing_x <= 0.0 {
    tree.interaction.scroll_offsets.remove(&path);
  } else {
    tree.interaction.scroll_offsets.insert(
      path,
      ScrollOffset {
        x: existing_x,
        y: scroll_y,
      },
    );
  }
}

pub fn scroll_element_thumb_to_x(tree: &mut Tree, layout: &LayoutBox, path: Vec<usize>, thumb_left: f32) {
  let Some(box_) = layout.box_at_path(&path) else {
    return;
  };
  let Some(geom) = element_scrollbar_geometry_x(box_, 0.0) else {
    return;
  };
  if geom.travel <= 0.0 {
    return;
  }
  let t = ((thumb_left - geom.track.x) / geom.travel).clamp(0.0, 1.0);
  let scroll_x = t * geom.max_scroll;
  let existing_y = tree.interaction.scroll_offsets.get(&path).map(|s| s.y).unwrap_or(0.0);
  if scroll_x <= 0.0 && existing_y <= 0.0 {
    tree.interaction.scroll_offsets.remove(&path);
  } else {
    tree.interaction.scroll_offsets.insert(
      path,
      ScrollOffset {
        x: scroll_x,
        y: existing_y,
      },
    );
  }
}
