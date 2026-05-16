use std::sync::{Arc, Mutex};

use lui_live::Signal;

#[test]
fn update_notifies_subscribers_after_mutation() {
  let signal = Signal::new(1);
  let seen = Arc::new(Mutex::new(Vec::new()));
  let seen_for_sub = seen.clone();
  let _sub = signal.subscribe(move |value| {
    seen_for_sub.lock().unwrap().push(*value);
  });

  signal.update(|value| *value += 10);

  assert_eq!(*seen.lock().unwrap(), vec![11]);
}
