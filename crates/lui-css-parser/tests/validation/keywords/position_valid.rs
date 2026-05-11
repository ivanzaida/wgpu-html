use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn static_is_valid_keyword_for_position() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::String(ArcStr::from("static")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn relative_is_valid_keyword_for_position() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::String(ArcStr::from("relative")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn absolute_is_valid_keyword_for_position() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::String(ArcStr::from("absolute")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}