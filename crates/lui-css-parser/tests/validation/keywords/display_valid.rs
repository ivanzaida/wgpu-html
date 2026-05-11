use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn grid_lanes_is_valid_keyword_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String("grid-lanes".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn inline_grid_lanes_is_valid_keyword_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String("inline-grid-lanes".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn math_is_valid_keyword_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String("math".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
