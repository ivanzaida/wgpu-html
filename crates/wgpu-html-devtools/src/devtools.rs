use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use wgpu_html::PipelineCache;
use wgpu_html_renderer::{FrameOutcome, GLYPH_ATLAS_SIZE, Renderer};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{MouseButton, Node, Tree};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::html_gen;

// ── Profiler ─────────────────────────────────────────────────────

#[derive(Default, Clone, Copy)]
struct Stage {
    sum_ms: f64,
    max_ms: f64,
}

impl Stage {
    fn add(&mut self, ms: f64) {
        self.sum_ms += ms;
        self.max_ms = self.max_ms.max(ms);
    }
    fn avg(&self, n: u64) -> f64 {
        if n == 0 { 0.0 } else { self.sum_ms / n as f64 }
    }
}

/// One-second rolling profile window for the devtools pipeline.
struct DevtoolsProfiler {
    started_at: Instant,
    frames: u64,
    build: Stage,
    cascade: Stage,
    layout: Stage,
    paint: Stage,
    upload: Stage,
    render: Stage,
    hover_moves: u64,
    hover_changed: u64,
    pointer_move: Stage,
}

impl DevtoolsProfiler {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            frames: 0,
            build: Stage::default(),
            cascade: Stage::default(),
            layout: Stage::default(),
            paint: Stage::default(),
            upload: Stage::default(),
            render: Stage::default(),
            hover_moves: 0,
            hover_changed: 0,
            pointer_move: Stage::default(),
        }
    }

    fn add_pointer_move(&mut self, ms: f64, changed: bool) {
        self.hover_moves += 1;
        if changed {
            self.hover_changed += 1;
        }
        self.pointer_move.add(ms);
    }

    fn take_summary_if_due(&mut self) -> Option<String> {
        if self.started_at.elapsed() < Duration::from_secs(1) {
            return None;
        }
        if self.frames == 0 && self.hover_moves == 0 {
            self.reset();
            return None;
        }
        let secs = self.started_at.elapsed().as_secs_f64().max(f64::EPSILON);
        let fps = self.frames as f64 / secs;
        let n = self.frames;
        let line = format!(
            "devtools: {secs:.2}s frames={n} fps={fps:.1}  \
             build={:.2}/{:.2}  cascade={:.2}/{:.2}  layout={:.2}/{:.2}  \
             paint={:.2}/{:.2}  upload={:.2}/{:.2}  render={:.2}/{:.2}  \
             hover[moves={} changed={} ptr={:.3}/{:.3}ms]",
            self.build.avg(n),
            self.build.max_ms,
            self.cascade.avg(n),
            self.cascade.max_ms,
            self.layout.avg(n),
            self.layout.max_ms,
            self.paint.avg(n),
            self.paint.max_ms,
            self.upload.avg(n),
            self.upload.max_ms,
            self.render.avg(n),
            self.render.max_ms,
            self.hover_moves,
            self.hover_changed,
            self.pointer_move.avg(self.hover_moves),
            self.pointer_move.max_ms,
        );
        self.reset();
        Some(line)
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Runtime state for the devtools window.
struct WindowState {
    window: Arc<Window>,
    renderer: Renderer,
    text_ctx: TextContext,
    image_cache: wgpu_html::layout::ImageCache,
    tree: Tree,
    /// Cached cascade + layout; skips expensive stages when only
    /// hover/scroll changed. Also holds the layout for hit-testing.
    cache: PipelineCache,
    /// Last known cursor position in viewport space.
    cursor_pos: (f32, f32),
    /// Active scrollbar thumb drag, if any.
    scrollbar_drag: Option<wgpu_html::scroll::ElementScrollbarDrag>,
    profiler: DevtoolsProfiler,
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
    /// Path into the inspected tree of the currently selected node.
    selected_path: Option<Vec<usize>>,
    /// Shared sink for click callbacks to communicate selected paths
    /// back from the devtools UI tree.
    click_sink: Arc<Mutex<Option<Vec<usize>>>>,
    /// Cloned root of the inspected tree — kept so the devtools can
    /// rebuild its UI on selection changes without waiting for the
    /// host to call `update_inspected_tree` again.
    inspected_root: Option<Node>,
    /// Generation of the inspected tree used in the last rebuild.
    last_inspected_gen: Option<u64>,
}

impl Devtools {
    pub fn new() -> Self {
        Self {
            enabled: false,
            window_state: None,
            pending_drop: None,
            fonts: Vec::new(),
            selected_path: None,
            click_sink: Arc::new(Mutex::new(None)),
            inspected_root: None,
            last_inspected_gen: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Borrow the devtools UI tree, if the window is open. Useful
    /// for feeding it into a second-level devtools instance.
    pub fn tree(&self) -> Option<&Tree> {
        self.window_state.as_ref().map(|s| &s.tree)
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
        let renderer = pollster::block_on(Renderer::new(window.clone(), size.width, size.height));
        let text_ctx = TextContext::new(GLYPH_ATLAS_SIZE);

        // Build an empty placeholder tree; the real content arrives
        // on the first `update_inspected_tree` call.
        let tree = self.new_empty_tree();

        window.request_redraw();
        let mut cache = PipelineCache::new();
        // The devtools CSS only uses :hover for background-color
        // (paint-only), so partial-cascade never needs re-layout.
        cache.paint_only_pseudo_rules = true;
        self.window_state = Some(WindowState {
            window,
            renderer,
            text_ctx,
            image_cache: wgpu_html::layout::ImageCache::new(),
            tree,
            cache,
            cursor_pos: (0.0, 0.0),
            scrollbar_drag: None,
            profiler: DevtoolsProfiler::new(),
        });
    }

    /// Rebuild the devtools UI from the current state of the
    /// inspected tree. Call this from `on_frame` whenever the
    /// devtools window is open.
    pub fn update_inspected_tree(&mut self, inspected: &Tree) {
        if self.window_state.is_none() {
            return;
        }

        // Store the inspected root so click-selection can rebuild
        // without waiting for the next host on_frame call.
        let inspected_gen = inspected.generation;
        let dom_changed = self.last_inspected_gen != Some(inspected_gen);
        if dom_changed {
            self.inspected_root = inspected.root.clone();
            self.last_inspected_gen = Some(inspected_gen);
        }

        // Drain any pending click selection.
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

    /// Rebuild the devtools UI tree from the stored inspected root
    /// and current selection, then invalidate the pipeline cache.
    fn rebuild_ui(&mut self) {
        let Some(state) = self.window_state.as_mut() else {
            return;
        };

        let t0 = Instant::now();
        let mut tree = html_gen::build(
            self.inspected_root.as_ref(),
            self.selected_path.as_deref(),
            &self.click_sink,
        );
        for face in &self.fonts {
            tree.register_font(face.clone());
        }
        let build_ms = t0.elapsed().as_secs_f64() * 1000.0;
        state.profiler.build.add(build_ms);

        // Preserve interaction state (hover, scroll offsets, etc.)
        // across tree rebuilds so scrolling and hover don't reset.
        std::mem::swap(&mut tree.interaction, &mut state.tree.interaction);
        state.tree = tree;
        // Force a full cascade + layout on the next render.
        state.cache.invalidate();
        state.window.request_redraw();
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
        match event {
            WindowEvent::CloseRequested => {
                // Defer the actual drop to flush() which runs in
                // about_to_wait, outside any WndProc.
                self.enabled = false;
                self.pending_drop = self.window_state.take();
            }
            WindowEvent::Resized(size) => {
                let state = self.window_state.as_mut().unwrap();
                state.renderer.resize(size.width, size.height);
                state.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let state = self.window_state.as_mut().unwrap();
                Self::render_frame(state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let state = self.window_state.as_mut().unwrap();
                state.cursor_pos = (position.x as f32, position.y as f32);
                if let Some(layout) = state.cache.layout() {
                    // Scrollbar drag in progress — update scroll
                    // and skip normal pointer dispatch.
                    if let Some(drag) = &state.scrollbar_drag {
                        drag.update(layout, &mut state.tree, state.cursor_pos.1);
                        state.window.request_redraw();
                        return;
                    }

                    let t0 = Instant::now();
                    let changed = wgpu_html::interactivity::pointer_move(
                        &mut state.tree,
                        layout,
                        state.cursor_pos,
                    );
                    let ms = t0.elapsed().as_secs_f64() * 1000.0;
                    state.profiler.add_pointer_move(ms, changed);
                    if changed {
                        state.window.request_redraw();
                    }
                }
            }
            WindowEvent::CursorLeft { .. } => {
                let state = self.window_state.as_mut().unwrap();
                state.tree.pointer_leave();
                state.window.request_redraw();
            }
            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                let Some(mb) = to_mouse_button(*button) else {
                    return;
                };
                let ws = self.window_state.as_mut().unwrap();

                // ── Mouse up: end scrollbar drag ────────────────
                if *button_state == ElementState::Released
                    && mb == MouseButton::Primary
                    && ws.scrollbar_drag.take().is_some()
                {
                    ws.window.request_redraw();
                    return;
                }

                // ── Mouse down: start scrollbar drag? ───────────
                if *button_state == ElementState::Pressed && mb == MouseButton::Primary {
                    // Two-phase borrow: hit-test immutably, then
                    // start the drag mutably.
                    let drag = ws.cache.layout().and_then(|layout| {
                        wgpu_html::scroll::ElementScrollbarDrag::try_start(
                            layout,
                            ws.cursor_pos,
                            &mut ws.tree,
                        )
                    });
                    if let Some(d) = drag {
                        ws.scrollbar_drag = Some(d);
                        ws.window.request_redraw();
                        return;
                    }
                }

                // ── Normal DOM dispatch ─────────────────────────
                if let Some(layout) = ws.cache.layout() {
                    match button_state {
                        ElementState::Pressed => {
                            wgpu_html::interactivity::mouse_down(
                                &mut ws.tree,
                                layout,
                                ws.cursor_pos,
                                mb,
                            );
                        }
                        ElementState::Released => {
                            wgpu_html::interactivity::mouse_up(
                                &mut ws.tree,
                                layout,
                                ws.cursor_pos,
                                mb,
                            );
                        }
                    }
                    ws.window.request_redraw();
                }
                // If a tree row was clicked (callback fires
                // synchronously during dispatch_mouse_up), apply
                // the selection and rebuild immediately from the
                // stored inspected root.
                let clicked = self.click_sink.lock().unwrap().take();
                if let Some(path) = clicked {
                    self.selected_path = Some(path);
                    self.rebuild_ui();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -*y * 40.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                let ws = self.window_state.as_mut().unwrap();
                if let Some(layout) = ws.cache.layout() {
                    if wgpu_html::scroll::scroll_element_at(&mut ws.tree, layout, ws.cursor_pos, dy)
                    {
                        // Re-dispatch hover after scroll so the
                        // cursor tracks the content that moved.
                        wgpu_html::interactivity::pointer_move(&mut ws.tree, layout, ws.cursor_pos);
                        ws.window.request_redraw();
                    }
                }
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

    /// Build the devtools shell with no inspected content.
    fn new_empty_tree(&self) -> Tree {
        let mut tree = html_gen::build(None, None, &self.click_sink);
        for face in &self.fonts {
            tree.register_font(face.clone());
        }
        tree
    }

    /// Render the devtools UI into its window.
    fn render_frame(state: &mut WindowState) {
        let size = state.window.inner_size();
        let scale = state
            .tree
            .effective_dpi_scale(state.window.scale_factor() as f32);
        let (list, _layout, timings) = wgpu_html::paint_tree_cached(
            &state.tree,
            &mut state.text_ctx,
            &mut state.image_cache,
            size.width as f32,
            size.height as f32,
            scale,
            &mut state.cache,
        );
        // Layout is retained inside `state.cache` for hit-testing
        // between frames (accessed via `state.cache.layout()`).

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

        // Accumulate into the rolling profiler.
        state.profiler.frames += 1;
        state.profiler.cascade.add(timings.cascade_ms);
        state.profiler.layout.add(timings.layout_ms);
        state.profiler.paint.add(timings.paint_ms);
        state.profiler.upload.add(upload_ms);
        state.profiler.render.add(render_ms);

        if let Some(line) = state.profiler.take_summary_if_due() {
            eprintln!("{line}");
        }
    }
}

// ── Winit button conversion ──────────────────────────────────────

fn to_mouse_button(b: winit::event::MouseButton) -> Option<MouseButton> {
    match b {
        winit::event::MouseButton::Left => Some(MouseButton::Primary),
        winit::event::MouseButton::Right => Some(MouseButton::Secondary),
        winit::event::MouseButton::Middle => Some(MouseButton::Middle),
        _ => None,
    }
}
