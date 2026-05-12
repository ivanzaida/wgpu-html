//! CSS value resolution: evaluates `calc()`, `min()`, `max()`, `clamp()`,
//! resolves `var()` references, and converts relative units (`em`, `rem`, `vw`, `vh`, etc.).
//!
//! ## Cascade integration
//!
//! Called from `lui-cascade` after inheritance:
//! 1. `resolve_custom_properties` — flatten `var()` chains in custom properties
//! 2. `resolve_var_value` + `resolve_math` — resolve each style property value

pub mod math;
pub(crate) mod math_helpers;
pub mod units;
pub mod vars;

use lui_css_parser::CssValue;

/// Context required for resolving relative CSS units and percentages.
#[derive(Debug, Clone, Copy)]
pub struct ResolverContext {
    /// Viewport width in CSS pixels.
    pub viewport_width: f32,
    /// Viewport height in CSS pixels.
    pub viewport_height: f32,
    /// Root element's computed `font-size` in px.
    pub root_font_size: f32,
    /// Parent element's computed `font-size` in px.
    pub parent_font_size: f32,
}

impl Default for ResolverContext {
    fn default() -> Self {
        Self {
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            root_font_size: 16.0,
            parent_font_size: 16.0,
        }
    }
}

impl ResolverContext {
    pub fn from_cascade(
        viewport_width: f32,
        viewport_height: f32,
        root_font_size: f32,
        parent_font_size: f32,
    ) -> Self {
        Self { viewport_width, viewport_height, root_font_size, parent_font_size }
    }
}

/// Resolve all math functions and relative units in a value.
pub fn resolve(value: &CssValue, ctx: &ResolverContext) -> CssValue {
    units::resolve_units(value, ctx)
}
