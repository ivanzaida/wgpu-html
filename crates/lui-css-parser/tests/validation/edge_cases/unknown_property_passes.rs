use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn unknown_custom_property_always_passes_validation() {
    let result = validate_value(
        &CssProperty::Unknown("--custom".into()),
        &CssValue::String(ArcStr::from("anything")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}