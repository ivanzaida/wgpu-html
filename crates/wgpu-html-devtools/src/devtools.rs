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

use crate::ui::{DevtoolsComponent, DevtoolsProps, DevtoolsStore};
use wgpu_html_events::HtmlEvent;
use wgpu_html_layout::LayoutBox;
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
  store: DevtoolsStore,

  last_inspected_gen: Option<u64>,
  last_cascade_gen: Option<u64>,
  last_selected_path: Option<Vec<usize>>,
  needs_redraw: bool,

  enabled: bool,
  toggle_requested: Arc<AtomicBool>,
  dump_html_requested: Arc<AtomicBool>,
}

impl Devtools {
  /// Create a devtools instance without attaching to a host tree.
  pub fn new(enable_profiler: bool) -> Self {
    let mut tree = Tree::default();
    let lucide = FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
    tree.register_font(lucide);
    tree.register_linked_stylesheet("devtools.css", "body { margin:0; padding: 0; }");
    if enable_profiler {
      tree.profiler = Some(Profiler::tagged("devtools"));
    }

    let store = DevtoolsStore::new();
    let mount = Mount::<DevtoolsComponent>::new(DevtoolsProps {
      store: store.clone(),
    });

    let dump_html_requested = Arc::new(AtomicBool::new(false));
    tree.hooks.push(TreeHookHandle::new(crate::devtools_hook::DumpHtmlHook {
      flag: dump_html_requested.clone(),
    }));

    Self {
      tree,
      mount,
      store,
      last_inspected_gen: None,
      last_cascade_gen: None,
      last_selected_path: None,
      needs_redraw: true,
      enabled: false,
      toggle_requested: Arc::new(AtomicBool::new(false)),
      dump_html_requested,
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
    self.poll_with_layout(host_tree, None);
  }

  /// Like [`poll`](Self::poll), but also binds the host layout tree
  /// so the Layout section in the styles panel can show box model data.
  pub fn poll_with_layout(&mut self, host_tree: &Tree, layout: Option<&LayoutBox>) {
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

    self.store.bind_host_tree(host_tree);
    if let Some(l) = layout { self.store.bind_layout(l); }

    let cascade_changed = self.last_cascade_gen != Some(host_tree.cascade_generation);
    let dom_changed = self.last_inspected_gen != Some(host_tree.generation);
    if dom_changed {
      self.last_inspected_gen = Some(host_tree.generation);
    }
    if cascade_changed {
      self.last_cascade_gen = Some(host_tree.cascade_generation);
      self.store.update_cascade(host_tree);
    }

    if self.mount.process(&mut self.tree) {
      self.needs_redraw = true;
    }

    let current_sel = self.store.selected_path.get();
    if current_sel != self.last_selected_path || cascade_changed {
      self.last_selected_path = current_sel;
      self.mount.force_render(&mut self.tree);
      self.needs_redraw = true;
    }

    self.store.unbind_host_tree();
    self.store.unbind_layout();
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
    self.store.hover_path.get()
  }

  pub fn is_pick_mode(&self) -> bool {
    self.store.pick_mode.load(Ordering::Relaxed)
  }

  pub fn pick_element(&mut self, path: Vec<usize>) {
    self.store.pending_pick.set(Some(path));
    self.store.hover_path.set(None);
    self.needs_redraw = true;
  }

  pub fn set_hover_path(&mut self, path: Option<Vec<usize>>) {
    self.store.hover_path.set(path);
  }
}

impl wgpu_html_driver::SecondaryWindow for Devtools {
  fn poll(&mut self, tree: &Tree) {
    self.poll_with_layout(tree, None);
  }

  fn needs_redraw(&self) -> bool {
    Devtools::needs_redraw(self)
  }
}
