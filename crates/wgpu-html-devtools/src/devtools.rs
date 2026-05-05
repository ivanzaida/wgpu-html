//! Platform-agnostic devtools inspector.
//!
//! `Devtools` manages a secondary OS window for inspecting the host
//! tree. It opens automatically on the first frame after `attach()`
//! and runs on the host's event loop (winit only allows one per
//! process).

use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use wgpu_html_driver_winit::{WinitRuntime, dispatch, new_window, register_system_fonts};
use wgpu_html_tree::{FontFace, Profiler, Tree, TreeHookHandle};
use wgpu_html_ui::Mount;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::ui::{DevtoolsComponent, DevtoolsProps, SharedHoverPath, SharedPendingPick, SharedPickMode};

/// Lucide icon font embedded at compile time (ISC license).
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

/// CSS for the devtools UI (inline — no external file dependency).
const CSS: &str = r#"
:root {
    --bg-primary: #202124;
    --bg-secondary: #292A2D;
    --bg-tertiary: #35363A;
    --bg-hover: #3C4043;
    --bg-selected: #073655;
    --bg-selected-hover: #0A4166;
    --border: #3C4043;
    --divider: #5F6368;
    --text-primary: #E8EAED;
    --text-secondary: #9AA0A6;
    --text-muted: #5F6368;
    --accent-blue: #8AB4F8;
    --tag-color: #5DB0D7;
    --attr-name: #9AA0A6;
    --attr-value: #F28B82;
    --selector: #D2E3FC;
    --property: #9AA0A6;
    --value: #FF8BCB;
    --unit: #FDD663;
}
html, body { height: 100%; }
body {
    margin: 0;
    font-family: sans-serif;
    font-size: 11px;
    background: var(--bg-primary);
    color: var(--text-primary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
.devtools-root {
    display: flex;
    flex-direction: column;
    flex-grow: 1;
    min-height: 0;
}
.toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
}
.icon { font-family: lucide; }
.pick-btn { color: var(--text-secondary); font-size: 14px; font-family: lucide; cursor: pointer; }
.pick-btn:hover { color: var(--text-primary); }
.pick-active { color: var(--accent-blue); }
.tb-divider { width: 1px; height: 16px; background: var(--divider); }
.filter {
    display: flex; align-items: center; gap: 6px; height: 22px;
    padding: 0 8px; background: var(--bg-tertiary); border-radius: 3px; width: 200px;
}
.filter-icon { color: var(--text-muted); font-family: lucide; font-size: 12px; }
.filter-text { color: var(--text-muted); font-size: 11px; }
.main { display: flex; flex-grow: 1; min-height: 0; }
.tree-panel {
    display: flex; flex-direction: column; width: 50%; min-width: 0;
    flex-shrink: 0;
    background: var(--bg-primary);
}
.divider {
    width: 4px; flex-shrink: 0; cursor: col-resize;
    background: var(--border);
}
.divider:hover { background: var(--accent-blue); }
.tree-rows {
    flex-grow: 1; display: flex; flex-direction: column;
    padding: 8px 0; overflow: auto; min-width: 0;
}
.tree-row {
    display: flex; align-items: center; height: 18px; flex-shrink: 0;
    padding-right: 8px; white-space: nowrap; overflow: hidden; cursor: default;
}
.tree-row:hover { background-color: var(--bg-hover); }
.tree-row-selected { background-color: var(--bg-selected); }
.tree-row-selected:hover { background-color: var(--bg-selected-hover); }
.chevron { color: var(--text-secondary); font-family: lucide; font-size: 10px; width: 12px; margin-right: 2px; flex-shrink: 0; }
.tag { color: var(--tag-color); }
.bracket { color: var(--text-secondary); }
.attr-n { margin-left: 4px; color: var(--attr-name); }
.attr-v { color: var(--attr-value); }
.text-node { color: var(--text-muted); font-style: italic; }
.breadcrumb {
    display: flex; align-items: center; gap: 4px; height: 22px;
    padding: 0 12px; background: var(--bg-secondary);
    border-top: 1px solid var(--border); flex-shrink: 0;
    font-size: 10px; color: var(--text-muted); white-space: nowrap; overflow: hidden;
}
.bc-active { color: var(--accent-blue); font-weight: 600; }
.styles-panel {
    display: flex; flex-direction: column; flex-grow: 1;
    background: var(--bg-primary); min-width: 0;
}
.tab-bar {
    display: flex; align-items: center; gap: 16px; height: 28px;
    padding: 0 12px; background: var(--bg-secondary);
    border-bottom: 1px solid var(--border); flex-shrink: 0;
}
.tab {
    height: 100%; display: flex; align-items: center; padding: 0 4px;
    color: var(--text-secondary); font-size: 11px; font-weight: 500;
    cursor: default; border-bottom: 2px solid transparent;
}
.tab:hover { color: var(--text-primary); }
.tab-active { color: var(--text-primary); font-weight: 600; border-bottom-color: var(--accent-blue); }
.style-search {
    display: flex; align-items: center; gap: 8px; height: 28px;
    padding: 0 12px; background: var(--bg-secondary);
    border-bottom: 1px solid var(--border); flex-shrink: 0;
}
.ss-label { color: var(--text-muted); font-size: 11px; }
.ss-spacer { flex-grow: 1; }
.ss-btn {
    padding: 0 6px; height: 18px; display: flex; align-items: center;
    color: var(--text-secondary); font-size: 10px; border-radius: 3px; cursor: default;
}
.ss-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
.ss-btn-active { background: var(--bg-tertiary); color: var(--accent-blue); }
.styles-content { flex-grow: 1; display: flex; flex-direction: column; overflow: auto; }
.rule { display: flex; flex-direction: column; border-bottom: 1px solid var(--border); cursor: default; flex-shrink: 0; }
.rule:hover { background: rgba(255, 255, 255, 0.02); }
.rule-header { display: flex; align-items: center; gap: 6px; height: 22px; padding: 0 12px; }
.selector-text { color: var(--selector); }
.brace { color: var(--text-secondary); }
.decl {
    display: flex; align-items: center; height: 18px;
    padding: 0 12px 0 28px; white-space: nowrap; overflow: hidden; cursor: default;
}
.decl:hover { background: var(--bg-hover); }
.cb { width: 10px; height: 10px; border: 1px solid var(--text-muted); border-radius: 2px; margin-right: 6px; flex-shrink: 0; }
.prop { color: var(--property); }
.colon { color: var(--text-secondary); }
.val { color: var(--value); }
.semi { color: var(--text-secondary); }
.rule-end { height: 18px; padding: 0 12px; display: flex; align-items: center; color: var(--text-secondary); }
"#;

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
}

