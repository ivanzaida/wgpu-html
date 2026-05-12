use std::sync::{Arc, Mutex};

use lui_events as ev;
use lui_models as m;

use super::*;
use crate::{EditCursor, Node, SelectionColors, TreeHook, TreeHookResponse};

/// Build a tree with a body containing children of mixed
/// focusability:
///   [0] <input type="text">          focusable, kbd
///   [1] <div>                        not focusable
///   [2] <button>                     focusable, kbd
///   [3] <input type="hidden">        not focusable
///   [4] <a href="#">                 focusable, kbd
fn focus_test_tree() -> Tree {
  let mut root = Node::new(m::Body::default());
  let mut input_text = m::Input::default();
  input_text.r#type = Some(m::common::html_enums::InputType::Text);
  root.children.push(Node::new(input_text));
  root.children.push(Node::new(m::Div::default()));
  root.children.push(Node::new(m::Button::default()));
  let mut input_hidden = m::Input::default();
  input_hidden.r#type = Some(m::common::html_enums::InputType::Hidden);
  root.children.push(Node::new(input_hidden));
  let mut anchor = m::A::default();
  anchor.href = Some("#".into());
  root.children.push(Node::new(anchor));
  Tree::new(root)
}

struct RecordingHook {
  events: Arc<Mutex<Vec<String>>>,
  mouse_paths: Arc<Mutex<Vec<Vec<usize>>>>,
}

impl TreeHook for RecordingHook {
  fn on_event(&mut self, _tree: &mut Tree, event: &mut ev::HtmlEvent) -> TreeHookResponse {
    self.events.lock().unwrap().push(event.event_type().to_string());
    TreeHookResponse::Continue
  }

  fn on_mouse_event(&mut self, _tree: &mut Tree, event: &mut MouseEvent) -> TreeHookResponse {
    self.mouse_paths.lock().unwrap().push(event.current_path.clone());
    TreeHookResponse::Continue
  }
}

#[test]
fn tree_hook_receives_keyboard_event_without_node_callback() {
  let events = Arc::new(Mutex::new(Vec::<String>::new()));
  let mouse_paths = Arc::new(Mutex::new(Vec::<Vec<usize>>::new()));
  let mut tree = focus_test_tree();
  tree.add_hook(RecordingHook {
    events: events.clone(),
    mouse_paths,
  });

  tree.key_down("a", "KeyA", false);

  let events = events.lock().unwrap().clone();
  assert!(events.contains(&"keydown".to_owned()), "got {events:?}");
}

#[test]
fn tree_hook_receives_mouse_events_without_node_callback() {
  let events = Arc::new(Mutex::new(Vec::<String>::new()));
  let mouse_paths = Arc::new(Mutex::new(Vec::<Vec<usize>>::new()));
  let mut root = Node::new(m::Body::default());
  root.children.push(Node::new(m::Div::default()));
  let mut tree = Tree::new(root);
  tree.add_hook(RecordingHook {
    events: events.clone(),
    mouse_paths: mouse_paths.clone(),
  });

  tree.dispatch_mouse_down(Some(&[0]), (1.0, 1.0), MouseButton::Primary, None);

  let events = events.lock().unwrap().clone();
  let mouse_paths = mouse_paths.lock().unwrap().clone();
  assert!(events.contains(&"mousedown".to_owned()), "got {events:?}");
  assert!(
    mouse_paths.iter().any(|p| p.as_slice() == [0usize]),
    "got {mouse_paths:?}"
  );
}

#[test]
fn focus_sets_focus_path_and_fires_focus_focusin() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = focus_test_tree();
  if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(0)) {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  assert!(tree.focus(Some(&[0])));
  assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"focus".into()), "expected focus in {evs:?}");
  assert!(evs.contains(&"focusin".into()), "expected focusin in {evs:?}");
}

