use std::{
  collections::HashMap,
  hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
  ArcStr, Declaration, HtmlElement,
  node::event_listeners_collection::{EventHandler, EventListenerOptions, EventListenersCollection, EventPhase},
};

/// A node in the parsed HTML tree.
#[derive(Debug, Clone)]
pub struct HtmlNode {
  pub element: HtmlElement,
  pub id: Option<ArcStr>,
  pub class_list: Vec<ArcStr>,
  pub styles: Vec<Declaration>,
  pub attrs: HashMap<ArcStr, ArcStr>,
  pub data_attrs: HashMap<ArcStr, ArcStr>,
  pub aria_attrs: HashMap<ArcStr, ArcStr>,
  pub children: Vec<HtmlNode>,
  pub node_hash: u64,

  event_listeners: EventListenersCollection,
}

impl PartialEq for HtmlNode {
  fn eq(&self, other: &Self) -> bool {
    self.element == other.element
      && self.id == other.id
      && self.class_list == other.class_list
      && self.styles == other.styles
      && self.attrs == other.attrs
      && self.data_attrs == other.data_attrs
      && self.aria_attrs == other.aria_attrs
      && self.children == other.children
  }
}

impl HtmlNode {
  pub fn new(element: HtmlElement) -> Self {
    let node_hash = hash_tag(element.tag_name());
    Self {
      element,
      id: None,
      class_list: Vec::new(),
      styles: Vec::new(),
      attrs: HashMap::new(),
      data_attrs: HashMap::new(),
      aria_attrs: HashMap::new(),
      children: Vec::new(),
      node_hash,
      event_listeners: EventListenersCollection::new(),
    }
  }

  pub fn text(content: impl Into<ArcStr>) -> Self {
    Self {
      element: HtmlElement::Text(content.into()),
      id: None,
      class_list: Vec::new(),
      styles: Vec::new(),
      attrs: HashMap::new(),
      data_attrs: HashMap::new(),
      aria_attrs: HashMap::new(),
      children: Vec::new(),
      node_hash: 0,
      event_listeners: EventListenersCollection::new(),
    }
  }

  pub fn attr(&self, name: &str) -> Option<&ArcStr> {
    if name == "id" {
      return self.id.as_ref();
    }
    if name == "class" {
      return None;
    }
    self
      .attrs
      .get(name)
      .or_else(|| self.data_attrs.get(name.strip_prefix("data-").unwrap_or("")))
      .or_else(|| self.aria_attrs.get(name.strip_prefix("aria-").unwrap_or("")))
  }

  pub fn with_children(mut self, children: Vec<HtmlNode>) -> Self {
    self.children = children;
    self
  }

  pub fn recompute_hash(&mut self) {
    self.node_hash = compute_node_hash(self);
  }

  pub fn add_event_listener(&mut self, event_type: &str, handler: EventHandler) {
    self
      .event_listeners
      .add_listener(event_type, handler, EventListenerOptions::default())
  }

  pub fn add_event_listener_with_options(
    &mut self,
    event_type: &str,
    handler: EventHandler,
    options: EventListenerOptions,
  ) {
    self.event_listeners.add_listener(event_type, handler, options)
  }

  pub fn remove_event_listener(&mut self, event_type: &str, handler: &EventHandler) {
    self.event_listeners.remove_listener(event_type, handler)
  }

  pub fn dispatch_event(&mut self, event: &mut crate::events::DocumentEvent) {
    let mut listeners = std::mem::take(&mut self.event_listeners);
    listeners.dispatch(self, event);
    self.event_listeners = listeners;
  }

  pub fn dispatch_event_phase(&mut self, event: &mut crate::events::DocumentEvent, phase: EventPhase) {
    let mut listeners = std::mem::take(&mut self.event_listeners);
    listeners.dispatch_phase(self, event, phase);
    self.event_listeners = listeners;
  }
}

pub fn hash_tag(tag: &str) -> u64 {
  let mut h = DefaultHasher::new();
  tag.hash(&mut h);
  h.finish()
}

pub fn hash_kv(k: &str, v: &str) -> u64 {
  let mut h = DefaultHasher::new();
  k.hash(&mut h);
  v.hash(&mut h);
  h.finish()
}

pub fn compute_node_hash(node: &HtmlNode) -> u64 {
  let mut h = DefaultHasher::new();
  node.element.tag_name().hash(&mut h);
  node.id.hash(&mut h);
  node.class_list.len().hash(&mut h);
  for c in &node.class_list {
    c.as_ref().hash(&mut h);
  }
  node.styles.len().hash(&mut h);
  for d in &node.styles {
    d.property.hash(&mut h);
    d.value.hash(&mut h);
    d.important.hash(&mut h);
  }
  let mut attr_xor = 0u64;
  for (k, v) in &node.attrs {
    attr_xor ^= hash_kv(k.as_ref(), v.as_ref());
  }
  for (k, v) in &node.data_attrs {
    attr_xor ^= hash_kv(k.as_ref(), v.as_ref());
  }
  for (k, v) in &node.aria_attrs {
    attr_xor ^= hash_kv(k.as_ref(), v.as_ref());
  }
  attr_xor.hash(&mut h);
  node.attrs.len().hash(&mut h);
  node.data_attrs.len().hash(&mut h);
  node.aria_attrs.len().hash(&mut h);
  h.finish()
}
