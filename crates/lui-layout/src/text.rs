//! Text context for the layout engine.
//! Wraps `lui_glyph::FontContext` and extracts style values from `ComputedStyle`.

use lui_cascade::ComputedStyle;
use lui_css_parser::{CssUnit, CssValue};
use lui_glyph::{FontContext, ShapedRun, TextStyle};

/// Text state owned by the layout pass.
pub struct TextContext {
    pub font_ctx: FontContext,
}

impl TextContext {
    pub fn new() -> Self {
        Self { font_ctx: FontContext::new() }
    }

    /// Shape a text span using the computed style.
    pub fn shape_run(&mut self, text: &str, style: &ComputedStyle) -> ShapedRun {
        let ts = text_style_from_cascade(style);
        self.font_ctx.shape(text, &ts)
    }
}

impl Default for TextContext {
    fn default() -> Self { Self::new() }
}

/// Extract text-relevant values from `ComputedStyle`.
pub fn text_style_from_cascade(style: &ComputedStyle) -> TextStyle {
    let font_size = match style.font_size {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(&CssValue::Number(n)) => n as f32,
        _ => 16.0,
    };
    let line_height = match style.line_height {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(&CssValue::Number(n)) => (n * font_size as f64) as f32,
        _ => font_size * 1.2,
    };
    let weight = match style.font_weight {
        Some(CssValue::Number(n)) => (*n as u16).min(1000),
        _ => 400,
    };
    let family = match style.font_family {
        Some(CssValue::String(ref s)) | Some(CssValue::Unknown(ref s)) => s.as_ref(),
        _ => "",
    };
    TextStyle { font_size, line_height, font_family: family, weight }
}
