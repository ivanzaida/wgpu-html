use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn unknown_custom_property_always_passes_validation() {
    let result = validate_value(
        &CssProperty::Unknown("--custom".into()),
        &CssValue::String("anything".into()),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
