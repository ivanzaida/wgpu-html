use lui::live::Signal;

#[test]
fn from_creates_signal() {
    let sig: Signal<i32> = Signal::from(42);
    assert_eq!(sig.get(), 42);
}
