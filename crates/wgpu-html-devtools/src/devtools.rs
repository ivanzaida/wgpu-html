use std::sync::Arc;
use std::time::Instant;

use wgpu_html::paint_tree_returning_layout;
use wgpu_html_renderer::{FrameOutcome, GLYPH_ATLAS_SIZE, Renderer};
use wgpu_html_text::TextContext;
use wgpu_html_tree::Tree;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::html_gen;

/// Runtime state for the devtools window.
struct WindowState {
    window: Arc<Window>,
    renderer: Renderer,
    text_ctx: TextContext,
    tree: Tree,
}

/// Browser-style inspector for wgpu-html.
///
/// Owns an optional secondary OS window. The window is created
/// when [`Devtools::enable`] is called with an `ActiveEventLoop`
/// and destroyed when [`Devtools::disable`] is called (or the
/// user closes the window).
///
/// The host's `AppHook` is responsible for:
/// 1. Forwarding secondary window events via [`Devtools::handle_window_event`].
/// 2. Calling [`Devtools::flush`] from [`AppHook::on_idle`] so
///    deferred window drops run outside any WndProc callback.
/// 3. Calling [`Devtools::update_inspected_tree`] from `on_frame`
///    to refresh the devtools view with the latest DOM state.
pub struct Devtools {
    enabled: bool,
    window_state: Option<WindowState>,
    /// Deferred drop: window state is moved here when the window
    /// should close, then actually dropped in [`flush`] which runs
    /// from `on_idle` / `about_to_wait` — outside any WndProc.
    pending_drop: Option<WindowState>,
    /// Font faces to register on the devtools UI tree. Collected
    /// via [`register_font`] before or after `enable()`.
    fonts: Vec<wgpu_html_tree::FontFace>,
}

