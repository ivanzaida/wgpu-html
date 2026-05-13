use crate::{
  node::event_listeners_collection::{EventHandler, EventListenerOptions, EventListenersCollection}, HtmlNode,
  Stylesheet,
};

/// Parsed HTML document — always rooted at a single `<html>` node.
#[derive(Debug, Clone)]
pub struct HtmlDocument {
  pub root: HtmlNode,
  pub stylesheets: Vec<Stylesheet>,

  event_listeners: EventListenersCollection,
}

impl HtmlDocument {
  pub fn new(root: HtmlNode, stylesheets: Vec<Stylesheet>) -> Self {
    Self { root, stylesheets, event_listeners: EventListenersCollection::new() }
  }

  pub fn add_event_listener(&mut self, event_type: &str, handler: EventHandler) {
    self
      .event_listeners
      .add_listener(event_type, handler, EventListenerOptions::default())
  }

  pub fn add_event_listener_with_options(&mut self, event_type: &str, handler: EventHandler, options: EventListenerOptions) {
    self.event_listeners.add_listener(event_type, handler, options)
  }

  pub fn remove_event_listener(&mut self, event_type: &str, handler: &EventHandler) {
    self.event_listeners.remove_listener(event_type, handler)
  }

  pub fn dispatch_event(&mut self, event: &mut crate::events::DocumentEvent) {
    let mut listeners = std::mem::take(&mut self.event_listeners);
    listeners.dispatch(&mut self.root, event);
    self.event_listeners = listeners;
  }
}
