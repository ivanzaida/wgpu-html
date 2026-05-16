use lui_live::Signal;

#[test]
fn cloned_signal_updates_same_value() {
  let signal = Signal::new(1);
  let cloned = signal.clone();

  cloned.update(|value| *value += 1);

  assert_eq!(signal.get(), 2);
}
