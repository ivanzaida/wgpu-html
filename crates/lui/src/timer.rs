use std::{
  sync::mpsc,
  time::{Duration, Instant},
};

use crate::Lui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimerHandle(u64);

type TimerCallback = Box<dyn FnMut(&mut Lui) + Send>;

enum TimerRequest {
  Once { delay: Duration, callback: TimerCallback, id: u64 },
  Repeat { interval: Duration, callback: TimerCallback, id: u64 },
  Immediate { callback: TimerCallback, id: u64 },
  Clear(u64),
}

struct ActiveTimer {
  id: u64,
  fire_at: Instant,
  interval: Option<Duration>,
  callback: TimerCallback,
}

pub struct Timers {
  tx: mpsc::Sender<TimerRequest>,
  rx: mpsc::Receiver<TimerRequest>,
  active: Vec<ActiveTimer>,
  next_id: u64,
}

impl Default for Timers {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Clone)]
pub struct TimerSender {
  tx: mpsc::Sender<TimerRequest>,
  next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl TimerSender {
  pub fn set_timeout(&self, delay: Duration, callback: impl FnMut(&mut Lui) + Send + 'static) -> TimerHandle {
    let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let _ = self.tx.send(TimerRequest::Once { delay, callback: Box::new(callback), id });
    TimerHandle(id)
  }

  pub fn set_interval(&self, interval: Duration, callback: impl FnMut(&mut Lui) + Send + 'static) -> TimerHandle {
    let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let _ = self.tx.send(TimerRequest::Repeat { interval, callback: Box::new(callback), id });
    TimerHandle(id)
  }

  pub fn set_immediate(&self, callback: impl FnMut(&mut Lui) + Send + 'static) -> TimerHandle {
    let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let _ = self.tx.send(TimerRequest::Immediate { callback: Box::new(callback), id });
    TimerHandle(id)
  }

  pub fn clear_timer(&self, handle: TimerHandle) {
    let _ = self.tx.send(TimerRequest::Clear(handle.0));
  }
}

impl Timers {
  pub fn new() -> Self {
    let (tx, rx) = mpsc::channel();
    Self { tx, rx, active: Vec::new(), next_id: 1 }
  }

  pub fn sender(&self) -> TimerSender {
    TimerSender {
      tx: self.tx.clone(),
      next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(self.next_id)),
    }
  }

  fn alloc_id(&mut self) -> u64 {
    let id = self.next_id;
    self.next_id += 1;
    id
  }

  pub fn set_timeout(&mut self, delay: Duration, callback: impl FnMut(&mut Lui) + Send + 'static) -> TimerHandle {
    let id = self.alloc_id();
    self.active.push(ActiveTimer {
      id,
      fire_at: Instant::now() + delay,
      interval: None,
      callback: Box::new(callback),
    });
    TimerHandle(id)
  }

  pub fn set_interval(&mut self, interval: Duration, callback: impl FnMut(&mut Lui) + Send + 'static) -> TimerHandle {
    let id = self.alloc_id();
    self.active.push(ActiveTimer {
      id,
      fire_at: Instant::now() + interval,
      interval: Some(interval),
      callback: Box::new(callback),
    });
    TimerHandle(id)
  }

  pub fn set_immediate(&mut self, callback: impl FnMut(&mut Lui) + Send + 'static) -> TimerHandle {
    let id = self.alloc_id();
    self.active.push(ActiveTimer {
      id,
      fire_at: Instant::now(),
      interval: None,
      callback: Box::new(callback),
    });
    TimerHandle(id)
  }

  pub fn clear_timer(&mut self, handle: TimerHandle) {
    self.active.retain(|t| t.id != handle.0);
  }

  pub fn has_pending(&self) -> bool {
    !self.active.is_empty()
  }

  fn drain_channel(&mut self) {
    while let Ok(req) = self.rx.try_recv() {
      match req {
        TimerRequest::Once { delay, callback, id } => {
          self.active.push(ActiveTimer {
            id,
            fire_at: Instant::now() + delay,
            interval: None,
            callback,
          });
        }
        TimerRequest::Repeat { interval, callback, id } => {
          self.active.push(ActiveTimer {
            id,
            fire_at: Instant::now() + interval,
            interval: Some(interval),
            callback,
          });
        }
        TimerRequest::Immediate { callback, id } => {
          self.active.push(ActiveTimer {
            id,
            fire_at: Instant::now(),
            interval: None,
            callback,
          });
        }
        TimerRequest::Clear(id) => {
          self.active.retain(|t| t.id != id);
        }
      }
    }
  }
}

pub(crate) fn tick_timers(lui: &mut Lui) -> bool {
  lui.timers.drain_channel();

  let now = Instant::now();
  let mut expired: Vec<ActiveTimer> = Vec::new();
  let mut i = 0;
  while i < lui.timers.active.len() {
    if now >= lui.timers.active[i].fire_at {
      expired.push(lui.timers.active.swap_remove(i));
    } else {
      i += 1;
    }
  }

  let fired = !expired.is_empty();
  for mut timer in expired {
    (timer.callback)(lui);
    if let Some(interval) = timer.interval {
      timer.fire_at = Instant::now() + interval;
      lui.timers.active.push(timer);
    }
  }
  fired
}
