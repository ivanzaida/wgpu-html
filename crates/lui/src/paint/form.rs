use std::collections::BTreeMap;

use lui_core::display_list::{DisplayCommand, DisplayCommandKind, DisplayList, GlyphQuad, Rect as DlRect};
use lui_core::form_state::FormControlState;
use lui_core::text_selection::EditCursor;
use lui_glyph::TextContext;
use lui_layout::LayoutBox;

use super::style;

#[derive(Clone)]
pub struct FormPaintCtx {
  pub focus_path: Option<Vec<usize>>,
  pub form_state: BTreeMap<Vec<usize>, FormControlState>,
}

impl Default for FormPaintCtx {
  fn default() -> Self {
    Self {
      focus_path: None,
      form_state: BTreeMap::new(),
    }
  }
}

pub fn paint_form_control(
  b: &LayoutBox,
  content_x: f32,
  content_y: f32,
  opacity: f32,
  text_ctx: &mut TextContext,
  dl: &mut DisplayList,
  dpi_scale: f32,
  path: &[usize],
  ctx: &FormPaintCtx,
) {
  let is_input = matches!(b.node.element(), lui_core::HtmlElement::Input);
  let is_textarea = matches!(b.node.element(), lui_core::HtmlElement::Textarea);
  if !is_input && !is_textarea {
    return;
  }

  if is_input {
    let input_type = b.node.attr("type").map(|v| v.as_ref()).unwrap_or("text");
    if !matches!(input_type, "text" | "password" | "email" | "search" | "tel" | "url" | "number") {
      return;
    }
  }

  let state = ctx.form_state.get(path);
  let is_focused = ctx.focus_path.as_deref() == Some(path);

  let value = state
    .map(|s| s.value.as_str())
    .or_else(|| b.node.attr("value").map(|v| v.as_ref()))
    .unwrap_or("");

  let is_password = is_input
    && b.node.attr("type").map(|v| v.as_ref()) == Some("password");

  let mut color = style::css_color(b.style.color).unwrap_or([0.0, 0.0, 0.0, 1.0]);
  color[3] *= opacity;

  let font_size = style::css_font_size(b.style.font_size);
  let line_height = resolve_line_height(b, font_size);
  let weight = match b.style.font_weight {
    Some(lui_core::CssValue::Number(n)) => (*n as u16).min(1000),
    _ => 400,
  };
  let font_family = style::css_str(b.style.font_family);

  if value.is_empty() {
    if let Some(placeholder) = b.node.attr("placeholder") {
      let mut ph_color = color;
      ph_color[3] *= 0.5;
      paint_text_run(
        placeholder.as_ref(),
        content_x,
        content_y,
        ph_color,
        font_size,
        line_height,
        weight,
        font_family,
        text_ctx,
        dl,
        dpi_scale,
        0.0,
      );
    }
    if is_focused {
      if let Some(state) = state {
        if state.caret_visible() {
          paint_caret(content_x, content_y, line_height, color, dl);
        }
      }
    }
    return;
  }

  let display_text: String;
  let text = if is_password {
    display_text = "\u{2022}".repeat(value.chars().count());
    &display_text
  } else {
    value
  };

  let scroll_x = state.map(|s| s.scroll_x).unwrap_or(0.0);

  let edit_cursor = if is_focused { state.map(|s| &s.edit_cursor) } else { None };

  if let Some(ec) = edit_cursor {
    if ec.has_selection() {
      paint_form_selection(
        text,
        content_x,
        content_y,
        font_size,
        line_height,
        weight,
        font_family,
        text_ctx,
        dl,
        dpi_scale,
        scroll_x,
        ec,
        opacity,
      );
    }
  }

  paint_text_run(
    text,
    content_x,
    content_y,
    color,
    font_size,
    line_height,
    weight,
    font_family,
    text_ctx,
    dl,
    dpi_scale,
    scroll_x,
  );

  if is_focused {
    if let Some(state) = state {
      if state.caret_visible() && !state.edit_cursor.has_selection() {
        let caret_x = byte_offset_to_x(
          text,
          state.edit_cursor.cursor,
          font_size,
          line_height,
          weight,
          font_family,
          text_ctx,
          dpi_scale,
        );
        paint_caret(content_x + caret_x - scroll_x, content_y, line_height, color, dl);
      }
    }
  }
}

