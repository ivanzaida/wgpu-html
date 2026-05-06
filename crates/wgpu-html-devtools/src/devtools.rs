//! Platform-agnostic devtools inspector.
//!
//! `Devtools` manages a secondary OS window for inspecting the host
//! tree. It opens automatically on the first frame after `attach()`
//! and runs on the host's event loop (winit only allows one per
//! process).

use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use wgpu_html_driver_winit::{dispatch, new_window, register_system_fonts, WinitRuntime};
use wgpu_html_tree::{FontFace, Profiler, Tree, TreeHookHandle};
use wgpu_html_ui::Mount;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::ui::{DevtoolsComponent, DevtoolsProps, SharedHostTree, SharedHoverPath, SharedPendingPick, SharedPickMode};

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── Devtools ─────────────────────────────────────────────────────

/// Devtools inspector.
///
/// Owns its own UI [`Tree`] and delegates rendering and input
/// handling to an [`HtmlWindow`] when active. The UI is produced
/// by [`DevtoolsComponent`] via the `wgpu-html-ui` framework.
///
/// # Usage
///
/// ```ignore
/// // In setup:
/// let devtools = Devtools::attach(&mut tree, false);
///
/// // Each frame (in on_frame hook):
/// devtools.poll(ctx.tree, ctx.event_loop);
///
/// // Route window events (in on_window_event hook):
/// if devtools.owns_window(window_id) {
///     devtools.handle_window_event(ctx.tree, event);
/// }
/// ```
pub struct Devtools {
  tree: Tree,
  mount: Mount<DevtoolsComponent>,

  last_inspected_gen: Option<u64>,
  needs_redraw: bool,

  html_window: Option<WinitRuntime>,
  enabled: bool,
  toggle_requested: Arc<AtomicBool>,
  dump_html_requested: Arc<AtomicBool>,
  shared_hover: SharedHoverPath,
  shared_pick_mode: SharedPickMode,
  shared_pending_pick: SharedPendingPick,
  shared_host_tree: SharedHostTree,
}

