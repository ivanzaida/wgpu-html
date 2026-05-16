use std::sync::{Arc, Mutex};

use lui_live::Signal;

#[test]
fn set_notifies_subscribers_with_new_values() {
  let signal = Signal::new(0);
  let seen = Arc::new(Mutex::new(Vec::new()));
  let seen_for_sub = seen.clone();
  let _sub = signal.subscribe(move |value| {
    seen_for_sub.lock().unwrap().push(*value);
  });

  signal.set(1);
  signal.set(2);

  assert_eq!(*seen.lock().unwrap(), vec![1, 2]);
}
