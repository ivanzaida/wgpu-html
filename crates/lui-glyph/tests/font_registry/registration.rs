use lui_glyph::{FontHandle, FontRegistry};
use crate::helpers::dummy_face;

#[test]
fn new_registry_is_empty() {
    let reg = FontRegistry::new();
    assert!(reg.is_empty());
    assert_eq!(reg.len(), 0);
}

#[test]
fn register_increments_length() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("A"));
    assert_eq!(reg.len(), 1);
    reg.register(dummy_face("B"));
    assert_eq!(reg.len(), 2);
}

#[test]
fn register_returns_sequential_handles() {
    let mut reg = FontRegistry::new();
    let h0 = reg.register(dummy_face("A"));
    let h1 = reg.register(dummy_face("B"));
    let h2 = reg.register(dummy_face("C"));
    assert_eq!(h0, FontHandle(0));
    assert_eq!(h1, FontHandle(1));
    assert_eq!(h2, FontHandle(2));
}

#[test]
fn get_returns_registered_face() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face("TestFont"));
    let face = reg.get(h).unwrap();
    assert_eq!(face.family, "TestFont");
}

#[test]
fn get_returns_none_for_invalid_handle() {
    let reg = FontRegistry::new();
    assert!(reg.get(FontHandle(0)).is_none());
    assert!(reg.get(FontHandle(999)).is_none());
}

#[test]
fn generation_starts_at_zero() {
    let reg = FontRegistry::new();
    assert_eq!(reg.generation(), 0);
}

#[test]
fn generation_increments_on_each_register() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("A"));
    assert_eq!(reg.generation(), 1);
    reg.register(dummy_face("B"));
    assert_eq!(reg.generation(), 2);
}

#[test]
fn iter_yields_all_registered_faces() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("Alpha"));
    reg.register(dummy_face("Beta"));
    reg.register(dummy_face("Gamma"));

    let items: Vec<_> = reg.iter().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].0, FontHandle(0));
    assert_eq!(items[0].1.family, "Alpha");
    assert_eq!(items[1].1.family, "Beta");
    assert_eq!(items[2].1.family, "Gamma");
}

#[test]
fn iter_empty_registry_yields_nothing() {
    let reg = FontRegistry::new();
    assert_eq!(reg.iter().count(), 0);
}

#[test]
fn default_is_same_as_new() {
    let a = FontRegistry::new();
    let b = FontRegistry::default();
    assert_eq!(a.len(), b.len());
    assert_eq!(a.generation(), b.generation());
    assert!(a.is_empty());
    assert!(b.is_empty());
}
