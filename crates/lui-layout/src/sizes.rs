//! Size resolution: width, height, min-width, min-height, max-width,
//! max-height from `ComputedStyle`.
//!
//! Returns `Option<f32>` — `None` means `auto` (deferred to content/layout mode).

use lui_cascade::ComputedStyle;
use lui_css_parser::{CssUnit, CssValue};

/// Resolve a length value against a containing-block dimension.
/// Returns `None` for `auto`, `Some(px)` otherwise.
pub fn resolve_length(value: Option<&CssValue>, containing: f32) -> Option<f32> {
    match value {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => Some(*value as f32),
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
