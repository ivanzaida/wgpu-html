//! Size resolution: width, height, min-width, min-height, max-width,
//! max-height from `ComputedStyle`.
//!
//! Returns `Option<f32>` — `None` means `auto` (deferred to content/layout mode).

use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};

use crate::context::LayoutContext;

/// Resolve a length value against a containing-block dimension.
/// Returns `None` for `auto`, `Some(px)` otherwise.
pub fn resolve_length(value: Option<&CssValue>, containing: f32) -> Option<f32> {
    resolve_length_with_font(value, containing, 16.0, 16.0)
}

/// Resolve a length value with font-size context for em/rem.
pub fn resolve_length_ctx(value: Option<&CssValue>, containing: f32, ctx: &LayoutContext) -> Option<f32> {
    resolve_length_with_font(value, containing, ctx.parent_font_size, ctx.root_font_size)
}

fn resolve_length_with_font(value: Option<&CssValue>, containing: f32, em: f32, rem: f32) -> Option<f32> {
    match value {
        Some(CssValue::Dimension { value, unit }) => match unit {
            CssUnit::Px => Some(*value as f32),
            CssUnit::Em => Some(*value as f32 * em),
            CssUnit::Rem => Some(*value as f32 * rem),
            CssUnit::Vw | CssUnit::Vh | CssUnit::Vmin | CssUnit::Vmax => None,
            CssUnit::Pt => Some(*value as f32 * 4.0 / 3.0),
            CssUnit::Cm => Some(*value as f32 * 96.0 / 2.54),
            CssUnit::Mm => Some(*value as f32 * 96.0 / 25.4),
            CssUnit::In => Some(*value as f32 * 96.0),
            CssUnit::Pc => Some(*value as f32 * 16.0),
            _ => Some(*value as f32),
        },
        Some(CssValue::Number(n)) if *n == 0.0 => Some(0.0),
        Some(CssValue::Percentage(pct)) => Some(*pct as f32 / 100.0 * containing),
        Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) if s.as_ref() == "auto" => None,
        _ => None,
    }
}

/// Resolved box dimensions.
pub struct BoxSizes {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
}

/// Clamp a value by resolved min/max constraints.
pub fn clamp_with_minmax(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = if let Some(max) = max { value.min(max) } else { value };
    if let Some(min) = min { v.max(min) } else { v }
}

/// Resolve all size properties from `ComputedStyle`.
pub fn resolve_box_sizes(style: &ComputedStyle, containing_width: f32, containing_height: f32) -> BoxSizes {
    BoxSizes {
        width: resolve_length(style.width, containing_width),
        height: resolve_length(style.height, containing_height),
        min_width: resolve_length(style.min_width, containing_width),
        min_height: resolve_length(style.min_height, containing_height),
        max_width: resolve_length(style.max_width, containing_width),
        max_height: resolve_length(style.max_height, containing_height),
    }
}

/// Extract font-size in px from a ComputedStyle (for threading through context).
pub fn resolve_font_size(style: &ComputedStyle, parent_font_size: f32) -> f32 {
    match style.font_size {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(CssValue::Dimension { value, unit: CssUnit::Em }) => *value as f32 * parent_font_size,
        Some(CssValue::Dimension { value, unit: CssUnit::Rem }) => *value as f32 * 16.0,
        Some(CssValue::Number(n)) => *n as f32,
        Some(CssValue::Percentage(pct)) => *pct as f32 / 100.0 * parent_font_size,
        _ => parent_font_size,
    }
}
