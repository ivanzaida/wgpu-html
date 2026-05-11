use lui_css::parse_value;

#[test]
fn returns_error_when_closing_paren_is_missing() {
    assert!(parse_value("acos(1").is_err());
}

#[test]
fn returns_error_when_opening_paren_is_missing() {
    assert!(parse_value("acos 1)").is_err());
}
