use lui_css::parse_value;

#[test]
fn returns_error_when_function_name_is_not_in_css_function_enum() {
    let err = parse_value("bogus(1)").unwrap_err();
    assert!(err.message.contains("unknown function"));
}
