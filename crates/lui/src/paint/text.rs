use lui_core::display_list::{DisplayCommand, DisplayCommandKind, DisplayList, GlyphQuad, Rect as DlRect};
use lui_glyph::TextContext;
use lui_layout::LayoutBox;

use super::style;

pub fn paint_text(
  b: &LayoutBox,
  content_x: f32,
  content_y: f32,
  opacity: f32,
  text_ctx: &mut TextContext,
  dl: &mut DisplayList,
  dpi_scale: f32,
) {
  let raw_text = match b.node.element() {
    lui_core::HtmlElement::Text(s) => s.as_ref(),
    _ => return,
  };
  if raw_text.is_empty() {
    return;
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
    return;
  }

  let mut color = style::css_color(b.style.color).unwrap_or([0.0, 0.0, 0.0, 1.0]);
  color[3] *= opacity;
  if color[3] <= 0.0 {
    return;
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

  let run = text_ctx.shape_and_pack(text, font_size, line_height, weight, color, font_family, dpi_scale);

  let snap_y = if dpi_scale > 1.0 {
    (content_y * dpi_scale).round() / dpi_scale
  } else {
    content_y
  };

  for glyph in &run.glyphs {
    if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] {
      continue;
    }
    let rect = DlRect::new(content_x + glyph.x, snap_y + glyph.y, glyph.w, glyph.h);
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

pub fn paint_text_with_selection(
  b: &LayoutBox,
  content_x: f32,
  content_y: f32,
  opacity: f32,
  text_ctx: &mut TextContext,
  dl: &mut DisplayList,
  dpi_scale: f32,
  path: &[usize],
  selection: Option<&lui_core::TextSelection>,
  sel_colors: &lui_core::SelectionColors,
) {
  let raw_text = match b.node.element() {
    lui_core::HtmlElement::Text(s) => s.as_ref(),
    _ => return,
  };
  if raw_text.is_empty() {
    return;
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
    return;
  }

  let mut color = style::css_color(b.style.color).unwrap_or([0.0, 0.0, 0.0, 1.0]);
  color[3] *= opacity;
  if color[3] <= 0.0 {
    return;
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

  let run = text_ctx.shape_and_pack(text, font_size, line_height, weight, color, font_family, dpi_scale);

  let sel_range = selection.and_then(|sel| selection_char_range(path, sel, run.char_count()));

  if let Some((from_char, to_char)) = sel_range {
    let from_glyph = run.char_to_glyph_index(from_char);
    let to_glyph = run.char_to_glyph_index(to_char);
    paint_selection_bg(&run, content_x, content_y, from_glyph, to_glyph, sel_colors.background, opacity, dl);
  }

  let snap_y = if dpi_scale > 1.0 {
    (content_y * dpi_scale).round() / dpi_scale
  } else {
    content_y
  };

  for (gi, glyph) in run.glyphs.iter().enumerate() {
    if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] {
      continue;
    }
    let char_idx = run.glyph_to_char_index(gi);
    let in_selection = sel_range.is_some_and(|(from, to)| char_idx >= from && char_idx < to);
    let glyph_color = if in_selection { sel_colors.foreground } else { color };

    let rect = DlRect::new(content_x + glyph.x, snap_y + glyph.y, glyph.w, glyph.h);
    let index = dl.glyphs.len() as u32;
    dl.glyphs.push(GlyphQuad {
      rect,
      color: glyph_color,
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

fn selection_char_range(
  path: &[usize],
  sel: &lui_core::TextSelection,
  char_count: usize,
) -> Option<(usize, usize)> {
  let (start, end) = sel.ordered();
  let cmp_start = path.cmp(&start.path.as_slice());
  let cmp_end = path.cmp(&end.path.as_slice());

  if cmp_start == std::cmp::Ordering::Greater && cmp_end == std::cmp::Ordering::Less {
    return None;
  }

  let from = match cmp_start {
    std::cmp::Ordering::Equal => start.char_index.min(char_count),
    std::cmp::Ordering::Greater => 0,
    std::cmp::Ordering::Less => return None,
  };
  let to = match cmp_end {
    std::cmp::Ordering::Equal => end.char_index.min(char_count),
    std::cmp::Ordering::Less => char_count,
    std::cmp::Ordering::Greater => return None,
  };

  if to > from { Some((from, to)) } else { None }
}

fn paint_selection_bg(
  run: &lui_glyph::ShapedRun,
  content_x: f32,
  content_y: f32,
  from_glyph: usize,
  to_glyph: usize,
  bg_color: [f32; 4],
  opacity: f32,
  dl: &mut DisplayList,
) {
  if from_glyph >= to_glyph { return; }
  let color = [bg_color[0], bg_color[1], bg_color[2], bg_color[3] * opacity];

  if run.lines.is_empty() {
    let x_min = run.glyphs.get(from_glyph).map(|g| g.x).unwrap_or(0.0);
    let x_max = run.glyphs.get(to_glyph.saturating_sub(1)).map(|g| g.x + g.w).unwrap_or(0.0);
    if x_max > x_min {
      dl.push_quad(DlRect::new(content_x + x_min, content_y, x_max - x_min, run.line_height), color);
    }
    return;
  }

  for line in &run.lines {
    let line_from = from_glyph.max(line.glyph_start);
    let line_to = to_glyph.min(line.glyph_end);
    if line_from >= line_to { continue; }
    let x_min = run.glyphs[line_from].x;
    let x_max = run.glyphs[line_to - 1].x + run.glyphs[line_to - 1].w;
    if x_max > x_min {
      dl.push_quad(
        DlRect::new(content_x + x_min, content_y + line.top, x_max - x_min, line.height),
        color,
      );
    }
  }
}

pub fn paint_text_decoration(
  b: &LayoutBox,
  content_x: f32,
  content_y: f32,
  content_w: f32,
  line_height: f32,
  opacity: f32,
  dl: &mut DisplayList,
) {
  let decoration = match &b.text_decoration {
    Some(d) => d.as_str(),
    None => return,
  };
  let mut color = style::css_color(b.style.text_decoration_color)
    .or_else(|| style::css_color(b.style.color))
    .unwrap_or([0.0, 0.0, 0.0, 1.0]);
  color[3] *= opacity;
  if color[3] <= 0.0 {
    return;
  }

  let thickness = 1.0_f32;
  for token in decoration.split_whitespace() {
    let y_offset = match token {
      "underline" => content_y + line_height - 2.0,
      "overline" => content_y + 1.0,
      "line-through" => content_y + line_height * 0.5,
      _ => continue,
    };
    let rect = DlRect::new(content_x, y_offset, content_w, thickness);
    dl.push_quad(rect, color);
  }
}

pub fn paint_list_marker(
  b: &LayoutBox,
  content_x: f32,
  content_y: f32,
  opacity: f32,
  text_ctx: &mut TextContext,
  dl: &mut DisplayList,
  dpi_scale: f32,
) {
  let marker = match &b.list_marker {
    Some(m) => m.as_str(),
    None => return,
  };

  let mut color = style::css_color(b.style.color).unwrap_or([0.0, 0.0, 0.0, 1.0]);
  color[3] *= opacity;
  if color[3] <= 0.0 {
    return;
  }

  let font_size = style::css_font_size(b.style.font_size);
  let line_height = font_size * 1.2;
  let weight = 400;

  let font_family = style::css_str(b.style.font_family);
  let run = text_ctx.shape_and_pack(marker, font_size, line_height, weight, color, font_family, dpi_scale);
  let marker_x = content_x - run.width;

  for glyph in &run.glyphs {
    if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] {
      continue;
    }
    let rect = DlRect::new(marker_x + glyph.x, content_y + glyph.y, glyph.w, glyph.h);
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
