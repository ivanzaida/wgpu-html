use lui_css_parser::{validate_value, CssColor, CssProperty, CssValue, Validation};

#[test]
fn named_color_is_valid_for_color_property() {
    let result = validate_value(
        &CssProperty::Color,
        &CssValue::Color(CssColor::Named("red".into())),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn named_color_is_valid_for_background_color_property() {
    let result = validate_value(
        &CssProperty::BackgroundColor,
        &CssValue::Color(CssColor::Named("red".into())),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
