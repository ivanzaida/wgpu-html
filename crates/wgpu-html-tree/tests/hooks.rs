use std::{
  sync::{Arc, Mutex},
  time::Duration,
};

use wgpu_html_events as dom;
use wgpu_html_tree::{
  Modifiers, MouseButton, MouseEvent as TreeMouseEvent, Node, Tree, TreeHook, TreeHookResponse, TreeLifecycleEvent,
  TreeLifecyclePhase, TreeLifecycleStage, TreeRenderEvent, TreeRenderViewport,
};

#[derive(Clone, Default)]
struct Log(Arc<Mutex<Vec<String>>>);

impl Log {
  fn push(&self, entry: impl Into<String>) {
    self.0.lock().unwrap().push(entry.into());
  }

  fn entries(&self) -> Vec<String> {
    self.0.lock().unwrap().clone()
  }
}

struct RecordingHook {
  log: Log,
  stop_on: Option<&'static str>,
}

impl RecordingHook {
  fn new(log: Log) -> Self {
    Self { log, stop_on: None }
  }

  fn stopping(log: Log, stop_on: &'static str) -> Self {
    Self {
      log,
      stop_on: Some(stop_on),
    }
  }

  fn record(&self, name: &'static str) -> TreeHookResponse {
    self.log.push(name);
    if self.stop_on == Some(name) {
      TreeHookResponse::Stop
    } else {
      TreeHookResponse::Continue
    }
  }
}

impl TreeHook for RecordingHook {
  fn on_render(&mut self, _tree: &mut Tree, event: &TreeRenderEvent<'_>) -> TreeHookResponse {
    assert_eq!(event.delta, Duration::from_millis(16));
    assert_eq!(event.elapsed, Some(Duration::from_secs(1)));
    assert_eq!(event.frame_index, Some(7));
    assert_eq!(event.viewport, Some(TreeRenderViewport::new(800.0, 600.0, 1.5)));
    assert_eq!(event.frame_duration, Some(Duration::from_millis(22)));
    assert_eq!(event.cascade_duration, Some(Duration::from_millis(1)));
    assert_eq!(event.layout_duration, Some(Duration::from_millis(2)));
    assert_eq!(event.paint_duration, Some(Duration::from_millis(3)));
    assert_eq!(event.render_duration, Some(Duration::from_millis(4)));
    assert_eq!(event.label, Some("frame"));
    self.record("render")
  }

  fn on_lifecycle_begin(&mut self, _tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    assert_eq!(event.phase, TreeLifecyclePhase::Begin);
    self.record("lifecycle-begin")
  }

  fn on_lifecycle_end(&mut self, _tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    assert_eq!(event.phase, TreeLifecyclePhase::End);
    assert_eq!(event.duration, Some(Duration::from_millis(5)));
    self.record("lifecycle-end")
  }

  fn on_lifecycle_instant(&mut self, _tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    assert_eq!(event.phase, TreeLifecyclePhase::Instant);
    assert_eq!(event.label, Some("mark"));
    self.record("lifecycle-instant")
  }

  fn on_mouse_event(&mut self, _tree: &mut Tree, event: &mut TreeMouseEvent) -> TreeHookResponse {
    assert_eq!(event.pos, (10.0, 20.0));
    assert_eq!(event.button, Some(MouseButton::Primary));
    assert_eq!(event.target_path, vec![0]);
    assert_eq!(event.current_path, vec![0]);
    self.record("tree-mouse")
  }

