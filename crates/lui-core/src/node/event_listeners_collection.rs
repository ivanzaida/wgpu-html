use std::{
  fmt,
  ops::{Deref, DerefMut},
  sync::Arc,
};

use crate::{ArcStr, events::DocumentEvent, HtmlNode};

#[derive(Default, Debug, Clone)]
pub struct EventListenerOptions {
  pub capture: bool,
  pub once: bool,
  pub passive: bool,
}

pub type EventHandler = Arc<dyn Fn(&mut HtmlNode, &mut DocumentEvent) + Send + Sync>;

struct NodeGuard<'a> {
  node: &'a mut HtmlNode,
  changed: bool,
}

impl Deref for NodeGuard<'_> {
  type Target = HtmlNode;
  fn deref(&self) -> &HtmlNode {
    self.node
  }
}

impl DerefMut for NodeGuard<'_> {
  fn deref_mut(&mut self) -> &mut HtmlNode {
    self.changed = true;
    self.node
  }
}

#[derive(Clone)]
struct ListenerWithOptions {
  pub event_type: ArcStr,
  pub handler: EventHandler,
  pub options: EventListenerOptions,
}

impl fmt::Debug for ListenerWithOptions {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ListenerWithOptions")
      .field("event_type", &self.event_type)
      .field("handler", &"<EventHandler>")
      .field("options", &self.options)
      .finish()
  }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EventListenersCollection {
  listeners: Vec<ListenerWithOptions>,
}

impl EventListenersCollection {
  pub fn new() -> Self {
    Self { listeners: Vec::new() }
  }

  pub fn add_listener(&mut self, event_type: &str, handler: EventHandler, options: EventListenerOptions) {
    self.listeners.push(ListenerWithOptions {
      event_type: ArcStr::from(event_type),
      handler,
      options,
    });
  }

  pub fn remove_listener(&mut self, event_type: &str, handler: &EventHandler) {
    self.listeners
      .retain(|l| !(&*l.event_type == event_type && Arc::ptr_eq(&l.handler, handler)));
  }

  pub fn dispatch(&mut self, node: &mut HtmlNode, event: &mut DocumentEvent) {
    let event_type: ArcStr = event.base().event_type.clone();
    let mut index = 0;

    while index < self.listeners.len() {
      if self.listeners[index].event_type != event_type {
        index += 1;
        continue;
      }

      let once = {
        let handler = Arc::clone(&self.listeners[index].handler);
        let options = &self.listeners[index].options;
        let is_once = options.once;
        let is_passive = options.passive;

        let mut guard = NodeGuard { node, changed: false };

        if is_passive {
          let saved = event.base().default_prevented;
          (handler)(&mut guard, event);
          event.base_mut().default_prevented = saved;
        } else {
          (handler)(&mut guard, event);
        }

        if guard.changed {
          guard.node.recompute_hash();
        }

        is_once
      };

      if once {
        self.listeners.remove(index);
      } else {
        index += 1;
      }
    }
  }
}