impl Devtools {
    pub fn new() -> Self {
        Self {
            enabled: false,
            window_state: None,
            pending_drop: None,
            fonts: Vec::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Register a font face for the devtools UI. Can be called
    /// before `enable()` — the fonts are applied when the window
    /// is created.
    pub fn register_font(&mut self, face: wgpu_html_tree::FontFace) {
        self.fonts.push(face.clone());
        if let Some(state) = self.window_state.as_mut() {
            state.tree.register_font(face);
        }
    }

    /// Open the devtools window. Requires the active event loop
    /// so a new OS window can be created on the current thread.
    pub fn enable(&mut self, event_loop: &ActiveEventLoop) {
        if self.enabled {
            return;
        }
        self.enabled = true;

        let attrs = Window::default_attributes()
            .with_title("DevTools")
            .with_inner_size(PhysicalSize::new(800u32, 600));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("wgpu-html-devtools: failed to create window"),
        );
        let size = window.inner_size();
        let renderer =
            pollster::block_on(Renderer::new(window.clone(), size.width, size.height));
        let text_ctx = TextContext::new(GLYPH_ATLAS_SIZE);

        // Build an empty placeholder tree; the real content arrives
        // on the first `update_inspected_tree` call.
        let tree = self.new_empty_tree();

        window.request_redraw();
        self.window_state = Some(WindowState {
            window,
            renderer,
            text_ctx,
            tree,
        });
    }

    /// Rebuild the devtools UI from the current state of the
    /// inspected tree. Call this from `on_frame` whenever the
    /// devtools window is open.
    pub fn update_inspected_tree(&mut self, inspected: &Tree) {
        let Some(state) = self.window_state.as_mut() else {
            return;
        };
        let t0 = Instant::now();
        let mut tree = html_gen::build(inspected);
        let build_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let t1 = Instant::now();
        for face in &self.fonts {
            tree.register_font(face.clone());
        }
        let fonts_ms = t1.elapsed().as_secs_f64() * 1000.0;

        state.tree = tree;
        state.window.request_redraw();
        eprintln!(
            "devtools: update  build={build_ms:.2}ms  fonts={fonts_ms:.2}ms  total={:.2}ms",
            t0.elapsed().as_secs_f64() * 1000.0
        );
    }

    /// Schedule the devtools window for destruction. The actual
    /// drop happens on the next [`flush`] call.
    pub fn disable(&mut self) {
        if !self.enabled {
            return;
        }
        self.enabled = false;
        self.pending_drop = self.window_state.take();
    }

    /// Toggle devtools on/off.
    pub fn toggle(&mut self, event_loop: &ActiveEventLoop) {
        if self.enabled {
            self.disable();
        } else {
            self.enable(event_loop);
        }
    }

    /// The `WindowId` of the devtools window, if open.
    pub fn window_id(&self) -> Option<WindowId> {
        self.window_state.as_ref().map(|s| s.window.id())
    }

    /// Returns `true` if `id` matches the devtools window
    /// (including a window that is pending deferred drop).
    pub fn owns_window(&self, id: WindowId) -> bool {
        self.window_id() == Some(id)
            || self
                .pending_drop
                .as_ref()
                .is_some_and(|s| s.window.id() == id)
    }

    /// Handle a winit `WindowEvent` for the devtools window.
    /// The caller should only forward events whose `WindowId`
    /// matches [`Devtools::owns_window`].
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        // Swallow events for a window that is pending drop.
        if self.window_state.is_none() {
            return;
        }
        let Some(state) = self.window_state.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                // Defer the actual drop to flush() which runs in
                // about_to_wait, outside any WndProc.
                self.enabled = false;
                self.pending_drop = self.window_state.take();
            }
            WindowEvent::Resized(size) => {
                state.renderer.resize(size.width, size.height);
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                Self::render_frame(state);
            }
            _ => {}
        }
    }

    /// Actually drop any window state that was deferred from a
    /// `CloseRequested` or `disable()` call. **Must** be called
    /// from `AppHook::on_idle` (which runs in `about_to_wait`,
    /// outside any WndProc callback).
    pub fn flush(&mut self) {
        self.pending_drop = None;
    }

    /// Build a minimal tree with just the stylesheet so the window
    /// isn't completely blank before the first update.
    fn new_empty_tree(&self) -> Tree {
        let style_css = include_str!("../html/devtools.css");
        let style = wgpu_html_tree::Node::new(wgpu_html_models::StyleElement::default())
            .with_children(vec![wgpu_html_tree::Node::new(style_css)]);
        let body = wgpu_html_tree::Node::new(wgpu_html_models::Body::default())
            .with_children(vec![style]);
        let mut tree = Tree::new(body);
        for face in &self.fonts {
            tree.register_font(face.clone());
        }
        tree
    }

    /// Render the devtools UI into its window.
    fn render_frame(state: &mut WindowState) {
        let t0 = Instant::now();
        let size = state.window.inner_size();
        let (list, _layout) = paint_tree_returning_layout(
            &state.tree,
            &mut state.text_ctx,
            size.width as f32,
            size.height as f32,
            1.0,
        );
        let paint_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // Upload freshly-rasterised glyphs into the GPU atlas.
        let t1 = Instant::now();
        state
            .text_ctx
            .atlas
            .upload(&state.renderer.queue, state.renderer.glyph_atlas_texture());
        let upload_ms = t1.elapsed().as_secs_f64() * 1000.0;

        let t2 = Instant::now();
        match state.renderer.render(&list) {
            FrameOutcome::Presented | FrameOutcome::Skipped => {}
            FrameOutcome::Reconfigure => {
                state.renderer.resize(size.width, size.height);
            }
        }
        let render_ms = t2.elapsed().as_secs_f64() * 1000.0;

        eprintln!(
            "devtools: render  paint={paint_ms:.2}ms  upload={upload_ms:.2}ms  gpu={render_ms:.2}ms  total={:.2}ms",
            t0.elapsed().as_secs_f64() * 1000.0
        );
    }
}
