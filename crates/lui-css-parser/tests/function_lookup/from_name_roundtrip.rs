use lui_css_parser::CssFunction;

#[test]
fn resolves_acos_from_string_name() {
    assert_eq!(CssFunction::from_name("acos"), CssFunction::Acos);
}

#[test]
fn name_roundtrips_through_from_name() {
    let f = CssFunction::Abs;
    assert_eq!(CssFunction::from_name(f.name()), f);
}