#[test]
fn focus_change_fires_blur_with_related_target() {
  let received = Arc::new(Mutex::new(Vec::<(String, Option<Vec<usize>>)>::new()));
  let r = received.clone();
  let mut tree = focus_test_tree();
  if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(0)) {
    n.on_event.push(Arc::new(move |ev| {
      if let ev::HtmlEvent::Focus(fe) = ev {
        r.lock()
          .unwrap()
          .push((ev.event_type().to_string(), fe.related_target.clone()));
      }
    }));
  }
  tree.focus(Some(&[0]));
  received.lock().unwrap().clear();
  tree.focus(Some(&[2]));
  let evs = received.lock().unwrap().clone();
  let blur_evs: Vec<_> = evs.iter().filter(|(t, _)| t == "blur").collect();
  assert_eq!(blur_evs.len(), 1, "got {evs:?}");
  assert_eq!(blur_evs[0].1.as_deref(), Some([2usize].as_slice()));
}

#[test]
fn blur_clears_focus_and_fires_blur() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = focus_test_tree();
  if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(2)) {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.focus(Some(&[2]));
  received.lock().unwrap().clear();
  assert!(tree.blur());
  assert_eq!(tree.interaction.focus_path, None);
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"blur".into()));
  assert!(evs.contains(&"focusout".into()));
}

#[test]
fn focus_walks_up_to_focusable_ancestor() {
  let mut button = m::Button::default();
  button.id = Some("ok".into());
  let mut btn_node = Node::new(button);
  btn_node.children.push(Node::new("OK"));
  let mut root = Node::new(m::Body::default());
  root.children.push(btn_node);
  let mut tree = Tree::new(root);
  assert!(tree.focus(Some(&[0, 0])));
  assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
}

#[test]
fn focus_next_cycles_in_document_order() {
  let mut tree = focus_test_tree();
  assert_eq!(tree.focus_next(false).as_deref(), Some([0usize].as_slice()));
  assert_eq!(tree.focus_next(false).as_deref(), Some([2usize].as_slice()));
  assert_eq!(tree.focus_next(false).as_deref(), Some([4usize].as_slice()));
  assert_eq!(tree.focus_next(false).as_deref(), Some([0usize].as_slice()));
}

#[test]
fn focus_next_reverse_cycles_backward() {
  let mut tree = focus_test_tree();
  assert_eq!(tree.focus_next(true).as_deref(), Some([4usize].as_slice()));
  assert_eq!(tree.focus_next(true).as_deref(), Some([2usize].as_slice()));
  assert_eq!(tree.focus_next(true).as_deref(), Some([0usize].as_slice()));
}

#[test]
fn key_down_dispatches_to_focused_element_on_event() {
  let received = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
  let r = received.clone();
  let mut tree = focus_test_tree();
  if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(0)) {
    n.on_event.push(Arc::new(move |ev| {
      if let ev::HtmlEvent::Keyboard(ke) = ev {
        r.lock().unwrap().push((ev.event_type().to_string(), ke.key.clone()));
      }
    }));
  }
  tree.focus(Some(&[0]));
  tree.key_down("a", "KeyA", false);
  let evs = received.lock().unwrap().clone();
  assert!(evs.iter().any(|(t, k)| t == "keydown" && k == "a"), "got {evs:?}");
}

#[test]
fn key_down_tab_advances_focus() {
  let mut tree = focus_test_tree();
  tree.focus(Some(&[0]));
  tree.key_down("Tab", "Tab", false);
  assert_eq!(tree.interaction.focus_path.as_deref(), Some([2usize].as_slice()));
}

#[test]
fn key_down_shift_tab_retreats_focus() {
  let mut tree = focus_test_tree();
  tree.focus(Some(&[2]));
  tree.set_modifier(Modifier::Shift, true);
  tree.key_down("Tab", "Tab", false);
  assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
}

#[test]
fn focus_returns_false_when_target_has_no_focusable_ancestor() {
  let mut root = Node::new(m::Body::default());
  root.children.push(Node::new(m::Div::default()));
  let mut tree = Tree::new(root);
  assert!(!tree.focus(Some(&[0])));
  assert_eq!(tree.interaction.focus_path, None);
}

#[test]
fn dispatch_mouse_down_with_no_target_clears_selection() {
  let mut tree = Tree::new(Node::new("text"));
  tree.dispatch_mouse_down(None, (0.0, 0.0), MouseButton::Primary, None);
  assert!(tree.interaction.selection.is_none());
}

