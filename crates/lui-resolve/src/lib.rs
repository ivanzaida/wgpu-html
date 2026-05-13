//! CSS value resolution: math functions, `var()` substitution, unit conversion.
//!
//! Main entry point: [`ResolutionContext`] — a registry of function handlers.
//! Built-in math functions (`calc`, `min`, `max`, `abs`, `sin`, …) are registered
//! automatically; custom functions can be added via [`ResolutionContext::register`].

pub mod context;
pub mod math;
pub(crate) mod math_helpers;
pub mod units;
pub mod vars;

pub use context::ResolutionContext;

use bumpalo::Bump;

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

/// Convenience: resolve a value with a default context (no unit conversion).
pub fn resolve(value: &lui_core::CssValue, ctx: &ResolverContext) -> lui_core::CssValue {
    let res = ResolutionContext::new(*ctx);
    let arena = Bump::new();
    res.resolve_value(value, &arena).clone()
}
