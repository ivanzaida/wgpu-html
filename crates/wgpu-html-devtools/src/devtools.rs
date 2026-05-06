//! Platform-agnostic devtools inspector.
//!
//! `Devtools` owns a UI [`Tree`] and a [`Mount`] that drives the
//! devtools component. It knows nothing about windowing — the host
//! is responsible for creating a window, routing events via
//! [`dispatch`](wgpu_html_driver_winit::dispatch), and calling
//! [`render_frame`](wgpu_html_driver::Runtime::render_frame) on
//! `devtools.tree()`.

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use crate::ui::{DevtoolsComponent, DevtoolsProps, SharedHostTree, SharedHoverPath, SharedPendingPick, SharedPickMode};
use wgpu_html_events::HtmlEvent;
use wgpu_html_tree::{FontFace, Profiler, Tree, TreeHookHandle};
use wgpu_html_ui::Mount;

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── Devtools ─────────────────────────────────────────────────────

/// Platform-agnostic devtools inspector.
///
/// Owns its own UI [`Tree`] driven by a [`Mount<DevtoolsComponent>`].
/// The host integration creates a window, dispatches platform events
/// into [`Devtools::tree_mut()`], and renders frames from it.
///
/// # Usage
///
/// ```ignore
/// let mut devtools = Devtools::attach(&mut host_tree, false);
///
/// // Each frame / event-loop iteration:
/// devtools.poll(&host_tree);
///
/// // The host checks is_enabled() / needs_redraw() to decide
/// // whether to show the window and request redraws.
/// ```
pub struct Devtools {
  tree: Tree,
  mount: Mount<DevtoolsComponent>,

  last_inspected_gen: Option<u64>,
  needs_redraw: bool,

  enabled: bool,
  toggle_requested: Arc<AtomicBool>,
  dump_html_requested: Arc<AtomicBool>,
  shared_hover: SharedHoverPath,
  shared_pick_mode: SharedPickMode,
  shared_pending_pick: SharedPendingPick,
  shared_host_tree: SharedHostTree,
}

impl Devtools {
  /// Create a devtools instance without attaching to a host tree.
  pub fn new(enable_profiler: bool) -> Self {
    let mut tree = Tree::default();
    let lucide = FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
    tree.register_font(lucide);

    if enable_profiler {
      tree.profiler = Some(Profiler::tagged("devtools"));
    }

    let shared_hover: SharedHoverPath = Arc::new(std::sync::Mutex::new(None));
    let shared_pick_mode: SharedPickMode = Arc::new(AtomicBool::new(false));
    let shared_pending_pick: SharedPendingPick = Arc::new(std::sync::Mutex::new(None));
    let shared_host_tree: SharedHostTree = Arc::new(std::sync::RwLock::new(None));
    let mount = Mount::<DevtoolsComponent>::new(DevtoolsProps {
      shared_hover: shared_hover.clone(),
      shared_pick_mode: shared_pick_mode.clone(),
      shared_pending_pick: shared_pending_pick.clone(),
      host_tree: shared_host_tree.clone(),
    });

    let dump_html_requested = Arc::new(AtomicBool::new(false));
    tree.hooks.push(TreeHookHandle::new(crate::devtools_hook::DumpHtmlHook {
      flag: dump_html_requested.clone(),
    }));

    Self {
      tree,
      mount,
      last_inspected_gen: None,
      needs_redraw: true,
      enabled: false,
      toggle_requested: Arc::new(AtomicBool::new(false)),
      dump_html_requested,
      shared_hover,
      shared_pick_mode,
      shared_pending_pick,
      shared_host_tree,
    }
  }

