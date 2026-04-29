//! Batteries-included winit harness.
//!
//! See [`WgpuHtmlWindow`] for the public surface and
//! [`create_window`] for the entry point. The harness handles:
//!
//! - Window + renderer setup, resize, redraw lifecycle
//! - Mouse + keyboard forwarding into [`Tree`] dispatch
//! - Focus + Tab navigation (via tree dispatchers)
//! - Viewport scroll (mouse wheel, scrollbar drag)
//! - Per-element scroll (`overflow:scroll` containers)
//! - Clipboard (Ctrl+A select-all, Ctrl+C copy) via `arboard`
//! - Screenshot (default F12) via `Renderer::capture_next_frame_to`
//!
//! Apps customise behaviour by implementing [`AppHook`] and
//! attaching it via [`WgpuHtmlWindow::with_hook`].

use std::sync::Arc;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use wgpu_html::interactivity;
use wgpu_html::layout::LayoutBox;
use wgpu_html::renderer::{FrameOutcome, GLYPH_ATLAS_SIZE, Renderer};
use wgpu_html::scroll::{
    clamp_scroll_y, deepest_element_scrollbar_at, paint_viewport_scrollbar, rect_contains,
    scroll_element_at, scroll_element_thumb_to, scroll_y_from_thumb_top, scrollbar_geometry,
    translate_display_list_y, viewport_to_document,
};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{Tree, TreeHook, TreeLifecycleStage, TreeRenderEvent, TreeRenderViewport};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::error::EventLoopError;
use winit::event::{
    ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

// ── Hook trait ──────────────────────────────────────────────────────────────

/// Extension hook invoked at well-known points in the harness's
/// event loop. Implementors override the methods they care about;
/// the default impls are no-ops.
///
/// To install a hook, call [`WgpuHtmlWindow::with_hook`]. Hooks
/// take precedence over the harness's built-in shortcuts when
/// [`AppHook::on_key`] returns [`EventResponse::Stop`].
pub trait AppHook {
    /// Called before the harness's default keyboard handling
    /// (Esc-to-exit, screenshot key, Ctrl+A, Ctrl+C). Return
    /// [`EventResponse::Stop`] to skip those defaults.
    fn on_key(&mut self, ctx: HookContext<'_>, event: &KeyEvent) -> EventResponse {
        let _ = (ctx, event);
        EventResponse::Continue
    }

    /// Called once per rendered frame, after the GPU submission
    /// has been issued. `timings` is the per-stage breakdown.
    fn on_frame(&mut self, ctx: HookContext<'_>, timings: &FrameTimings) {
        let _ = (ctx, timings);
    }

    /// Called after each pointer-move dispatch. `pointer_move_ms`
    /// is the wall-clock elapsed time of the dispatch itself
    /// (useful for hover-driven profiling); `changed` is `true`
    /// iff the hover path changed.
    fn on_pointer_move(&mut self, ctx: HookContext<'_>, pointer_move_ms: f64, changed: bool) {
        let _ = (ctx, pointer_move_ms, changed);
    }

    /// Called once per event-loop iteration after all pending
    /// window events have been dispatched, before the loop waits
    /// for new events. This runs **outside** any window's WndProc
    /// callback, making it the safe place to drop secondary
    /// windows or other resources that can't be released from
    /// inside a window event handler.
    fn on_idle(&mut self) {}

    /// Called for window events that target a secondary window
    /// (i.e. not the main harness window). Return `true` if the
    /// event was handled, `false` to ignore it.
    ///
    /// This enables hooks to manage additional OS windows (e.g. a
    /// devtools panel) while sharing the same event loop.
    fn on_window_event(
        &mut self,
        ctx: HookContext<'_>,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> bool {
        let _ = (ctx, window_id, event);
        false
    }
}

/// Borrows handed to [`AppHook`] callbacks.
pub struct HookContext<'a> {
    pub tree: &'a mut Tree,
    pub renderer: &'a mut Renderer,
    /// Text shaping / atlas context. Mutable because hooks that
    /// re-run layout (e.g. capturing a node screenshot via
    /// [`wgpu_html::screenshot_node_to`]) need to feed it through
    /// the cascade pipeline.
    pub text_ctx: &'a mut TextContext,
    /// The most recent layout box, populated after at least one
    /// frame has been rendered. `None` before the first redraw or
    /// when the document collapsed to nothing during cascade.
    pub last_layout: Option<&'a LayoutBox>,
    /// Reference to the Arc-wrapped window. Hooks that need a
    /// `Send` handle (e.g. to spawn a stdin reader thread that
    /// wakes the event loop via `request_redraw`) clone the Arc;
    /// hooks that only call window methods can keep using
    /// `ctx.window.foo()` thanks to `Arc<Window>`'s `Deref` to
    /// `Window`.
    pub window: &'a Arc<Window>,
    pub event_loop: &'a ActiveEventLoop,
}

