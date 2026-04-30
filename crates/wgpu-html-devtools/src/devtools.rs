//! Platform-agnostic devtools inspector.
//!
//! `Devtools` manages a UI tree that visualises an inspected tree's
//! DOM structure, styles, and breadcrumb. Rendering and input handling
//! are delegated to an [`HtmlWindow`] when the devtools window is active.

use std::sync::Arc;
use std::sync::Mutex;

use wgpu_html_tree::{Node, Profiler, Tree};
use wgpu_html_winit::HtmlWindow;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::html_gen;

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── Devtools ─────────────────────────────────────────────────────

/// Platform-agnostic devtools inspector.
///
/// Owns its own UI [`Tree`] and delegates rendering and input handling
/// to an [`HtmlWindow`] when active.
pub struct Devtools {
    // ── UI state ────────────────────────────────────────────
    tree: Tree,

    // ── Inspected-tree tracking ─────────────────────────────
    selected_path: Option<Vec<usize>>,
    click_sink: Arc<Mutex<Option<Vec<usize>>>>,
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
    /// The host must call [`update_inspected_tree`] manually.
    pub fn new(enable_profiler: bool) -> Self {
        let click_sink = Arc::new(Mutex::new(None));
        let mut tree = html_gen::build(None, None, &click_sink);
        let lucide = wgpu_html_tree::FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
        tree.register_font(lucide.clone());

        if enable_profiler {
            tree.profiler = Some(Profiler::tagged("devtools"));
        }

        Self {
            tree,
            selected_path: None,
            click_sink,
            inspected_root: None,
            last_inspected_gen: None,
            needs_redraw: true,
            html_window: None,
            pending_drop: None,
            enabled: false,
        }
    }

    /// Create a devtools instance pre-populated with the host tree's
    /// state and fonts. The host calls [`poll`] each frame with
    /// `&tree` to sync changes.
    ///
    /// Copies the tree's registered fonts into the devtools.
    pub fn attach(tree: &mut Tree, enable_profiler: bool) -> Self {
        // Enable profiling on the host tree so the cascade → layout →
        // paint pipeline records and auto-flushes timings.
        if enable_profiler && tree.profiler.is_none() {
            tree.profiler = Some(Profiler::tagged("host"));
        }

        let mut devtools = Self::new(enable_profiler);
        devtools.inspected_root = tree.root.clone();
        devtools.last_inspected_gen = Some(tree.generation);
        // Copy the tree's fonts so the devtools UI can render text.
        for (_handle, face) in tree.fonts.iter() {
            devtools.register_font(face.clone());
        }
        // Populate the tree rows with the initial inspected tree.
        devtools.update_tree_rows();
        devtools.update_selection();
        devtools
    }

    // ── Font registration ───────────────────────────────────

    /// Register a font face for the devtools UI.
    pub fn register_font(&mut self, face: wgpu_html_tree::FontFace) {
        self.tree.register_font(face);
    }

    // ── Tree access ─────────────────────────────────────────

    /// Borrow the devtools UI tree. Useful for feeding it into a
    /// second-level devtools or for host-side inspection.
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// Whether the devtools needs a repaint.
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    // ── Polling ─────────────────────────────────────────────

    /// Sync with the host tree. Call once per frame. Only updates
    /// the parts of the devtools UI that actually changed:
    /// tree-rows on DOM mutation, breadcrumb + styles on selection.
    pub fn poll(&mut self, host_tree: &Tree) {
        let dom_changed = self.last_inspected_gen != Some(host_tree.generation);
        if dom_changed {
            self.inspected_root = host_tree.root.clone();
            self.last_inspected_gen = Some(host_tree.generation);
            self.update_tree_rows();
        }

        let clicked = self.click_sink.lock().unwrap().take();
        if let Some(path) = clicked {
            self.selected_path = Some(path);
            self.update_selection();
        }
    }

    // ── Internal: incremental tree updates ───────────────────

    /// Rebuild only the tree-rows container (inspected DOM changed).
    fn update_tree_rows(&mut self) {
        html_gen::update_tree_rows(
            &mut self.tree,
            self.inspected_root.as_ref(),
            self.selected_path.as_deref(),
            &self.click_sink,
        );
        self.needs_redraw = true;
    }

    /// Update breadcrumb + styles panel (selection changed).
    fn update_selection(&mut self) {
        html_gen::update_breadcrumb(
            &mut self.tree,
            self.inspected_root.as_ref(),
            self.selected_path.as_deref(),
        );
        html_gen::update_styles(
            &mut self.tree,
            self.inspected_root.as_ref(),
            self.selected_path.as_deref(),
        );
        // Also update tree rows to reflect the new selection highlight.
        html_gen::update_tree_rows(
            &mut self.tree,
            self.inspected_root.as_ref(),
            self.selected_path.as_deref(),
            &self.click_sink,
        );
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
    /// Delegates all event→tree forwarding (pointer, mouse, scroll,
    /// keyboard, resize) to `HtmlWindow::handle_event`.
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
        // Check if a tree-row click produced a selection.
        if matches!(event, WindowEvent::MouseInput { state: ElementState::Released, .. }) {
            let clicked = self.click_sink.lock().unwrap().take();
            if let Some(path) = clicked {
                self.selected_path = Some(path);
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
