//! Text shaping: converts a string + font properties into positioned glyphs.

use cosmic_text::{Attrs, Buffer, Family, Metrics, Shaping, Weight};

use crate::font::FontContext;

/// Properties that affect text shaping.
pub struct TextStyle<'a> {
    pub font_size: f32,
    pub line_height: f32,
    pub font_family: &'a str,
    pub weight: u16,
}

impl Default for TextStyle<'_> {
    fn default() -> Self {
        Self { font_size: 16.0, line_height: 19.2, font_family: "sans-serif", weight: 400 }
    }
}

/// A single positioned glyph from shaping.
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub glyph_id: u16,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// A shaped text run — one or more lines of glyphs.
#[derive(Debug, Clone)]
pub struct ShapedRun {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
    pub line_height: f32,
    pub font_size: f32,
    /// Number of lines.
    pub line_count: usize,
}

/// A single line of shaped text.
#[derive(Debug, Clone)]
pub struct ShapedLine {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
}

// ---------------------------------------------------------------------------
// Shaping
// ---------------------------------------------------------------------------

impl FontContext {
    /// Shape a single text span without line breaking.
    pub fn shape(&mut self, text: &str, style: &TextStyle) -> ShapedRun {
        let attrs = build_attrs(style);
        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(&mut self.system, metrics);
        buffer.set_size(&mut self.system, Some(style.font_size), Some(style.line_height));
        buffer.set_text(&mut self.system, text, attrs, Shaping::Advanced);

        let mut glyphs = Vec::new();
        let mut line_count = 0;
        for line in &buffer.lines {
            if let Some(layout) = line.layout_opt().as_ref() {
                line_count += 1;
                for g in layout.iter() {
                    glyphs.push(PositionedGlyph {
                        glyph_id: g.glyph_id,
                        x: g.x,
                        y: g.y,
                        w: g.w,
                        h: style.line_height,
                    });
                }
            }
        }

        let width = glyphs.iter().map(|g| g.x + g.w).fold(0.0f32, f32::max);
        ShapedRun {
            glyphs,
            width,
            height: line_count as f32 * style.line_height,
            line_height: style.line_height,
            font_size: style.font_size,
            line_count,
        }
    }

    /// Shape text and break into lines fitting `max_width`.
    pub fn break_into_lines(&mut self, text: &str, style: &TextStyle, max_width: f32) -> Vec<ShapedLine> {
        let attrs = build_attrs(style);
        let metrics = Metrics::new(style.font_size, style.line_height);
        let mut buffer = Buffer::new(&mut self.system, metrics);
        buffer.set_size(&mut self.system, Some(style.font_size), Some(style.line_height));
        buffer.set_text(&mut self.system, text, attrs, Shaping::Advanced);
        buffer.set_size(&mut self.system, Some(max_width), Some(style.line_height));
        buffer.shape_until_scroll(&mut self.system, false);

        buffer.lines.iter().map(|line| {
            let mut glyphs = Vec::new();
            if let Some(layout) = line.layout_opt().as_ref() {
                for g in layout.iter() {
                    glyphs.push(PositionedGlyph {
                        glyph_id: g.glyph_id, x: g.x, y: g.y, w: g.w, h: style.line_height,
                    });
                }
            }
            let line_width = glyphs.iter().map(|g| g.x + g.w).fold(0.0f32, f32::max);
            ShapedLine { glyphs, width: line_width, height: style.line_height }
        }).collect()
    }
}

// ---------------------------------------------------------------------------
// Style → cosmic-text attrs
// ---------------------------------------------------------------------------

fn build_attrs<'a>(style: &'a TextStyle<'a>) -> Attrs<'a> {
    let family = if style.font_family.is_empty() || style.font_family == "sans-serif" {
        Family::SansSerif
    } else if style.font_family == "serif" {
        Family::Serif
    } else if style.font_family == "monospace" {
        Family::Monospace
    } else {
        Family::Name(style.font_family)
    };
    Attrs::new().family(family).weight(Weight(style.weight))
}
