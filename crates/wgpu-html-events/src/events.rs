//! All DOM event structs, modelling the W3C event hierarchy via composition.
//!
//! Each event that "extends" a parent event embeds it in a `base` field so
//! every field on the ancestor is trivially reachable without trait dispatch:
//!
//! ```
//! # use wgpu_html_events::events::MouseEvent;
//! # fn demo(ev: &MouseEvent) {
//! let _ = ev.base.base.bubbles;   // Event field through UIEvent
//! let _ = ev.base.detail;         // UIEvent field
//! let _ = ev.client_x;            // MouseEvent field
//! # }
//! ```

use crate::enums::{
    EventPhase, InputType, KeyboardLocation, PointerType, ToggleState, WheelDeltaMode,
};
use crate::{ClipboardDataId, DataTransferId, FormDataId, HtmlEventType, NodeId};

// ── Event ────────────────────────────────────────────────────────────────────

/// The base DOM `Event` interface.
///
/// All other event types embed this struct as `base`.
#[derive(Debug, Clone)]
pub struct Event {
    /// The name of this event (e.g. `"click"`, `"keydown"`).
    pub event_type: HtmlEventType,
    /// Whether the event bubbles up through the DOM tree.
    pub bubbles: bool,
    /// Whether `prevent_default()` can be called on this event.
    pub cancelable: bool,
    /// Whether the event propagates across shadow DOM boundaries.
    pub composed: bool,
    /// The element that originally dispatched the event.
    pub target: Option<NodeId>,
    /// The element whose listener is currently being invoked.
    pub current_target: Option<NodeId>,
    /// Current propagation phase.
    pub event_phase: EventPhase,
    /// Whether `prevent_default()` has been called.
    pub default_prevented: bool,
    /// Whether the event was dispatched by the user agent (not script).
    pub is_trusted: bool,
    /// Time (in milliseconds since the time origin) at which the event was created.
    pub time_stamp: f64,
}

// ── UIEvent ──────────────────────────────────────────────────────────────────

/// Extends `Event`. Base for all user-interface events.
#[derive(Debug, Clone)]
pub struct UIEvent {
    pub base: Event,
    /// Event-specific detail value (e.g. click count for `"click"`).
    pub detail: i32,
}

// ── MouseEvent ───────────────────────────────────────────────────────────────

/// Extends `UIEvent`. Fired for pointer-device interactions.
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub base: UIEvent,
    /// X coordinate relative to the top-left of the screen.
    pub screen_x: f64,
    /// Y coordinate relative to the top-left of the screen.
    pub screen_y: f64,
    /// X coordinate relative to the top-left of the viewport.
    pub client_x: f64,
    /// Y coordinate relative to the top-left of the viewport.
    pub client_y: f64,
    /// X coordinate relative to the target element's padding edge.
    pub offset_x: f64,
    /// Y coordinate relative to the target element's padding edge.
    pub offset_y: f64,
    /// X coordinate relative to the left edge of the document.
    pub page_x: f64,
    /// Y coordinate relative to the top edge of the document.
    pub page_y: f64,
    /// Delta X from the previous `mousemove` event.
    pub movement_x: f64,
    /// Delta Y from the previous `mousemove` event.
    pub movement_y: f64,
    /// Which button was pressed/released (0 = primary, 1 = middle, 2 = secondary).
    pub button: i16,
    /// Bitmask of all buttons currently pressed.
    pub buttons: u16,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    /// For `mouseenter`/`mouseleave`/`mouseover`/`mouseout`: the element
    /// the pointer entered from or moved to.
    pub related_target: Option<NodeId>,
}

// ── PointerEvent ─────────────────────────────────────────────────────────────

