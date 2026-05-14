use std::path::Path;

use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_display_list::DisplayList;
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

/// Renderer-agnostic HTML engine.
///
/// Owns the full pipeline from HTML to `DisplayList`:
/// ```text
/// HTML → parse → cascade → layout → paint → DisplayList
/// ```
///
/// Platform drivers (winit, egui, bevy) own a `Lui` instance and feed
/// the produced `DisplayList` to their renderer each frame.
pub struct Lui {
    doc: HtmlDocument,
    pub text_ctx: TextContext,
    cascade_ctx: CascadeContext,
    layout_engine: LayoutEngine,
    dpi_scale: f32,
    viewport_width: f32,
    viewport_height: f32,
}

impl Lui {
    pub fn new() -> Self {
        Self {
            doc: lui_parse::parse("<html><body></body></html>"),
            text_ctx: TextContext::new(),
            cascade_ctx: CascadeContext::new(),
            layout_engine: LayoutEngine::new(),
            dpi_scale: 1.0,
            viewport_width: 0.0,
            viewport_height: 0.0,
        }
    }

    pub fn from_html(html: &str) -> Self {
        let mut lui = Self::new();
        lui.set_html(html);
        lui
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let html = std::fs::read_to_string(path)?;
        Ok(Self::from_html(&html))
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

    // ── Viewport & DPI ───────────────────────────────────────────────

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn set_dpi_scale(&mut self, scale: f32) {
        self.dpi_scale = scale.max(0.25);
    }

    pub fn dpi_scale(&self) -> f32 {
        self.dpi_scale
    }

    pub fn viewport(&self) -> (f32, f32) {
        (self.viewport_width, self.viewport_height)
    }

    // ── Pipeline ─────────────────────────────────────────────────────

    /// Run cascade → layout → paint and return a `DisplayList`.
    ///
    /// Coordinates in the list are in logical (CSS) pixels.
    /// `list.dpi_scale` tells the renderer how to map to physical pixels.
    pub fn paint(&mut self) -> DisplayList {
        let scale = self.dpi_scale;
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        let media = MediaContext {
            viewport_width: vw,
            viewport_height: vh,
            dpi: 96.0 * scale,
            ..MediaContext::default()
        };
        let interaction = InteractionState::default();
        let styled = self.cascade_ctx.cascade(&self.doc.root, &media, &interaction);
        let tree = self.layout_engine.layout(&styled, vw, vh, &mut self.text_ctx);
        let mut list = lui_paint::paint_scaled(&tree, &mut self.text_ctx, scale);
        list.dpi_scale = scale;
        list
    }

    /// Flush dirty atlas regions. Call this after `paint()` before
    /// submitting the display list to the renderer.
    pub fn flush_atlas(&mut self, mut sink: impl FnMut(u32, u32, u32, u32, &[u8])) {
        self.text_ctx.flush_dirty(|rect, data| {
            sink(rect.x, rect.y, rect.w, rect.h, data);
        });
    }
}

impl Default for Lui {
    fn default() -> Self {
        Self::new()
    }
}
