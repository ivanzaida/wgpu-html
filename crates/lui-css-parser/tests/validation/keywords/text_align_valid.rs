use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn left_is_valid_keyword_for_text_align() {
    let result = validate_value(
        &CssProperty::TextAlign,
        &CssValue::String("left".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn center_is_valid_keyword_for_text_align() {
    let result = validate_value(
        &CssProperty::TextAlign,
        &CssValue::String("center".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn right_is_valid_keyword_for_text_align() {
    let result = validate_value(
        &CssProperty::TextAlign,
        &CssValue::String("right".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
