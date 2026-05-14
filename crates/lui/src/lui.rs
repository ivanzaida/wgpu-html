use std::path::Path;
use std::sync::Arc;

use lui_cascade::{
    cascade::{CascadeContext, InteractionState},
    media::MediaContext,
};
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

use crate::{
    display_list::{DisplayList, FrameOutcome},
    Driver, RenderBackend, RenderError, WindowHandle,
};

pub struct Lui {
    pub doc: HtmlDocument,
    pub(crate) text_ctx: TextContext,
    cascade_ctx: CascadeContext,
    layout_engine: LayoutEngine,
    dpi_scale_override: Option<f32>,
    pub driver: Box<dyn Driver>,
    pub renderer: Box<dyn RenderBackend>,
}

impl Lui {
    pub fn new(driver: Box<dyn Driver>, renderer: Box<dyn RenderBackend>) -> Self {
        Self {
            doc: lui_parse::parse("<html><body></body></html>"),
            text_ctx: TextContext::new(),
            cascade_ctx: CascadeContext::new(),
            layout_engine: LayoutEngine::new(),
            dpi_scale_override: None,
            driver,
            renderer,
        }
    }

    pub fn set_html(&mut self, html: &str) {
        self.doc = lui_parse::parse(html);
    }

    pub fn doc(&self) -> &HtmlDocument { &self.doc }
    pub fn doc_mut(&mut self) -> &mut HtmlDocument { &mut self.doc }

    pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
        self.cascade_ctx.set_stylesheets(sheets);
    }

    pub fn register_font(&mut self, face: FontFace) -> FontHandle {
        self.text_ctx.register_font(face)
    }

    pub fn set_dpi_scale(&mut self, scale: Option<f32>) {
        self.dpi_scale_override = scale;
    }

    pub fn run(mut self) {
        let driver = std::mem::replace(&mut self.driver, Box::new(crate::NullDriver));
        driver.run(self);
    }

    /// Called by the driver when the window is ready.
    pub fn init_renderer(&mut self, window: Arc<dyn WindowHandle>, width: u32, height: u32) {
        self.renderer.init(window, width, height);
    }

    pub fn render_frame(
        &mut self,
        physical_width: u32,
        physical_height: u32,
        scale: f32,
    ) -> FrameOutcome {
        let list = self.paint(physical_width, physical_height, scale);
        self.flush_atlas();
        self.renderer.render(&list)
    }

    pub fn screenshot_to(
        &mut self,
        physical_width: u32,
        physical_height: u32,
        scale: f32,
        path: impl AsRef<Path>,
    ) -> Result<(), RenderError> {
        let list = self.paint(physical_width, physical_height, scale);
        self.flush_atlas();
        self.renderer.capture_to(&list, physical_width, physical_height, path.as_ref())
    }

    fn paint(&mut self, pw: u32, ph: u32, scale: f32) -> DisplayList {
        let scale = self.dpi_scale_override.unwrap_or(scale);
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
        self.text_ctx.flush_dirty(|rect, data| {
            self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });
    }
}
