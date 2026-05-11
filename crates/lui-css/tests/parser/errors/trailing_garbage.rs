use lui_css::parse_value;

#[test]
fn returns_error_when_extra_tokens_appear_after_valid_expression() {
    assert!(parse_value("acos(1) blah").is_err());
}
