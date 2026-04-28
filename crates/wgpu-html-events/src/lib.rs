//! Full DOM event hierarchy for `wgpu-html`.
//!
//! # Architecture
//!
//! Every DOM event type is represented as a plain Rust struct. Inheritance is
//! modelled via **composition**: each event struct embeds its parent as a
//! `base` field, so any ancestor field is reachable without trait dispatch.
//!
//! ```
//! # use wgpu_html_events::events::MouseEvent;
//! # fn demo(ev: &MouseEvent) {
//! let _ = ev.base.base.bubbles;   // Event field through UIEvent
//! let _ = ev.base.detail;         // UIEvent field
//! let _ = ev.client_x;            // MouseEvent's own field
//! # }
//! ```
//!
//! # Modules
//!
//! - [`enums`] — `EventPhase`, `PointerType`, `WheelDeltaMode`,
//!   `KeyboardLocation`, `ToggleState`, `InputType`.
//! - [`events`] — every event struct in the hierarchy.
//!
//! # Opaque handle types
//!
//! `ClipboardDataId`, `DataTransferId`, and `FormDataId` are newtype wrappers
//! around `u64`. The host allocates these and hands them to events; the events
//! crate never inspects the payload.

pub mod enums;
pub mod events;

pub use enums::*;
pub use events::*;

// ── Core identity types ───────────────────────────────────────────────────────

/// Node address as a child-index path from the tree root.
///
/// A `NodeId` of `vec![1, 0, 3]` means: second child of root → first child of
/// that → fourth child of that. This mirrors the path convention used throughout
/// `wgpu-html-tree`.
pub type NodeId = Vec<usize>;

/// The string name of a DOM event (e.g. `"click"`, `"keydown"`).
///
/// Well-known names are available as associated constants:
///
/// ```
/// # use wgpu_html_events::HtmlEventType;
/// let t = HtmlEventType::from(HtmlEventType::CLICK);
/// assert_eq!(t.as_str(), "click");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HtmlEventType(String);

