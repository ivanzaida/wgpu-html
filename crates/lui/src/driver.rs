use std::path::Path;

pub use lui_display_list::FrameOutcome;

use crate::{Lui, RenderBackend, RenderError};

/// Minimal trait for platform windows.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
}

/// Owns a `Lui` engine + a platform driver + a render backend.
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
