//! Pointer event types attached to `Node`s.
//!
//! State lives on the `Tree`, mirroring the rule that fonts do. See
//! `spec/interactivity.md` §4. First slice ships only the bits the
//! demo needs: a hover path, an active path, and per-node `on_click /
//! on_mouse_{down,up,enter,leave}` callback slots.

use std::sync::Arc;

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

/// Per-document interaction state. Reset on document reload.
#[derive(Debug, Clone, Default)]
pub struct InteractionState {
    /// Path to the deepest element currently under the pointer, or
    /// `None` if the pointer is outside the document or never moved.
    pub hover_path: Option<Vec<usize>>,
    /// Path to the element that received the most recent primary
    /// press and has not yet seen its release.
    pub active_path: Option<Vec<usize>>,
    /// Last known pointer position in physical pixels.
    pub pointer_pos: Option<(f32, f32)>,
}
