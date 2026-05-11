use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn static_is_valid_keyword_for_position() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::String("static".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn relative_is_valid_keyword_for_position() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::String("relative".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn absolute_is_valid_keyword_for_position() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::String("absolute".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
