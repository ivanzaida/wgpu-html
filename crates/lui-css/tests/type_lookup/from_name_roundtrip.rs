use lui_css::CssType;

#[test]
fn resolves_color_type_from_string_name() {
    assert_eq!(CssType::from_name("color"), Some(CssType::Color));
}

#[test]
fn resolves_kebab_case_type_from_string_name() {
    assert_eq!(CssType::from_name("length-percentage"), Some(CssType::LengthPercentage));
}

#[test]
fn name_roundtrips_through_from_name() {
    let t = CssType::Number;
    assert_eq!(CssType::from_name(t.name()), Some(t));
}
