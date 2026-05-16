use lui::live::Signal;

#[test]
fn signal_stores_and_returns_value() {
    let sig = Signal::new(42);
    assert_eq!(sig.get(), 42);
}

#[test]
fn set_replaces_value() {
    let sig = Signal::new(0);
    sig.set(99);
    assert_eq!(sig.get(), 99);
}

#[test]
fn update_mutates_in_place() {
    let sig = Signal::new(10);
    sig.update(|n| *n += 5);
    assert_eq!(sig.get(), 15);
}

#[test]
fn clone_shares_value() {
    let a = Signal::new(1);
    let b = a.clone();
    a.set(2);
    assert_eq!(b.get(), 2);
}

#[test]
fn get_untracked_reads_without_tracking() {
    let sig = Signal::new(7);
    assert_eq!(sig.get_untracked(), 7);
}

#[test]
fn with_untracked_reads_by_ref() {
    let sig = Signal::new(String::from("hello"));
    let len = sig.with_untracked(|s| s.len());
    assert_eq!(len, 5);
}
