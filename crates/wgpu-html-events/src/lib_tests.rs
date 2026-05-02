use super::{enums::EventPhase, *};

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
