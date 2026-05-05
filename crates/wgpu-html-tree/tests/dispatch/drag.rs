use std::sync::{Arc, Mutex};

use wgpu_html_models as m;
use wgpu_html_tree::{MouseButton, MouseEvent, Node, SelectionColors, Tree};

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

  tree.dispatch_mouse_down(Some(&[]), (10.0, 10.0), MouseButton::Primary, None);
  tree.dispatch_pointer_move(Some(&[]), (13.0, 10.0), None);
  assert!(received.lock().unwrap().is_empty());
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
  tree.dispatch_pointer_move(Some(&[]), (16.0, 10.0), None);
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
