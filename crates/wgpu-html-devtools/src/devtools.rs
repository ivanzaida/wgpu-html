//! Platform-agnostic devtools inspector.
//!
//! `Devtools` manages a UI tree that visualises an inspected tree's
//! DOM structure, styles, and breadcrumb. It accepts abstract input
//! (pointer position, clicks, scroll) and produces a `DisplayList`
//! for the host to render — no winit, no GPU, no window management.
//!
//! Hosts (winit harness, Bevy plugin, egui panel, …) create a
//! rendering surface, forward input events, and call [`Devtools::paint`]
//! each frame.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use wgpu_html::{PipelineCache, PipelineTimings};
use wgpu_html_renderer::DisplayList;
use wgpu_html_text::TextContext;
use wgpu_html_tree::{MouseButton, Node, Tree, TreeHook, TreeHookResponse, TreeRenderEvent};
use wgpu_html_winit::HtmlWindow;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::html_gen;

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── TreeHook for auto-snapshot ────────────────────────────────────

struct SharedSnapshot {
    root: Option<Node>,
    generation: u64,
    dirty: bool,
}

struct DevtoolsHook {
    shared: Arc<Mutex<SharedSnapshot>>,
}

impl TreeHook for DevtoolsHook {
    fn on_render(
        &mut self,
        tree: &mut Tree,
        _event: &TreeRenderEvent<'_>,
    ) -> TreeHookResponse {
        let mut shared = self.shared.lock().unwrap();
        if tree.generation != shared.generation {
            shared.root = tree.root.clone();
            shared.generation = tree.generation;
            shared.dirty = true;
        }
        TreeHookResponse::Continue
    }
}

// ── Devtools ─────────────────────────────────────────────────────

/// Platform-agnostic devtools inspector.
///
/// Owns its own UI [`Tree`], [`TextContext`], and [`PipelineCache`].
/// The host is responsible for:
/// 1. Calling [`poll`] once per frame to pick up inspected-tree changes.
/// 2. Forwarding input via [`pointer_move`], [`mouse_down`],
///    [`mouse_up`], [`scroll`], [`pointer_leave`].
/// 3. Calling [`paint`] to obtain a [`DisplayList`] for rendering.
/// 4. Uploading the glyph atlas via [`text_ctx`] and rendering
///    the display list with the host's GPU backend.
pub struct Devtools {
    // ── UI state ────────────────────────────────────────────
    tree: Tree,
    text_ctx: TextContext,
    image_cache: wgpu_html::layout::ImageCache,
    cache: PipelineCache,

    // ── Inspected-tree tracking ─────────────────────────────
    fonts: Vec<wgpu_html_tree::FontFace>,
    selected_path: Option<Vec<usize>>,
    click_sink: Arc<Mutex<Option<Vec<usize>>>>,
    inspected_root: Option<Node>,
    last_inspected_gen: Option<u64>,
    /// Shared state with the TreeHook installed by `attach()`.
    snapshot: Option<Arc<Mutex<SharedSnapshot>>>,

    // ── Interaction ─────────────────────────────────────────
    cursor_pos: (f32, f32),
    scrollbar_drag: Option<wgpu_html::scroll::ElementScrollbarDrag>,

    // ── Viewport ────────────────────────────────────────────
    viewport_w: f32,
    viewport_h: f32,
    scale: f32,

    /// Set to `true` whenever the UI needs a repaint.
    needs_redraw: bool,

    // ── Window (managed by devtools itself) ──────────────────
    html_window: Option<HtmlWindow>,
    /// Deferred window drop (must happen outside WndProc).
    pending_drop: Option<HtmlWindow>,
    enabled: bool,
}

