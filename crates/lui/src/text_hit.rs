use lui_core::TextCursor;
use lui_glyph::TextContext;
use lui_layout::{LayoutBox, LayoutTree};

use crate::paint::style;

pub fn hit_text_cursor(
  tree: &LayoutTree<'_>,
  x: f32,
  y: f32,
  text_ctx: &mut TextContext,
) -> Option<TextCursor> {
  let mut path = Vec::new();
  hit_text_cursor_box(&tree.root, x, y, 0.0, 0.0, &mut path, text_ctx)
}

fn hit_text_cursor_box(
  b: &LayoutBox<'_>,
  x: f32,
  y: f32,
  scroll_offset_x: f32,
  scroll_offset_y: f32,
  path: &mut Vec<usize>,
  text_ctx: &mut TextContext,
) -> Option<TextCursor> {
  let ax = x + scroll_offset_x;
  let ay = y + scroll_offset_y;

  let br = b.border_rect();
  if ax < br.x || ax > br.x + br.width || ay < br.y || ay > br.y + br.height {
    return None;
  }

  if let Some(clip) = b.clip {
    let cx = clip.x - scroll_offset_x;
    let cy = clip.y - scroll_offset_y;
    if x < cx || x > cx + clip.width || y < cy || y > cy + clip.height {
      return None;
    }
  }

  if b.node.element.is_text() {
    if let Some(cursor) = resolve_text_cursor(b, ax, ay, path, text_ctx) {
      return Some(cursor);
    }
  }

  let child_sx = scroll_offset_x + b.scroll.map(|s| s.scroll_x).unwrap_or(0.0);
  let child_sy = scroll_offset_y + b.scroll.map(|s| s.scroll_y).unwrap_or(0.0);

  for idx in (0..b.children.len()).rev() {
    path.push(idx);
    if let Some(cursor) = hit_text_cursor_box(&b.children[idx], x, y, child_sx, child_sy, path, text_ctx) {
      return Some(cursor);
    }
    path.pop();
  }

  None
}

fn resolve_text_cursor(
  b: &LayoutBox<'_>,
  doc_x: f32,
  doc_y: f32,
  path: &[usize],
  text_ctx: &mut TextContext,
) -> Option<TextCursor> {
  let raw_text = match &b.node.element {
    lui_core::HtmlElement::Text(s) => s.as_ref(),
    _ => return None,
  };
  if raw_text.is_empty() {
    return None;
  }

  let ws = style::css_str(b.style.white_space);
  let collapsed;
  let text = if !matches!(ws, "pre" | "pre-wrap" | "nowrap") {
    collapsed = lui_layout::flow::collapse_whitespace(raw_text);
    collapsed.as_str()
  } else {
    raw_text
  };
  if text.is_empty() {
    return None;
  }

  let font_size = style::css_font_size(b.style.font_size);
  let line_height = match b.style.line_height {
    Some(lui_core::CssValue::Dimension {
      value,
      unit: lui_core::CssUnit::Px,
    }) => *value as f32,
    Some(lui_core::CssValue::Number(n)) => *n as f32 * font_size,
    _ => font_size * 1.2,
  };
  let weight = match b.style.font_weight {
    Some(lui_core::CssValue::Number(n)) => (*n as u16).min(1000),
    _ => 400,
  };
  let font_family = style::css_str(b.style.font_family);

  let ts = lui_glyph::TextStyle {
    font_size,
    line_height,
    font_family,
    weight,
    ..Default::default()
  };
  let run = text_ctx.shape(text, &ts);

  let local_x = doc_x - b.content.x;
  let local_y = doc_y - b.content.y;
  let char_index = run.hit_char_boundary(local_x, local_y);

  Some(TextCursor {
    path: path.to_vec(),
    char_index,
  })
}