impl HtmlEventType {
    /// Construct an event type from any string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Borrow the event type string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Well-known DOM event type name constants.
impl HtmlEventType {
    pub const ABORT: &'static str = "abort";
    pub const ANIMATIONCANCEL: &'static str = "animationcancel";
    pub const ANIMATIONEND: &'static str = "animationend";
    pub const ANIMATIONITERATION: &'static str = "animationiteration";
    pub const ANIMATIONSTART: &'static str = "animationstart";
    pub const AUXCLICK: &'static str = "auxclick";
    pub const BEFOREINPUT: &'static str = "beforeinput";
    pub const BEFORETOGGLE: &'static str = "beforetoggle";
    pub const BLUR: &'static str = "blur";
    pub const CANCEL: &'static str = "cancel";
    pub const CHANGE: &'static str = "change";
    pub const CLICK: &'static str = "click";
    pub const CLOSE: &'static str = "close";
    pub const COMPOSITIONEND: &'static str = "compositionend";
    pub const COMPOSITIONSTART: &'static str = "compositionstart";
    pub const COMPOSITIONUPDATE: &'static str = "compositionupdate";
    pub const CONTEXTMENU: &'static str = "contextmenu";
    pub const COPY: &'static str = "copy";
    pub const CUT: &'static str = "cut";
    pub const DBLCLICK: &'static str = "dblclick";
    pub const DRAG: &'static str = "drag";
    pub const DRAGEND: &'static str = "dragend";
    pub const DRAGENTER: &'static str = "dragenter";
    pub const DRAGEXIT: &'static str = "dragexit";
    pub const DRAGLEAVE: &'static str = "dragleave";
    pub const DRAGOVER: &'static str = "dragover";
    pub const DRAGSTART: &'static str = "dragstart";
    pub const DROP: &'static str = "drop";
    pub const ERROR: &'static str = "error";
    pub const FOCUS: &'static str = "focus";
    pub const FOCUSIN: &'static str = "focusin";
    pub const FOCUSOUT: &'static str = "focusout";
    pub const FORMDATA: &'static str = "formdata";
    pub const GOTPOINTERCAPTURE: &'static str = "gotpointercapture";
    pub const INPUT: &'static str = "input";
    pub const INVALID: &'static str = "invalid";
    pub const KEYDOWN: &'static str = "keydown";
    pub const KEYUP: &'static str = "keyup";
    pub const LOAD: &'static str = "load";
    pub const LOADEND: &'static str = "loadend";
    pub const LOADSTART: &'static str = "loadstart";
    pub const LOSTPOINTERCAPTURE: &'static str = "lostpointercapture";
    pub const MOUSEDOWN: &'static str = "mousedown";
    pub const MOUSEENTER: &'static str = "mouseenter";
    pub const MOUSELEAVE: &'static str = "mouseleave";
    pub const MOUSEMOVE: &'static str = "mousemove";
    pub const MOUSEOUT: &'static str = "mouseout";
    pub const MOUSEOVER: &'static str = "mouseover";
    pub const MOUSEUP: &'static str = "mouseup";
    pub const PASTE: &'static str = "paste";
    pub const POINTERCANCEL: &'static str = "pointercancel";
    pub const POINTERDOWN: &'static str = "pointerdown";
    pub const POINTERENTER: &'static str = "pointerenter";
    pub const POINTERLEAVE: &'static str = "pointerleave";
    pub const POINTERMOVE: &'static str = "pointermove";
    pub const POINTEROUT: &'static str = "pointerout";
    pub const POINTEROVER: &'static str = "pointerover";
    pub const POINTERUP: &'static str = "pointerup";
    pub const PROGRESS: &'static str = "progress";
    pub const RESET: &'static str = "reset";
    pub const RESIZE: &'static str = "resize";
    pub const SCROLL: &'static str = "scroll";
    pub const SCROLLEND: &'static str = "scrollend";
    pub const SELECT: &'static str = "select";
    pub const SELECTIONCHANGE: &'static str = "selectionchange";
    pub const SELECTSTART: &'static str = "selectstart";
    pub const SUBMIT: &'static str = "submit";
    pub const TOGGLE: &'static str = "toggle";
    pub const TOUCHCANCEL: &'static str = "touchcancel";
    pub const TOUCHEND: &'static str = "touchend";
    pub const TOUCHMOVE: &'static str = "touchmove";
    pub const TOUCHSTART: &'static str = "touchstart";
    pub const TRANSITIONCANCEL: &'static str = "transitioncancel";
    pub const TRANSITIONEND: &'static str = "transitionend";
    pub const TRANSITIONRUN: &'static str = "transitionrun";
    pub const TRANSITIONSTART: &'static str = "transitionstart";
    pub const WHEEL: &'static str = "wheel";
}

impl From<&str> for HtmlEventType {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for HtmlEventType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for HtmlEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── Opaque external data handles ──────────────────────────────────────────────

/// Opaque handle to a clipboard data object.
///
/// The host assigns and manages these IDs; `wgpu-html-events` never looks
/// inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClipboardDataId(pub u64);

/// Opaque handle to a drag-and-drop data transfer object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataTransferId(pub u64);

/// Opaque handle to a `FormData` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FormDataId(pub u64);

// ── Dispatch enum ─────────────────────────────────────────────────────────────

/// A fully-typed, owned DOM event ready for dispatch.
///
/// Wrap each concrete event type in this enum to pass through a single
/// dispatch channel without boxing or trait objects.
#[derive(Debug, Clone)]
pub enum HtmlEvent {
    Mouse(events::MouseEvent),
    Pointer(events::PointerEvent),
    Wheel(events::WheelEvent),
    Keyboard(events::KeyboardEvent),
    Focus(events::FocusEvent),
    Input(events::InputEvent),
    Composition(events::CompositionEvent),
    Clipboard(events::ClipboardEvent),
    Drag(events::DragEvent),
    Touch(events::TouchEvent),
    Animation(events::AnimationEvent),
    Transition(events::TransitionEvent),
    Submit(events::SubmitEvent),
    FormData(events::FormDataEvent),
    Toggle(events::ToggleEvent),
    BeforeToggle(events::BeforeToggleEvent),
    Progress(events::ProgressEvent),
    /// A generic event whose type does not map to a more specific variant.
    Generic(events::Event),
}

impl HtmlEvent {
    /// Return the `Event` base of whichever variant is active.
    pub fn base(&self) -> &events::Event {
        match self {
            HtmlEvent::Mouse(e) => &e.base.base,
            HtmlEvent::Pointer(e) => &e.base.base.base,
            HtmlEvent::Wheel(e) => &e.base.base.base,
            HtmlEvent::Keyboard(e) => &e.base.base,
            HtmlEvent::Focus(e) => &e.base.base,
            HtmlEvent::Input(e) => &e.base.base,
            HtmlEvent::Composition(e) => &e.base.base,
            HtmlEvent::Clipboard(e) => &e.base,
            HtmlEvent::Drag(e) => &e.base.base.base,
            HtmlEvent::Touch(e) => &e.base.base,
            HtmlEvent::Animation(e) => &e.base,
            HtmlEvent::Transition(e) => &e.base,
            HtmlEvent::Submit(e) => &e.base,
            HtmlEvent::FormData(e) => &e.base,
            HtmlEvent::Toggle(e) => &e.base,
            HtmlEvent::BeforeToggle(e) => &e.base.base,
            HtmlEvent::Progress(e) => &e.base,
            HtmlEvent::Generic(e) => e,
        }
    }

