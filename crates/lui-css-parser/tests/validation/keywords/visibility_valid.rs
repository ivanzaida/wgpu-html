use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn visible_is_valid_keyword_for_visibility() {
    let result = validate_value(
        &CssProperty::Visibility,
        &CssValue::String("visible".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn hidden_is_valid_keyword_for_visibility() {
    let result = validate_value(
        &CssProperty::Visibility,
        &CssValue::String("hidden".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn collapse_is_valid_keyword_for_visibility() {
    let result = validate_value(
        &CssProperty::Visibility,
        &CssValue::String("collapse".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
