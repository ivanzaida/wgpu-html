//! Batteries-included winit harness.
//!
//! See [`WgpuHtmlWindow`] for the public surface and
//! [`create_window`] for the entry point.

use std::sync::Arc;

use wgpu_html::interactivity;
use wgpu_html::layout::LayoutBox;
use wgpu_html::renderer::{FrameOutcome, GLYPH_ATLAS_SIZE, Renderer};
use wgpu_html_text::TextContext;
use wgpu_html_tree::Tree;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::error::EventLoopError;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

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
/// [`Tree`]. Drives the cascade → layout → paint → render loop
/// and forwards mouse / keyboard input into the tree's dispatch
/// API.
///
/// Builder methods configure window attributes; [`Self::run`]
/// blocks until the user closes the window (or [`WindowEvent::CloseRequested`]
/// is emitted).
pub struct WgpuHtmlWindow<'tree> {
    tree: &'tree mut Tree,
    title: String,
    initial_size: (u32, u32),
    exit_on_escape: bool,
    state: Option<RuntimeState>,
}

/// Live runtime state, populated in `resumed` once winit hands us
/// an `ActiveEventLoop` to create a window from.
struct RuntimeState {
    window: Arc<Window>,
    renderer: Renderer,
    text_ctx: TextContext,
    last_layout: Option<LayoutBox>,
    cursor_pos: Option<(f32, f32)>,
}

impl<'tree> WgpuHtmlWindow<'tree> {
    /// Build a new harness around `tree`. The borrow lasts for the
    /// duration of [`Self::run`]; after `run` returns, the tree is
    /// usable again.
    pub fn new(tree: &'tree mut Tree) -> Self {
        Self {
            tree,
            title: String::from("wgpu-html"),
            initial_size: (1280, 720),
            exit_on_escape: true,
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

    /// Block on the winit event loop until the window closes.
    ///
    /// Returns any `EventLoopError` that winit produces; the
    /// happy path returns `Ok(())` on a clean exit.
    pub fn run(mut self) -> Result<(), EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self)
    }

    /// Run the cascade → layout → paint → render pipeline once.
    /// Called from `RedrawRequested`.
    fn render_frame(&mut self) {
        let Some(state) = self.state.as_mut() else {
            return;
        };
        let size = state.window.inner_size();
        let (list, layout) = wgpu_html::paint_tree_returning_layout(
            self.tree,
            &mut state.text_ctx,
            size.width as f32,
            size.height as f32,
            // Fixed 1.0 scale; HiDPI handling is intentionally out of
            // scope for the minimal harness — apps that need it can
            // copy the demo's scale-factor handling.
            1.0,
        );
        state.last_layout = layout;

        // Push any newly-rasterised glyph rasters into the GPU atlas
        // before the draw. Cheap when nothing is dirty.
        state
            .text_ctx
            .atlas
            .upload(&state.renderer.queue, state.renderer.glyph_atlas_texture());

        match state.renderer.render(&list) {
            FrameOutcome::Presented | FrameOutcome::Skipped => {}
            FrameOutcome::Reconfigure => {
                state.renderer.resize(size.width, size.height);
            }
        }
    }
}

impl<'tree> ApplicationHandler for WgpuHtmlWindow<'tree> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(PhysicalSize::new(
                self.initial_size.0,
                self.initial_size.1,
            ));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("wgpu-html-winit: failed to create window"),
        );
        let size = window.inner_size();
        let renderer =
            pollster::block_on(Renderer::new(window.clone(), size.width, size.height));
        let text_ctx = TextContext::new(GLYPH_ATLAS_SIZE);
        window.request_redraw();
        self.state = Some(RuntimeState {
            window,
            renderer,
            text_ctx,
            last_layout: None,
            cursor_pos: None,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        // A clone of the window handle so we can `request_redraw`
        // without holding the `&mut state` borrow open.
        let Some(window) = self.state.as_ref().map(|s| s.window.clone()) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(state) = self.state.as_mut() {
                    state.renderer.resize(size.width, size.height);
                }
                window.request_redraw();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pos = physical_to_pos(position);
                if let Some(state) = self.state.as_mut() {
                    state.cursor_pos = Some(pos);
                    if let Some(layout) = state.last_layout.as_ref() {
                        let changed = interactivity::pointer_move(self.tree, layout, pos);
                        if changed || self.tree.interaction.selecting_text {
                            window.request_redraw();
                        }
                    }
                }
            }

            WindowEvent::CursorLeft { .. } => {
                if let Some(state) = self.state.as_mut() {
                    state.cursor_pos = None;
                }
                self.tree.pointer_leave();
                window.request_redraw();
            }

            WindowEvent::MouseInput { state: btn_state, button, .. } => {
                let Some(state) = self.state.as_ref() else {
                    return;
                };
                let Some(pos) = state.cursor_pos else { return };
                // Layout might still be building on the first
                // frame; if we have nothing to hit-test, skip.
                let Some(layout) = state.last_layout.as_ref() else {
                    return;
                };
                let btn = crate::mouse_button(button);
                match btn_state {
                    ElementState::Pressed => {
                        interactivity::mouse_down(self.tree, layout, pos, btn);
                    }
                    ElementState::Released => {
                        interactivity::mouse_up(self.tree, layout, pos, btn);
                    }
                }
                window.request_redraw();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                crate::handle_keyboard(self.tree, &event);
                if let PhysicalKey::Code(key) = event.physical_key {
                    if self.exit_on_escape
                        && key == KeyCode::Escape
                        && event.state == ElementState::Pressed
                    {
                        event_loop.exit();
                        return;
                    }
                }
                if event.state == ElementState::Pressed {
                    window.request_redraw();
                }
            }

            WindowEvent::RedrawRequested => {
                self.render_frame();
            }

            _ => {}
        }
    }
}

fn physical_to_pos(p: PhysicalPosition<f64>) -> (f32, f32) {
    (p.x as f32, p.y as f32)
}