  /// Create a devtools instance pre-populated with the host tree's
  /// fonts. Registers an F11 keyup handler on the host tree root.
  pub fn attach(host_tree: &mut Tree, enable_profiler: bool) -> Self {
    let mut devtools = Self::new(enable_profiler);
    for (_handle, face) in host_tree.fonts.iter() {
      devtools.tree.register_font(face.clone());
    }
    devtools.enabled = true;

    let toggle_flag = devtools.toggle_requested.clone();
    host_tree.root.as_mut().unwrap().on_keyup.push(Arc::new(move |e| {
      let HtmlEvent::Keyboard(kb) = e else { return };
      if kb.code == "F11" {
        toggle_flag.store(true, Ordering::Relaxed);
      }
    }));

    devtools
  }

  // ── Tree access ────────────────────────────────────────────

  /// The devtools UI tree. The host dispatches platform events
  /// into this and calls `render_frame` on it.
  pub fn tree(&self) -> &Tree {
    &self.tree
  }

  /// Mutable access to the devtools UI tree.
  pub fn tree_mut(&mut self) -> &mut Tree {
    &mut self.tree
  }

  // ── Font registration ──────────────────────────────────────

  pub fn register_font(&mut self, face: FontFace) {
    self.tree.register_font(face);
  }

  // ── Polling ────────────────────────────────────────────────

  /// Sync with the host tree and process component messages.
  /// Call once per frame or event-loop iteration.
  pub fn poll(&mut self, host_tree: &Tree) {
    if self.toggle_requested.swap(false, Ordering::Relaxed) {
      self.toggle();
    }

    if !self.enabled {
      return;
    }

    // Shift+F12: dump devtools tree as HTML for browser debugging.
    if self.dump_html_requested.swap(false, Ordering::Relaxed) {
      let html = self.tree.to_html();
      let path = format!("devtools-dump-{}.html", self.tree.generation);
      match std::fs::write(&path, &html) {
        Ok(()) => println!("[devtools] saved HTML \u{2192} {path} ({} bytes)", html.len()),
        Err(e) => eprintln!("[devtools] failed to save HTML: {e}"),
      }
    }

    // Update the shared host tree reference.
    {
      let mut ht = self.shared_host_tree.write().unwrap();
      *ht = Some(host_tree.clone());
    }

    // Process pending component messages.
    if self.mount.process(&mut self.tree) {
      self.needs_redraw = true;
    }

    // Re-render if the host tree changed.
    let dom_changed = self.last_inspected_gen != Some(host_tree.generation);
    if dom_changed {
      self.last_inspected_gen = Some(host_tree.generation);
      self.mount.force_render(&mut self.tree);
      self.needs_redraw = true;
    }
  }

  /// Called by the host after rendering a frame.
  pub fn frame_rendered(&mut self) {
    self.needs_redraw = false;
  }

  // ── State ──────────────────────────────────────────────────

  pub fn needs_redraw(&self) -> bool {
    self.needs_redraw
  }

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn enable(&mut self) {
    self.enabled = true;
    self.needs_redraw = true;
  }

  pub fn disable(&mut self) {
    self.enabled = false;
  }

  pub fn toggle(&mut self) {
    if self.enabled {
      self.disable();
    } else {
      self.enable();
    }
  }

  pub fn hovered_path(&self) -> Option<Vec<usize>> {
    self.shared_hover.lock().ok()?.clone()
  }

  pub fn is_pick_mode(&self) -> bool {
    self.shared_pick_mode.load(Ordering::Relaxed)
  }

  pub fn pick_element(&mut self, path: Vec<usize>) {
    if let Ok(mut pending) = self.shared_pending_pick.lock() {
      *pending = Some(path);
    }
    if let Ok(mut hover) = self.shared_hover.lock() {
      *hover = None;
    }
    self.needs_redraw = true;
  }

  pub fn set_hover_path(&mut self, path: Option<Vec<usize>>) {
    if let Ok(mut hover) = self.shared_hover.lock() {
      *hover = path;
    }
  }
}

impl wgpu_html_driver::SecondaryWindow for Devtools {
  fn poll(&mut self, tree: &Tree) {
    Devtools::poll(self, tree);
  }

  fn needs_redraw(&self) -> bool {
    Devtools::needs_redraw(self)
  }
}