/// Returned from [`AppHook::on_key`] to either let the harness
/// run its default behaviour or to short-circuit it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResponse {
    Continue,
    Stop,
}

/// Per-frame timing breakdown. Zeros if profiling info isn't
/// available (the harness always populates these from
/// [`wgpu_html::paint_tree_returning_layout_profiled`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameTimings {
    pub cascade_ms: f64,
    pub layout_ms: f64,
    pub paint_ms: f64,
    pub render_ms: f64,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Convenience constructor: equivalent to [`WgpuHtmlWindow::new`].
///
/// ```ignore
/// let mut tree = build_tree();
/// wgpu_html_winit::create_window(&mut tree)
///     .with_title("My App")
///     .run()
///     .unwrap();
/// ```
pub fn create_window(tree: &mut Tree) -> WgpuHtmlWindow<'_> {
    WgpuHtmlWindow::new(tree)
}

/// A self-contained winit harness wrapped around a borrowed
/// [`Tree`].
pub struct WgpuHtmlWindow<'tree> {
    tree: &'tree mut Tree,
    title: String,
    initial_size: (u32, u32),
    exit_on_escape: bool,
    enable_clipboard: bool,
    screenshot_key: Option<KeyCode>,
    hook: Option<Box<dyn AppHook>>,
    state: Option<RuntimeState>,
}

struct RuntimeState {
    window: Arc<Window>,
    renderer: Renderer,
    text_ctx: TextContext,
    last_layout: Option<LayoutBox>,
    cursor_pos: Option<(f32, f32)>,
    scroll_y: f32,
    scrollbar_drag: Option<ScrollbarDrag>,
    /// Lazy clipboard handle. `arboard` connects on first use.
    clipboard: Option<Clipboard>,
    started_at: Instant,
    last_render_at: Option<Instant>,
    frame_index: u64,
}

#[derive(Debug, Clone)]
struct ScrollbarDrag {
    target: ScrollTarget,
    grab_offset_y: f32,
}

#[derive(Debug, Clone)]
enum ScrollTarget {
    Viewport,
    Element(Vec<usize>),
}

impl<'tree> WgpuHtmlWindow<'tree> {
    /// Build a new harness around `tree`. The borrow lasts for
    /// the duration of [`Self::run`]; after `run` returns, the
    /// tree is usable again.
    pub fn new(tree: &'tree mut Tree) -> Self {
        Self {
            tree,
            title: String::from("wgpu-html"),
            initial_size: (1280, 720),
            exit_on_escape: true,
            enable_clipboard: true,
            screenshot_key: Some(KeyCode::F12),
            hook: None,
            state: None,
        }
    }

