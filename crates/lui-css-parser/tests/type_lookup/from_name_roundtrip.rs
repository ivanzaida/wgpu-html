use lui_css_parser::CssType;

#[test]
fn resolves_color_type_from_string_name() {
    assert_eq!(CssType::from_name("color"), CssType::Color);
}

#[test]
fn resolves_kebab_case_type_from_string_name() {
    assert_eq!(CssType::from_name("length-percentage"), CssType::LengthPercentage);
}

#[test]
fn name_roundtrips_through_from_name() {
    let t = CssType::Number;
    assert_eq!(CssType::from_name(t.name()), t);
}

#[test]
fn unknown_type_returns_unknown_variant() {
    assert_eq!(
        CssType::from_name("nonexistent"),
        CssType::Unknown("nonexistent".into())
    );
}