  fn on_dom_mouse_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::MouseEvent) -> TreeHookResponse {
    self.record("mouse")
  }

  fn on_pointer_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::PointerEvent) -> TreeHookResponse {
    self.record("pointer")
  }

  fn on_wheel_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::WheelEvent) -> TreeHookResponse {
    self.record("wheel")
  }

  fn on_keyboard_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::KeyboardEvent) -> TreeHookResponse {
    self.record("keyboard")
  }

  fn on_focus_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::FocusEvent) -> TreeHookResponse {
    self.record("focus")
  }

  fn on_input_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::InputEvent) -> TreeHookResponse {
    self.record("input")
  }

  fn on_composition_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::CompositionEvent) -> TreeHookResponse {
    self.record("composition")
  }

  fn on_clipboard_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::ClipboardEvent) -> TreeHookResponse {
    self.record("clipboard")
  }

  fn on_drag_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::DragEvent) -> TreeHookResponse {
    self.record("drag")
  }

  fn on_touch_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::TouchEvent) -> TreeHookResponse {
    self.record("touch")
  }

  fn on_animation_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::AnimationEvent) -> TreeHookResponse {
    self.record("animation")
  }

  fn on_transition_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::TransitionEvent) -> TreeHookResponse {
    self.record("transition")
  }

  fn on_submit_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::SubmitEvent) -> TreeHookResponse {
    self.record("submit")
  }

  fn on_form_data_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::FormDataEvent) -> TreeHookResponse {
    self.record("formdata")
  }

  fn on_toggle_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::ToggleEvent) -> TreeHookResponse {
    self.record("toggle")
  }

  fn on_before_toggle_event(
    &mut self,
    _tree: &mut Tree,
    _event: &mut dom::events::BeforeToggleEvent,
  ) -> TreeHookResponse {
    self.record("beforetoggle")
  }

  fn on_progress_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::ProgressEvent) -> TreeHookResponse {
    self.record("progress")
  }

  fn on_generic_event(&mut self, _tree: &mut Tree, _event: &mut dom::events::Event) -> TreeHookResponse {
    self.record("generic")
  }
}

fn tree() -> Tree {
  Tree::new(Node::new("root"))
}

fn base(event_type: &str) -> dom::events::Event {
  dom::events::Event {
    event_type: dom::HtmlEventType::from(event_type),
    bubbles: true,
    cancelable: true,
    composed: false,
    target: Some(vec![0]),
    current_target: Some(vec![0]),
    event_phase: dom::EventPhase::AtTarget,
    default_prevented: false,
    is_trusted: true,
    time_stamp: 12.0,
  }
}

fn ui(event_type: &str) -> dom::events::UIEvent {
  dom::events::UIEvent {
    base: base(event_type),
    detail: 1,
  }
}

fn mouse(event_type: &str) -> dom::events::MouseEvent {
  dom::events::MouseEvent {
    base: ui(event_type),
    screen_x: 1.0,
    screen_y: 2.0,
    client_x: 3.0,
    client_y: 4.0,
    offset_x: 5.0,
    offset_y: 6.0,
    page_x: 7.0,
    page_y: 8.0,
    movement_x: 0.0,
    movement_y: 0.0,
    button: 0,
    buttons: 1,
    ctrl_key: false,
    shift_key: false,
    alt_key: false,
    meta_key: false,
    related_target: None,
  }
}

fn touch() -> dom::events::Touch {
  dom::events::Touch {
    identifier: 1,
    target: vec![0],
    screen_x: 1.0,
    screen_y: 2.0,
    client_x: 3.0,
    client_y: 4.0,
    page_x: 5.0,
    page_y: 6.0,
    radius_x: 7.0,
    radius_y: 8.0,
    rotation_angle: 9.0,
    force: 0.5,
  }
}

