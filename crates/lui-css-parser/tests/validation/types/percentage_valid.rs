use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn percentage_is_valid_for_font_stretch() {
    let result = validate_value(
        &CssProperty::FontStretch,
        &CssValue::Percentage(50.0),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn percentage_is_valid_for_border_image_slice() {
    let result = validate_value(
        &CssProperty::BorderImageSlice,
        &CssValue::Percentage(50.0),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
