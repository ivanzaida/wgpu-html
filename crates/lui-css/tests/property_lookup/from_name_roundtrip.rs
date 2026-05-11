use lui_css::CssProperty;

#[test]
fn resolves_background_color_from_name() {
    assert_eq!(CssProperty::from_name("background-color"), CssProperty::BackgroundColor);
}

#[test]
fn resolves_border_radius_from_name() {
    assert_eq!(CssProperty::from_name("border-radius"), CssProperty::BorderRadius);
}

#[test]
fn name_roundtrips_through_from_name() {
    let p = CssProperty::Display;
    assert_eq!(CssProperty::from_name(p.name()), p);
}

#[test]
fn unknown_property_returns_unknown_variant() {
    assert_eq!(
        CssProperty::from_name("not-a-real-property"),
        CssProperty::Unknown("not-a-real-property".into())
    );
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
