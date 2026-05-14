//! V2 driver runtime — thin wrapper around `Lui` + a render backend.
//!
//! ```text
//! Lui (cascade → layout → paint → DisplayList) → RenderBackend → GPU
//! ```

use std::path::Path;

use lui::Lui;
pub use lui_display_list::{DisplayList, FrameOutcome};
pub use lui_render_api::{RenderBackend, RenderError};

/// Minimal trait for platform windows.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
}

/// Owns a `Lui` engine + a render backend.
pub struct Runtime<D: Driver, B: RenderBackend> {
    pub driver: D,
    pub renderer: B,
    pub lui: Lui,
}

impl<D: Driver, B: RenderBackend> Runtime<D, B> {
    pub fn new(driver: D, renderer: B) -> Self {
        Self {
            driver,
            renderer,
            lui: Lui::new(),
        }
    }

    /// Set the stylesheets used for cascade.
    pub fn set_stylesheets(&mut self, sheets: &[lui_parse::Stylesheet]) {
        self.lui.set_stylesheets(sheets);
    }

    /// Sync viewport + DPI from the platform driver, then paint + render.
    pub fn render_frame(&mut self) -> FrameOutcome {
        let (pw, ph) = self.driver.inner_size();
        let scale = self.driver.scale_factor() as f32;
        self.lui.set_dpi_scale(scale);
        self.lui.set_viewport(pw as f32 / scale, ph as f32 / scale);

        let list = self.lui.paint();

        self.lui.flush_atlas(|x, y, w, h, data| {
            self.renderer.upload_atlas_region(x, y, w, h, data);
        });

        self.renderer.render(&list)
    }

    /// Capture to a PNG file at the current viewport + DPI.
    pub fn screenshot_to(&mut self, path: impl AsRef<Path>) -> Result<(), RenderError> {
        let (pw, ph) = self.driver.inner_size();
        let scale = self.driver.scale_factor() as f32;
        self.lui.set_dpi_scale(scale);
        self.lui.set_viewport(pw as f32 / scale, ph as f32 / scale);

        let list = self.lui.paint();

        self.lui.flush_atlas(|x, y, w, h, data| {
            self.renderer.upload_atlas_region(x, y, w, h, data);
        });

        self.renderer.capture_to(&list, pw, ph, path.as_ref())
    }

    /// Render to RGBA pixels at the current viewport + DPI.
    pub fn render_to_rgba(&mut self) -> Result<Vec<u8>, RenderError> {
        let (pw, ph) = self.driver.inner_size();
        let scale = self.driver.scale_factor() as f32;
        self.lui.set_dpi_scale(scale);
        self.lui.set_viewport(pw as f32 / scale, ph as f32 / scale);

        let list = self.lui.paint();

        self.lui.flush_atlas(|x, y, w, h, data| {
            self.renderer.upload_atlas_region(x, y, w, h, data);
        });

        self.renderer.render_to_rgba(&list, pw, ph)
    }
}