impl Devtools {
  /// Create a devtools instance without attaching to a tree.
  pub fn new(enable_profiler: bool) -> Self {
    let mut tree = Tree::default();
    let lucide = FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
    tree.register_font(lucide);
    register_system_fonts(&mut tree, "sans-serif");

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
      html_window: None,
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
  /// fonts. Opens automatically on the first `poll()` call.
  ///
  /// Registers an F11 keyboard hook on the host tree so the devtools
  /// window can be toggled without the host needing to forward keys.
  pub fn attach(host_tree: &mut Tree, enable_profiler: bool) -> Self {
    let mut devtools = Self::new(enable_profiler);
    for (_handle, face) in host_tree.fonts.iter() {
      devtools.tree.register_font(face.clone());
    }
    // Auto-enable so it opens on first poll.
    devtools.enabled = true;

 
    host_tree.root.as_mut().unwrap().on_keyup.push(Arc::new(|e| {
      println!("keyup on host tree")
    }));

    devtools
  }

  // ── Font registration ───────────────────────────────────

  pub fn register_font(&mut self, face: FontFace) {
    self.tree.register_font(face);
  }

  // ── Polling ─────────────────────────────────────────────

  /// Sync with the host tree and manage the window lifecycle.
  /// Call once per frame from the host's `on_frame` hook.
  ///
  /// If the window hasn't been created yet and devtools is enabled,
  /// it will be opened using `event_loop`.
  pub fn poll(&mut self, host_tree: &Tree, event_loop: &ActiveEventLoop) {
    // Check if the host tree hook requested a toggle (F11).
    if self.toggle_requested.swap(false, Ordering::Relaxed) {
      self.toggle();
    }

    if !self.enabled {
      return;
    }

    // Lazily create the window on first enable.
    if self.html_window.is_none() {
      let hw = new_window(event_loop, "DevTools", 1280, 720);
      hw.driver.window.request_redraw();
      self.html_window = Some(hw);
      self.last_inspected_gen = None;
      self.needs_redraw = true;
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

    // Update the shared host tree reference so the shell component
    // can read it during view().
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

    if self.needs_redraw {
      if let Some(rt) = &self.html_window {
        rt.driver.window.request_redraw();
      }
    }
  }

  // ── Window lifecycle ────────────────────────────────────

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

  pub fn is_enabled(&self) -> bool {
    self.enabled
  }

  pub fn enable(&mut self) {
    self.enabled = true;
    if let Some(rt) = &self.html_window {
      rt.driver.window.set_visible(true);
      rt.driver.window.request_redraw();
    }
  }

  pub fn disable(&mut self) {
    self.enabled = false;
    if let Some(rt) = &self.html_window {
      rt.driver.window.set_visible(false);
    }
  }

  pub fn toggle(&mut self) {
    if self.enabled {
      self.disable();
    } else {
      self.enable();
    }
  }

  pub fn window_id(&self) -> Option<WindowId> {
    self.html_window.as_ref().map(|rt| rt.driver.window.id())
  }

  pub fn owns_window(&self, id: WindowId) -> bool {
    self.window_id() == Some(id)
  }

  /// Handle a winit `WindowEvent` for the devtools window.
  pub fn handle_window_event(&mut self, host_tree: &Tree, event: &WindowEvent) {
    let Some(rt) = self.html_window.as_mut() else {
      return;
    };
    match event {
      WindowEvent::CloseRequested => {
        self.disable();
        return;
      }
      WindowEvent::RedrawRequested => {
        self.render_to_window();
        return;
      }
      _ => {}
    }
    let needs_redraw = dispatch(event, rt, &mut self.tree);

    // Update shared host tree so view() has fresh data.
    {
      let mut ht = self.shared_host_tree.write().unwrap();
      *ht = Some(host_tree.clone());
    }

    // Drain any messages queued by callbacks.
    if self.mount.process(&mut self.tree) {
      self.needs_redraw = true;
    }

    // Check Shift+F12 dump flag.
    if self.dump_html_requested.swap(false, Ordering::Relaxed) {
      let html = self.tree.to_html();
      let path = format!("devtools-dump-{}.html", self.tree.generation);
      match std::fs::write(&path, &html) {
        Ok(()) => println!("[devtools] saved HTML \u{2192} {path} ({} bytes)", html.len()),
        Err(e) => eprintln!("[devtools] failed to save HTML: {e}"),
      }
    }

    if needs_redraw || self.needs_redraw {
      rt.driver.window.request_redraw();
    }
  }

  fn render_to_window(&mut self) {
    if let Some(rt) = self.html_window.as_mut() {
      rt.render_frame(&mut self.tree);
    }
    self.needs_redraw = false;
  }
}

// ── F11 TreeHook ────────────────────────────────────────────────

struct DevtoolsKeyHook {
  toggle_flag: Arc<AtomicBool>,
}

impl wgpu_html_tree::TreeHook for DevtoolsKeyHook {
  fn on_keyboard_event(
    &mut self,
    _tree: &mut Tree,
    event: &mut wgpu_html_events::events::KeyboardEvent,
  ) -> wgpu_html_tree::TreeHookResponse {
    // Only toggle on keydown, not keyup (both fire through this hook).
    let is_keydown = event.base.base.event_type.as_str() == "keydown";
    if event.code == "F11" && is_keydown && !event.repeat {
      self.toggle_flag.store(true, Ordering::Relaxed);
    }
    wgpu_html_tree::TreeHookResponse::Continue
  }
}

// ── SecondaryWindow impl ────────────────────────────────────────

impl wgpu_html_ui::SecondaryWindow for Devtools {
  fn poll(&mut self, tree: &Tree, event_loop: &ActiveEventLoop) {
    Devtools::poll(self, tree, event_loop);
  }

  fn on_key(&mut self, _tree: &Tree, _event_loop: &ActiveEventLoop, _event: &winit::event::KeyEvent) -> bool {
    // F11 is handled by the TreeHook → toggle_requested flag → poll().
    // No action needed here.
    false
  }

  fn owns_window(&self, id: WindowId) -> bool {
    Devtools::owns_window(self, id)
  }

  fn handle_window_event(&mut self, tree: &Tree, event: &WindowEvent) {
    Devtools::handle_window_event(self, tree, event);
  }
}
