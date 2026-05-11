use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn auto_is_valid_keyword_for_pointer_events() {
    let result = validate_value(
        &CssProperty::PointerEvents,
        &CssValue::String(ArcStr::from("auto")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn visible_is_valid_keyword_for_pointer_events() {
    let result = validate_value(
        &CssProperty::PointerEvents,
        &CssValue::String(ArcStr::from("visible")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn none_is_valid_keyword_for_pointer_events() {
    let result = validate_value(
        &CssProperty::PointerEvents,
        &CssValue::String(ArcStr::from("none")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}