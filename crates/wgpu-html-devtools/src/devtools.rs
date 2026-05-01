//! Platform-agnostic devtools inspector.
//!
//! `Devtools` wraps a [`DevtoolsComponent`] driven by
//! [`Mount`](wgpu_html_ui::Mount), managing the secondary OS window
//! and synchronisation with the inspected host tree.

use std::sync::Arc;

use wgpu_html_tree::{Profiler, Tree};
use wgpu_html_ui::Mount;
use wgpu_html_winit::HtmlWindow;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::component::{DevtoolsComponent, DevtoolsProps};

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

/// CSS for the devtools UI.
const CSS: &str = include_str!("../html/devtools.css");

// ── Devtools ─────────────────────────────────────────────────────

/// Platform-agnostic devtools inspector.
///
/// Owns its own UI [`Tree`] and delegates rendering and input
/// handling to an [`HtmlWindow`] when active.  The UI is produced
/// by [`DevtoolsComponent`] via the `wgpu-html-ui` framework.
pub struct Devtools {
    tree: Tree,
    mount: Mount<DevtoolsComponent>,

    last_inspected_gen: Option<u64>,
    needs_redraw: bool,

    html_window: Option<HtmlWindow>,
    pending_drop: Option<HtmlWindow>,
    enabled: bool,
}

impl Devtools {
    /// Create a devtools instance without attaching to a tree.
    pub fn new(enable_profiler: bool) -> Self {
        let mut tree = Tree::default();
        let lucide = wgpu_html_tree::FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
        tree.register_font(lucide);
        tree.register_linked_stylesheet("devtools.css", CSS);

        if enable_profiler {
            tree.profiler = Some(Profiler::tagged("devtools"));
        }

        let mount = Mount::<DevtoolsComponent>::new(DevtoolsProps);

        Self {
            tree,
            mount,
            last_inspected_gen: None,
            needs_redraw: true,
            html_window: None,
            pending_drop: None,
            enabled: false,
        }
    }

    /// Create a devtools instance pre-populated with the host tree's
    /// fonts.  The first render is deferred to the first [`poll`]
    /// call (which only runs when the devtools window is enabled).
    pub fn attach(host_tree: &mut Tree, enable_profiler: bool) -> Self {
        if enable_profiler && host_tree.profiler.is_none() {
            host_tree.profiler = Some(Profiler::tagged("host"));
        }

        let mut devtools = Self::new(enable_profiler);
        for (_handle, face) in host_tree.fonts.iter() {
            devtools.register_font(face.clone());
        }
        devtools
    }

    // ── Font registration ───────────────────────────────────

    pub fn register_font(&mut self, face: wgpu_html_tree::FontFace) {
        self.tree.register_font(face);
    }

    // ── Tree access ─────────────────────────────────────────

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    // ── Polling ─────────────────────────────────────────────

    /// Sync with the host tree.  Call once per frame.
    pub fn poll(&mut self, host_tree: &Tree) {
        // Process any pending component messages (clicks, toggles)
        // first, so we don't force-render and then immediately
        // re-render again from queued messages.
        if self.mount.process(&mut self.tree, host_tree) {
            println!("[Devtools::poll] process() triggered re-render");
            self.needs_redraw = true;
        }

        let dom_changed = self.last_inspected_gen != Some(host_tree.generation);
        if dom_changed {
            println!("[Devtools::poll] dom_changed, host_gen={}", host_tree.generation);
            self.last_inspected_gen = Some(host_tree.generation);
            self.mount.force_render(&mut self.tree, host_tree);
            self.needs_redraw = true;
        }
    }

    // ── Window lifecycle ────────────────────────────────────

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self, event_loop: &ActiveEventLoop) {
        if self.enabled {
            return;
        }
        self.enabled = true;
        let hw = HtmlWindow::new(event_loop, "DevTools", 1280, 720);
        hw.request_redraw();
        self.html_window = Some(hw);
    }

    pub fn disable(&mut self) {
        if !self.enabled {
            return;
        }
        self.enabled = false;
        self.pending_drop = self.html_window.take();
    }

    pub fn toggle(&mut self, event_loop: &ActiveEventLoop) {
        if self.enabled {
            self.disable();
        } else {
            self.enable(event_loop);
        }
    }

    pub fn flush(&mut self) {
        self.pending_drop = None;
    }

    pub fn window_id(&self) -> Option<WindowId> {
        self.html_window.as_ref().map(|hw| hw.window_id())
    }

    pub fn owns_window(&self, id: WindowId) -> bool {
        self.window_id() == Some(id)
            || self
                .pending_drop
                .as_ref()
                .is_some_and(|hw| hw.window_id() == id)
    }

    /// Handle a winit `WindowEvent` for the devtools window.
    pub fn handle_window_event(&mut self, host_tree: &Tree, event: &WindowEvent) {
        let Some(hw) = self.html_window.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                self.enabled = false;
                self.pending_drop = self.html_window.take();
                return;
            }
            WindowEvent::RedrawRequested => {
                self.render_to_window();
                return;
            }
            _ => {}
        }
        let needs_redraw = hw.handle_event(&mut self.tree, event);

        // After mouse release, component callbacks may have queued messages.
        if matches!(
            event,
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            }
        ) {
            println!("[Devtools::handle_window_event] MouseRelease, running process()");
            if self.mount.process(&mut self.tree, host_tree) {
                println!("[Devtools::handle_window_event] process() triggered re-render");
                self.needs_redraw = true;
            }
        }

        if needs_redraw || self.needs_redraw {
            if let Some(hw) = &self.html_window {
                hw.request_redraw();
            }
        }
    }

    /// Sync with host tree and request a redraw if needed.
    pub fn poll_and_redraw(&mut self, host_tree: &Tree) {
        self.poll(host_tree);
        if self.needs_redraw {
            if let Some(hw) = &self.html_window {
                hw.request_redraw();
            }
        }
    }

    fn render_to_window(&mut self) {
        if let Some(hw) = self.html_window.as_mut() {
            hw.render_frame(&self.tree);
        }
        self.needs_redraw = false;
    }
}
