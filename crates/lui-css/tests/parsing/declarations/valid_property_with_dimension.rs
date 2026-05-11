use lui_css::{parse_declaration, CssProperty, CssUnit, CssValue};

#[test]
fn parses_valid_property_with_dimension_value() {
    let result = parse_declaration("margin-top", "10px").unwrap();
    assert_eq!(
        result,
        (
            CssProperty::MarginTop,
            CssValue::Dimension { value: 10.0, unit: CssUnit::Px },
        )
    );
}
