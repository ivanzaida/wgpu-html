use lui_display_list::{DisplayList, Rect as DlRect, GlyphQuad, DisplayCommand, DisplayCommandKind};
use lui_glyph::TextContext;
use lui_layout::LayoutBox;

use crate::style;

pub fn paint_text(
    b: &LayoutBox,
    content_x: f32,
    content_y: f32,
    opacity: f32,
    text_ctx: &mut TextContext,
    dl: &mut DisplayList,
    dpi_scale: f32,
) {
    let text = match &b.node.element {
        lui_core::HtmlElement::Text(s) => s.as_ref(),
        _ => return,
    };
    if text.is_empty() { return; }

    let mut color = style::css_color(b.style.color).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    color[3] *= opacity;
    if color[3] <= 0.0 { return; }

    let font_size = style::css_f32(b.style.font_size).max(1.0);
    let line_height = match b.style.line_height {
        Some(lui_core::CssValue::Dimension { value, unit: lui_core::CssUnit::Px }) => *value as f32,
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
        if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] { continue; }
        let rect = DlRect::new(
            content_x + glyph.x,
            snap_y + glyph.y,
            glyph.w,
            glyph.h,
        );
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
    if color[3] <= 0.0 { return; }

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
    if color[3] <= 0.0 { return; }

    let font_size = style::css_f32(b.style.font_size).max(1.0);
    let line_height = font_size * 1.2;
    let weight = 400;

    let font_family = style::css_str(b.style.font_family);
    let run = text_ctx.shape_and_pack(marker, font_size, line_height, weight, color, font_family, dpi_scale);
    let marker_x = content_x - run.width;

    for glyph in &run.glyphs {
        if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] { continue; }
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
