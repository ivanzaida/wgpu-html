use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

/// WebkitBoxAlign has an empty syntax string, so any value should pass.
#[test]
fn property_with_empty_syntax_always_passes() {
    let result = validate_value(
        &CssProperty::WebkitBoxAlign,
        &CssValue::String("anything".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
