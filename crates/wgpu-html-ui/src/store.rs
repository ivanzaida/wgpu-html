//! Shared reactive state — [`Store<T>`].
//!
//! A `Store` wraps a value of type `T` behind an `Arc<Mutex<T>>`.  Any
//! number of components can hold a clone of the same store.  When the
//! value is mutated via [`set`](Store::set) or [`update`](Store::update),
//! all registered subscriber callbacks are called synchronously on the
//! calling thread.
//!
//! # Subscribing from a component
//!
//! Subscribe inside [`Component::mounted`] using the provided
//! [`MsgSender`]:
//!
//! ```ignore
//! use wgpu_html_ui::{Component, Store, MsgSender, ShouldRender};
//!
//! struct MyComponent { /* ... */ }
//!
//! #[derive(Clone)]
//! enum Msg { ThemeChanged(String) }
//!
//! impl Component for MyComponent {
//!     // ...
//!
//!     fn mounted(&mut self, sender: MsgSender<Msg>) {
//!         THEME_STORE.subscribe(&sender, |theme| {
//!             Msg::ThemeChanged(theme.clone())
//!         });
//!     }
//! }
//! ```
//!
//! # Limitations
//!
//! Subscriptions are never automatically removed.  If a component is
//! destroyed while a `Store` it subscribed to still lives, the callback
//! keeps a `MsgSender` clone alive; messages sent to the orphaned queue
//! are silently discarded on the next `process` cycle.  This is a minor
//! memory overhead, not a crash.  A future version will add
//! `SubscriptionHandle` with automatic cleanup on drop.

use std::sync::{Arc, Mutex};

use crate::core::ctx::MsgSender;

// ── Inner ────────────────────────────────────────────────────────────────────

struct StoreInner<T> {
  value: Mutex<T>,
  listeners: Mutex<Vec<Box<dyn Fn(&T) + Send + Sync>>>,
}

// ── Store ────────────────────────────────────────────────────────────────────

/// Shared reactive state container.
///
/// Cheap to clone — all clones share the same underlying value.
pub struct Store<T> {
  inner: Arc<StoreInner<T>>,
}

impl<T> Clone for Store<T> {
  fn clone(&self) -> Self {
    Self {
      inner: Arc::clone(&self.inner),
    }
  }
}

impl<T: Send + Sync + 'static> Store<T> {
  /// Create a new store with an initial value.
  pub fn new(value: T) -> Self {
    Self {
      inner: Arc::new(StoreInner {
        value: Mutex::new(value),
        listeners: Mutex::new(Vec::new()),
      }),
    }
  }

  /// Read the current value (returns a clone).
  pub fn get(&self) -> T
  where
    T: Clone,
  {
    self.inner.value.lock().unwrap().clone()
  }

  /// Replace the value and notify all subscribers.
  pub fn set(&self, value: T) {
    {
      *self.inner.value.lock().unwrap() = value;
    }
    self.notify();
  }

  /// Mutate the value in-place and notify all subscribers.
  pub fn update(&self, f: impl FnOnce(&mut T)) {
    {
      f(&mut self.inner.value.lock().unwrap());
    }
    self.notify();
  }

  /// Register a raw callback that is called with `&T` on every mutation.
  ///
  /// Prefer [`subscribe`](Store::subscribe) for component integration.
  pub fn on_change(&self, f: impl Fn(&T) + Send + Sync + 'static) {
    self.inner.listeners.lock().unwrap().push(Box::new(f));
  }

  /// Subscribe a [`MsgSender`] to value changes.
  ///
  /// Each time the store is mutated, `map` is called with the new value
  /// and the returned message is enqueued in `sender`.
  ///
  /// Call this inside [`Component::mounted`](crate::Component::mounted):
  ///
  /// ```ignore
  /// fn mounted(&mut self, sender: MsgSender<Msg>) {
  ///     my_store.subscribe(&sender, |val| Msg::Updated(val.clone()));
  /// }
  /// ```
  pub fn subscribe<M>(&self, sender: &MsgSender<M>, map: impl Fn(&T) -> M + Send + Sync + 'static)
  where
    M: Clone + Send + Sync + 'static,
  {
    let sender = sender.clone();
    self.on_change(move |v| sender.send(map(v)));
  }

  fn notify(&self) {
    let value = self.inner.value.lock().unwrap();
    let listeners = self.inner.listeners.lock().unwrap();
    for cb in listeners.iter() {
      cb(&value);
    }
  }
}
