use bumpalo::Bump;
use lui_cascade::style::{ComputedStyle, alloc_value};
use lui_core::{ArcStr, CssValue};

#[test]
fn inherits_color() {
    let arena = Bump::new();
    let red = alloc_value(&arena, CssValue::Unknown("red".into()));

    let mut parent = ComputedStyle::default();
    parent.color = Some(red);

    let mut child = ComputedStyle::default();
    child.inherit_from(&parent);

    assert_eq!(child.color, Some(red));
}

#[test]
fn does_not_inherit_non_inherited_property() {
    let arena = Bump::new();
    let val = alloc_value(&arena, CssValue::Unknown("10px".into()));

    let mut parent = ComputedStyle::default();
    parent.margin_top = Some(val);

    let mut child = ComputedStyle::default();
    child.inherit_from(&parent);

    assert!(child.margin_top.is_none());
}

#[test]
fn child_value_wins_over_parent() {
    let arena = Bump::new();
    let red = alloc_value(&arena, CssValue::Unknown("red".into()));
    let blue = alloc_value(&arena, CssValue::Unknown("blue".into()));

    let mut parent = ComputedStyle::default();
    parent.color = Some(red);

    let mut child = ComputedStyle::default();
    child.color = Some(blue);
    child.inherit_from(&parent);

    assert_eq!(child.color, Some(blue));
}

#[test]
fn inherits_custom_properties() {
    let arena = Bump::new();
    let val = alloc_value(&arena, CssValue::Unknown("42".into()));

    let mut parent = ComputedStyle::default();
    parent.custom_properties
        .get_or_insert_with(Default::default)
        .insert(ArcStr::from("--my-var"), val);

    let mut child = ComputedStyle::default();
    child.inherit_from(&parent);

    let child_cp = child.custom_properties.as_ref().unwrap();
    assert_eq!(*child_cp.get("--my-var" as &str).unwrap(), val);
}

#[test]
fn inherits_font_family() {
    let arena = Bump::new();
    let font = alloc_value(&arena, CssValue::Unknown("Arial".into()));

    let mut parent = ComputedStyle::default();
    parent.font_family = Some(font);

    let mut child = ComputedStyle::default();
    child.inherit_from(&parent);

    assert_eq!(child.font_family, Some(font));
}

#[test]
fn does_not_inherit_display() {
    let arena = Bump::new();
    let val = alloc_value(&arena, CssValue::Unknown("flex".into()));

    let mut parent = ComputedStyle::default();
    parent.display = Some(val);

    let mut child = ComputedStyle::default();
    child.inherit_from(&parent);

    assert!(child.display.is_none());
}

#[test]
fn inherits_visibility() {
    let arena = Bump::new();
    let val = alloc_value(&arena, CssValue::Unknown("hidden".into()));

    let mut parent = ComputedStyle::default();
    parent.visibility = Some(val);

    let mut child = ComputedStyle::default();
    child.inherit_from(&parent);

    assert_eq!(child.visibility, Some(val));
}
