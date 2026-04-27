use wgpu_html_models::common::css_enums::CssLength;

use crate::Ctx;

const DEFAULT_FONT_PX: f32 = 16.0;

/// Resolve a CSS length to physical pixels.
///
/// - `Px` is taken verbatim.
/// - `Percent` is resolved against `parent_size_px`.
/// - `Vw` / `Vh` / `Vmin` / `Vmax` against the viewport.
/// - `Em` / `Rem` against `DEFAULT_FONT_PX` (real font metrics come later).
/// - `Auto` and `Raw(_)` return `None` (the caller picks a default).
pub(crate) fn resolve(len: Option<&CssLength>, parent_size_px: f32, ctx: &Ctx) -> Option<f32> {
    match len? {
        CssLength::Px(v) => Some(*v),
        CssLength::Percent(v) => Some(parent_size_px * v / 100.0),
        CssLength::Em(v) | CssLength::Rem(v) => Some(*v * DEFAULT_FONT_PX),
        CssLength::Vw(v) => Some(ctx.viewport_w * v / 100.0),
        CssLength::Vh(v) => Some(ctx.viewport_h * v / 100.0),
        CssLength::Vmin(v) => Some(ctx.viewport_w.min(ctx.viewport_h) * v / 100.0),
        CssLength::Vmax(v) => Some(ctx.viewport_w.max(ctx.viewport_h) * v / 100.0),
        CssLength::Zero => Some(0.0),
        CssLength::Auto | CssLength::Raw(_) => None,
    }
}
