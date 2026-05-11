use lui_css_parser::{validate_value, CssProperty, CssUnit, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn dimension_value_is_invalid_for_position_property() {
    let result = validate_value(
        &CssProperty::Position,
        &CssValue::Dimension {
            value: 10.0,
            unit: CssUnit::Px,
        },
    );
    assert_eq!(result.valid, false);
    assert!(result.warning.is_some());
    let warning = result.warning.unwrap();
    assert!(warning.contains("dimension"));
    assert!(warning.contains("position"));
}