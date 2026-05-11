use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

/// WebkitBoxAlign has an empty syntax string, so any value should pass.
#[test]
fn property_with_empty_syntax_always_passes() {
    let result = validate_value(
        &CssProperty::WebkitBoxAlign,
        &CssValue::String(ArcStr::from("anything")),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}