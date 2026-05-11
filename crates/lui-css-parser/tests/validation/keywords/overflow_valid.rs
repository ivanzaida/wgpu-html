use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn visible_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String("visible".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn hidden_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String("hidden".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn scroll_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String("scroll".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn auto_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String("auto".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
