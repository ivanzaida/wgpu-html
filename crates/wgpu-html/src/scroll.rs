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
use wgpu_html_tree::Tree;

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
/// adding the current vertical scroll.
pub fn viewport_to_document(pos: (f32, f32), scroll_y: f32) -> (f32, f32) {
    (pos.0, pos.1 + scroll_y)
}

/// Clamp `scroll_y` into `[0, max_scroll_y]`.
pub fn clamp_scroll_y(scroll_y: f32, layout: &LayoutBox, viewport_h: f32) -> f32 {
    scroll_y.clamp(0.0, max_scroll_y(layout, viewport_h))
}

/// Maximum vertical scroll for the document root. `0.0` if the
/// document fits in the viewport.
pub fn max_scroll_y(layout: &LayoutBox, viewport_h: f32) -> f32 {
    (document_bottom(layout) - viewport_h).max(0.0)
}

/// Bottom edge (in document space) of the deepest descendant.
pub fn document_bottom(b: &LayoutBox) -> f32 {
    b.children
        .iter()
        .map(document_bottom)
        .fold(b.margin_rect.y + b.margin_rect.h, f32::max)
}

/// Compute the viewport scrollbar's track + thumb. Returns `None`
/// if the document fits or the viewport is too small to host one.
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

    let track_w = 10.0;
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

    Some(ScrollbarGeometry { track, thumb, max_scroll, travel })
}

/// Inverse of the thumb position calculation: given a desired
/// thumb-top in viewport space, return the corresponding `scroll_y`.
pub fn scroll_y_from_thumb_top(
    thumb_top: f32,
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
) -> f32 {
    let Some(geom) = scrollbar_geometry(layout, viewport_w, viewport_h, 0.0) else {
        return 0.0;
    };
    if geom.travel <= 0.0 {
        return 0.0;
    }
    let t = ((thumb_top - geom.track.y) / geom.travel).clamp(0.0, 1.0);
    t * geom.max_scroll
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

/// Append the viewport scrollbar (track + thumb) to `list` as a
/// final clip-less section. No-op if the document fits.
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

    list.push_clip(None, [0.0; 4], [0.0; 4]);
    list.push_quad(geom.track, [0.0, 0.0, 0.0, 0.18]);
    list.push_quad(geom.thumb, [0.0, 0.0, 0.0, 0.55]);
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

fn element_subtree_bottom(b: &LayoutBox) -> f32 {
    let mut bottom = b.margin_rect.y + b.margin_rect.h;
    for child in &b.children {
        bottom = bottom.max(element_subtree_bottom(child));
    }
    bottom
}

/// Maximum vertical scroll inside `b`'s padding box.
pub fn max_element_scroll_y(b: &LayoutBox) -> f32 {
    (scrollable_content_height(b) - element_padding_box(b).h).max(0.0)
}

/// Compute the element-level scrollbar for an `overflow:scroll`
/// container, parameterised by its current `scroll_y` offset.
/// Returns `None` if the element doesn't need a scrollbar.
pub fn element_scrollbar_geometry(b: &LayoutBox, scroll_y: f32) -> Option<ScrollbarGeometry> {
    if !matches!(
        b.overflow.y,
        crate::models::common::css_enums::Overflow::Scroll
            | crate::models::common::css_enums::Overflow::Auto
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
    let track_w = 10.0_f32.min(pad.w);
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
    Some(ScrollbarGeometry { track, thumb, max_scroll, travel })
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
    offsets: &BTreeMap<Vec<usize>, f32>,
    path: &mut Vec<usize>,
) -> Option<Vec<usize>> {
    let own_scroll = offsets
        .get(path)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, max_element_scroll_y(b));
    let child_pos = (pos.0, pos.1 + own_scroll);
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

    let pad = element_padding_box(b);
    if matches!(
        b.overflow.y,
        crate::models::common::css_enums::Overflow::Scroll
            | crate::models::common::css_enums::Overflow::Auto
    ) && max_element_scroll_y(b) > 0.0
        && rect_contains(pad, pos)
    {
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
    offsets: &BTreeMap<Vec<usize>, f32>,
    path: &mut Vec<usize>,
) -> Option<(Vec<usize>, ScrollbarGeometry)> {
    let own_scroll = offsets
        .get(path)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, max_element_scroll_y(b));
    let child_pos = (pos.0, pos.1 + own_scroll);
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

    let geom = element_scrollbar_geometry(b, own_scroll)?;
    (rect_contains(geom.track, pos) || rect_contains(geom.thumb, pos)).then(|| (path.clone(), geom))
}

/// Apply `delta_y` (positive = scroll down) to the deepest
/// `overflow:scroll` container under `doc_pos`. Returns `true` if
/// any scroll offset actually changed.
pub fn scroll_element_at(
    tree: &mut Tree,
    layout: &LayoutBox,
    doc_pos: (f32, f32),
    delta_y: f32,
) -> bool {
    let Some(path) = deepest_scrollable_path_at(
        layout,
        doc_pos,
        &tree.interaction.scroll_offsets_y,
        &mut Vec::new(),
    ) else {
        return false;
    };
    let Some(box_) = layout.box_at_path(&path) else {
        return false;
    };
    let max_scroll = max_element_scroll_y(box_);
    if max_scroll <= 0.0 {
        return false;
    }

    let old = tree
        .interaction
        .scroll_offsets_y
        .get(&path)
        .copied()
        .unwrap_or(0.0)
        .clamp(0.0, max_scroll);
    let new = (old + delta_y).clamp(0.0, max_scroll);
    if (new - old).abs() <= 0.5 {
        return false;
    }

    if new <= 0.0 {
        tree.interaction.scroll_offsets_y.remove(&path);
    } else {
        tree.interaction.scroll_offsets_y.insert(path, new);
    }
    true
}

/// Drag an element-level scrollbar's thumb to `thumb_top` (in
/// document space).
pub fn scroll_element_thumb_to(
    tree: &mut Tree,
    layout: &LayoutBox,
    path: Vec<usize>,
    thumb_top: f32,
) {
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
    if scroll_y <= 0.0 {
        tree.interaction.scroll_offsets_y.remove(&path);
    } else {
        tree.interaction.scroll_offsets_y.insert(path, scroll_y);
    }
}
