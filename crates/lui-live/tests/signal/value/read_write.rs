use lui_live::Signal;

#[test]
fn get_returns_current_value_after_set() {
  let signal = Signal::new(1);

  assert_eq!(signal.get(), 1);

  signal.set(2);

  assert_eq!(signal.get(), 2);
}
