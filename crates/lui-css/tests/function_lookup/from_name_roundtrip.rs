use lui_css::CssFunction;

#[test]
fn resolves_acos_from_string_name() {
    assert_eq!(CssFunction::from_name("acos"), Some(CssFunction::Acos));
}

#[test]
fn name_roundtrips_through_from_name() {
    let f = CssFunction::Abs;
    let name = f.clone().name().to_string();
    assert_eq!(CssFunction::from_name(&name), Some(f));
}