#[test]
fn dispatch_mouse_down_focuses_focusable_target() {
  // Body → [0] = Button. Pressing the button should focus it.
  let mut root = Node::new(m::Body::default());
  root.children.push(Node::new(m::Button::default()));
  let mut tree = Tree::new(root);
  tree.dispatch_mouse_down(Some(&[0]), (0.0, 0.0), MouseButton::Primary, None);
  assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
}

#[test]
fn dispatch_mouse_down_then_up_synthesises_click() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut node = Node::new("text");
  node.on_event.push(Arc::new(move |ev| {
    r.lock().unwrap().push(ev.event_type().to_string());
  }));
  let mut tree = Tree::new(node);
  let path: &[usize] = &[];
  tree.dispatch_mouse_down(Some(path), (1.0, 1.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(path), (1.0, 1.0), MouseButton::Primary, None);
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"mousedown".into()), "got {evs:?}");
  assert!(evs.contains(&"mouseup".into()), "got {evs:?}");
  assert!(evs.contains(&"click".into()), "got {evs:?}");
}

#[test]
fn dispatch_pointer_move_fires_enter_then_leave() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut node = Node::new("text");
  node.on_event.push(Arc::new(move |ev| {
    r.lock().unwrap().push(ev.event_type().to_string());
  }));
  let mut tree = Tree::new(node);
  tree.dispatch_pointer_move(Some(&[]), (1.0, 1.0), None);
  tree.pointer_leave();
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"mouseenter".into()), "got {evs:?}");
  assert!(evs.contains(&"mouseleave".into()), "got {evs:?}");
}

#[test]
fn buttons_down_bitmask_tracks_press_and_release() {
  let mut tree = Tree::new(Node::new("text"));
  let path: &[usize] = &[];
  assert_eq!(tree.interaction.buttons_down, 0);
  tree.dispatch_mouse_down(Some(path), (0.0, 0.0), MouseButton::Primary, None);
  assert_eq!(tree.interaction.buttons_down, 1);
  tree.dispatch_mouse_up(Some(path), (0.0, 0.0), MouseButton::Primary, None);
  assert_eq!(tree.interaction.buttons_down, 0);
}

#[test]
fn set_modifier_updates_interaction_state() {
  let mut tree = Tree::new(Node::new("text"));
  assert!(!tree.modifiers().shift);
  tree.set_modifier(Modifier::Shift, true);
  assert!(tree.modifiers().shift);
  tree.set_modifier(Modifier::Ctrl, true);
  assert!(tree.modifiers().ctrl);
  assert!(tree.modifiers().shift); // unchanged by previous call
  tree.set_modifier(Modifier::Shift, false);
  assert!(!tree.modifiers().shift);
  assert!(tree.modifiers().ctrl);
}

// ── Input / Change / Submit event tests ──────────────────────────────────────

fn edit_test_tree() -> Tree {
  let mut input = m::Input::default();
  input.r#type = Some(m::common::html_enums::InputType::Text);
  let mut tree = Tree::new(Node::new(input));
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn text_input_fires_input_event() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = edit_test_tree();
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.focus(Some(&[]));

  tree.text_input("hello");

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"input".into()), "expected input event, got {evs:?}");
  assert_eq!(evs.iter().filter(|e| *e == "input").count(), 1);
}

#[test]
fn text_input_does_not_fire_input_on_non_editable() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut node = Node::new("text");
  node.on_event.push(Arc::new(move |ev| {
    r.lock().unwrap().push(ev.event_type().to_string());
  }));
  let mut tree = Tree::new(node);
  assert!(!tree.text_input("hello"));
  assert!(received.lock().unwrap().is_empty());
}

#[test]
fn backspace_fires_input_with_delete_content_backward() {
  let received = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
  let r = received.clone();
  let mut input = m::Input::default();
  input.r#type = Some(m::common::html_enums::InputType::Text);
  input.value = Some("abc".into());
  let mut tree = Tree::new(Node::new(input));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      if let ev::HtmlEvent::Input(ie) = ev {
        r.lock()
          .unwrap()
          .push((ev.event_type().to_string(), format!("{:?}", ie.input_type)));
      }
    }));
  }
  tree.focus(Some(&[]));
  tree.interaction.edit_cursor = Some(EditCursor::collapsed(3));

  tree.key_down("Backspace", "Backspace", false);

  let evs = received.lock().unwrap().clone();
  assert!(evs.iter().any(|(t, _)| t == "input"), "got {evs:?}");
  assert!(evs.iter().any(|(_, it)| it.contains("DeleteContentBackward")));
}

