//! V2 driver runtime — owns the full pipeline from HTML to GPU.
//!
//! ```text
//! HTML → parse → cascade → layout → paint → DisplayList → RenderBackend
//! ```
//!
//! The `Runtime` struct owns all persistent state (cascade context, layout
//! engine, text context). Platform integration crates (winit, egui, bevy)
//! own a `Runtime` and call `render_frame()` each frame.

use std::path::Path;

use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_glyph::TextContext;
use lui_layout::engine::LayoutEngine;
use lui_parse::HtmlDocument;
pub use lui_display_list::{DisplayList, FrameOutcome};
pub use lui_render_api::{RenderBackend, RenderError};

/// Minimal trait for platform windows.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
}

/// Owns the full v2 pipeline. Generic over the platform driver and render backend.
pub struct Runtime<D: Driver, B: RenderBackend> {
    pub driver: D,
    pub renderer: B,
    pub text_ctx: TextContext,
    cascade_ctx: CascadeContext,
    layout_engine: LayoutEngine,
}

impl<D: Driver, B: RenderBackend> Runtime<D, B> {
    pub fn new(driver: D, renderer: B) -> Self {
        Self {
            driver,
            renderer,
            text_ctx: TextContext::new(),
            cascade_ctx: CascadeContext::new(),
            layout_engine: LayoutEngine::new(),
        }
    }

    /// Set the stylesheets used for cascade. Call once at startup or when CSS changes.
    pub fn set_stylesheets(&mut self, sheets: &[lui_parse::Stylesheet]) {
        self.cascade_ctx.set_stylesheets(sheets);
    }

    /// Run the full pipeline: cascade → layout → paint → render.
    pub fn render_frame(&mut self, doc: &HtmlDocument) -> FrameOutcome {
        let (pw, ph) = self.driver.inner_size();
        let scale = self.driver.scale_factor() as f32;
        let vw = pw as f32;
        let vh = ph as f32;
        let list = self.paint_frame_scaled(doc, vw, vh, scale);

        self.text_ctx.flush_dirty(|rect, data| {
            self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });

        self.renderer.render(&list)
    }

    /// Paint without rendering — useful for testing or headless use.
    pub fn paint_frame(&mut self, doc: &HtmlDocument, vw: f32, vh: f32) -> DisplayList {
        self.paint_frame_scaled(doc, vw, vh, 1.0)
    }

    fn paint_frame_scaled(&mut self, doc: &HtmlDocument, vw: f32, vh: f32, scale: f32) -> DisplayList {
        let media = MediaContext {
            viewport_width: vw,
            viewport_height: vh,
            dpi: 96.0 * scale,
            ..MediaContext::default()
        };
        let interaction = InteractionState::default();
        let styled = self.cascade_ctx.cascade(&doc.root, &media, &interaction);
        let tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
        lui_paint::paint(&tree, &mut self.text_ctx)
    }

    /// Capture the current frame to a PNG file.
    pub fn screenshot_to(
        &mut self,
        doc: &HtmlDocument,
        width: u32,
        height: u32,
        path: impl AsRef<Path>,
    ) -> Result<(), RenderError> {
        let list = self.paint_frame(doc, width as f32, height as f32);

        self.text_ctx.flush_dirty(|rect, data| {
            self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });

        self.renderer.capture_to(&list, width, height, path.as_ref())
    }

    /// Render the current frame to RGBA pixels in memory.
    pub fn render_to_rgba(
        &mut self,
        doc: &HtmlDocument,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, RenderError> {
        let list = self.paint_frame(doc, width as f32, height as f32);

        self.text_ctx.flush_dirty(|rect, data| {
            self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });

        self.renderer.render_to_rgba(&list, width, height)
    }
}
