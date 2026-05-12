//! Tree-level keyboard hook for the devtools window.
//!
//! Shift+F12: dump the devtools tree as HTML for browser debugging.

use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use lui_tree::{Tree, TreeHook, TreeHookResponse};

/// Shared flag checked by [`super::Devtools::poll`] to trigger the
/// HTML dump outside the hook (which only has `&mut Tree`, not the
/// full `Devtools` state).
pub(crate) struct DumpHtmlHook {
  pub flag: Arc<AtomicBool>,
}

impl TreeHook for DumpHtmlHook {
  fn on_keyboard_event(&mut self, _tree: &mut Tree, event: &mut lui_events::events::KeyboardEvent) -> TreeHookResponse {
    let is_keydown = event.base.base.event_type.as_str() == "keydown";
    if event.code == "F12" && event.shift_key && is_keydown && !event.repeat {
      eprintln!("[devtools-hook] Shift+F12 detected, setting dump flag");
      self.flag.store(true, Ordering::Relaxed);
    }
    TreeHookResponse::Continue
  }
}
