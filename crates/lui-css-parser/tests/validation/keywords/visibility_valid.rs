use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn visible_is_valid_keyword_for_visibility() {
    let result = validate_value(
        &CssProperty::Visibility,
        &CssValue::String(ArcStr::from("visible")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn hidden_is_valid_keyword_for_visibility() {
    let result = validate_value(
        &CssProperty::Visibility,
        &CssValue::String(ArcStr::from("hidden")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn collapse_is_valid_keyword_for_visibility() {
    let result = validate_value(
        &CssProperty::Visibility,
        &CssValue::String(ArcStr::from("collapse")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}