impl Devtools {
    /// Create a devtools instance without attaching to a tree.
    /// The host must call [`update_inspected_tree`] manually.
    pub fn new() -> Self {
        let click_sink = Arc::new(Mutex::new(None));
        let mut tree = html_gen::build(None, None, &click_sink);
        // Register the Lucide icon font so the devtools UI renders
        // icons regardless of what fonts the host has registered.
        let lucide = wgpu_html_tree::FontFace::regular(
            "lucide",
            Arc::from(LUCIDE_FONT),
        );
        tree.register_font(lucide.clone());
        let mut cache = PipelineCache::new();
        cache.paint_only_pseudo_rules = true;
        Self {
            tree,
            text_ctx: TextContext::new(wgpu_html_renderer::GLYPH_ATLAS_SIZE),
            image_cache: wgpu_html::layout::ImageCache::new(),
            cache,
            fonts: vec![lucide],
            selected_path: None,
            click_sink,
            inspected_root: None,
            last_inspected_gen: None,
            snapshot: None,
            cursor_pos: (0.0, 0.0),
            scrollbar_drag: None,
            viewport_w: 800.0,
            viewport_h: 600.0,
            scale: 1.0,
            needs_redraw: true,
            html_window: None,
            pending_drop: None,
            enabled: false,
        }
    }

    /// Attach to `tree` by installing a [`TreeHook`] that
    /// auto-snapshots the tree's root on every render. The
    /// devtools will pick up changes via [`poll`] — no manual
    /// [`update_inspected_tree`] calls needed.
    ///
    /// Copies the tree's registered fonts into the devtools.
    pub fn attach(tree: &mut Tree) -> Self {
        let shared = Arc::new(Mutex::new(SharedSnapshot {
            root: tree.root.clone(),
            generation: tree.generation,
            dirty: true,
        }));
        tree.add_hook(DevtoolsHook {
            shared: shared.clone(),
        });

        let mut devtools = Self::new();
        devtools.inspected_root = tree.root.clone();
        devtools.last_inspected_gen = Some(tree.generation);
        devtools.snapshot = Some(shared);
        // Copy the tree's fonts so the devtools UI can render text.
        for (_handle, face) in tree.fonts.iter() {
            devtools.register_font(face.clone());
        }
        devtools
    }

    // ── Font registration ───────────────────────────────────

    /// Register a font face for the devtools UI.
    pub fn register_font(&mut self, face: wgpu_html_tree::FontFace) {
        self.fonts.push(face.clone());
        self.tree.register_font(face);
    }

    // ── Tree access ─────────────────────────────────────────

    /// Borrow the devtools UI tree. Useful for feeding it into a
    /// second-level devtools or for host-side inspection.
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// Mutable borrow of the text context. The host needs this
    /// to upload the glyph atlas to the GPU after [`paint`].
    pub fn text_ctx(&mut self) -> &mut TextContext {
        &mut self.text_ctx
    }

    /// Current cursor position in viewport space.
    pub fn cursor_pos(&self) -> (f32, f32) {
        self.cursor_pos
    }

    /// Whether the devtools needs a repaint. Reset by [`paint`].
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    // ── Viewport ────────────────────────────────────────────

    /// Set the viewport size (physical pixels) and DPI scale.
    pub fn resize(&mut self, width: f32, height: f32, scale: f32) {
        self.viewport_w = width;
        self.viewport_h = height;
        self.scale = scale;
        self.needs_redraw = true;
    }

    // ── Polling ─────────────────────────────────────────────

    /// Check for tree changes (from the auto-snapshot hook) and
    /// pending click selections. Rebuilds the UI if needed.
    pub fn poll(&mut self) {
        let mut dom_changed = false;
        if let Some(shared) = &self.snapshot {
            let mut snap = shared.lock().unwrap();
            if snap.dirty {
                snap.dirty = false;
                self.inspected_root = snap.root.take();
                self.last_inspected_gen = Some(snap.generation);
                dom_changed = true;
            }
        }

        let selection_changed = self
            .click_sink
            .lock()
            .unwrap()
            .take()
            .map(|path| {
                self.selected_path = Some(path);
            })
            .is_some();

        if dom_changed || selection_changed {
            self.rebuild_ui();
        }
    }

    /// Feed the inspected tree directly (manual mode, no hook).
    pub fn update_inspected_tree(&mut self, inspected: &Tree) {
        let inspected_gen = inspected.generation;
        let dom_changed = self.last_inspected_gen != Some(inspected_gen);
        if dom_changed {
            self.inspected_root = inspected.root.clone();
            self.last_inspected_gen = Some(inspected_gen);
        }

        let selection_changed = self
            .click_sink
            .lock()
            .unwrap()
            .take()
            .map(|path| {
                self.selected_path = Some(path);
            })
            .is_some();

        if dom_changed || selection_changed {
            self.rebuild_ui();
        }
    }