    /// The event type name.
    pub fn event_type(&self) -> &HtmlEventType {
        &self.base().event_type
    }

    /// Whether `prevent_default()` has been called on this event.
    pub fn default_prevented(&self) -> bool {
        self.base().default_prevented
    }

    /// Whether the event bubbles.
    pub fn bubbles(&self) -> bool {
        self.base().bubbles
    }
}

#[cfg(test)]
mod tests {
    use super::enums::EventPhase;
    use super::*;

    fn make_base(event_type: &str) -> Event {
        Event {
            event_type: HtmlEventType::from(event_type),
            bubbles: true,
            cancelable: true,
            composed: false,
            target: None,
            current_target: None,
            event_phase: EventPhase::AtTarget,
            default_prevented: false,
            is_trusted: true,
            time_stamp: 0.0,
        }
    }

    fn make_ui(event_type: &str, detail: i32) -> UIEvent {
        UIEvent {
            base: make_base(event_type),
            detail,
        }
    }

    #[test]
    fn event_type_display() {
        let t = HtmlEventType::from(HtmlEventType::CLICK);
        assert_eq!(t.as_str(), "click");
        assert_eq!(t.to_string(), "click");
    }

    #[test]
    fn html_event_type_constants_are_lowercase() {
        // Spot-check a few well-known names.
        assert_eq!(HtmlEventType::CLICK, "click");
        assert_eq!(HtmlEventType::KEYDOWN, "keydown");
        assert_eq!(HtmlEventType::POINTERDOWN, "pointerdown");
    }

    #[test]
    fn html_event_base_accessor() {
        let ev = HtmlEvent::Generic(make_base("custom"));
        assert_eq!(ev.event_type().as_str(), "custom");
        assert!(ev.bubbles());
        assert!(!ev.default_prevented());
    }

    #[test]
    fn keyboard_event_fields() {
        use enums::KeyboardLocation;
        let kev = KeyboardEvent {
            base: make_ui("keydown", 0),
            key: "Enter".into(),
            code: "Enter".into(),
            location: KeyboardLocation::Standard,
            ctrl_key: false,
            shift_key: true,
            alt_key: false,
            meta_key: false,
            repeat: false,
            is_composing: false,
        };
        let wrapped = HtmlEvent::Keyboard(kev);
        assert_eq!(wrapped.event_type().as_str(), "keydown");
    }

    #[test]
    fn mouse_event_chain_fields_reachable() {
        let mev = MouseEvent {
            base: make_ui("click", 1),
            screen_x: 100.0,
            screen_y: 200.0,
            client_x: 50.0,
            client_y: 80.0,
            offset_x: 5.0,
            offset_y: 10.0,
            page_x: 50.0,
            page_y: 80.0,
            movement_x: 0.0,
            movement_y: 0.0,
            button: 0,
            buttons: 1,
            ctrl_key: false,
            shift_key: false,
            alt_key: false,
            meta_key: false,
            related_target: None,
        };
        // Access through chain.
        assert_eq!(mev.base.base.event_type.as_str(), "click");
        assert_eq!(mev.base.detail, 1);
        assert_eq!(mev.client_x, 50.0);
        assert!(mev.base.base.bubbles);
    }

    #[test]
    fn toggle_event_states() {
        use enums::ToggleState;
        let t = ToggleEvent {
            base: make_base("toggle"),
            old_state: ToggleState::Closed,
            new_state: ToggleState::Open,
        };
        assert_eq!(t.new_state, ToggleState::Open);
        let before = BeforeToggleEvent { base: t };
        assert_eq!(before.base.old_state, ToggleState::Closed);
    }

    #[test]
    fn input_type_variants_exist() {
        use enums::InputType;
        let _ = InputType::InsertText;
        let _ = InputType::DeleteContentBackward;
        let _ = InputType::FormatBold;
        let _ = InputType::HistoryUndo;
    }

    #[test]
    fn node_id_is_path() {
        let id: NodeId = vec![0, 2, 1];
        assert_eq!(id.len(), 3);
    }

    #[test]
    fn opaque_ids_are_distinct_types() {
        let clip = ClipboardDataId(1);
        let drag = DataTransferId(1);
        let form = FormDataId(1);
        assert_eq!(clip.0, drag.0);
        assert_eq!(drag.0, form.0);
        // Type-system check: the three cannot be mixed without `.0`.
        let _: ClipboardDataId = clip;
        let _: DataTransferId = drag;
        let _: FormDataId = form;
    }
}
