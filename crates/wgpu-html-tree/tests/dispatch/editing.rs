use std::sync::{Arc, Mutex};

use wgpu_html_events as ev;
use wgpu_html_models as m;
use wgpu_html_models::common::html_enums::InputType;
use wgpu_html_tree::{EditCursor, Node, SelectionColors, Tree};

fn edit_test_tree() -> Tree {
  let mut input = m::Input::default();
  input.r#type = Some(InputType::Text);
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
  input.r#type = Some(InputType::Text);
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
  input.r#type = Some(InputType::Text);
  input.value = Some("before".into());
  let mut tree = Tree::new(Node::new(input));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  tree.focus(Some(&[]));
  tree.text_input("X");
  tree.blur();

  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"change".into()), "expected change event, got {evs:?}");
}

#[test]
fn change_event_does_not_fire_when_value_unchanged_on_blur() {
  let received = Arc::new(Mutex::new(Vec::<String>::new()));
  let r = received.clone();
  let mut input = m::Input::default();
  input.r#type = Some(InputType::Text);
  input.value = Some("unchanged".into());
  let mut tree = Tree::new(Node::new(input));
  if let Some(n) = tree.root.as_mut() {
    n.on_event.push(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
  }
  tree.interaction.selection_colors = SelectionColors::default();

  tree.focus(Some(&[]));
  tree.blur();

  let evs = received.lock().unwrap().clone();
  assert!(!evs.contains(&"change".into()), "unexpected change, got {evs:?}");
}

fn form_test_tree() -> Tree {
  let mut input = m::Input::default();
  input.r#type = Some(InputType::Text);
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
  input.r#type = Some(InputType::Text);
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
  tree.interaction.hover_path = Some(vec![]);

  tree.wheel_event((10.0, 10.0), 0.0, -120.0, ev::enums::WheelDeltaMode::Line);

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
  tree.wheel_event((10.0, 10.0), 0.0, -120.0, ev::enums::WheelDeltaMode::Line);
  let evs = received.lock().unwrap().clone();
  assert!(evs.contains(&"wheel".into()), "expected wheel event, got {evs:?}");
}
