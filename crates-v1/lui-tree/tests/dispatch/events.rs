use std::sync::{Arc, Mutex};

use lui_events as ev;
use lui_models as m;
use lui_tree::{Modifier, MouseButton, MouseEvent, Node, Tree, TreeHook, TreeHookResponse};

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
  assert!(tree.modifiers().shift);
  tree.set_modifier(Modifier::Shift, false);
  assert!(!tree.modifiers().shift);
  assert!(tree.modifiers().ctrl);
}
