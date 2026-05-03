//! Pointer event types attached to `Node`s.
//!
//! State lives on the `Tree`, mirroring the rule that fonts do. See
//! `spec/interactivity.md` §4. First slice ships only the bits the
//! demo needs: a hover path, an active path, and per-node `on_click /
//! on_mouse_{down,up,enter,leave}` callback slots.

use std::{collections::BTreeMap, sync::Arc, time::Instant};

// Re-export the full event type hierarchy so tree users only need one crate.
pub use wgpu_html_events::{HtmlEvent, HtmlEventType};

/// A caret/selection endpoint in shaped text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCursor {
  /// Path to the text `LayoutBox` in the layout/tree mirror.
  pub path: Vec<usize>,
  /// Grapheme-agnostic boundary index in that text run.
  ///
  /// `0` is before the first glyph, `glyph_count` is after the last.
  pub glyph_index: usize,
}

/// Active text selection range (anchor + focus).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSelection {
  pub anchor: TextCursor,
  pub focus: TextCursor,
}

impl TextSelection {
  pub fn is_collapsed(&self) -> bool {
    self.anchor == self.focus
  }
}

/// Caret / selection state inside a focused `<input>` or `<textarea>`.
///
/// Byte offsets are into the field's logical value string. They must
/// always sit on a `char` boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditCursor {
  /// Byte offset of the insertion caret.
  pub cursor: usize,
  /// Byte offset of the selection anchor, or `None` for a collapsed
  /// (no-selection) caret. When `Some(a)`, the selected range is
  /// `min(a, cursor)..max(a, cursor)`.
  pub selection_anchor: Option<usize>,
}

impl EditCursor {
  /// A collapsed caret at `pos` with no selection.
  pub fn collapsed(pos: usize) -> Self {
    Self {
      cursor: pos,
      selection_anchor: None,
    }
  }

  /// Whether a non-empty selection exists.
  pub fn has_selection(&self) -> bool {
    self.selection_anchor.is_some_and(|a| a != self.cursor)
  }

  /// `(start_byte, end_byte)` of the selected range, or
  /// `(cursor, cursor)` when collapsed.
  pub fn selection_range(&self) -> (usize, usize) {
    let anchor = self.selection_anchor.unwrap_or(self.cursor);
    (anchor.min(self.cursor), anchor.max(self.cursor))
  }
}

/// Host-configurable text selection paint colors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionColors {
  pub background: [f32; 4],
  pub foreground: [f32; 4],
}

impl Default for SelectionColors {
  fn default() -> Self {
    // Browser-like defaults (blue highlight, white selected text).
    Self {
      background: [0.23, 0.51, 0.96, 0.45],
      foreground: [1.0, 1.0, 1.0, 1.0],
    }
  }
}

/// Three named buttons + a numeric escape hatch for everything else
/// (winit's "additional" buttons, e.g. side-mouse keys).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
  Primary,
  Secondary,
  Middle,
  Other(u8),
}

/// Modifier bitmask captured at the time the event was dispatched.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
  pub shift: bool,
  pub ctrl: bool,
  pub alt: bool,
  pub meta: bool,
}

/// One of the four modifier keys tracked by [`Modifiers`].
///
/// Used with [`Modifiers::set`] / [`crate::Tree::set_modifier`] to
/// flip a single bit without naming the field directly. Lets host
/// code write a generic mapping from its native key code to a
/// modifier change without the engine caring how the host names
/// its keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
  Shift,
  Ctrl,
  Alt,
  Meta,
}

impl Modifiers {
  /// Set one modifier bit. The other three are unchanged.
  pub fn set(&mut self, modifier: Modifier, down: bool) {
    match modifier {
      Modifier::Shift => self.shift = down,
      Modifier::Ctrl => self.ctrl = down,
      Modifier::Alt => self.alt = down,
      Modifier::Meta => self.meta = down,
    }
  }
}

/// Payload handed to every mouse-related callback.
///
/// `target_path` is the deepest element that was hit; `current_path`
/// is the element the listener is attached to (set by the dispatcher
/// while bubbling). For move-only events without a button, `button`
/// is `None`.
#[derive(Debug, Clone)]
pub struct MouseEvent {
  pub pos: (f32, f32),
  pub button: Option<MouseButton>,
  pub modifiers: Modifiers,
  pub target_path: Vec<usize>,
  pub current_path: Vec<usize>,
}

/// Shareable, immutable callback. `Arc<dyn Fn>` keeps `Tree: Clone`
/// without forcing closures through `Box`. Use interior mutability
/// (e.g. `Arc<AtomicUsize>`, `Arc<Mutex<T>>`) for mutable state.
pub type MouseCallback = Arc<dyn Fn(&MouseEvent) + Send + Sync + 'static>;

