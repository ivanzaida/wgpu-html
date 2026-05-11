use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn left_is_valid_keyword_for_text_align() {
    let result = validate_value(
        &CssProperty::TextAlign,
        &CssValue::String(ArcStr::from("left")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn center_is_valid_keyword_for_text_align() {
    let result = validate_value(
        &CssProperty::TextAlign,
        &CssValue::String(ArcStr::from("center")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn right_is_valid_keyword_for_text_align() {
    let result = validate_value(
        &CssProperty::TextAlign,
        &CssValue::String(ArcStr::from("right")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}