use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn auto_is_valid_keyword_for_pointer_events() {
    let result = validate_value(
        &CssProperty::PointerEvents,
        &CssValue::String("auto".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn visible_is_valid_keyword_for_pointer_events() {
    let result = validate_value(
        &CssProperty::PointerEvents,
        &CssValue::String("visible".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn none_is_valid_keyword_for_pointer_events() {
    let result = validate_value(
        &CssProperty::PointerEvents,
        &CssValue::String("none".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
