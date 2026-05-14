use bumpalo::Bump;
use lui_cascade::style::{ComputedStyle, alloc_value};
use lui_core::{CssProperty, CssUnit, CssValue};

#[test]
fn set_and_get_hot_property() {
  let arena = Bump::new();
  let val = alloc_value(&arena, CssValue::Unknown("block".into()));
  let mut style = ComputedStyle::default();
  style.set(&CssProperty::Display, val);

  assert_eq!(style.get(&CssProperty::Display), Some(val));
  assert!(style.has(&CssProperty::Display));
}

#[test]
fn get_returns_none_for_unset_hot_property() {
  let style = ComputedStyle::default();
  assert_eq!(style.get(&CssProperty::Display), None);
  assert!(!style.has(&CssProperty::Display));
}

#[test]
fn set_and_get_cold_property() {
  let arena = Bump::new();
  let val = alloc_value(&arena, CssValue::Unknown("normal".into()));
  let mut style = ComputedStyle::default();
  style.set(&CssProperty::UnicodeBidi, val);

  assert_eq!(style.get(&CssProperty::UnicodeBidi), Some(val));
  assert!(style.has(&CssProperty::UnicodeBidi));
}

#[test]
fn cold_property_does_not_allocate_extra_until_needed() {
  let style = ComputedStyle::default();
  assert!(style.extra.is_none());
}

#[test]
fn cold_property_allocates_extra_on_first_set() {
  let arena = Bump::new();
  let val = alloc_value(&arena, CssValue::Unknown("x".into()));
  let mut style = ComputedStyle::default();
  style.set(&CssProperty::UnicodeBidi, val);

  assert!(style.extra.is_some());
}

#[test]
fn overwrite_hot_property() {
  let arena = Bump::new();
  let val1 = alloc_value(&arena, CssValue::Unknown("block".into()));
  let val2 = alloc_value(&arena, CssValue::Unknown("flex".into()));

  let mut style = ComputedStyle::default();
  style.set(&CssProperty::Display, val1);
  style.set(&CssProperty::Display, val2);

  assert_eq!(style.get(&CssProperty::Display), Some(val2));
}

#[test]
fn multiple_hot_properties_independent() {
  let arena = Bump::new();
  let block = alloc_value(&arena, CssValue::Unknown("block".into()));
  let red = alloc_value(&arena, CssValue::Unknown("red".into()));
  let px10 = alloc_value(
    &arena,
    CssValue::Dimension {
      value: 10.0,
      unit: CssUnit::Px,
    },
  );

  let mut style = ComputedStyle::default();
  style.set(&CssProperty::Display, block);
  style.set(&CssProperty::Color, red);
  style.set(&CssProperty::Width, px10);

  assert_eq!(style.get(&CssProperty::Display), Some(block));
  assert_eq!(style.get(&CssProperty::Color), Some(red));
  assert_eq!(style.get(&CssProperty::Width), Some(px10));
  assert_eq!(style.get(&CssProperty::Height), None);
}

#[test]
fn arena_allocated_values_survive() {
  let arena = Bump::new();
  let mut style = ComputedStyle::default();

  let val = alloc_value(
    &arena,
    CssValue::Dimension {
      value: 42.0,
      unit: CssUnit::Px,
    },
  );
  style.set(&CssProperty::MarginTop, val);

  if let Some(CssValue::Dimension { value, unit }) = style.get(&CssProperty::MarginTop) {
    assert_eq!(*value, 42.0);
    assert_eq!(*unit, CssUnit::Px);
  } else {
    panic!("expected Dimension");
  }
}

#[test]
fn direct_field_access_matches_get() {
  let arena = Bump::new();
  let val = alloc_value(&arena, CssValue::Unknown("relative".into()));
  let mut style = ComputedStyle::default();
  style.set(&CssProperty::Position, val);

  assert_eq!(style.position, Some(val));
  assert_eq!(style.get(&CssProperty::Position), style.position);
}
