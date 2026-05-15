use std::sync::Arc;

use lui_core::{
  EventHandler, HtmlElement, HtmlNode,
  events::{DocumentEvent, EventInit},
};

fn click_event() -> DocumentEvent {
  DocumentEvent::Event(EventInit {
    event_type: "click".into(),
    bubbles: true,
    cancelable: true,
    ..Default::default()
  })
}

#[test]
fn handler_add_class_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();

  let handler: EventHandler = Arc::new(|node, _| {
    node.class_list_mut().add("active");
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());

  assert!(node.class_list().contains("active"));
  assert!(node.is_dirty());
}

#[test]
fn hash_recomputed_when_handler_sets_attribute() {
  let mut node = HtmlNode::new(HtmlElement::Button);
  node.clear_dirty();
  node.recompute_hash();
  let before = node.node_hash();

  let handler: EventHandler = Arc::new(|node, _| {
    node.set_attribute("disabled", "true");
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());

  assert_ne!(node.node_hash(), before);
  assert!(node.is_dirty());
}

#[test]
fn hash_stable_when_handler_only_reads() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("existing");
  node.clear_dirty();
  node.recompute_hash();
  let before = node.node_hash();

  let handler: EventHandler = Arc::new(|node, _| {
    let _ = node.element().tag_name();
    let _ = node.class_list();
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());

  assert_eq!(node.node_hash(), before);
  assert!(!node.is_dirty());
}

#[test]
fn hash_stable_across_repeated_noop_dispatches() {
  let mut node = HtmlNode::new(HtmlElement::Span);
  node.clear_dirty();
  node.recompute_hash();

  let handler: EventHandler = Arc::new(|_, _| {});
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());
  let first = node.node_hash();

  node.dispatch_event(&mut click_event());
  let second = node.node_hash();

  node.dispatch_event(&mut click_event());
  let third = node.node_hash();

  assert_eq!(first, second);
  assert_eq!(second, third);
}

#[test]
fn successive_class_mutations_accumulate() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();

  let handler: EventHandler = Arc::new(|node, _| {
    let count = node.class_list().len();
    node.class_list_mut().add(&format!("cls-{count}"));
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());
  assert!(node.class_list().contains("cls-0"));

  node.dispatch_event(&mut click_event());
  assert!(node.class_list().contains("cls-1"));
  assert!(node.is_dirty());
}

#[test]
fn listener_not_called_for_wrong_event_type() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.recompute_hash();
  let before = node.node_hash();

  let handler: EventHandler = Arc::new(|node, _| {
    node.class_list_mut().add("should-not-run");
  });
  node.add_event_listener("keydown", handler);

  node.dispatch_event(&mut click_event());

  assert_eq!(node.node_hash(), before);
  assert!(node.class_list().is_empty());
  assert!(!node.is_dirty());
}
