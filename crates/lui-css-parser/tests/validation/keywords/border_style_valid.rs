use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn solid_is_valid_keyword_for_text_decoration_style() {
    let result = validate_value(
        &CssProperty::TextDecorationStyle,
        &CssValue::String("solid".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dashed_is_valid_keyword_for_text_decoration_style() {
    let result = validate_value(
        &CssProperty::TextDecorationStyle,
        &CssValue::String("dashed".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dotted_is_valid_keyword_for_text_decoration_style() {
    let result = validate_value(
        &CssProperty::TextDecorationStyle,
        &CssValue::String("dotted".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