/// General-purpose event callback that receives the full [`HtmlEvent`].
///
/// Attached to `Node::on_event`; fired for *every* event that reaches the
/// node (after the type-specific slot, if both are wired). Prefer
/// `on_click` / `on_mouse_down` etc. for ordinary pointer work; use this
/// for keyboard, focus, wheel, or other events that have no dedicated slot.
pub type EventCallback = Arc<dyn Fn(&HtmlEvent) + Send + Sync + 'static>;

/// Per-document interaction state. Reset on document reload.
#[derive(Debug, Clone)]
pub struct InteractionState {
  /// Path to the deepest element currently under the pointer, or
  /// `None` if the pointer is outside the document or never moved.
  pub hover_path: Option<Vec<usize>>,
  /// Path to the element that received the most recent primary
  /// press and has not yet seen its release.
  pub active_path: Option<Vec<usize>>,
  /// Path to the element that currently has keyboard focus, or
  /// `None` if no element is focused. The dispatcher in
  /// `wgpu_html::interactivity` is the only writer; the cascade
  /// reads it via `MatchContext::for_path` to resolve `:focus`.
  ///
  /// `:focus` matches only the exact path stored here; it does
  /// not propagate to ancestors (unlike `:hover`). `:focus-within`
  /// is not implemented.
  pub focus_path: Option<Vec<usize>>,
  /// Last known pointer position in physical pixels.
  pub pointer_pos: Option<(f32, f32)>,
  /// Current text selection, if any.
  pub selection: Option<TextSelection>,
  /// Whether a primary-button drag currently owns text selection.
  pub selecting_text: bool,
  /// Colors used to paint selected text/background.
  pub selection_colors: SelectionColors,
  /// Vertical scroll offsets keyed by layout/tree child-index path.
  pub scroll_offsets_y: BTreeMap<Vec<usize>, f32>,
  /// Instant at which the document interaction state was created.
  /// Used to compute `Event::time_stamp` (milliseconds since origin,
  /// matching `performance.now()` semantics).
  pub time_origin: Instant,
  /// DOM-style bitmask of mouse buttons currently held down.
  ///
  /// Bit 0 = primary, bit 1 = secondary, bit 2 = middle,
  /// bit 3/4 = back/forward (matches the W3C `MouseEvent.buttons` spec).
  pub buttons_down: u16,
  /// Currently-held modifier keys. Hosts update this through
  /// [`crate::Tree::set_modifier`] (or by writing the field
  /// directly); dispatchers read it when they fire events, so
  /// callers no longer have to thread `Modifiers` through every
  /// call.
  pub modifiers: Modifiers,
  /// Caret/selection state inside the currently focused `<input>` or
  /// `<textarea>`. `None` when focus is not on a text-editable control.
  /// Written by the text-edit dispatcher; read by layout (for scroll-
  /// into-view) and paint (for caret + selection highlight).
  pub edit_cursor: Option<EditCursor>,
  /// Instant of last cursor movement or value edit. Paint uses this
  /// to animate caret blink (500 ms on, 500 ms off, restarting on
  /// every mutation so the caret stays visible while typing).
  pub caret_blink_epoch: Instant,
  /// Snapshot of the text-editable value at the moment the element
  /// gained focus. Compared with the current value on `blur` to
  /// decide whether to fire a `change` event.
  pub focus_value_snapshot: Option<String>,
}

impl Default for InteractionState {
  fn default() -> Self {
    Self {
      hover_path: None,
      active_path: None,
      focus_path: None,
      pointer_pos: None,
      selection: None,
      selecting_text: false,
      selection_colors: SelectionColors::default(),
      scroll_offsets_y: BTreeMap::new(),
      time_origin: Instant::now(),
      buttons_down: 0,
      modifiers: Modifiers::default(),
      edit_cursor: None,
      caret_blink_epoch: Instant::now(),
      focus_value_snapshot: None,
    }
  }
}

/// Lightweight snapshot of the `InteractionState` fields that affect
/// cascade output (pseudo-class resolution). Two snapshots are equal
/// iff the cascade would produce identical `CascadedTree`s for the
/// same DOM tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionSnapshot {
  pub hover_path: Option<Vec<usize>>,
  pub active_path: Option<Vec<usize>>,
  pub focus_path: Option<Vec<usize>>,
}

impl InteractionState {
  /// Capture the fields that affect cascade (pseudo-class resolution).
  pub fn cascade_snapshot(&self) -> InteractionSnapshot {
    InteractionSnapshot {
      hover_path: self.hover_path.clone(),
      active_path: self.active_path.clone(),
      focus_path: self.focus_path.clone(),
    }
  }
}
