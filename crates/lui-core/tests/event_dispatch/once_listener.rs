use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

use lui_core::{
  EventHandler, EventListenerOptions, HtmlElement, HtmlNode,
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
fn once_listener_called_then_removed() {
  let call_count = Arc::new(AtomicUsize::new(0));
  let count = call_count.clone();

  let handler: EventHandler = Arc::new(move |_, _| {
    count.fetch_add(1, Ordering::SeqCst);
  });

  let mut node = HtmlNode::new(HtmlElement::Button);
  node.add_event_listener_with_options(
    "click",
    handler,
    EventListenerOptions {
      once: true,
      ..Default::default()
    },
  );

  node.dispatch_event(&mut click_event());
  assert_eq!(call_count.load(Ordering::SeqCst), 1);

  node.dispatch_event(&mut click_event());
  assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn regular_listener_persists_across_dispatches() {
  let call_count = Arc::new(AtomicUsize::new(0));
  let count = call_count.clone();

  let handler: EventHandler = Arc::new(move |_, _| {
    count.fetch_add(1, Ordering::SeqCst);
  });

  let mut node = HtmlNode::new(HtmlElement::Div);
  node.add_event_listener("click", handler);

  node.dispatch_event(&mut click_event());
  node.dispatch_event(&mut click_event());
  node.dispatch_event(&mut click_event());

  assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

#[test]
fn multiple_listeners_all_invoked() {
  let call_count = Arc::new(AtomicUsize::new(0));

  let mut node = HtmlNode::new(HtmlElement::Div);
  for _ in 0..3 {
    let count = call_count.clone();
    let handler: EventHandler = Arc::new(move |_, _| {
      count.fetch_add(1, Ordering::SeqCst);
    });
    node.add_event_listener("click", handler);
  }

  node.dispatch_event(&mut click_event());

  assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

#[test]
fn once_listener_removed_while_regular_persists() {
  let once_count = Arc::new(AtomicUsize::new(0));
  let regular_count = Arc::new(AtomicUsize::new(0));

  let oc = once_count.clone();
  let once_handler: EventHandler = Arc::new(move |_, _| {
    oc.fetch_add(1, Ordering::SeqCst);
  });

  let rc = regular_count.clone();
  let regular_handler: EventHandler = Arc::new(move |_, _| {
    rc.fetch_add(1, Ordering::SeqCst);
  });

  let mut node = HtmlNode::new(HtmlElement::Div);
  node.add_event_listener_with_options(
    "click",
    once_handler,
    EventListenerOptions {
      once: true,
      ..Default::default()
    },
  );
  node.add_event_listener("click", regular_handler);

  node.dispatch_event(&mut click_event());
  assert_eq!(once_count.load(Ordering::SeqCst), 1);
  assert_eq!(regular_count.load(Ordering::SeqCst), 1);

  node.dispatch_event(&mut click_event());
  assert_eq!(once_count.load(Ordering::SeqCst), 1);
  assert_eq!(regular_count.load(Ordering::SeqCst), 2);
}

#[test]
fn remove_listener_by_reference() {
  let call_count = Arc::new(AtomicUsize::new(0));
  let count = call_count.clone();

  let handler: EventHandler = Arc::new(move |_, _| {
    count.fetch_add(1, Ordering::SeqCst);
  });

  let mut node = HtmlNode::new(HtmlElement::Div);
  node.add_event_listener("click", handler.clone());

  node.dispatch_event(&mut click_event());
  assert_eq!(call_count.load(Ordering::SeqCst), 1);

  node.remove_event_listener("click", &handler);

  node.dispatch_event(&mut click_event());
  assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn remove_only_affects_matching_event_type() {
  let call_count = Arc::new(AtomicUsize::new(0));
  let count = call_count.clone();

  let handler: EventHandler = Arc::new(move |_, _| {
    count.fetch_add(1, Ordering::SeqCst);
  });

  let mut node = HtmlNode::new(HtmlElement::Div);
  node.add_event_listener("click", handler.clone());

  node.remove_event_listener("keydown", &handler);

  node.dispatch_event(&mut click_event());
  assert_eq!(call_count.load(Ordering::SeqCst), 1);
}