/// Extends `MouseEvent`. Unified pointer-device events (mouse, touch, pen).
#[derive(Debug, Clone)]
pub struct PointerEvent {
    pub base: MouseEvent,
    /// Unique identifier for this pointer.
    pub pointer_id: i32,
    /// Width of the contact geometry in CSS pixels.
    pub width: f64,
    /// Height of the contact geometry in CSS pixels.
    pub height: f64,
    /// Normalized pressure in the range `[0, 1]`.
    pub pressure: f64,
    /// Normalized tangential (barrel) pressure in the range `[-1, 1]`.
    pub tangential_pressure: f64,
    /// Plane angle (degrees, `[-90, 90]`) between the Y–Z plane and the plane
    /// containing the pointer axis and the Y axis.
    pub tilt_x: i32,
    /// Plane angle (degrees, `[-90, 90]`) between the X–Z plane and the plane
    /// containing the pointer axis and the X axis.
    pub tilt_y: i32,
    /// Clockwise rotation (degrees, `[0, 359]`) of the pointer.
    pub twist: i32,
    /// The kind of device that produced this event.
    pub pointer_type: PointerType,
    /// Whether this pointer is the primary pointer for its type.
    pub is_primary: bool,
}

// ── WheelEvent ───────────────────────────────────────────────────────────────

/// Extends `MouseEvent`. Fired when the wheel button is rotated.
#[derive(Debug, Clone)]
pub struct WheelEvent {
    pub base: MouseEvent,
    /// Horizontal scroll delta.
    pub delta_x: f64,
    /// Vertical scroll delta.
    pub delta_y: f64,
    /// Z-axis scroll delta.
    pub delta_z: f64,
    /// Unit in which deltas are expressed.
    pub delta_mode: WheelDeltaMode,
}

// ── KeyboardEvent ─────────────────────────────────────────────────────────────

/// Extends `UIEvent`. Fired when a key is pressed or released.
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub base: UIEvent,
    /// The printed representation of the key (e.g. `"a"`, `"Enter"`, `"ArrowRight"`).
    pub key: String,
    /// A physical key code independent of keyboard layout (e.g. `"KeyA"`, `"ArrowRight"`).
    pub code: String,
    /// Which part of the keyboard generated the event.
    pub location: KeyboardLocation,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    /// `true` if the key is being held down so that it auto-repeats.
    pub repeat: bool,
    /// `true` if the event is fired within a composition session.
    pub is_composing: bool,
}

// ── FocusEvent ───────────────────────────────────────────────────────────────

/// Extends `UIEvent`. Fired when an element gains or loses focus.
#[derive(Debug, Clone)]
pub struct FocusEvent {
    pub base: UIEvent,
    /// For `focusin`/`focusout`: the element losing/gaining focus.
    pub related_target: Option<NodeId>,
}

// ── InputEvent ────────────────────────────────────────────────────────────────

/// Extends `UIEvent`. Fired when the value of an editable element changes.
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub base: UIEvent,
    /// The inserted/deleted characters, or `None` for non-character inputs.
    pub data: Option<String>,
    /// The kind of change that produced this event.
    pub input_type: InputType,
    /// `true` if the event is fired within a composition session.
    pub is_composing: bool,
}

// ── CompositionEvent ──────────────────────────────────────────────────────────

/// Extends `UIEvent`. Fired during IME composition.
#[derive(Debug, Clone)]
pub struct CompositionEvent {
    pub base: UIEvent,
    /// The current composition string.
    pub data: String,
}

// ── ClipboardEvent ────────────────────────────────────────────────────────────

/// Extends `Event`. Fired for cut/copy/paste operations.
#[derive(Debug, Clone)]
pub struct ClipboardEvent {
    pub base: Event,
    /// Handle to the underlying clipboard data object, if available.
    pub clipboard_data: Option<ClipboardDataId>,
}

// ── DragEvent ─────────────────────────────────────────────────────────────────

/// Extends `MouseEvent`. Fired during drag-and-drop interactions.
#[derive(Debug, Clone)]
pub struct DragEvent {
    pub base: MouseEvent,
    /// Handle to the drag data store, if available.
    pub data_transfer: Option<DataTransferId>,
}

// ── Touch / TouchEvent ────────────────────────────────────────────────────────

