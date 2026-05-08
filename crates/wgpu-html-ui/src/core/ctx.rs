//! Callback factory and message channel.

use std::{
  any::{Any, TypeId},
  cell::{Cell, RefCell},
  collections::HashMap,
  sync::{Arc, Mutex},
};

use wgpu_html_models::{self as m, ArcStr};
use wgpu_html_tree::{EventCallback, HtmlEvent, MouseCallback, MouseEvent, Node};

use crate::el::El;

// ── MsgSender ───────────────────────────────────────────────────────────────

/// Thread-safe message queue.
///
/// Cloned into callback closures so that event handlers (which must be
/// `Send + Sync`) can enqueue messages for the component's update loop.
///
/// You can also obtain one from [`Ctx::sender`] inside `view()` to pass
/// into [`Observable::subscribe_msg`](crate::Observable::subscribe_msg)
/// inside [`Component::mounted`](crate::Component::mounted).
#[derive(Clone)]
pub struct MsgSender<M: 'static> {
  queue: Arc<Mutex<Vec<M>>>,
  wake: Arc<dyn Fn() + Send + Sync>,
}

impl<M: 'static> MsgSender<M> {
  pub(crate) fn new(wake: Arc<dyn Fn() + Send + Sync>) -> Self {
    Self {
      queue: Arc::new(Mutex::new(Vec::new())),
      wake,
    }
  }

  /// Enqueue a message and request a redraw.
  pub fn send(&self, msg: M) {
    self.queue.lock().unwrap().push(msg);
    (self.wake)();
  }

  /// Drain all pending messages.
  pub(crate) fn drain(&self) -> Vec<M> {
    std::mem::take(&mut *self.queue.lock().unwrap())
  }
}

// ── ChildSlot ───────────────────────────────────────────────────────────────

/// Descriptor for a child component declared during [`Ctx::child`] /
/// [`Ctx::keyed_child`].
pub(crate) struct ChildSlot {
  /// Identity key used to match children across re-renders.
  /// `ctx.child` auto-generates `"__pos_{n}"`.
  /// `ctx.keyed_child` uses the caller-supplied string.
  pub key: String,
  /// The DOM placeholder element id inserted into the parent's node tree.
  pub marker_id: String,
  pub component_type_id: TypeId,
  pub props: Box<dyn Any>,
  /// Factory: `(props: &dyn Any, wake) -> Box<dyn AnyComponent>`
  pub create: Box<dyn FnOnce(&dyn Any, Arc<dyn Fn() + Send + Sync>) -> Box<dyn crate::core::runtime::AnyComponent>>,
}

// ── Ctx ─────────────────────────────────────────────────────────────────────

/// Callback factory for a component.
///
/// Created by the runtime and passed to [`Component::view`].  Provides
/// methods to create event handlers that send messages, and to embed
/// child components.
/// Shared cache for scoped class names. Persists across renders so
/// repeated `ctx.scoped("foo")` calls return the same `ArcStr`
/// without allocating.
pub type ScopedClassCache = Arc<RefCell<HashMap<&'static str, ArcStr>>>;

pub struct Ctx<Msg: 'static> {
  pub(crate) sender: MsgSender<Msg>,
  pub(crate) children: RefCell<Vec<ChildSlot>>,
  pub(crate) child_counter: Cell<usize>,
  pub(crate) scope_prefix: &'static str,
  pub(crate) scoped_cache: ScopedClassCache,
}

