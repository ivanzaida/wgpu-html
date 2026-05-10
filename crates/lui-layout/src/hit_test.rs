//! Hit-testing helpers for the layout tree.
//!
//! Functions in this module answer "which element is at (x, y)?" by
//! walking the [`LayoutBox`] tree with optional scroll-offset
//! compensation and overflow-clip support.

use std::collections::BTreeMap;

use lui_models::common::css_enums::PointerEvents;
use lui_text::{PositionedGlyph, ShapedLine, ShapedRun};
use lui_tree::ScrollOffset;

use crate::{LayoutBox, Rect};

/// If `b` has a CSS transform, apply the inverse to `(x, y)` so
/// hit-testing uses the element's untransformed local coordinates.
fn untransform_point(b: &LayoutBox, x: f32, y: f32) -> (f32, f32) {
  if let Some(ref t) = b.transform {
    // Translate origin before inverse (transform is relative to origin)
    let ox = b.border_rect.x + b.transform_origin.0;
    let oy = b.border_rect.y + b.transform_origin.1;
    let (lx, ly) = t.apply_inverse(x - ox, y - oy);
    (lx + ox, ly + oy)
  } else {
    (x, y)
  }
}

/// Axis-aligned padding box of `b` in physical pixels.
fn padding_box_rect(b: &LayoutBox) -> Rect {
  Rect::new(
    b.border_rect.x + b.border.left,
    b.border_rect.y + b.border.top,
    (b.border_rect.w - b.border.horizontal()).max(0.0),
    (b.border_rect.h - b.border.vertical()).max(0.0),
  )
}

