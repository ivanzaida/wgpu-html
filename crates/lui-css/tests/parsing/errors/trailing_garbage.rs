use lui_css::parse_value;

#[test]
fn returns_error_for_text_after_a_complete_value() {
    assert!(parse_value("acos(1) blah").is_err());
}
