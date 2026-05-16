use std::sync::{
  Arc, Mutex, RwLock, Weak,
  atomic::{AtomicUsize, Ordering},
};

pub type SignalSubscriber<T> = dyn Fn(&T) + Send + Sync + 'static;

struct SignalInner<T> {
  value: RwLock<T>,
  next_subscriber_id: AtomicUsize,
  subscribers: Mutex<Vec<(usize, Arc<SignalSubscriber<T>>)>>,
}

/// Cloneable reactive value.
///
/// Clones of a signal point at the same value and subscriber list, so they can
/// be moved into UI callbacks and updated without requiring `&mut Signal<T>`.
pub struct Signal<T> {
  inner: Arc<SignalInner<T>>,
}

/// RAII subscription handle.
///
/// Dropping the handle unsubscribes the callback.
#[must_use = "dropping the subscription immediately unsubscribes it"]
pub struct Subscription<T> {
  id: usize,
  inner: Weak<SignalInner<T>>,
}

impl<T> Clone for Signal<T> {
  fn clone(&self) -> Self {
    Self {
      inner: Arc::clone(&self.inner),
    }
  }
}

impl<T> Signal<T> {
  pub fn new(value: T) -> Self {
    Self {
      inner: Arc::new(SignalInner {
        value: RwLock::new(value),
        next_subscriber_id: AtomicUsize::new(0),
        subscribers: Mutex::new(Vec::new()),
      }),
    }
  }

  pub fn get(&self) -> T
  where
    T: Clone,
  {
    self.inner.value.read().unwrap().clone()
  }

  pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
    let value = self.inner.value.read().unwrap();
    f(&value)
  }

  pub fn set(&self, value: T) {
    {
      *self.inner.value.write().unwrap() = value;
    }
    self.changed();
  }

  pub fn update(&self, f: impl FnOnce(&mut T)) {
    {
      let mut value = self.inner.value.write().unwrap();
      f(&mut value);
    }
    self.changed();
  }

  pub fn subscribe(&self, sub: impl Fn(&T) + Send + Sync + 'static) -> Subscription<T> {
    let id = self.inner.next_subscriber_id.fetch_add(1, Ordering::Relaxed);
    self.inner.subscribers.lock().unwrap().push((id, Arc::new(sub)));
    Subscription {
      id,
      inner: Arc::downgrade(&self.inner),
    }
  }

  pub fn subscriber_count(&self) -> usize {
    self.inner.subscribers.lock().unwrap().len()
  }

  fn changed(&self) {
    let subscribers = self.inner.subscribers.lock().unwrap().clone();
    let value = self.inner.value.read().unwrap();
    for (_, sub) in subscribers {
      sub(&value);
    }
  }
}

impl<T> Drop for Subscription<T> {
  fn drop(&mut self) {
    if let Some(inner) = self.inner.upgrade() {
      inner.subscribers.lock().unwrap().retain(|(id, _)| *id != self.id);
    }
  }
}

impl<T> From<T> for Signal<T> {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}
