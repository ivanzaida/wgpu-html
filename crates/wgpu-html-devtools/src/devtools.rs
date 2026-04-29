use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use wgpu_html::layout::{LayoutBox, Rect};
use wgpu_html::PipelineCache;
use wgpu_html_renderer::{FrameOutcome, GLYPH_ATLAS_SIZE, Renderer};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{MouseButton, Tree};
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
            image_cache: wgpu_html::layout::ImageCache::new(),
            tree,
            cache: PipelineCache::new(),
            cursor_pos: (0.0, 0.0),
            profiler: DevtoolsProfiler::new(),
        });
    }

    /// Rebuild the devtools UI from the current state of the
    /// inspected tree. Call this from `on_frame` whenever the
    /// devtools window is open.
    pub fn update_inspected_tree(&mut self, inspected: &Tree) {
        let Some(state) = self.window_state.as_mut() else {
            return;
        };

        // Check for pending selection from a click event.
        if let Some(path) = self.click_sink.lock().unwrap().take() {
            self.selected_path = Some(path);
        }

        let t0 = Instant::now();
        let mut tree = html_gen::build(
            inspected,
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
                    let t0 = Instant::now();
                    let target = hit_path_scrolled(
                        layout,
                        state.cursor_pos,
                        &state.tree.interaction.scroll_offsets_y,
                    );
                    let changed = state.tree.dispatch_pointer_move(
                        target.as_deref(),
                        state.cursor_pos,
                        None,
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
                if let Some(layout) = ws.cache.layout() {
                    let target = hit_path_scrolled(
                        layout,
                        ws.cursor_pos,
                        &ws.tree.interaction.scroll_offsets_y,
                    );
                    match button_state {
                        ElementState::Pressed => {
                            ws.tree.dispatch_mouse_down(
                                target.as_deref(),
                                ws.cursor_pos,
                                mb,
                                None,
                            );
                        }
                        ElementState::Released => {
                            ws.tree.dispatch_mouse_up(
                                target.as_deref(),
                                ws.cursor_pos,
                                mb,
                                None,
                            );
                        }
                    }
                    ws.window.request_redraw();
                }
                // Check if a tree row was clicked (callback fires
                // synchronously during dispatch_mouse_up).
                if let Some(path) = self.click_sink.lock().unwrap().take() {
                    self.selected_path = Some(path);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -*y * 40.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                let ws = self.window_state.as_mut().unwrap();
                if let Some(layout) = ws.cache.layout() {
                    if wgpu_html::scroll::scroll_element_at(
                        &mut ws.tree,
                        layout,
                        ws.cursor_pos,
                        dy,
                    ) {
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
        let size = state.window.inner_size();
        let (list, _layout, timings) = wgpu_html::paint_tree_cached(
            &state.tree,
            &mut state.text_ctx,
            &mut state.image_cache,
            size.width as f32,
            size.height as f32,
            1.0,
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

// ── Scroll-aware hit-testing ─────────────────────────────────────
//
// The standard `LayoutBox::hit_path` does not compensate for
// per-element scroll offsets stored in `InteractionState`. These
// helpers adjust the test point as they descend through scrollable
// containers so that rows scrolled into view are properly hit.

fn hit_path_scrolled(
    root: &LayoutBox,
    point: (f32, f32),
    scroll_offsets: &BTreeMap<Vec<usize>, f32>,
) -> Option<Vec<usize>> {
    let mut path = Vec::new();
    collect_hit_scrolled(root, point.0, point.1, scroll_offsets, &mut path, None)
}

fn collect_hit_scrolled(
    b: &LayoutBox,
    x: f32,
    y: f32,
    scroll_offsets: &BTreeMap<Vec<usize>, f32>,
    path: &mut Vec<usize>,
    clip: Option<Rect>,
) -> Option<Vec<usize>> {
    // Respect parent clip.
    if let Some(c) = clip {
        if !c.contains(x, y) {
            return None;
        }
    }

    // Compute the clip region this element imposes on its children.
    let next_clip = if b.overflow.clips_any() {
        let pad = padding_box(b);
        Some(match clip {
            Some(c) => intersect_rects(c, pad),
            None => pad,
        })
    } else {
        clip
    };

    // Compensate for this element's scroll offset so children are
    // tested at their true layout positions.
    let own_scroll = scroll_offsets
        .get(path.as_slice())
        .copied()
        .unwrap_or(0.0);
    let child_y = y + own_scroll;

    // Walk children last-to-first (topmost painted wins).
    for (i, child) in b.children.iter().enumerate().rev() {
        path.push(i);
        if let Some(result) =
            collect_hit_scrolled(child, x, child_y, scroll_offsets, path, next_clip)
        {
            path.pop();
            return Some(result);
        }
        path.pop();
    }

    // Self.
    if b.border_rect.contains(x, y) {
        Some(path.clone())
    } else {
        None
    }
}

fn padding_box(b: &LayoutBox) -> Rect {
    Rect::new(
        b.border_rect.x + b.border.left,
        b.border_rect.y + b.border.top,
        (b.border_rect.w - b.border.horizontal()).max(0.0),
        (b.border_rect.h - b.border.vertical()).max(0.0),
    )
}

fn intersect_rects(a: Rect, b: Rect) -> Rect {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.w).min(b.x + b.w);
    let y2 = (a.y + a.h).min(b.y + b.h);
    Rect::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}