#[test]
fn change_event_fires_when_value_mutated_then_blurred() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut input = m::Input::default();
  input.r#type = Some(m::common::html_enums::InputType::Text);
  input.value = Some("before".into());
  let mut tree = Tree::new(Node::new(input));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  // Focus and snapshot "before"
  tree.focus(Some(&[]));
  // Edit to "beforeX"
  tree.text_input("X");
  // Blur — should fire change
  tree.blur();

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"change".into()), "expected change event, got {evs:?}");
}

#[test]
fn change_event_does_not_fire_when_value_unchanged_on_blur() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut input = m::Input::default();
  input.r#type = Some(m::common::html_enums::InputType::Text);
  input.value = Some("unchanged".into());
  let mut tree = Tree::new(Node::new(input));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  tree.focus(Some(&[]));
  // No mutation — blur should NOT fire change
  tree.blur();

  let evs = received.lock().unwrap().clone();
  assert!(!evs.contains(&"change".into()), "unexpected change, got {evs:?}");
}

fn form_test_tree() -> Tree {
  let mut input = m::Input::default();
  input.r#type = Some(m::common::html_enums::InputType::Text);
  let mut form = m::Form::default();
  form.id = Some("myform".into());
  let mut tree = Tree::new(Node::new(m::Body::default()));
  if let Some(body) = tree.root.as_mut() {
    let mut form_node = Node::new(form);
    form_node.children.push(Node::new(input));
    body.children.push(form_node);
  }
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn enter_in_form_input_fires_submit() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = form_test_tree();
  if let Some(body) = tree.root.as_mut() {
    if let Some(form) = body.children.get_mut(0) {
      form.on_event.push(Arc::new(move |ev| {
        r.lock().unwrap().push(ev.event_type().to_string());
      }));
    }
  }

  // Focus the input inside the form: path [0, 0]
  tree.focus(Some(&[0, 0]));
  tree.key_down("Enter", "Enter", false);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"submit".into()), "expected submit event, got {evs:?}");
}

#[test]
fn enter_in_non_form_input_does_not_fire_submit() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut input = m::Input::default();
  input.r#type = Some(m::common::html_enums::InputType::Text);
  let mut tree = Tree::new(Node::new(input));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  tree.focus(Some(&[]));
  tree.key_down("Enter", "Enter", false);

  let evs = received.lock().unwrap().clone();
  assert!(!evs.contains(&"submit".into()));
}

#[test]
fn wheel_event_dispatches_to_hovered_element() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new("text"));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  // Set hover path so wheel_event has a target
  tree.interaction.hover_path = Some(vec![]);

  wheel_event(&mut tree, (10.0, 10.0), 0.0, -120.0, ev::enums::WheelDeltaMode::Line);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"wheel".into()), "expected wheel event, got {evs:?}");
}

#[test]
fn wheel_event_with_no_hover_dispatches_to_root() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new("text"));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  wheel_event(&mut tree, (10.0, 10.0), 0.0, -120.0, ev::enums::WheelDeltaMode::Line);
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"wheel".into()), "expected wheel event, got {evs:?}");
}

// ── Checkbox / Radio toggle tests ────────────────────────────────────────────

fn checkbox_tree() -> Tree {
  let mut inp = m::Input::default();
  inp.r#type = Some(m::common::html_enums::InputType::Checkbox);
  let mut tree = Tree::new(Node::new(inp));
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn click_toggles_checkbox() {
  let mut tree = checkbox_tree();
  let root = tree.root.as_ref().unwrap();
  let was_checked = if let Element::Input(inp) = &root.element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(!was_checked);

  tree.dispatch_mouse_down(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);

  let now_checked = if let Element::Input(inp) = &tree.root.as_ref().unwrap().element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(now_checked);
}

#[test]
fn checkbox_click_fires_change_and_input() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = checkbox_tree();
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }

  tree.dispatch_mouse_down(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[]), (0.0, 0.0), MouseButton::Primary, None);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"input".into()), "expected input, got {evs:?}");
  assert!(evs.contains(&"change".into()), "expected change, got {evs:?}");
}