    /// Window title shown in the OS title bar.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Initial window size in physical pixels.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.initial_size = (width, height);
        self
    }

    /// Whether pressing Escape exits the event loop. Default
    /// `true`. Disable if your document handles Escape itself.
    pub fn with_exit_on_escape(mut self, exit: bool) -> Self {
        self.exit_on_escape = exit;
        self
    }

    /// Whether Ctrl+A / Ctrl+C trigger select-all + copy via
    /// `arboard`. Default `true`.
    pub fn with_clipboard_enabled(mut self, enabled: bool) -> Self {
        self.enable_clipboard = enabled;
        self
    }

    /// Key that captures a PNG screenshot to
    /// `screenshot-<timestamp>.png` in the current directory.
    /// Default `Some(F12)`. Pass `None` to disable.
    pub fn with_screenshot_key(mut self, key: Option<KeyCode>) -> Self {
        self.screenshot_key = key;
        self
    }

    /// Install an [`AppHook`] that intercepts key events, frame
    /// timings, and pointer-move dispatches.
    pub fn with_hook(mut self, hook: impl AppHook + 'static) -> Self {
        self.hook = Some(Box::new(hook));
        self
    }

    /// Register a tree-level hook on the borrowed [`Tree`].
    ///
    /// The harness emits through `Tree::emit_*`; the hook itself is stored on
    /// the tree, not on the winit event loop.
    pub fn with_tree_hook(self, hook: impl TreeHook + Send + 'static) -> Self {
        self.tree.add_hook(hook);
        self
    }

    /// Block on the winit event loop until the window closes.
    pub fn run(mut self) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self)
    }
}

// ── Internal helpers ────────────────────────────────────────────────────────

fn physical_to_pos(p: PhysicalPosition<f64>) -> (f32, f32) {
    (p.x as f32, p.y as f32)
}

/// Convert winit's wheel delta into vertical pixels. Positive =
/// content moves up (i.e. user scrolled down).
fn scroll_delta_to_pixels(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => -y * 48.0,
        MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
    }
}

/// Seconds since UNIX epoch, used for screenshot filenames.
fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn duration_from_ms(ms: f64) -> Duration {
    Duration::from_secs_f64((ms.max(0.0)) / 1000.0)
}

// ── Application handler ─────────────────────────────────────────────────────

impl<'tree> ApplicationHandler for WgpuHtmlWindow<'tree> {
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Called after all pending window events have been dispatched,
        // outside any WndProc. Safe to drop secondary windows here.
        if let Some(h) = self.hook.as_mut() {
            h.on_idle();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(PhysicalSize::new(self.initial_size.0, self.initial_size.1));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("wgpu-html-winit: failed to create window"),
        );
        let size = window.inner_size();
        let renderer = pollster::block_on(Renderer::new(window.clone(), size.width, size.height));
        let text_ctx = TextContext::new(GLYPH_ATLAS_SIZE);
        window.request_redraw();
        self.state = Some(RuntimeState {
            window,
            renderer,
            text_ctx,
            last_layout: None,
            cursor_pos: None,
            scroll_y: 0.0,
            scrollbar_drag: None,
            clipboard: None,
            started_at: Instant::now(),
            last_render_at: None,
            frame_index: 0,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.state.as_ref().map(|s| s.window.clone()) else {
            return;
        };

        // Route events for secondary windows (e.g. devtools) to the hook.
        if id != window.id() {
            let mut hook = self.hook.take();
            if let Some(h) = hook.as_mut() {
                if let Some(state) = self.state.as_mut() {
                    let ctx = HookContext {
                        tree: &mut *self.tree,
                        renderer: &mut state.renderer,
                        text_ctx: &mut state.text_ctx,
                        last_layout: state.last_layout.as_ref(),
                        window: &state.window,
                        event_loop,
                    };
                    h.on_window_event(ctx, id, &event);
                }
            }
            self.hook = hook;
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    state.renderer.resize(size.width, size.height);
                    if let Some(layout) = state.last_layout.as_ref() {
                        state.scroll_y = clamp_scroll_y(state.scroll_y, layout, size.height as f32);
                    }
                    state.scrollbar_drag = None;
                }
                window.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pos = physical_to_pos(position);
                self.handle_cursor_moved(event_loop, pos, &window);
            }

            WindowEvent::CursorLeft { .. } => {
                if let Some(state) = self.state.as_mut() {
                    state.cursor_pos = None;
                }
                self.tree.pointer_leave();
                window.request_redraw();
            }

            WindowEvent::MouseInput {
                state: btn_state,
                button,
                ..
            } => {
                self.handle_mouse_input(btn_state, button, &window);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let dy = scroll_delta_to_pixels(delta);
                self.handle_wheel(dy, &window);
            }

            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard(event_loop, &window, event);
            }