pub(crate) fn hit_glyph_boundary(b: &LayoutBox, run: &ShapedRun, point: (f32, f32)) -> usize {
  if run.glyphs.is_empty() {
    return 0;
  }

  let local_x = point.0 - b.content_rect.x;
  let local_y = point.1 - b.content_rect.y;

  let selected_line = if !run.lines.is_empty() {
    nearest_line(local_y, &run.lines)
  } else {
    // Fallback for synthetic runs that didn't populate line metadata.
    ShapedLine {
      top: 0.0,
      height: run.height.max(1.0),
      glyph_range: (0, run.glyphs.len()),
    }
  };

  let mut line: Vec<(usize, &PositionedGlyph)> = run
    .glyphs
    .iter()
    .enumerate()
    .skip(selected_line.glyph_range.0)
    .take(selected_line.glyph_range.1.saturating_sub(selected_line.glyph_range.0))
    .collect();
  if line.is_empty() {
    return run.glyphs.len();
  }
  line.sort_by(|(_, a), (_, b)| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

  for (idx, g) in &line {
    let mid = g.x + g.w * 0.5;
    if local_x < mid {
      // Return the character position of this glyph (cursor
      // is placed *before* it).
      return run.glyph_to_char_index(*idx);
    }
  }

  // Cursor is past all rendered glyphs on this line → place it
  // after the last glyph's character.
  let max_idx = line.iter().map(|(idx, _)| *idx).max().unwrap_or(0);
  let after_char = run.glyph_to_char_index(max_idx) + 1;
  after_char.min(run.text.chars().count())
}

fn nearest_line(local_y: f32, lines: &[ShapedLine]) -> ShapedLine {
  let mut best = lines[0];
  let mut best_d = distance_to_line(local_y, best.top, best.height);
  for line in &lines[1..] {
    let d = distance_to_line(local_y, line.top, line.height);
    if d < best_d {
      best_d = d;
      best = *line;
    }
  }
  best
}

fn distance_to_line(y: f32, top: f32, height: f32) -> f32 {
  if y < top {
    top - y
  } else if y > top + height {
    y - (top + height)
  } else {
    0.0
  }
}

pub(crate) fn collect_hit_path(b: &LayoutBox, x: f32, y: f32, active_clip: Option<Rect>) -> Option<Vec<usize>> {
  if active_clip.is_some_and(|clip| !clip.contains(x, y)) {
    return None;
  }

  let next_clip = overflow_hit_clip(b, active_clip);

  // If this element has a transform, inverse-transform the point
  // for children — they're laid out in the local coordinate space.
  let (child_x, child_y) = if b.transform.is_some() {
    untransform_point(b, x, y)
  } else {
    (x, y)
  };

  for (i, child) in b.children.iter().enumerate().rev() {
    if let Some(mut path) = collect_hit_path(child, child_x, child_y, next_clip) {
      path.insert(0, i);
      return Some(path);
    }
  }

  if b.pointer_events == PointerEvents::None {
    return None;
  }
  // For the element itself, also apply inverse transform.
  let (lx, ly) = untransform_point(b, x, y);
  b.border_rect.contains(lx, ly).then(Vec::new)
}

/// Scroll-aware variant of [`collect_hit_path`].  For each element
/// that has a scroll offset in `offsets`, the test `y` and the clip
/// region are shifted by the offset so children scrolled into view
/// are correctly matched.
pub(crate) fn collect_hit_path_scrolled(
  b: &LayoutBox,
  x: f32,
  y: f32,
  offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  path: &mut Vec<usize>,
  clip: Option<Rect>,
) -> Option<Vec<usize>> {
  if clip.is_some_and(|c| !c.contains(x, y)) {
    return None;
  }

  let next_clip = overflow_hit_clip(b, clip);

  // Clamp scroll offset to the element's actual scrollable range
  // (mirrors paint behaviour). Without this, a stale offset after
  // resize causes hit-test to target wrong children.
  let pad = padding_box_rect(b);

  let raw_scroll_x = offsets.get(path.as_slice()).map(|s| s.x).unwrap_or(0.0);
  let own_scroll_x = if raw_scroll_x != 0.0 {
    let content_right = b.children.iter().fold(pad.x + pad.w, |acc, child| {
      acc.max(child.margin_rect.x + child.margin_rect.w)
    });
    let max_scroll_x = (content_right - pad.x - pad.w).max(0.0);
    raw_scroll_x.clamp(0.0, max_scroll_x)
  } else {
    0.0
  };

  let raw_scroll_y = offsets.get(path.as_slice()).map(|s| s.y).unwrap_or(0.0);
  let own_scroll_y = if raw_scroll_y != 0.0 {
    let content_bottom = b.children.iter().fold(pad.y + pad.h, |acc, child| {
      acc.max(child.margin_rect.y + child.margin_rect.h)
    });
    let max_scroll_y = (content_bottom - pad.y - pad.h).max(0.0);
    raw_scroll_y.clamp(0.0, max_scroll_y)
  } else {
    0.0
  };

  let child_x = x + own_scroll_x;
  let child_y = y + own_scroll_y;

  // If this element has a transform, inverse-transform for children.
  let (child_x, child_y) = if b.transform.is_some() {
    let (lx, ly) = untransform_point(b, child_x, child_y);
    (lx, ly)
  } else {
    (child_x, child_y)
  };

  let child_clip = if own_scroll_x != 0.0 || own_scroll_y != 0.0 {
    next_clip.map(|c| Rect::new(c.x + own_scroll_x, c.y + own_scroll_y, c.w, c.h))
  } else {
    next_clip
  };

  for (i, child) in b.children.iter().enumerate().rev() {
    path.push(i);
    if let Some(result) = collect_hit_path_scrolled(child, child_x, child_y, offsets, path, child_clip) {
      path.pop();
      return Some(result);
    }
    path.pop();
  }

  if b.pointer_events == PointerEvents::None {
    return None;
  }
  let (lx, ly) = untransform_point(b, x, y);
  if b.border_rect.contains(lx, ly) {
    Some(path.clone())
  } else {
    None
  }
}

fn overflow_hit_clip(b: &LayoutBox, parent_clip: Option<Rect>) -> Option<Rect> {
  if !b.overflow.clips_any() {
    return parent_clip;
  }

  let pad = padding_box_rect(b);
  let local = match (b.overflow.clips_x(), b.overflow.clips_y(), parent_clip) {
    (true, true, _) => pad,
    (true, false, Some(parent)) => Rect::new(pad.x, parent.y, pad.w, parent.h),
    (false, true, Some(parent)) => Rect::new(parent.x, pad.y, parent.w, pad.h),
    (true, false, None) => Rect::new(pad.x, f32::MIN / 4.0, pad.w, f32::MAX / 2.0),
    (false, true, None) => Rect::new(f32::MIN / 4.0, pad.y, f32::MAX / 2.0, pad.h),
    (false, false, _) => return parent_clip,
  };

  Some(match parent_clip {
    Some(parent) => intersect_rects_for_hit(parent, local),
    None => local,
  })
}

fn intersect_rects_for_hit(a: Rect, b: Rect) -> Rect {
  let x1 = a.x.max(b.x);
  let y1 = a.y.max(b.y);
  let x2 = (a.x + a.w).min(b.x + b.w);
  let y2 = (a.y + a.h).min(b.y + b.h);
  Rect::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}
