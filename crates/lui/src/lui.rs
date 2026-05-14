use std::path::Path;

use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use crate::display_list::DisplayList;
use lui_glyph::{FontFace, FontHandle, TextContext};
use lui_layout::engine::LayoutEngine;
use lui_parse::{HtmlDocument, Stylesheet};

use crate::RenderBackend;

/// HTML rendering engine.
///
/// Owns the full pipeline and optionally a GPU renderer.
///
/// ```text
/// HTML → parse → cascade → layout → paint → DisplayList → [Renderer] → GPU
/// ```
///
/// # Without features (pipeline only)
/// ```ignore
/// let mut lui = Lui::from_html("<h1>hello</h1>");
/// lui.set_viewport(800.0, 600.0);
/// let list = lui.paint(); // DisplayList — feed to your own renderer
/// ```
///
/// # With `winit` feature
/// ```ignore
/// Lui::from_html("<h1>hello</h1>").run(800, 600, "demo");
/// ```
pub struct Lui {
    pub doc: HtmlDocument,
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

    // ── Viewport & DPI ───────────────────────────────────────────────

    pub fn set_viewport(&mut self, width: f32, height: f32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn set_dpi_scale(&mut self, scale: f32) {
        self.dpi_scale = scale.max(0.25);
    }

    pub fn dpi_scale(&self) -> f32 { self.dpi_scale }
    pub fn viewport(&self) -> (f32, f32) { (self.viewport_width, self.viewport_height) }

    // ── Pipeline ─────────────────────────────────────────────────────

    /// Run cascade → layout → paint and return a `DisplayList`.
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
        let mut list = crate::paint::paint_scaled(&tree, &mut self.text_ctx, scale);
        list.dpi_scale = scale;
        list
    }

    /// Flush dirty atlas regions to a custom sink.
    pub fn flush_atlas(&mut self, mut sink: impl FnMut(u32, u32, u32, u32, &[u8])) {
        self.text_ctx.flush_dirty(|rect, data| {
            sink(rect.x, rect.y, rect.w, rect.h, data);
        });
    }

    // ── Render to an external backend ────────────────────────────────

    /// Paint and submit to any `RenderBackend`.
    pub fn render_with<B: RenderBackend>(&mut self, renderer: &mut B) -> crate::display_list::FrameOutcome {
        let list = self.paint();
        self.text_ctx.flush_dirty(|rect, data| {
            renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });
        renderer.render(&list)
    }

    /// Paint and capture to PNG via any `RenderBackend`.
    pub fn screenshot_with<B: RenderBackend>(
        &mut self,
        renderer: &mut B,
        width: u32,
        height: u32,
        path: impl AsRef<Path>,
    ) -> Result<(), crate::RenderError> {
        let list = self.paint();
        self.text_ctx.flush_dirty(|rect, data| {
            renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });
        renderer.capture_to(&list, width, height, path.as_ref())
    }

    /// Paint and return RGBA pixels via any `RenderBackend`.
    pub fn render_to_rgba_with<B: RenderBackend>(
        &mut self,
        renderer: &mut B,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, crate::RenderError> {
        let list = self.paint();
        self.text_ctx.flush_dirty(|rect, data| {
            renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
        });
        renderer.render_to_rgba(&list, width, height)
    }
}

// ── winit + wgpu: windowed app ───────────────────────────────────────

#[cfg(feature = "winit")]
impl Lui {
    /// Open a wgpu-accelerated window and run the event loop. Blocks until closed.
    pub fn run(self, width: u32, height: u32, title: &str) {
        use std::sync::Arc;
        use winit::application::ApplicationHandler;
        use winit::event::WindowEvent;
        use winit::event_loop::{ActiveEventLoop, EventLoop};
        use winit::window::{WindowAttributes, WindowId};

        struct App {
            lui: Lui,
            renderer: Option<crate::renderer_wgpu::Renderer>,
            window: Option<Arc<winit::window::Window>>,
            title: String,
            initial_size: (u32, u32),
        }

        impl ApplicationHandler for App {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                if self.window.is_some() { return; }
                let attrs = WindowAttributes::default()
                    .with_title(&self.title)
                    .with_inner_size(winit::dpi::LogicalSize::new(
                        self.initial_size.0, self.initial_size.1,
                    ));
                let window = Arc::new(event_loop.create_window(attrs).unwrap());
                let (w, h) = {
                    let s = window.inner_size();
                    (s.width.max(1), s.height.max(1))
                };
                self.renderer = Some(pollster::block_on(
                    crate::renderer_wgpu::Renderer::new(window.clone(), w, h),
                ));
                self.window = Some(window);
            }

            fn window_event(
                &mut self,
                event_loop: &ActiveEventLoop,
                _id: WindowId,
                event: WindowEvent,
            ) {
                match &event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::RedrawRequested => {
                        let Some(window) = &self.window else { return };
                        let Some(renderer) = &mut self.renderer else { return };
                        let size = window.inner_size();
                        let scale = window.scale_factor() as f32;
                        self.lui.set_dpi_scale(scale);
                        self.lui.set_viewport(
                            size.width as f32 / scale,
                            size.height as f32 / scale,
                        );
                        let outcome = self.lui.render_with(renderer);
                        if matches!(outcome, crate::display_list::FrameOutcome::Reconfigure) {
                            renderer.resize(size.width, size.height);
                            window.request_redraw();
                        }
                    }
                    WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                        if let Some(r) = &mut self.renderer { r.resize(size.width, size.height); }
                        if let Some(w) = &self.window { w.request_redraw(); }
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        if let (Some(w), Some(r)) = (&self.window, &mut self.renderer) {
                            let s = w.inner_size();
                            r.resize(s.width, s.height);
                            w.request_redraw();
                        }
                    }
                    _ => {}
                }
            }

            fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
                if let Some(w) = &self.window { w.request_redraw(); }
            }
        }

        let event_loop = EventLoop::new().unwrap();
        let mut app = App {
            lui: self,
            renderer: None,
            window: None,
            title: title.to_string(),
            initial_size: (width, height),
        };
        event_loop.run_app(&mut app).unwrap();
    }
}

impl Default for Lui {
    fn default() -> Self { Self::new() }
}