fn radio_tree() -> Tree {
  let mut r1 = m::Input::default();
  r1.r#type = Some(m::common::html_enums::InputType::Radio);
  r1.name = Some("group1".into());
  r1.checked = Some(true);
  let mut r2 = m::Input::default();
  r2.r#type = Some(m::common::html_enums::InputType::Radio);
  r2.name = Some("group1".into());
  let mut root = Node::new(m::Div::default());
  root.children.push(Node::new(r1));
  root.children.push(Node::new(r2));
  let mut tree = Tree::new(root);
  tree.interaction.selection_colors = SelectionColors::default();
  tree
}

#[test]
fn click_on_radio_unchecks_sibling_and_checks_self() {
  let mut tree = radio_tree();

  tree.dispatch_mouse_down(Some(&[1]), (0.0, 0.0), MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[1]), (0.0, 0.0), MouseButton::Primary, None);

  let body = tree.root.as_ref().unwrap();

  let r1 = if let Element::Input(inp) = &body.children[0].element {
    inp.checked.unwrap_or(false)
  } else {
    true
  };
  let r2 = if let Element::Input(inp) = &body.children[1].element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(!r1, "first radio should be unchecked");
  assert!(r2, "second radio should be checked");
}

#[test]
fn enter_on_button_synthesises_click() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut button = m::Button::default();
  button.id = Some("btn".into());
  let mut tree = Tree::new(Node::new(button));
  if let Some(n) = tree.root.as_mut() {
    n.on_click.push(Arc::new(move |ev| {
      r.lock().unwrap().push(format!("click:{:?}", ev.pos));
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  tree.focus(Some(&[]));
  tree.key_down("Enter", "Enter", false);

  let evs = received.lock().unwrap().clone();
  assert!(!evs.is_empty(), "expected click from Enter, got nothing");
  assert!(evs[0].starts_with("click:"));
}

#[test]
fn space_on_checkbox_toggles() {
  let mut tree = checkbox_tree();
  tree.focus(Some(&[]));

  tree.key_down(" ", "Space", false);

  let now_checked = if let Element::Input(inp) = &tree.root.as_ref().unwrap().element {
    inp.checked.unwrap_or(false)
  } else {
    false
  };
  assert!(now_checked, "Space should toggle checkbox via synthetic click");
}

// ── Drag tests ───────────────────────────────────────────────────────────────

#[test]
fn dragstart_fires_after_5px_movement_on_draggable_element() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut node = Node::new(m::Div::default());
  node.draggable = true;
  node.on_dragstart.push(Arc::new(move |_: &MouseEvent| {
    r.lock().unwrap().push("dragstart".into());
  }));
  let mut tree = Tree::new(node);
  tree.interaction.selection_colors = SelectionColors::default();

  // Mousedown at (10, 10)
  tree.dispatch_mouse_down(Some(&[]), (10.0, 10.0), MouseButton::Primary, None);
  // Move 3px — not enough
  tree.dispatch_pointer_move(Some(&[]), (13.0, 10.0), None);
  assert!(received.lock().unwrap().is_empty());
  // Move 6px — triggers dragstart
  tree.dispatch_pointer_move(Some(&[]), (16.0, 10.0), None);
  assert!(received.lock().unwrap().contains(&"dragstart".to_string()));
}

#[test]
fn drag_suppresses_click() {
  let click_count = Arc::new(Mutex::new(0u32));
  let c = click_count.clone();
  let mut node = Node::new(m::Div::default());
  node.draggable = true;
  node.on_click.push(Arc::new(move |_: &MouseEvent| {
    *c.lock().unwrap() += 1;
  }));
  let mut tree = Tree::new(node);
  tree.interaction.selection_colors = SelectionColors::default();

  tree.dispatch_mouse_down(Some(&[]), (10.0, 10.0), MouseButton::Primary, None);
  tree.dispatch_pointer_move(Some(&[]), (16.0, 10.0), None); // triggers dragstart
  tree.dispatch_mouse_up(Some(&[]), (16.0, 10.0), MouseButton::Primary, None);

  assert_eq!(*click_count.lock().unwrap(), 0, "click should be suppressed after drag");
}

#[test]
fn dragend_and_drop_fire_on_mouseup_after_drag() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let r2 = received.clone();
  let mut node = Node::new(m::Div::default());
  node.draggable = true;
  node.on_dragend.push(Arc::new(move |_: &MouseEvent| {
    r.lock().unwrap().push("dragend".into());
  }));
  node.on_drop.push(Arc::new(move |_: &MouseEvent| {
    r2.lock().unwrap().push("drop".into());
  }));
  let mut tree = Tree::new(node);
  tree.interaction.selection_colors = SelectionColors::default();

  tree.dispatch_mouse_down(Some(&[]), (10.0, 10.0), MouseButton::Primary, None);
  tree.dispatch_pointer_move(Some(&[]), (16.0, 10.0), None);
  tree.dispatch_mouse_up(Some(&[]), (16.0, 10.0), MouseButton::Primary, None);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"dragend".to_string()), "expected dragend, got {evs:?}");
  assert!(evs.contains(&"drop".to_string()), "expected drop, got {evs:?}");
}

