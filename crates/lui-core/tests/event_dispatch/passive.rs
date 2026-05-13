use std::sync::Arc;

use lui_core::events::{DocumentEvent, EventInit};
use lui_core::{EventHandler, EventListenerOptions, HtmlElement, HtmlNode};

fn cancelable_click() -> DocumentEvent {
  DocumentEvent::Event(EventInit {
    event_type: "click".into(),
    bubbles: true,
    cancelable: true,
    ..Default::default()
  })
}

#[test]
fn non_passive_handler_can_prevent_default() {
  let handler: EventHandler = Arc::new(|_, event| {
    event.prevent_default();
  });

  let mut node = HtmlNode::new(HtmlElement::Button);
  node.add_event_listener("click", handler);

  let mut event = cancelable_click();
  node.dispatch_event(&mut event);

  assert!(event.is_default_prevented());
}

#[test]
fn passive_handler_cannot_prevent_default() {
  let handler: EventHandler = Arc::new(|_, event| {
    event.prevent_default();
  });

  let mut node = HtmlNode::new(HtmlElement::Button);
  node.add_event_listener_with_options(
    "click",
    handler,
    EventListenerOptions { passive: true, ..Default::default() },
  );

  let mut event = cancelable_click();
  node.dispatch_event(&mut event);

  assert!(!event.is_default_prevented());
}

#[test]
fn non_cancelable_event_ignores_prevent_default() {
  let handler: EventHandler = Arc::new(|_, event| {
    event.prevent_default();
  });

  let mut node = HtmlNode::new(HtmlElement::Div);
  node.add_event_listener("click", handler);

  let mut event = DocumentEvent::Event(EventInit {
    event_type: "click".into(),
    cancelable: false,
    ..Default::default()
  });
  node.dispatch_event(&mut event);

  assert!(!event.is_default_prevented());
}

#[test]
fn passive_does_not_undo_earlier_prevent_default() {
  let preventer: EventHandler = Arc::new(|_, event| {
    event.prevent_default();
  });
  let passive_noop: EventHandler = Arc::new(|_, _| {});

  let mut node = HtmlNode::new(HtmlElement::Button);
  node.add_event_listener("click", preventer);
  node.add_event_listener_with_options(
    "click",
    passive_noop,
    EventListenerOptions { passive: true, ..Default::default() },
  );

  let mut event = cancelable_click();
  node.dispatch_event(&mut event);

  assert!(event.is_default_prevented());
}

#[test]
fn passive_handler_prevent_default_ignored_but_later_non_passive_can_still_prevent() {
  let passive_preventer: EventHandler = Arc::new(|_, event| {
    event.prevent_default();
  });
  let non_passive_preventer: EventHandler = Arc::new(|_, event| {
    event.prevent_default();
  });

  let mut node = HtmlNode::new(HtmlElement::Button);
  node.add_event_listener_with_options(
    "click",
    passive_preventer,
    EventListenerOptions { passive: true, ..Default::default() },
  );
  node.add_event_listener("click", non_passive_preventer);

  let mut event = cancelable_click();
  node.dispatch_event(&mut event);

  assert!(event.is_default_prevented());
}
