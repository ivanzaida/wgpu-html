use lui_css_parser::{validate_value, CssProperty, CssValue};

#[test]
fn unknown_keyword_is_invalid_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String("invalid-value".into()),
    );
    assert_eq!(result.valid, false);
    assert!(result.warning.is_some());
    let warning = result.warning.unwrap();
    assert!(warning.contains("invalid-value"));
    assert!(warning.contains("display"));
}