// ── Clipboard tests ──────────────────────────────────────────────────────────

#[test]
fn clipboard_copy_event_dispatches_to_focused_element() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.focus(Some(&[]));

  let prevented = clipboard_event(&mut tree, ev::HtmlEventType::COPY);

  assert!(!prevented);
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"copy".to_string()), "expected copy, got {evs:?}");
}

#[test]
fn clipboard_cut_event_dispatches() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.focus(Some(&[]));

  clipboard_event(&mut tree, ev::HtmlEventType::CUT);

  assert!(received.lock().unwrap().contains(&"cut".to_string()));
}

#[test]
fn clipboard_event_prevent_default_returns_true() {
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      ev.prevent_default();
    }));
  }
  tree.focus(Some(&[]));

  let prevented = clipboard_event(&mut tree, ev::HtmlEventType::COPY);
  assert!(prevented);
}

// ── Scroll / Selectionchange tests ───────────────────────────────────────────

#[test]
fn scroll_event_dispatches_non_bubbling() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_scroll.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }

  scroll_event(&mut tree, &[]);

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"scroll".to_string()), "expected scroll, got {evs:?}");
}

#[test]
fn selectionchange_dispatches_on_root() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut tree = Tree::new(Node::new(m::Div::default()));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }

  selectionchange_event(&mut tree);

  assert!(received.lock().unwrap().contains(&"selectionchange".to_string()));
}

// ── Capture phase tests ──────────────────────────────────────────────────────

/// Helper: build a tree `body > div#outer > span#inner` and fire mousedown
/// on the span.  Callbacks record `(event_type, phase, current_target_len)`.
fn capture_test_tree() -> Tree {
  let span = Node::new(m::Span::default());
  let mut outer = Node::new(m::Div::default());
  outer.children.push(span);
  let mut body = Node::new(m::Body::default());
  body.children.push(outer);
  Tree::new(body)
}

/// Install recording callbacks on every node that push `(event_type, phase,
/// current_target_len)` into `log`. Dedicated `on_mouse_down` slots just log the
/// path length; `on_event` logs phase + len.
fn install_mousedown_path_loggers(tree: &mut Tree, log: Arc<Mutex<Vec<usize>>>) {
  let paths = vec![vec![], vec![0], vec![0, 0]]; // body, outer div, inner span
  for path in &paths {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_mouse_down.push(Arc::new(move |_ev| {
        l.lock().unwrap().push(p.len());
      }));
    }
  }
}

