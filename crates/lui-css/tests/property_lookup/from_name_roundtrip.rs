use lui_css::CssProperty;

#[test]
fn resolves_background_color_from_name() {
    assert_eq!(CssProperty::from_name("background-color"), Some(CssProperty::BackgroundColor));
}

#[test]
fn resolves_border_radius_from_name() {
    assert_eq!(CssProperty::from_name("border-radius"), Some(CssProperty::BorderRadius));
}

#[test]
fn name_roundtrips_through_from_name() {
    let p = CssProperty::Display;
    let name = p.clone().name().to_string();
    assert_eq!(CssProperty::from_name(&name), Some(p));
}

#[test]
fn returns_none_for_unknown_property() {
    assert_eq!(CssProperty::from_name("not-a-real-property"), None);
}

#[test]
fn display_initial_is_inline() {
    assert_eq!(CssProperty::Display.initial(), "inline");
}

#[test]
fn color_is_inherited() {
    assert!(CssProperty::Color.inherited());
}

#[test]
fn background_color_is_not_inherited() {
    assert!(!CssProperty::BackgroundColor.inherited());
}
