use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn grid_lanes_is_valid_keyword_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String(ArcStr::from("grid-lanes")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn inline_grid_lanes_is_valid_keyword_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String(ArcStr::from("inline-grid-lanes")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn math_is_valid_keyword_for_display() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::String(ArcStr::from("math")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}