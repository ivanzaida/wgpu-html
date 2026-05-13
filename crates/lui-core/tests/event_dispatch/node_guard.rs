use std::sync::Arc;

use lui_core::events::{DocumentEvent, EventInit};
use lui_core::{EventHandler, HtmlElement, HtmlNode};

fn click_event() -> DocumentEvent {
  DocumentEvent::Event(EventInit { event_type: "click".into(), bubbles: true, cancelable: true, ..Default::default() })
}

#[test]
fn hash_recomputed_when_handler_adds_class() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.recompute_hash();
  let before = node.node_hash;

  let handler: EventHandler = Arc::new(|node, _| {
    node.class_list.push("active".into());
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());

  assert_ne!(node.node_hash, before);
}

#[test]
fn hash_recomputed_when_handler_sets_attribute() {
  let mut node = HtmlNode::new(HtmlElement::Button);
  node.recompute_hash();
  let before = node.node_hash;

  let handler: EventHandler = Arc::new(|node, _| {
    node.attrs.insert("disabled".into(), "true".into());
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());

  assert_ne!(node.node_hash, before);
}

#[test]
fn hash_stable_when_handler_only_reads() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list.push("existing".into());
  node.recompute_hash();
  let before = node.node_hash;

  let handler: EventHandler = Arc::new(|node, _| {
    let _ = node.element.tag_name();
    let _ = &node.class_list;
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());

  assert_eq!(node.node_hash, before);
}

#[test]
fn hash_stable_across_repeated_noop_dispatches() {
  let mut node = HtmlNode::new(HtmlElement::Span);
  node.recompute_hash();

  let handler: EventHandler = Arc::new(|_, _| {});
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());
  let first = node.node_hash;

  node.dispatch_event(&mut click_event());
  let second = node.node_hash;

  node.dispatch_event(&mut click_event());
  let third = node.node_hash;

  assert_eq!(first, second);
  assert_eq!(second, third);
}

#[test]
fn hash_changes_per_successive_mutation() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.recompute_hash();

  let handler: EventHandler = Arc::new(|node, _| {
    let count = node.class_list.len();
    node.class_list.push(format!("cls-{count}").into());
  });
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());
  let after_first = node.node_hash;

  node.dispatch_event(&mut click_event());
  let after_second = node.node_hash;

  assert_ne!(after_first, after_second);
}

#[test]
fn listener_not_called_for_wrong_event_type() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.recompute_hash();
  let before = node.node_hash;

  let handler: EventHandler = Arc::new(|node, _| {
    node.class_list.push("should-not-run".into());
  });
  node.add_event_listener("keydown", handler);

  node.dispatch_event(&mut click_event());

  assert_eq!(node.node_hash, before);
  assert!(node.class_list.is_empty());
}
