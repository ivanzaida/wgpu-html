use lui_css_parser::CssFunction;

#[test]
fn unknown_function_returns_unknown_variant() {
    assert_eq!(
        CssFunction::from_name("nonexistent"),
        CssFunction::Unknown("nonexistent".into())
    );
}
