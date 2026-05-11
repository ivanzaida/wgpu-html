use lui_css_parser::{validate_value, CssProperty, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn unknown_keyword_is_invalid_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String(ArcStr::from("invalid-value")),
    );
    assert_eq!(result.valid, false);
    assert!(result.warning.is_some());
    let warning = result.warning.unwrap();
    assert!(warning.contains("invalid-value"));
    assert!(warning.contains("display"));
}