/// Install recording callbacks on every node that push `(event_type, phase,
/// current_target_len)` into `log`. `on_mouse_down` is a dedicated slot;
/// `on_event` is the generic DOM slot.
fn install_capture_loggers(
  tree: &mut Tree,
  log: Arc<Mutex<Vec<(String, String, usize)>>>,
  use_on_event: bool,
) {
  let paths = vec![vec![], vec![0], vec![0, 0]]; // body, outer div, inner span
  for path in &paths {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(path)) {
      let l = log.clone();
      let p = path.clone();
      if use_on_event {
        node.on_event.push(Arc::new(move |ev| {
          let phase = format!("{:?}", ev.base().event_phase);
          l.lock()
            .unwrap()
            .push((ev.event_type().to_string(), phase, p.len()));
        }));
      }
    }
  }
}

#[test]
fn mousedown_capture_phase_fires_root_first_then_target_then_bubble() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  install_capture_loggers(&mut tree, log.clone(), true); // use on_event

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  // Expected: capture body(0) → outer(1) → target span(2) → bubble outer(1) → body(0)
  assert_eq!(
    events,
    vec![
      ("mousedown".to_string(), "CapturingPhase".to_string(), 0), // body capture
      ("mousedown".to_string(), "CapturingPhase".to_string(), 1), // outer capture
      ("mousedown".to_string(), "AtTarget".to_string(), 2),       // span target
      ("mousedown".to_string(), "BubblingPhase".to_string(), 1), // outer bubble
      ("mousedown".to_string(), "BubblingPhase".to_string(), 0), // body bubble
    ],
    "expected capture→target→bubble order for mousedown"
  );
}

#[test]
fn keydown_capture_phase_root_first() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  // Make the span focusable so keydown targets it (path [0,0])
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
    if let Element::Span(s) = &mut node.element {
      s.tabindex = Some(0);
    }
  }
  install_capture_loggers(&mut tree, log.clone(), true);
  tree.focus(Some(&[0, 0]));
  // Clear focus events from the log so we only see keydown events.
  log.lock().unwrap().clear();

  tree.key_down("a", "KeyA", false);

  let events = log.lock().unwrap().clone();
  // keydown dispatches to focused element: body=[], outer=[0], span=[0,0]
  // Capture: body (0) → outer (1) → target span (2) → bubble outer (1) → body (0)
  assert_eq!(events[0], ("keydown".to_string(), "CapturingPhase".to_string(), 0));
  assert_eq!(events[1], ("keydown".to_string(), "CapturingPhase".to_string(), 1));
  assert_eq!(events[2], ("keydown".to_string(), "AtTarget".to_string(), 2));
  assert_eq!(events[3], ("keydown".to_string(), "BubblingPhase".to_string(), 1));
  assert_eq!(events[4], ("keydown".to_string(), "BubblingPhase".to_string(), 0));
}

