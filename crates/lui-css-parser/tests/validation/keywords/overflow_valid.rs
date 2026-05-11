use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn visible_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String(ArcStr::from("visible")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn hidden_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String(ArcStr::from("hidden")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn scroll_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String(ArcStr::from("scroll")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn auto_is_valid_keyword_for_overflow_x() {
    let result = validate_value(
        &CssProperty::OverflowX,
        &CssValue::String(ArcStr::from("auto")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}