use std::path::Path;

use lui_cascade::{
    cascade::{CascadeContext, InteractionState},
    media::MediaContext,
};
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

use crate::{
    display_list::{DisplayList, FrameOutcome}, Driver, RenderBackend,
    RenderError,
};

/// HTML rendering engine.
///
/// Owns the full pipeline, a platform driver, and a render backend.
/// The user never touches DisplayList, atlas uploads, or paint internals.
///
/// ```ignore
/// let mut lui = Lui::new(my_driver, my_renderer);
/// lui.set_html("<h1>hello</h1>");
/// lui.render(); // cascade → layout → paint → GPU
/// ```
pub struct Lui {
  pub doc: HtmlDocument,
  text_ctx: TextContext,
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

  // ── Document ─────────────────────────────────────────────────────

  pub fn set_html(&mut self, html: &str) {
    self.doc = lui_parse::parse(html);
  }

  pub fn doc(&self) -> &HtmlDocument {
    &self.doc
  }
  pub fn doc_mut(&mut self) -> &mut HtmlDocument {
    &mut self.doc
  }

  // ── Stylesheets ──────────────────────────────────────────────────

  pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
    self.cascade_ctx.set_stylesheets(sheets);
  }

  // ── Fonts ────────────────────────────────────────────────────────

  pub fn register_font(&mut self, face: FontFace) -> FontHandle {
    self.text_ctx.register_font(face)
  }

  // ── DPI ──────────────────────────────────────────────────────────

  /// Override the DPI scale. `None` = use driver's scale_factor.
  pub fn set_dpi_scale(&mut self, scale: Option<f32>) {
    self.dpi_scale_override = scale;
  }

  pub fn dpi_scale(&self) -> f32 {
    self.dpi_scale_override.unwrap_or(self.driver.scale_factor() as f32)
  }

  // ── Render ───────────────────────────────────────────────────────

  /// Run the full pipeline: cascade → layout → paint → render.
  pub fn render(&mut self) -> FrameOutcome {
    let list = self.paint_frame();
    self.flush_atlas();
    self.renderer.render(&list)
  }

  /// Capture to PNG at the current viewport + DPI.
  pub fn screenshot_to(&mut self, path: impl AsRef<Path>) -> Result<(), RenderError> {
    let (pw, ph) = self.driver.inner_size();
    let list = self.paint_frame();
    self.flush_atlas();
    self.renderer.capture_to(&list, pw, ph, path.as_ref())
  }

  /// Render to RGBA pixels at the current viewport + DPI.
  pub fn render_to_rgba(&mut self) -> Result<Vec<u8>, RenderError> {
    let (pw, ph) = self.driver.inner_size();
    let list = self.paint_frame();
    self.flush_atlas();
    self.renderer.render_to_rgba(&list, pw, ph)
  }

  // ── Internal ─────────────────────────────────────────────────────

  fn paint_frame(&mut self) -> DisplayList {
    let (pw, ph) = self.driver.inner_size();
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
    self.text_ctx.flush_dirty(|rect, data| {
      self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
    });
  }
}