    // ── Input ───────────────────────────────────────────────

    /// Update the cursor position. Returns `true` if the hover
    /// path changed (the host should repaint).
    pub fn pointer_move(&mut self, x: f32, y: f32) -> bool {
        self.cursor_pos = (x, y);
        let Some(layout) = self.cache.layout() else {
            return false;
        };

        // Scrollbar drag in progress.
        if let Some(drag) = &self.scrollbar_drag {
            drag.update(layout, &mut self.tree, y);
            self.needs_redraw = true;
            return true;
        }

        let changed =
            wgpu_html::interactivity::pointer_move(&mut self.tree, layout, self.cursor_pos);
        if changed {
            self.needs_redraw = true;
        }
        changed
    }

    /// Notify that the cursor left the surface.
    pub fn pointer_leave(&mut self) {
        self.tree.pointer_leave();
        self.needs_redraw = true;
    }

    /// Primary or secondary button press. Returns `true` if
    /// the event was consumed (scrollbar drag started or DOM
    /// click dispatched).
    pub fn mouse_down(&mut self, x: f32, y: f32, button: MouseButton) -> bool {
        self.cursor_pos = (x, y);

        // Scrollbar drag start?
        if button == MouseButton::Primary {
            let drag = self.cache.layout().and_then(|layout| {
                wgpu_html::scroll::ElementScrollbarDrag::try_start(
                    layout,
                    self.cursor_pos,
                    &mut self.tree,
                )
            });
            if let Some(d) = drag {
                self.scrollbar_drag = Some(d);
                self.needs_redraw = true;
                return true;
            }
        }

        let Some(layout) = self.cache.layout() else {
            return false;
        };
        wgpu_html::interactivity::mouse_down(&mut self.tree, layout, self.cursor_pos, button);
        self.needs_redraw = true;
        true
    }

    /// Button release. Returns `true` if the event was consumed.
    pub fn mouse_up(&mut self, x: f32, y: f32, button: MouseButton) -> bool {
        self.cursor_pos = (x, y);

        // End scrollbar drag.
        if button == MouseButton::Primary && self.scrollbar_drag.take().is_some() {
            self.needs_redraw = true;
            return true;
        }

        let Some(layout) = self.cache.layout() else {
            return false;
        };
        wgpu_html::interactivity::mouse_up(&mut self.tree, layout, self.cursor_pos, button);
        self.needs_redraw = true;

        // Check if a tree row was clicked.
        let clicked = self.click_sink.lock().unwrap().take();
        if let Some(path) = clicked {
            self.selected_path = Some(path);
            self.rebuild_ui();
        }
        true
    }

    /// Mouse wheel or trackpad scroll. `delta_y` is positive for
    /// scroll-down. Returns `true` if any element scrolled.
    pub fn scroll(&mut self, x: f32, y: f32, delta_y: f32) -> bool {
        self.cursor_pos = (x, y);
        let Some(layout) = self.cache.layout() else {
            return false;
        };
        if wgpu_html::scroll::scroll_element_at(&mut self.tree, layout, self.cursor_pos, delta_y) {
            wgpu_html::interactivity::pointer_move(&mut self.tree, layout, self.cursor_pos);
            self.needs_redraw = true;
            true
        } else {
            false
        }
    }

    // ── Painting ────────────────────────────────────────────

    /// Run cascade + layout + paint and return the display list.
    /// The host should then:
    /// 1. Upload glyphs via `self.text_ctx().atlas.upload(…)`
    /// 2. Render the display list with its GPU backend.
    pub fn paint(&mut self) -> (DisplayList, PipelineTimings) {
        self.needs_redraw = false;
        let (list, _layout, timings) = wgpu_html::paint_tree_cached(
            &self.tree,
            &mut self.text_ctx,
            &mut self.image_cache,
            self.viewport_w,
            self.viewport_h,
            self.scale,
            &mut self.cache,
        );
        (list, timings)
    }