fn paint_text_run(
  text: &str,
  content_x: f32,
  content_y: f32,
  color: [f32; 4],
  font_size: f32,
  line_height: f32,
  weight: u16,
  font_family: &str,
  text_ctx: &mut TextContext,
  dl: &mut DisplayList,
  dpi_scale: f32,
  scroll_x: f32,
) {
  if text.is_empty() || color[3] <= 0.0 {
    return;
  }

  let run = text_ctx.shape_and_pack(text, font_size, line_height, weight, color, font_family, dpi_scale, None);

  let snap_y = if dpi_scale > 1.0 {
    (content_y * dpi_scale).round() / dpi_scale
  } else {
    content_y
  };

  for glyph in &run.glyphs {
    if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] {
      continue;
    }
    let rect = DlRect::new(content_x + glyph.x - scroll_x, snap_y + glyph.y, glyph.w, glyph.h);
    let index = dl.glyphs.len() as u32;
    dl.glyphs.push(GlyphQuad {
      rect,
      color,
      uv_min: glyph.uv_min,
      uv_max: glyph.uv_max,
      transform: [1.0, 0.0, 0.0, 1.0],
      transform_origin: [rect.w * 0.5, rect.h * 0.5],
    });
    dl.commands.push(DisplayCommand {
      kind: DisplayCommandKind::Glyph,
      index,
      clip_index: dl.clips.len().saturating_sub(1) as u32,
    });
  }
}

fn paint_caret(x: f32, y: f32, height: f32, color: [f32; 4], dl: &mut DisplayList) {
  dl.push_quad(DlRect::new(x, y, 1.5, height), color);
}

fn paint_form_selection(
  text: &str,
  content_x: f32,
  content_y: f32,
  font_size: f32,
  line_height: f32,
  weight: u16,
  font_family: &str,
  text_ctx: &mut TextContext,
  dl: &mut DisplayList,
  dpi_scale: f32,
  scroll_x: f32,
  cursor: &EditCursor,
  opacity: f32,
) {
  let (start, end) = cursor.selection_range();
  let start_x = byte_offset_to_x(text, start, font_size, line_height, weight, font_family, text_ctx, dpi_scale);
  let end_x = byte_offset_to_x(text, end, font_size, line_height, weight, font_family, text_ctx, dpi_scale);

  if end_x > start_x {
    let sel_color = [0.23, 0.51, 0.96, 0.45 * opacity];
    dl.push_quad(
      DlRect::new(content_x + start_x - scroll_x, content_y, end_x - start_x, line_height),
      sel_color,
    );
  }
}

fn byte_offset_to_x(
  text: &str,
  byte_offset: usize,
  font_size: f32,
  line_height: f32,
  weight: u16,
  font_family: &str,
  text_ctx: &mut TextContext,
  dpi_scale: f32,
) -> f32 {
  if byte_offset == 0 || text.is_empty() {
    return 0.0;
  }
  let offset = byte_offset.min(text.len());
  let prefix = &text[..offset];
  let color = [0.0; 4];
  let run = text_ctx.shape_and_pack(prefix, font_size, line_height, weight, color, font_family, dpi_scale, None);
  run.width
}

fn resolve_line_height(b: &LayoutBox, font_size: f32) -> f32 {
  match b.style.line_height {
    Some(lui_core::CssValue::Dimension {
      value,
      unit: lui_core::CssUnit::Px,
    }) => *value as f32,
    Some(lui_core::CssValue::Number(n)) => *n as f32 * font_size,
    _ => font_size * 1.2,
  }
}
