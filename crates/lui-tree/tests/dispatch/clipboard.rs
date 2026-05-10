use std::sync::{Arc, Mutex};

use lui_models as m;
use lui_tree::{Node, Tree};

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

  let prevented = tree.clipboard_event("copy");

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

  tree.clipboard_event("cut");

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

  let prevented = tree.clipboard_event("copy");
  assert!(prevented);
}
