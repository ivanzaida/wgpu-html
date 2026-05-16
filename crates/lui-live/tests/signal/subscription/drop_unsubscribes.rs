use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

use lui_live::Signal;

#[test]
fn dropping_subscription_unsubscribes_callback() {
  let signal = Signal::new(0);
  let calls = Arc::new(AtomicUsize::new(0));
  let calls_for_sub = calls.clone();
  let sub = signal.subscribe(move |_| {
    calls_for_sub.fetch_add(1, Ordering::Relaxed);
  });

  signal.set(1);
  drop(sub);
  signal.set(2);

  assert_eq!(calls.load(Ordering::Relaxed), 1);
  assert_eq!(signal.subscriber_count(), 0);
}
