use lui_glyph::FontHandle;

#[test]
fn handles_with_same_index_are_equal() {
  assert_eq!(FontHandle(0), FontHandle(0));
  assert_eq!(FontHandle(42), FontHandle(42));
}

#[test]
fn handles_with_different_index_are_not_equal() {
  assert_ne!(FontHandle(0), FontHandle(1));
}

#[test]
fn handles_are_ordered_by_index() {
  assert!(FontHandle(0) < FontHandle(1));
  assert!(FontHandle(1) < FontHandle(100));
}

#[test]
fn handles_are_copyable() {
  let a = FontHandle(5);
  let b = a;
  assert_eq!(a, b);
}

#[test]
fn handles_are_hashable() {
  use std::collections::HashSet;
  let mut set = HashSet::new();
  set.insert(FontHandle(0));
  set.insert(FontHandle(1));
  set.insert(FontHandle(0));
  assert_eq!(set.len(), 2);
}