fn all_dom_events() -> Vec<(dom::HtmlEvent, &'static str)> {
  vec![
    (dom::HtmlEvent::Mouse(mouse("click")), "mouse"),
    (
      dom::HtmlEvent::Pointer(dom::events::PointerEvent {
        base: mouse("pointerdown"),
        pointer_id: 1,
        width: 1.0,
        height: 1.0,
        pressure: 0.5,
        tangential_pressure: 0.0,
        tilt_x: 0,
        tilt_y: 0,
        twist: 0,
        pointer_type: dom::PointerType::Mouse,
        is_primary: true,
      }),
      "pointer",
    ),
    (
      dom::HtmlEvent::Wheel(dom::events::WheelEvent {
        base: mouse("wheel"),
        delta_x: 0.0,
        delta_y: 1.0,
        delta_z: 0.0,
        delta_mode: dom::WheelDeltaMode::Pixel,
      }),
      "wheel",
    ),
    (
      dom::HtmlEvent::Keyboard(dom::events::KeyboardEvent {
        base: ui("keydown"),
        key: "A".into(),
        code: "KeyA".into(),
        location: dom::KeyboardLocation::Standard,
        ctrl_key: false,
        shift_key: false,
        alt_key: false,
        meta_key: false,
        repeat: false,
        is_composing: false,
      }),
      "keyboard",
    ),
    (
      dom::HtmlEvent::Focus(dom::events::FocusEvent {
        base: ui("focus"),
        related_target: None,
      }),
      "focus",
    ),
    (
      dom::HtmlEvent::Input(dom::events::InputEvent {
        base: ui("input"),
        data: Some("a".into()),
        input_type: dom::InputType::InsertText,
        is_composing: false,
      }),
      "input",
    ),
    (
      dom::HtmlEvent::Composition(dom::events::CompositionEvent {
        base: ui("compositionupdate"),
        data: "x".into(),
      }),
      "composition",
    ),
    (
      dom::HtmlEvent::Clipboard(dom::events::ClipboardEvent {
        base: base("copy"),
        clipboard_data: Some(dom::ClipboardDataId(1)),
      }),
      "clipboard",
    ),
    (
      dom::HtmlEvent::Drag(dom::events::DragEvent {
        base: mouse("drag"),
        data_transfer: Some(dom::DataTransferId(2)),
      }),
      "drag",
    ),
    (
      dom::HtmlEvent::Touch(dom::events::TouchEvent {
        base: ui("touchstart"),
        touches: vec![touch()],
        target_touches: vec![touch()],
        changed_touches: vec![touch()],
        ctrl_key: false,
        shift_key: false,
        alt_key: false,
        meta_key: false,
      }),
      "touch",
    ),
    (
      dom::HtmlEvent::Animation(dom::events::AnimationEvent {
        base: base("animationstart"),
        animation_name: "fade".into(),
        elapsed_time: 0.0,
        pseudo_element: None,
      }),
      "animation",
    ),
    (
      dom::HtmlEvent::Transition(dom::events::TransitionEvent {
        base: base("transitionstart"),
        property_name: "opacity".into(),
        elapsed_time: 0.0,
        pseudo_element: None,
      }),
      "transition",
    ),
    (
      dom::HtmlEvent::Submit(dom::events::SubmitEvent {
        base: base("submit"),
        submitter: Some(vec![0]),
      }),
      "submit",
    ),
    (
      dom::HtmlEvent::FormData(dom::events::FormDataEvent {
        base: base("formdata"),
        form_data: dom::FormDataId(3),
      }),
      "formdata",
    ),
    (
      dom::HtmlEvent::Toggle(dom::events::ToggleEvent {
        base: base("toggle"),
        old_state: dom::ToggleState::Closed,
        new_state: dom::ToggleState::Open,
      }),
      "toggle",
    ),
    (
      dom::HtmlEvent::BeforeToggle(dom::events::BeforeToggleEvent {
        base: dom::events::ToggleEvent {
          base: base("beforetoggle"),
          old_state: dom::ToggleState::Closed,
          new_state: dom::ToggleState::Open,
        },
      }),
      "beforetoggle",
    ),
    (
      dom::HtmlEvent::Progress(dom::events::ProgressEvent {
        base: base("progress"),
        length_computable: true,
        loaded: 1,
        total: 2,
      }),
      "progress",
    ),
    (dom::HtmlEvent::Generic(base("custom")), "generic"),
  ]
}

