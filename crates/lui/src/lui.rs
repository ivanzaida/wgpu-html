use std::path::Path;

use lui_cascade::{
    cascade::{CascadeContext, InteractionState},
    media::MediaContext,
};
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

use crate::{
    display_list::{DisplayList, FrameOutcome},
    Driver, RenderBackend, RenderError,
};

/// HTML rendering engine.
///
/// ```ignore
/// let mut lui = Lui::new();
/// lui.set_html("<h1>hello</h1>");
/// lui.run(800, 600, "demo"); // winit feature
/// ```
pub struct Lui {
    pub doc: HtmlDocument,
    pub(crate) text_ctx: TextContext,
    cascade_ctx: CascadeContext,
    layout_engine: LayoutEngine,
    dpi_scale_override: Option<f32>,
    pub driver: Option<Box<dyn Driver>>,
    pub renderer: Option<Box<dyn RenderBackend>>,
}

impl Lui {
    pub fn new() -> Self {
        Self {
            doc: lui_parse::parse("<html><body></body></html>"),
            text_ctx: TextContext::new(),
            cascade_ctx: CascadeContext::new(),
            layout_engine: LayoutEngine::new(),
            dpi_scale_override: None,
            driver: None,
            renderer: None,
        }
    }

    // ── Document ─────────────────────────────────────────────────────

    pub fn set_html(&mut self, html: &str) {
        self.doc = lui_parse::parse(html);
    }

    pub fn doc(&self) -> &HtmlDocument { &self.doc }
    pub fn doc_mut(&mut self) -> &mut HtmlDocument { &mut self.doc }

    // ── Stylesheets ──────────────────────────────────────────────────

    pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
        self.cascade_ctx.set_stylesheets(sheets);
    }

    // ── Fonts ────────────────────────────────────────────────────────

    pub fn register_font(&mut self, face: FontFace) -> FontHandle {
        self.text_ctx.register_font(face)
    }

    // ── DPI ──────────────────────────────────────────────────────────

    pub fn set_dpi_scale(&mut self, scale: Option<f32>) {
        self.dpi_scale_override = scale;
    }

    pub fn dpi_scale(&self) -> f32 {
        if let Some(s) = self.dpi_scale_override { return s; }
        if let Some(d) = &self.driver { return d.scale_factor() as f32; }
        1.0
    }

    // ── Render (requires driver + renderer) ──────────────────────────

    pub fn render(&mut self) -> FrameOutcome {
        let list = self.paint_frame();
        self.flush_atlas();
        self.renderer.as_mut().unwrap().render(&list)
    }

    pub fn screenshot_to(&mut self, path: impl AsRef<Path>) -> Result<(), RenderError> {
        let (pw, ph) = self.driver.as_ref().unwrap().inner_size();
        let list = self.paint_frame();
        self.flush_atlas();
        self.renderer.as_mut().unwrap().capture_to(&list, pw, ph, path.as_ref())
    }

    pub fn render_to_rgba(&mut self) -> Result<Vec<u8>, RenderError> {
        let (pw, ph) = self.driver.as_ref().unwrap().inner_size();
        let list = self.paint_frame();
        self.flush_atlas();
        self.renderer.as_mut().unwrap().render_to_rgba(&list, pw, ph)
    }

    // ── Internal ─────────────────────────────────────────────────────

    fn paint_frame(&mut self) -> DisplayList {
        let (pw, ph) = self.driver.as_ref().unwrap().inner_size();
        let scale = self.dpi_scale();
        let vw = pw as f32 / scale;
        let vh = ph as f32 / scale;

        let media = MediaContext {
            viewport_width: vw,
            viewport_height: vh,
            dpi: 96.0 * scale,
            ..MediaContext::default()
        };
        let interaction = InteractionState::default();
        let styled = self.cascade_ctx.cascade(&self.doc.root, &media, &interaction);
        let tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
        let mut list = crate::paint::paint_scaled(&tree, &mut self.text_ctx, scale);
        list.dpi_scale = scale;
        list
    }

    fn flush_atlas(&mut self) {
        let renderer = self.renderer.as_mut().unwrap();
        self.text_ctx.flush_dirty(|rect, data| {
            renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });
    }
}

impl Default for Lui {
    fn default() -> Self { Self::new() }
}