impl Devtools {
  /// Create a devtools instance without attaching to a tree.
  pub fn new(enable_profiler: bool) -> Self {
    let mut tree = Tree::default();
    let lucide = FontFace::regular("lucide", Arc::from(LUCIDE_FONT));
    tree.register_font(lucide);
    tree.register_linked_stylesheet("devtools.css", CSS);
    register_system_fonts(&mut tree, "sans-serif");

    if enable_profiler {
      tree.profiler = Some(Profiler::tagged("devtools"));
    }

    let shared_hover: SharedHoverPath = Arc::new(std::sync::Mutex::new(None));
    let shared_pick_mode: SharedPickMode = Arc::new(AtomicBool::new(false));
    let shared_pending_pick: SharedPendingPick = Arc::new(std::sync::Mutex::new(None));
    let mount = Mount::<DevtoolsComponent>::new(DevtoolsProps {
      shared_hover: shared_hover.clone(),
      shared_pick_mode: shared_pick_mode.clone(),
      shared_pending_pick: shared_pending_pick.clone(),
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

    // Register F11 hook on the host tree.
    let toggle_flag = devtools.toggle_requested.clone();
    host_tree
      .hooks
      .push(TreeHookHandle::new(DevtoolsKeyHook { toggle_flag }));

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

    // Process pending component messages.
    if self.mount.process(&mut self.tree, host_tree) {
      self.needs_redraw = true;
    }

    // Re-render if the host tree changed.
    let dom_changed = self.last_inspected_gen != Some(host_tree.generation);
    if dom_changed {
      self.last_inspected_gen = Some(host_tree.generation);
      self.mount.force_render(&mut self.tree, host_tree);
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

    // Drain any messages queued by callbacks.
    if self.mount.process(&mut self.tree, host_tree) {
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