impl<Msg: Clone + Send + Sync + 'static> Ctx<Msg> {
  pub(crate) fn new(sender: MsgSender<Msg>, scope_prefix: &'static str, scoped_cache: ScopedClassCache) -> Self {
    Self {
      sender,
      children: RefCell::new(Vec::new()),
      child_counter: Cell::new(0),
      scope_prefix,
      scoped_cache,
    }
  }

  /// Returns the component's scope prefix (empty if unscoped).
  pub fn scope(&self) -> &'static str {
    self.scope_prefix
  }

  /// Build a scoped class name: `"{prefix}-{class}"`.
  /// Cached across renders — second call with the same class is free.
  pub fn scoped(&self, class: &'static str) -> ArcStr {
    let cache = &self.scoped_cache;
    if let Some(cached) = cache.borrow().get(class) {
      return cached.clone();
    }
    let result = if self.scope_prefix.is_empty() {
      ArcStr::from(class)
    } else {
      ArcStr::from(format!("{}-{}", self.scope_prefix, class).as_str())
    };
    cache.borrow_mut().insert(class, result.clone());
    result
  }

  // ── Callback factories ──────────────────────────────────────────────

  /// Create a [`MouseCallback`] that sends a pre-built message.
  ///
  /// ```ignore
  /// el::button().text("+").on_click_cb(ctx.on_click(Msg::Inc))
  /// ```
  pub fn on_click(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that maps the event to a message.
  pub fn callback(&self, f: impl Fn(&MouseEvent) -> Msg + Send + Sync + 'static) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |ev: &MouseEvent| {
      sender.send(f(ev));
    })
  }

  /// Create an [`EventCallback`] that optionally maps an event to a
  /// message.
  pub fn event_callback(&self, f: impl Fn(&HtmlEvent) -> Option<Msg> + Send + Sync + 'static) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |ev: &HtmlEvent| {
      if let Some(m) = f(ev) {
        sender.send(m);
      }
    })
  }

  /// Create an [`EventCallback`] that sends a fixed message on `input` events.
  pub fn on_input(&self, msg: Msg) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &HtmlEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create an [`EventCallback`] that sends a fixed message on `change` events.
  pub fn on_change(&self, msg: Msg) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &HtmlEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create an [`EventCallback`] that sends a fixed message on `keydown` events.
  pub fn on_keydown(&self, msg: Msg) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &HtmlEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create an [`EventCallback`] that sends a fixed message on `focus` events.
  pub fn on_focus(&self, msg: Msg) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &HtmlEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create an [`EventCallback`] that sends a fixed message on `blur` events.
  pub fn on_blur(&self, msg: Msg) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &HtmlEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create an [`EventCallback`] that sends a fixed message on `wheel` events.
  pub fn on_wheel(&self, msg: Msg) -> EventCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &HtmlEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that sends a fixed message on `dblclick`.
  pub fn on_dblclick(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that sends a fixed message on `contextmenu`.
  pub fn on_contextmenu(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that sends a fixed message on `auxclick`.
  pub fn on_auxclick(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that sends a fixed message on `dragstart`.
  pub fn on_dragstart(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that sends a fixed message on `dragend`.
  pub fn on_dragend(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Create a [`MouseCallback`] that sends a fixed message on `drop`.
  pub fn on_drop(&self, msg: Msg) -> MouseCallback {
    let sender = self.sender.clone();
    Arc::new(move |_: &MouseEvent| {
      sender.send(msg.clone());
    })
  }

  /// Get a clone of the message sender for building custom callbacks
  /// (e.g. parent-provided closures in props) or passing to
  /// [`Observable::subscribe_msg`](crate::Observable::subscribe_msg).
  pub fn sender(&self) -> MsgSender<Msg> {
    self.sender.clone()
  }

  // ── Background tasks (#12) ──────────────────────────────────────────

  /// Spawn a blocking task on a background thread.
  ///
  /// `f` runs on a new OS thread. When it returns a message, that
  /// message is enqueued just like a normal callback message.
  ///
  /// ```ignore
  /// ctx.spawn(|| {
  ///     let data = std::fs::read_to_string("data.json").unwrap();
  ///     Msg::Loaded(data)
  /// });
  /// ```
  pub fn spawn<F>(&self, f: F)
  where
    F: FnOnce() -> Msg + Send + 'static,
  {
    let sender = self.sender.clone();
    std::thread::spawn(move || {
      sender.send(f());
    });
  }

  // ── Child components ────────────────────────────────────────────────

  /// Embed a child component.  The call-site position determines
  /// identity across re-renders (positional keying).
  ///
  /// Use [`keyed_child`](Ctx::keyed_child) when the child appears in a
  /// dynamic list so that it survives reordering without losing state.
  pub fn child<C: crate::Component>(&self, props: C::Props) -> El
  where
    C::Props: 'static,
    C::Msg: Clone + Send + Sync + 'static,
  {
    let idx = self.child_counter.get();
    self.child_counter.set(idx + 1);
    self.push_child::<C>(format!("__pos_{idx}"), props)
  }

  /// Embed a child component with an explicit string key.
  ///
  /// Children with the same key and type are considered the same
  /// instance across re-renders regardless of their call-site position.
  /// Use this for dynamic lists where items can be reordered or removed:
  ///
  /// ```ignore
  /// for item in &self.items {
  ///     row = row.child(ctx.keyed_child::<ItemRow>(
  ///         item.id.to_string(),
  ///         ItemProps { data: item.clone() },
  ///     ));
  /// }
  /// ```
  pub fn keyed_child<C: crate::Component>(&self, key: impl Into<String>, props: C::Props) -> El
  where
    C::Props: 'static,
    C::Msg: Clone + Send + Sync + 'static,
  {
    self.push_child::<C>(key.into(), props)
  }

  fn push_child<C: crate::Component>(&self, key: String, props: C::Props) -> El
  where
    C::Props: 'static,
    C::Msg: Clone + Send + Sync + 'static,
  {
    let marker_id = format!("__ui_child_{key}");

    let placeholder = El {
      node: Node::new(m::Div {
        id: Some(wgpu_html_models::ArcStr::from(marker_id.as_str())),
        ..Default::default()
      }),
    };

    self.children.borrow_mut().push(ChildSlot {
      key,
      marker_id,
      component_type_id: TypeId::of::<C>(),
      props: Box::new(props),
      create: Box::new(|props_any, wake| {
        let props = props_any.downcast_ref::<C::Props>().unwrap();
        Box::new(crate::core::runtime::ComponentState::<C>::new(props, wake))
      }),
    });

    placeholder
  }
}
