//! Layout context — viewport, containing block, font metrics.

/// Context threaded through the layout tree during layout.
#[derive(Debug, Clone, Copy)]
pub struct LayoutContext {
    /// Viewport width in px.
    pub viewport_width: f32,
    /// Viewport height in px.
    pub viewport_height: f32,
    /// Content-box width of the current containing block.
    pub containing_width: f32,
    /// Content-box height of the current containing block (may be NaN for auto).
    pub containing_height: f32,
    /// Root font-size (for `rem`).
    pub root_font_size: f32,
    /// Parent font-size (for `em`).
    pub parent_font_size: f32,
}

impl LayoutContext {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Self {
        Self {
            viewport_width,
            viewport_height,
            containing_width: viewport_width,
            containing_height: f32::NAN,
            root_font_size: 16.0,
            parent_font_size: 16.0,
        }
    }
}