#[test]
fn stoppropagation_in_capture_prevents_target_and_bubble() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  // Add a handler on the outer div (path [0]) that stops propagation
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0])) {
    node.on_event.push(Arc::new(move |ev| {
      ev.stop_propagation();
    }));
  }
  let l = log.clone();
  // Log all events
  for path in [vec![], vec![0], vec![0, 0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let p = path.clone();
      let ll = l.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        ll.lock()
          .unwrap()
          .push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  // body capture fires, then outer capture fires (stopProp called here),
  // target and bubble should NOT fire
  assert_eq!(
    events,
    vec![
      ("mousedown".to_string(), "CapturingPhase".to_string(), 0), // body capture
      ("mousedown".to_string(), "CapturingPhase".to_string(), 1), // outer capture (stops)
    ],
    "stopPropagation during capture should prevent target and bubble phases"
  );
}

#[test]
fn stoppropagation_at_target_prevents_bubble_phase() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  // Stop propagation at target (inner span)
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
    let l = log.clone();
    node.on_event.push(Arc::new(move |ev| {
      let phase = format!("{:?}", ev.base().event_phase);
      l.lock()
        .unwrap()
        .push((ev.event_type().to_string(), phase, 2));
      ev.stop_propagation();
    }));
  }
  // Log on ancestors too
  for path in [vec![], vec![0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock()
          .unwrap()
          .push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  // body capture, outer capture, target, then no bubble since stopped
  assert_eq!(events.len(), 3);
  assert_eq!(events[0], ("mousedown".to_string(), "CapturingPhase".to_string(), 0));
  assert_eq!(events[1], ("mousedown".to_string(), "CapturingPhase".to_string(), 1));
  assert_eq!(events[2], ("mousedown".to_string(), "AtTarget".to_string(), 2));
}

#[test]
fn scroll_event_fires_capture_on_ancestors() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  for path in [vec![], vec![0], vec![0, 0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock()
          .unwrap()
          .push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  scroll_event(&mut tree, &[0, 0]);

  let events = log.lock().unwrap().clone();
  // non-bubbling: capture body → outer, then target at span
  assert_eq!(
    events,
    vec![
      ("scroll".to_string(), "CapturingPhase".to_string(), 0),
      ("scroll".to_string(), "CapturingPhase".to_string(), 1),
      ("scroll".to_string(), "AtTarget".to_string(), 2),
    ],
    "scroll should fire capture on ancestors then target"
  );
}

#[test]
fn focus_event_capture_fires_on_non_bubbling_event() {
  let mut tree = capture_test_tree();
  // Make the span focusable so focus targets it (path [0,0])
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&[0, 0])) {
    if let Element::Span(s) = &mut node.element {
      s.tabindex = Some(0);
    }
  }
  let log = Arc::new(Mutex::new(Vec::new()));
  // Install on_event loggers
  for path in [vec![], vec![0], vec![0, 0]] {
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&path)) {
      let l = log.clone();
      let p = path.clone();
      node.on_event.push(Arc::new(move |ev| {
        let phase = format!("{:?}", ev.base().event_phase);
        l.lock()
          .unwrap()
          .push((ev.event_type().to_string(), phase, p.len()));
      }));
    }
  }

  tree.focus(Some(&[0, 0]));

  let events: Vec<_> = log
    .lock()
    .unwrap()
    .iter()
    .filter(|(t, _, _)| t == "focusin")
    .cloned()
    .collect();
  // focusin bubbles, so: capture body → outer, target span, bubble outer → body
  assert_eq!(events.len(), 5);
  assert_eq!(events[0], ("focusin".to_string(), "CapturingPhase".to_string(), 0));
  assert_eq!(events[1], ("focusin".to_string(), "CapturingPhase".to_string(), 1));
  assert_eq!(events[2], ("focusin".to_string(), "AtTarget".to_string(), 2));
  assert_eq!(events[3], ("focusin".to_string(), "BubblingPhase".to_string(), 1));
  assert_eq!(events[4], ("focusin".to_string(), "BubblingPhase".to_string(), 0));
}

#[test]
fn submit_event_capture_phase_root_first() {
  let mut tree = Tree::new(Node::new(m::Form::default()));
  let log = Arc::new(Mutex::new(Vec::new()));
  // Form is root (path []), add listener on root
  if let Some(node) = tree.root.as_mut() {
    let l = log.clone();
    node.on_event.push(Arc::new(move |ev| {
      let phase = format!("{:?}", ev.base().event_phase);
      l.lock()
        .unwrap()
        .push((ev.event_type().to_string(), phase, 0));
    }));
  }

  bubble_submit_event(&mut tree, &[], None);

  let events = log.lock().unwrap().clone();
  // Target is root, so only AtTarget fires (no ancestors)
  assert_eq!(events.len(), 1);
  assert_eq!(events[0], ("submit".to_string(), "AtTarget".to_string(), 0));
}

#[test]
fn dedicated_slots_receive_mousedown_in_capture_to_bubble_order() {
  let mut tree = capture_test_tree();
  let log = Arc::new(Mutex::new(Vec::new()));
  install_mousedown_path_loggers(&mut tree, log.clone());

  tree.dispatch_mouse_down(Some(&[0, 0]), (10.0, 10.0), MouseButton::Primary, None);

  let events = log.lock().unwrap().clone();
  // Dedicated on_mouse_down slots fire in capture→target→bubble order:
  // capture: body(0), outer(1); target: span(2); bubble: outer(1), body(0)
  assert_eq!(
    events,
    vec![0, 1, 2, 1, 0],
    "dedicated mouse callbacks should fire capture→target→bubble"
  );
}
