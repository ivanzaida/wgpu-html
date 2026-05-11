use lui_css::parse_value;

#[test]
fn returns_error_for_unknown_function_name() {
    let err = parse_value("bogus(1)").unwrap_err();
    assert!(err.message.contains("unknown function"));
}
