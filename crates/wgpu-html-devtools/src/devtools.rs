//! Platform-agnostic devtools inspector.
//!
//! `Devtools` manages a UI tree that visualises an inspected tree's
//! DOM structure, styles, and breadcrumb. Rendering and input handling
//! are delegated to an [`HtmlWindow`] when the devtools window is active.

use std::sync::Arc;

use wgpu_html_tree::{Node, Profiler, Tree};
use wgpu_html_winit::HtmlWindow;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::components::breadcrumb::Breadcrumb;
use crate::components::styles_panel::StylesPanel;
use crate::components::tree_view::TreeView;
use crate::html_gen;

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── Devtools ─────────────────────────────────────────────────────

/// Platform-agnostic devtools inspector.
///
/// Owns its own UI [`Tree`] and delegates rendering and input handling
/// to an [`HtmlWindow`] when active. Each panel section is a
/// component struct that owns its own state.
pub struct Devtools {
    // ── UI state ────────────────────────────────────────────
    tree: Tree,

    // ── Components ──────────────────────────────────────────
    tree_view: TreeView,
    breadcrumb: Breadcrumb,
    styles_panel: StylesPanel,

    // ── Inspected-tree tracking ─────────────────────────────
    inspected_root: Option<Node>,
    last_inspected_gen: Option<u64>,

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
    pub fn new(enable_profiler: bool) -> Self {
        let tree_view = TreeView::new();
        let mut tree = html_gen::build_shell();
        let lucide = wgpu_html_tree::FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
        tree.register_font(lucide);

        if enable_profiler {
            tree.profiler = Some(Profiler::tagged("devtools"));
        }

        Self {
            tree,
            tree_view,
            breadcrumb: Breadcrumb::new(),
            styles_panel: StylesPanel::new(),
            inspected_root: None,
            last_inspected_gen: None,
            needs_redraw: true,
            html_window: None,
            pending_drop: None,
            enabled: false,
        }
    }

    /// Create a devtools instance pre-populated with the host tree's
    /// state and fonts.
    pub fn attach(tree: &mut Tree, enable_profiler: bool) -> Self {
        if enable_profiler && tree.profiler.is_none() {
            tree.profiler = Some(Profiler::tagged("host"));
        }

        let mut devtools = Self::new(enable_profiler);
        devtools.inspected_root = tree.root.clone();
        devtools.last_inspected_gen = Some(tree.generation);
        for (_handle, face) in tree.fonts.iter() {
            devtools.register_font(face.clone());
        }
        devtools.update_tree_rows();
        devtools.update_selection();
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

    /// Sync with the host tree. Call once per frame.
    pub fn poll(&mut self, host_tree: &Tree) {
        let dom_changed = self.last_inspected_gen != Some(host_tree.generation);
        if dom_changed {
            self.inspected_root = host_tree.root.clone();
            self.last_inspected_gen = Some(host_tree.generation);
            self.update_tree_rows();
        }

        let (toggled, clicked) = self.tree_view.drain();
        if toggled {
            self.update_tree_rows();
        }
        if clicked {
            self.update_selection();
        }
    }

    // ── Internal: incremental tree updates ───────────────────

    fn update_tree_rows(&mut self) {
        self.tree_view
            .update(&mut self.tree, self.inspected_root.as_ref());
        self.needs_redraw = true;
    }

    fn update_selection(&mut self) {
        let root = self.inspected_root.as_ref();
        let sel = self.tree_view.selected_path.as_deref();
        self.breadcrumb.update(&mut self.tree, root, sel);
        self.styles_panel.update(&mut self.tree, root, sel);
        self.update_tree_rows();
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
    pub fn handle_window_event(&mut self, event: &WindowEvent) {
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

        // Check if a tree-row click or chevron toggle happened.
        if matches!(
            event,
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            }
        ) {
            let (toggled, clicked) = self.tree_view.drain();
            if toggled {
                self.update_tree_rows();
            }
            if clicked {
                self.update_selection();
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