/// Data for a single contact point in a `TouchEvent`.
#[derive(Debug, Clone)]
pub struct Touch {
    /// Unique identifier for this touch point.
    pub identifier: i32,
    /// The element this touch point started over.
    pub target: NodeId,
    pub screen_x: f64,
    pub screen_y: f64,
    pub client_x: f64,
    pub client_y: f64,
    pub page_x: f64,
    pub page_y: f64,
    /// Horizontal radius of the ellipse that most closely circumscribes the
    /// contact area, in CSS pixels.
    pub radius_x: f64,
    /// Vertical radius of the ellipse.
    pub radius_y: f64,
    /// Rotation angle (degrees, `[0, 90]`) of the contact ellipse.
    pub rotation_angle: f64,
    /// Normalized pressure in the range `[0, 1]`.
    pub force: f64,
}

/// Extends `UIEvent`. Fired when a touch surface is touched or lifted.
#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub base: UIEvent,
    /// All touches currently on the surface.
    pub touches: Vec<Touch>,
    /// All touches on the same target element.
    pub target_touches: Vec<Touch>,
    /// Touches whose state changed for this event.
    pub changed_touches: Vec<Touch>,
    pub ctrl_key: bool,
    pub shift_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

// ── AnimationEvent ────────────────────────────────────────────────────────────

/// Extends `Event`. Fired at milestones of a CSS animation.
#[derive(Debug, Clone)]
pub struct AnimationEvent {
    pub base: Event,
    /// The value of the `animation-name` property.
    pub animation_name: String,
    /// Elapsed time in seconds since the animation started.
    pub elapsed_time: f64,
    /// The pseudo-element selector (e.g. `"::before"`) that generated this
    /// event, or `None` if the event is for the element itself.
    pub pseudo_element: Option<String>,
}

// ── TransitionEvent ───────────────────────────────────────────────────────────

/// Extends `Event`. Fired at milestones of a CSS transition.
#[derive(Debug, Clone)]
pub struct TransitionEvent {
    pub base: Event,
    /// The CSS property whose transition triggered this event.
    pub property_name: String,
    /// Elapsed time in seconds since the transition started.
    pub elapsed_time: f64,
    /// The pseudo-element selector that generated this event, or `None`.
    pub pseudo_element: Option<String>,
}

// ── SubmitEvent ───────────────────────────────────────────────────────────────

/// Extends `Event`. Fired when a `<form>` is submitted.
#[derive(Debug, Clone)]
pub struct SubmitEvent {
    pub base: Event,
    /// The element that triggered the submission (e.g. a submit `<button>`).
    pub submitter: Option<NodeId>,
}

// ── FormDataEvent ─────────────────────────────────────────────────────────────

/// Extends `Event`. Fired after the `FormData` object is constructed.
#[derive(Debug, Clone)]
pub struct FormDataEvent {
    pub base: Event,
    /// Handle to the `FormData` object being built.
    pub form_data: FormDataId,
}

// ── ToggleEvent / BeforeToggleEvent ───────────────────────────────────────────

/// Extends `Event`. Fired when a `<details>` element is opened or closed.
#[derive(Debug, Clone)]
pub struct ToggleEvent {
    pub base: Event,
    /// The previous state.
    pub old_state: ToggleState,
    /// The new (current) state.
    pub new_state: ToggleState,
}

/// Extends `ToggleEvent`. Fired *before* a `<details>` toggles, allowing
/// cancellation.
///
/// No additional fields beyond `ToggleEvent`.
#[derive(Debug, Clone)]
pub struct BeforeToggleEvent {
    pub base: ToggleEvent,
}

// ── ProgressEvent ─────────────────────────────────────────────────────────────

/// Extends `Event`. Fired to report progress of a long-running operation
/// (e.g. resource loading).
#[derive(Debug, Clone)]
pub struct ProgressEvent {
    pub base: Event,
    /// `true` if the total size is known.
    pub length_computable: bool,
    /// Amount of work already done.
    pub loaded: u64,
    /// Total amount of work, valid only when `length_computable` is `true`.
    pub total: u64,
}
