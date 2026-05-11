use lui_css::CssType;

#[test]
fn resolves_angle_type_from_string() {
    let t = CssType::from_name("angle").unwrap();
    assert_eq!(t, CssType::Angle);
    assert_eq!(t.name(), "angle");
}

#[test]
fn resolves_color_type_from_string() {
    let t = CssType::from_name("color").unwrap();
    assert_eq!(t, CssType::Color);
    assert_eq!(t.name(), "color");
}

#[test]
fn resolves_length_type_from_string() {
    let t = CssType::from_name("length").unwrap();
    assert_eq!(t, CssType::Length);
    assert_eq!(t.name(), "length");
}

#[test]
fn resolves_number_type_from_string() {
    let t = CssType::from_name("number").unwrap();
    assert_eq!(t, CssType::Number);
    assert_eq!(t.name(), "number");
}

#[test]
fn resolves_percentage_type_from_string() {
    let t = CssType::from_name("percentage").unwrap();
    assert_eq!(t, CssType::Percentage);
    assert_eq!(t.name(), "percentage");
}

#[test]
fn roundtrip_name_from_name_is_identity_for_basic_types() {
    let types = [
        CssType::Angle,
        CssType::Color,
        CssType::Length,
        CssType::Number,
        CssType::Percentage,
        CssType::Ident,
    ];
    for ty in types {
        assert_eq!(CssType::from_name(ty.name()), Some(ty));
    }
}

#[test]
fn returns_none_for_unknown_type_name() {
    assert_eq!(CssType::from_name("bogus"), None);
}