#[test]
fn render_hook_receives_all_frame_metadata() {
  let log = Log::default();
  let mut tree = tree();
  tree.add_hook(RecordingHook::new(log.clone()));

  let event = TreeRenderEvent::new(Duration::from_millis(16))
    .with_elapsed(Duration::from_secs(1))
    .with_frame_index(7)
    .with_viewport(TreeRenderViewport::new(800.0, 600.0, 1.5))
    .with_frame_duration(Duration::from_millis(22))
    .with_pipeline_durations(
      Some(Duration::from_millis(1)),
      Some(Duration::from_millis(2)),
      Some(Duration::from_millis(3)),
      Some(Duration::from_millis(4)),
    )
    .with_label("frame");

  assert_eq!(tree.emit_render(&event), TreeHookResponse::Continue);
  assert_eq!(log.entries(), ["render"]);
}

#[test]
fn lifecycle_hook_fans_out_by_phase() {
  let log = Log::default();
  let mut tree = tree();
  tree.add_hook(RecordingHook::new(log.clone()));

  assert_eq!(
    tree.emit_lifecycle_begin(TreeLifecycleStage::Frame),
    TreeHookResponse::Continue
  );
  assert_eq!(
    tree.emit_lifecycle_end(TreeLifecycleStage::Render, Duration::from_millis(5)),
    TreeHookResponse::Continue
  );
  assert_eq!(
    tree.emit_lifecycle_event(&TreeLifecycleEvent::instant(TreeLifecycleStage::Custom).with_label("mark")),
    TreeHookResponse::Continue
  );

  assert_eq!(log.entries(), ["lifecycle-begin", "lifecycle-end", "lifecycle-instant"]);
}

#[test]
fn low_level_mouse_hook_receives_tree_mouse_event() {
  let log = Log::default();
  let mut tree = tree();
  tree.add_hook(RecordingHook::new(log.clone()));

  let mut event = TreeMouseEvent {
    pos: (10.0, 20.0),
    button: Some(MouseButton::Primary),
    modifiers: Modifiers::default(),
    target_path: vec![0],
    current_path: vec![0],
  };

  assert_eq!(tree.emit_mouse_event(&mut event), TreeHookResponse::Continue);
  assert_eq!(log.entries(), ["tree-mouse"]);
}

#[test]
fn dom_event_hook_fans_out_to_every_typed_hook() {
  for (mut event, expected) in all_dom_events() {
    let log = Log::default();
    let mut tree = tree();
    tree.add_hook(RecordingHook::new(log.clone()));

    assert_eq!(tree.emit_event(&mut event), TreeHookResponse::Continue);
    assert_eq!(log.entries(), [expected], "event {expected}");
  }
}

#[test]
fn stop_action_prevents_later_hooks() {
  let log = Log::default();
  let mut tree = tree();
  tree.add_hook(RecordingHook::stopping(log.clone(), "keyboard"));
  tree.add_hook(RecordingHook::new(log.clone()));
  let mut event = dom::HtmlEvent::Keyboard(dom::events::KeyboardEvent {
    base: ui("keydown"),
    key: "A".into(),
    code: "KeyA".into(),
    location: dom::KeyboardLocation::Standard,
    ctrl_key: false,
    shift_key: false,
    alt_key: false,
    meta_key: false,
    repeat: false,
    is_composing: false,
  });

  assert_eq!(tree.emit_event(&mut event), TreeHookResponse::Stop);
  assert_eq!(log.entries(), ["keyboard"]);
}

#[test]
fn hook_handles_can_be_shared_removed_and_cleared() {
  let log = Log::default();
  let handle = wgpu_html_tree::TreeHookHandle::new(RecordingHook::new(log.clone()));
  let mut tree = tree();

  tree.add_hook_handle(handle.clone());
  tree.add_hook_handle(handle.clone());
  assert_eq!(tree.hook_count(), 2);

  assert!(tree.remove_hook(&handle));
  assert_eq!(tree.hook_count(), 0);
  assert!(!tree.remove_hook(&handle));

  tree.add_hook(RecordingHook::new(log));
  assert_eq!(tree.hook_count(), 1);
  tree.clear_hooks();
  assert_eq!(tree.hook_count(), 0);
}
