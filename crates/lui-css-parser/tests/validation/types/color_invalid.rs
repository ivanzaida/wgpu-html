use lui_css_parser::{validate_value, CssColor, CssProperty, CssValue};

#[test]
fn color_value_is_invalid_for_display_property() {
    let result = validate_value(
        &CssProperty::Display,
        &CssValue::Color(CssColor::Named("red".into())),
    );
    assert_eq!(result.valid, false);
    assert!(result.warning.is_some());
    let warning = result.warning.unwrap();
    assert!(warning.contains("color"));
    assert!(warning.contains("display"));
}
