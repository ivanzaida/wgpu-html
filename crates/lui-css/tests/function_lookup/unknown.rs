use lui_css::CssFunction;

#[test]
fn returns_none_for_unknown_function_name() {
    assert_eq!(CssFunction::from_name("bogus"), None);
}

#[test]
fn returns_none_for_empty_string() {
    assert_eq!(CssFunction::from_name(""), None);
}