            WindowEvent::RedrawRequested => {
                self.render_frame(event_loop);
            }

            _ => {}
        }
    }
}

// ── Event handlers (broken out for borrow-checker friendliness) ─────────────

impl<'tree> WgpuHtmlWindow<'tree> {
    fn handle_cursor_moved(
        &mut self,
        event_loop: &ActiveEventLoop,
        pos: (f32, f32),
        window: &Window,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        state.cursor_pos = Some(pos);

        // Continue an active scrollbar drag.
        if let Some(drag) = state.scrollbar_drag.clone() {
            if let Some(layout) = state.last_layout.as_ref() {
                let size = state.window.inner_size();
                match drag.target {
                    ScrollTarget::Viewport => {
                        state.scroll_y = scroll_y_from_thumb_top(
                            pos.1 - drag.grab_offset_y,
                            layout,
                            size.width as f32,
                            size.height as f32,
                        );
                    }
                    ScrollTarget::Element(path) => {
                        let doc_pos = viewport_to_document(pos, state.scroll_y);
                        scroll_element_thumb_to(
                            self.tree,
                            layout,
                            path,
                            doc_pos.1 - drag.grab_offset_y,
                        );
                    }
                }
                window.request_redraw();
            }
        }

        // Pointer-move dispatch.
        let scroll_y = state.scroll_y;
        let Some(layout) = state.last_layout.as_ref() else {
            return;
        };
        let doc_pos = viewport_to_document(pos, scroll_y);
        let t0 = Instant::now();
        let changed = interactivity::pointer_move(self.tree, layout, doc_pos);
        let pointer_ms = t0.elapsed().as_secs_f64() * 1000.0;
        if changed || self.tree.interaction.selecting_text {
            window.request_redraw();
        }
        // Fire on_pointer_move hook.
        let mut hook = self.hook.take();
        if let Some(h) = hook.as_mut() {
            if let Some(state) = self.state.as_mut() {
                let ctx = HookContext {
                    tree: &mut *self.tree,
                    renderer: &mut state.renderer,
                    text_ctx: &mut state.text_ctx,
                    last_layout: state.last_layout.as_ref(),
                    window: &state.window,
                    event_loop,
                };
                h.on_pointer_move(ctx, pointer_ms, changed);
            }
        }
        self.hook = hook;
    }

    fn handle_mouse_input(
        &mut self,
        btn_state: ElementState,
        button: WinitMouseButton,
        window: &Window,
    ) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let Some(pos) = state.cursor_pos else { return };

        // Scrollbar drag start / end on the primary button.
        if button == WinitMouseButton::Left {
            match btn_state {
                ElementState::Pressed => {
                    if start_scrollbar_drag(state, self.tree, pos) {
                        window.request_redraw();
                        return;
                    }
                }
                ElementState::Released => {
                    if state.scrollbar_drag.take().is_some() {
                        window.request_redraw();
                        return;
                    }
                }
            }
        }

