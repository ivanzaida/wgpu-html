use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn solid_is_valid_keyword_for_text_decoration_style() {
    let result = validate_value(
        &CssProperty::TextDecorationStyle,
        &CssValue::String(ArcStr::from("solid")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dashed_is_valid_keyword_for_text_decoration_style() {
    let result = validate_value(
        &CssProperty::TextDecorationStyle,
        &CssValue::String(ArcStr::from("dashed")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dotted_is_valid_keyword_for_text_decoration_style() {
    let result = validate_value(
        &CssProperty::TextDecorationStyle,
        &CssValue::String(ArcStr::from("dotted")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}