use crate::{HtmlElement, HtmlNode, Stylesheet, node::event_listeners_collection::{EventHandler, EventListenerOptions, EventListenersCollection}};

/// Parsed HTML document — always rooted at a single `<html>` node.
#[derive(Debug, Clone)]
pub struct HtmlDocument {
  pub root: HtmlNode,
  pub stylesheets: Vec<Stylesheet>,
  pub focus_path: Option<Vec<usize>>,
  pub generation: u64,
  pub dirty_paths: Vec<Vec<usize>>,

  event_listeners: EventListenersCollection,
}

impl Default for HtmlDocument {
  fn default() -> Self {
    Self {
      root: HtmlNode::new(HtmlElement::Html),
      stylesheets: Vec::new(),
      focus_path: None,
      generation: 0,
      dirty_paths: Vec::new(),
      event_listeners: EventListenersCollection::new(),
    }
  }
}

impl HtmlDocument {
  pub fn new(root: HtmlNode, stylesheets: Vec<Stylesheet>) -> Self {
    Self {
      root,
      stylesheets,
      focus_path: None,
      generation: 0,
      dirty_paths: Vec::new(),
      event_listeners: EventListenersCollection::new(),
    }
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
    listeners.dispatch(&mut self.root, event);
    self.event_listeners = listeners;
  }

  pub fn collect_dirty_paths(&mut self) -> bool {
    self.dirty_paths.clear();
    let mut path = Vec::new();
    collect_dirty_recursive(&mut self.root, &mut path, &mut self.dirty_paths);
    let any = !self.dirty_paths.is_empty();
    if any {
      self.generation += 1;
    }
    any
  }

  pub fn active_element(&self) -> Option<&HtmlNode> {
    let path = self.focus_path.as_deref()?;
    self.root.at_path(path)
  }

  pub fn active_element_mut(&mut self) -> Option<&mut HtmlNode> {
    let path = self.focus_path.as_deref()?;
    self.root.at_path_mut(&path)
  }
}

fn collect_dirty_recursive(node: &mut HtmlNode, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
  if node.is_dirty() {
    node.recompute_hash();
    node.clear_dirty();
    out.push(path.clone());
  }
  for i in 0..node.children.len() {
    path.push(i);
    collect_dirty_recursive(&mut node.children[i], path, out);
    path.pop();
  }
}
