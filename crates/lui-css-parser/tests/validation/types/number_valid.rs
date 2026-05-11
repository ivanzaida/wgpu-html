use lui_css_parser::{validate_value, CssProperty, CssValue, Validation};

#[test]
fn number_is_valid_for_z_index() {
    let result = validate_value(
        &CssProperty::ZIndex,
        &CssValue::Number(2.0),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn number_is_valid_for_orphans() {
    let result = validate_value(
        &CssProperty::Orphans,
        &CssValue::Number(2.0),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn number_is_valid_for_widows() {
    let result = validate_value(
        &CssProperty::Widows,
        &CssValue::Number(2.0),
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}
