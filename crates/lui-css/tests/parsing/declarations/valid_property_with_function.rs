use lui_css::{parse_declaration, CssFunction, CssProperty, CssUnit, CssValue};

#[test]
fn parses_valid_property_with_function_value() {
    let result = parse_declaration("width", "calc(100%, 20px)").unwrap();
    assert_eq!(
        result,
        (
            CssProperty::Width,
            CssValue::Function {
                function: CssFunction::Calc,
                args: vec![
                    CssValue::Percentage(100.0),
                    CssValue::Dimension { value: 20.0, unit: CssUnit::Px },
                ],
            },
        )
    );
}
