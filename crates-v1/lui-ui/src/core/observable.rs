//! Shared reactive state.

use std::sync::{Arc, Mutex, RwLock, Weak};

use crate::core::ctx::MsgSender;

type Listener<T> = Arc<dyn Fn(&T) + Send + Sync>;

struct ObservableInner<T> {
  value: RwLock<T>,
  next_listener_id: Mutex<usize>,
  listeners: Mutex<Vec<(usize, Listener<T>)>>,
}

/// Cloneable reactive value.
///
/// All clones point at the same value. Mutations notify current
/// subscribers synchronously and can wake component runtimes through
/// [`subscribe_msg`](Self::subscribe_msg).
pub struct Observable<T> {
  inner: Arc<ObservableInner<T>>,
}

impl<T: Send + Sync + 'static> Default for Observable<T>
where
  T: Default,
{
  fn default() -> Self {
    Self::new(T::default())
  }
}

impl<T> Clone for Observable<T> {
  fn clone(&self) -> Self {
    Self {
      inner: Arc::clone(&self.inner),
    }
  }
}

#[must_use = "subscriptions are cancelled when dropped — store in a field or Subscriptions bag"]
pub struct Subscription<T> {
  id: usize,
  inner: Weak<ObservableInner<T>>,
}

/// Type-erased collection of subscriptions. Dropping the bag cancels
/// all contained subscriptions. Used by the component runtime to
/// auto-clean subscriptions when a component is destroyed.
pub struct Subscriptions {
  entries: Vec<Box<dyn std::any::Any>>,
}

impl Subscriptions {
  pub fn new() -> Self {
    Self { entries: Vec::new() }
  }

  pub fn add<T: 'static>(&mut self, sub: Subscription<T>) {
    self.entries.push(Box::new(sub));
  }

  pub fn clear(&mut self) {
    self.entries.clear();
  }
}

impl Default for Subscriptions {
  fn default() -> Self {
    Self::new()
  }
}

impl<T> Drop for Subscription<T> {
  fn drop(&mut self) {
    if let Some(inner) = self.inner.upgrade() {
      inner.listeners.lock().unwrap().retain(|(id, _)| *id != self.id);
    }
  }
}

impl<T: Send + Sync + 'static> Observable<T> {
  /// Create a reactive value.
  pub fn new(value: impl Into<T>) -> Self {
    Self {
      inner: Arc::new(ObservableInner {
        value: RwLock::new(value.into()),
        next_listener_id: Mutex::new(0),
        listeners: Mutex::new(Vec::new()),
      }),
    }
  }

  /// Read the current value (shared read lock — no contention).
  pub fn get(&self) -> T
  where
    T: Clone,
  {
    self.inner.value.read().unwrap().clone()
  }

  /// Access the current value by reference without cloning.
  pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
    let guard = self.inner.value.read().unwrap();
    f(&*guard)
  }

  /// Replace the value and notify subscribers.
  pub fn set(&self, value: impl Into<T>) {
    {
      *self.inner.value.write().unwrap() = value.into();
    }
    self.notify();
  }

  /// Mutate the value in place and notify subscribers.
  pub fn update(&self, f: impl FnOnce(&mut T)) {
    {
      f(&mut self.inner.value.write().unwrap());
    }
    self.notify();
  }

  /// Subscribe to value changes.
  pub fn subscribe(&self, f: impl Fn(&T) + Send + Sync + 'static) -> Subscription<T> {
    let mut next = self.inner.next_listener_id.lock().unwrap();
    let id = *next;
    *next += 1;
    self.inner.listeners.lock().unwrap().push((id, Arc::new(f)));
    Subscription {
      id,
      inner: Arc::downgrade(&self.inner),
    }
  }

  /// Subscribe a component message sender to value changes.
  pub fn subscribe_msg<M>(
    &self,
    sender: &MsgSender<M>,
    map: impl Fn(&T) -> M + Send + Sync + 'static,
  ) -> Subscription<T>
  where
    M: Clone + Send + Sync + 'static,
  {
    let sender = sender.clone();
    self.subscribe(move |value| sender.send(map(value)))
  }

  fn notify(&self) {
    let listeners = self.inner.listeners.lock().unwrap().clone();
    let value = self.inner.value.read().unwrap();
    for (_, cb) in listeners {
      cb(&value);
    }
  }
}