        let doc_pos = viewport_to_document(pos, state.scroll_y);
        let btn = crate::mouse_button(button);
        let Some(layout) = state.last_layout.as_ref() else {
            return;
        };
        match btn_state {
            ElementState::Pressed => {
                interactivity::mouse_down(self.tree, layout, doc_pos, btn);
            }
            ElementState::Released => {
                interactivity::mouse_up(self.tree, layout, doc_pos, btn);
            }
        }
        window.request_redraw();
    }

    fn handle_wheel(&mut self, dy: f32, window: &Window) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let Some(layout) = state.last_layout.as_ref() else {
            return;
        };
        let Some(pos) = state.cursor_pos else { return };

        let doc_pos = viewport_to_document(pos, state.scroll_y);
        // Element-level scroll first (overflow:scroll containers).
        if scroll_element_at(self.tree, layout, doc_pos, dy) {
            interactivity::pointer_move(self.tree, layout, doc_pos);
            window.request_redraw();
            return;
        }

        // Fall through to viewport scroll.
        let size = state.window.inner_size();
        state.scroll_y = clamp_scroll_y(state.scroll_y + dy, layout, size.height as f32);
        let new_doc_pos = viewport_to_document(pos, state.scroll_y);
        // Re-borrow layout because `state` was previously borrowed
        // mutably (clamp_scroll_y was given &state.last_layout,
        // but state.scroll_y assignment happened in between).
        if let Some(layout) = state.last_layout.as_ref() {
            interactivity::pointer_move(self.tree, layout, new_doc_pos);
        }
        window.request_redraw();
    }

    fn handle_keyboard(&mut self, event_loop: &ActiveEventLoop, window: &Window, event: KeyEvent) {
        // 1. User hook gets first dibs.
        let mut hook = self.hook.take();
        let response = if let Some(h) = hook.as_mut() {
            let Some(state) = self.state.as_mut() else {
                self.hook = hook;
                return;
            };
            let ctx = HookContext {
                tree: &mut *self.tree,
                renderer: &mut state.renderer,
                text_ctx: &mut state.text_ctx,
                last_layout: state.last_layout.as_ref(),
                window: &state.window,
                event_loop,
            };
            h.on_key(ctx, &event)
        } else {
            EventResponse::Continue
        };
        self.hook = hook;
        if response == EventResponse::Stop {
            return;
        }

        // 2. Forward to the tree (modifier sync + keydown/keyup +
        //    Tab navigation).
        crate::handle_keyboard(self.tree, &event);

        // 3. Built-in shortcuts (Esc, screenshot, clipboard).
        if event.state == ElementState::Pressed {
            window.request_redraw();
        }
        let PhysicalKey::Code(key) = event.physical_key else {
            return;
        };
        if event.state == ElementState::Pressed && !event.repeat {
            if self.exit_on_escape && key == KeyCode::Escape {
                event_loop.exit();
                return;
            }
            if self.screenshot_key == Some(key) {
                if let Some(state) = self.state.as_mut() {
                    let path: std::path::PathBuf = format!("screenshot-{}.png", timestamp()).into();
                    state.renderer.capture_next_frame_to(path);
                    window.request_redraw();
                }
                return;
            }
            if self.enable_clipboard && self.tree.modifiers().ctrl {
                if self.tree.interaction.edit_cursor.is_some() {
                    // Form-control-level clipboard shortcuts.
                    match key {
                        // Ctrl+A is handled by handle_edit_key (select_all).
                        KeyCode::KeyA => window.request_redraw(),
                        KeyCode::KeyC => {
                            self.run_edit_copy();
                        }
                        KeyCode::KeyX => {
                            self.run_edit_cut(window);
                        }
                        KeyCode::KeyV => {
                            self.run_edit_paste(window);
                        }
                        _ => {}
                    }
                } else {
                    // Document-level clipboard shortcuts.
                    match key {
                        KeyCode::KeyA => self.run_select_all(window),
                        KeyCode::KeyC => self.run_copy_selection(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn run_select_all(&mut self, window: &Window) {
        let Some(state) = self.state.as_ref() else {
            return;
        };
        let Some(layout) = state.last_layout.as_ref() else {
            return;
        };
        if wgpu_html::select_all_text(self.tree, layout) {
            window.request_redraw();
        }
    }

    /// Copy the selected text from a focused form control to clipboard.
    fn run_edit_copy(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let Some(ec) = &self.tree.interaction.edit_cursor else {
            return;
        };
        if !ec.has_selection() {
            return;
        }
        let Some(focus_path) = self.tree.interaction.focus_path.as_deref() else {
            return;
        };
        let value = match self
            .tree
            .root
            .as_ref()
            .and_then(|r| r.at_path(focus_path))
            .map(|n| &n.element)
        {
            Some(wgpu_html_tree::Element::Input(inp)) => {
                inp.value.clone().unwrap_or_default()
            }
            Some(wgpu_html_tree::Element::Textarea(ta)) => {
                ta.value.clone().unwrap_or_default()
            }
            _ => return,
        };
        let (start, end) = ec.selection_range();
        let start = start.min(value.len());
        let end = end.min(value.len());
        if start >= end {
            return;
        }
        let selected = &value[start..end];
        let cb = state
            .clipboard
            .get_or_insert_with(|| Clipboard::new().expect("arboard: clipboard"));
        let _ = cb.set_text(selected);
    }

    /// Cut: copy selection to clipboard then delete it.
    fn run_edit_cut(&mut self, window: &Window) {
        self.run_edit_copy();
        // Delete the selection via a zero-length insert.
        if let Some(ec) = &self.tree.interaction.edit_cursor {
            if ec.has_selection() {
                wgpu_html_tree::text_input(self.tree, "");
                window.request_redraw();
            }
        }
    }

    /// Paste clipboard text into the focused form control.
    fn run_edit_paste(&mut self, window: &Window) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let cb = state
            .clipboard
            .get_or_insert_with(|| Clipboard::new().expect("arboard: clipboard"));
        let Ok(text) = cb.get_text() else {
            return;
        };
        if !text.is_empty() {
            wgpu_html_tree::text_input(self.tree, &text);
            window.request_redraw();
        }
    }

    fn run_copy_selection(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let Some(layout) = state.last_layout.as_ref() else {
            return;
        };
        let Some(text) = wgpu_html::selected_text(self.tree, layout) else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let cb = state
            .clipboard
            .get_or_insert_with(|| Clipboard::new().expect("arboard: failed to open clipboard"));
        let _ = cb.set_text(text);
    }

    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let frame_t0 = Instant::now();
        self.tree.emit_lifecycle_begin(TreeLifecycleStage::Frame);
        let size = state.window.inner_size();

        let (mut list, layout, timings) = wgpu_html::paint_tree_returning_layout_profiled(
            self.tree,
            &mut state.text_ctx,
            size.width as f32,
            size.height as f32,
            1.0,
        );
        self.tree.emit_lifecycle_end(
            TreeLifecycleStage::Cascade,
            duration_from_ms(timings.cascade_ms),
        );
        self.tree.emit_lifecycle_end(
            TreeLifecycleStage::Layout,
            duration_from_ms(timings.layout_ms),
        );
        self.tree.emit_lifecycle_end(
            TreeLifecycleStage::Paint,
            duration_from_ms(timings.paint_ms),
        );

        if let Some(layout) = layout.as_ref() {
            state.scroll_y = clamp_scroll_y(state.scroll_y, layout, size.height as f32);
            translate_display_list_y(&mut list, -state.scroll_y);
            paint_viewport_scrollbar(
                &mut list,
                layout,
                size.width as f32,
                size.height as f32,
                state.scroll_y,
            );
        } else {
            state.scroll_y = 0.0;
        }
        state.last_layout = layout;

        // Push freshly-rasterised glyph rasters into the GPU atlas.
        state
            .text_ctx
            .atlas
            .upload(&state.renderer.queue, state.renderer.glyph_atlas_texture());

        self.tree.emit_lifecycle_begin(TreeLifecycleStage::Render);
        let render_t0 = Instant::now();
        match state.renderer.render(&list) {
            FrameOutcome::Presented | FrameOutcome::Skipped => {}
            FrameOutcome::Reconfigure => {
                state.renderer.resize(size.width, size.height);
            }
        }
        let render_duration = render_t0.elapsed();
        self.tree
            .emit_lifecycle_end(TreeLifecycleStage::Render, render_duration);
        let render_ms = render_duration.as_secs_f64() * 1000.0;
        let frame_duration = frame_t0.elapsed();

        let now = Instant::now();
        let delta = state
            .last_render_at
            .map(|prev| now.saturating_duration_since(prev))
            .unwrap_or(Duration::ZERO);
        state.last_render_at = Some(now);
        let frame_index = state.frame_index;
        state.frame_index = state.frame_index.saturating_add(1);
        let render_event = TreeRenderEvent::new(delta)
            .with_elapsed(now.saturating_duration_since(state.started_at))
            .with_frame_index(frame_index)
            .with_viewport(TreeRenderViewport::new(
                size.width as f32,
                size.height as f32,
                1.0,
            ))
            .with_frame_duration(frame_duration)
            .with_pipeline_durations(
                Some(duration_from_ms(timings.cascade_ms)),
                Some(duration_from_ms(timings.layout_ms)),
                Some(duration_from_ms(timings.paint_ms)),
                Some(render_duration),
            );
        self.tree.emit_render(&render_event);
        self.tree
            .emit_lifecycle_end(TreeLifecycleStage::Frame, frame_duration);

        // Hook callback with frame timings.
        let frame_timings = FrameTimings {
            cascade_ms: timings.cascade_ms,
            layout_ms: timings.layout_ms,
            paint_ms: timings.paint_ms,
            render_ms,
        };
        let mut hook = self.hook.take();
        if let Some(h) = hook.as_mut() {
            if let Some(state) = self.state.as_mut() {
                let ctx = HookContext {
                    tree: &mut *self.tree,
                    renderer: &mut state.renderer,
                    text_ctx: &mut state.text_ctx,
                    last_layout: state.last_layout.as_ref(),
                    window: &state.window,
                    event_loop,
                };
                h.on_frame(ctx, &frame_timings);
            }
        }
        self.hook = hook;
    }
}

/// Hit-test scrollbars (element first, then viewport) and start a
/// drag if the press is on a thumb or track.
fn start_scrollbar_drag(state: &mut RuntimeState, tree: &mut Tree, pos: (f32, f32)) -> bool {
    let Some(layout) = state.last_layout.as_ref() else {
        return false;
    };
    let size = state.window.inner_size();
    let doc_pos = viewport_to_document(pos, state.scroll_y);

    // Element-level scrollbars first.
    if let Some((path, geom)) = deepest_element_scrollbar_at(
        layout,
        doc_pos,
        &tree.interaction.scroll_offsets_y,
        &mut Vec::new(),
    ) {
        if rect_contains(geom.thumb, doc_pos) {
            state.scrollbar_drag = Some(ScrollbarDrag {
                target: ScrollTarget::Element(path),
                grab_offset_y: doc_pos.1 - geom.thumb.y,
            });
            return true;
        }
        if rect_contains(geom.track, doc_pos) {
            let thumb_top = doc_pos.1 - geom.thumb.h * 0.5;
            scroll_element_thumb_to(tree, layout, path.clone(), thumb_top);
            // Recompute the geometry now that scroll has moved.
            if let Some(box_) = layout.box_at_path(&path) {
                let new_scroll = tree
                    .interaction
                    .scroll_offsets_y
                    .get(&path)
                    .copied()
                    .unwrap_or(0.0);
                if let Some(updated) =
                    wgpu_html::scroll::element_scrollbar_geometry(box_, new_scroll)
                {
                    state.scrollbar_drag = Some(ScrollbarDrag {
                        target: ScrollTarget::Element(path),
                        grab_offset_y: doc_pos.1 - updated.thumb.y,
                    });
                }
            }
            return true;
        }
    }

    // Viewport scrollbar.
    let Some(geom) = scrollbar_geometry(
        layout,
        size.width as f32,
        size.height as f32,
        state.scroll_y,
    ) else {
        return false;
    };
    if rect_contains(geom.thumb, pos) {
        state.scrollbar_drag = Some(ScrollbarDrag {
            target: ScrollTarget::Viewport,
            grab_offset_y: pos.1 - geom.thumb.y,
        });
        return true;
    }
    if rect_contains(geom.track, pos) {
        let thumb_top = pos.1 - geom.thumb.h * 0.5;
        state.scroll_y =
            scroll_y_from_thumb_top(thumb_top, layout, size.width as f32, size.height as f32);
        if let Some(updated) = scrollbar_geometry(
            layout,
            size.width as f32,
            size.height as f32,
            state.scroll_y,
        ) {
            state.scrollbar_drag = Some(ScrollbarDrag {
                target: ScrollTarget::Viewport,
                grab_offset_y: pos.1 - updated.thumb.y,
            });
        }
        return true;
    }
    false
}