    // ── Internal ────────────────────────────────────────────

    fn rebuild_ui(&mut self) {
        let t0 = Instant::now();
        let mut tree = html_gen::build(
            self.inspected_root.as_ref(),
            self.selected_path.as_deref(),
            &self.click_sink,
        );
        for face in &self.fonts {
            tree.register_font(face.clone());
        }
        let _build_ms = t0.elapsed().as_secs_f64() * 1000.0;

        std::mem::swap(&mut tree.interaction, &mut self.tree.interaction);
        self.tree = tree;
        self.cache.invalidate();
        self.needs_redraw = true;
    }

    // ── Window lifecycle ────────────────────────────────────

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Open the devtools window.
    pub fn enable(&mut self, event_loop: &ActiveEventLoop) {
        if self.enabled {
            return;
        }
        self.enabled = true;
        let hw = HtmlWindow::new(event_loop, "DevTools", 1280, 720);
        let (w, h) = hw.inner_size();
        let scale = hw.scale_factor();
        self.resize(w as f32, h as f32, scale);
        hw.request_redraw();
        self.html_window = Some(hw);
    }

    /// Close the devtools window. Actual drop is deferred to
    /// [`flush`] (must run outside WndProc).
    pub fn disable(&mut self) {
        if !self.enabled {
            return;
        }
        self.enabled = false;
        self.pending_drop = self.html_window.take();
    }

    /// Toggle devtools on/off.
    pub fn toggle(&mut self, event_loop: &ActiveEventLoop) {
        if self.enabled {
            self.disable();
        } else {
            self.enable(event_loop);
        }
    }

    /// Drop the window state deferred from [`disable`] or close.
    /// Must be called from `about_to_wait` / `on_idle`.
    pub fn flush(&mut self) {
        self.pending_drop = None;
    }

    /// The `WindowId` of the devtools window, if open.
    pub fn window_id(&self) -> Option<WindowId> {
        self.html_window.as_ref().map(|hw| hw.window_id())
    }

    /// Whether `id` matches the devtools window (including one
    /// that is pending deferred drop).
    pub fn owns_window(&self, id: WindowId) -> bool {
        self.window_id() == Some(id)
            || self
                .pending_drop
                .as_ref()
                .is_some_and(|hw| hw.window_id() == id)
    }

    /// Handle a winit `WindowEvent` for the devtools window.
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        if self.html_window.is_none() {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.enabled = false;
                self.pending_drop = self.html_window.take();
            }
            WindowEvent::Resized(size) => {
                let scale = self
                    .html_window
                    .as_ref()
                    .map(|hw| hw.scale_factor())
                    .unwrap_or(1.0);
                if let Some(hw) = self.html_window.as_mut() {
                    hw.resize(size.width, size.height);
                }
                self.resize(size.width as f32, size.height as f32, scale);
            }
            WindowEvent::RedrawRequested => {
                self.render_to_window();
                return; // paint clears needs_redraw, skip the check below
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_move(position.x as f32, position.y as f32);
            }
            WindowEvent::CursorLeft { .. } => {
                self.pointer_leave();
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                let mb = wgpu_html_winit::mouse_button(*button);
                let pos = self.cursor_pos;
                match button_state {
                    ElementState::Pressed => {
                        self.mouse_down(pos.0, pos.1, mb);
                    }
                    ElementState::Released => {
                        self.mouse_up(pos.0, pos.1, mb);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -*y * 40.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                let pos = self.cursor_pos;
                self.scroll(pos.0, pos.1, dy);
            }
            _ => {}
        }
        // Single redraw check for all event types.
        if self.needs_redraw {
            if let Some(hw) = &self.html_window {
                hw.request_redraw();
            }
        }
    }

    /// Poll for changes and request a redraw if needed.
    pub fn poll_and_redraw(&mut self) {
        self.poll();
        if self.needs_redraw {
            if let Some(hw) = &self.html_window {
                hw.request_redraw();
            }
        }
    }

    fn render_to_window(&mut self) {
        let (list, _timings) = self.paint();
        if let Some(hw) = self.html_window.as_mut() {
            hw.render(&list, &mut self.text_ctx);
        }
    }
}
