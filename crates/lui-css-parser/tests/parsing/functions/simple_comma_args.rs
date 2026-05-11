use lui_css_parser::{parse_value, CssFunction, CssValue};

#[test]
fn parses_function_with_three_comma_separated_number_args() {
    assert_eq!(parse_value("rgb(255, 0, 128)").unwrap(),
        CssValue::Function {
            function: CssFunction::Rgb,
            args: vec![CssValue::Number(255.0), CssValue::Number(0.0), CssValue::Number(128.0)],
        }
    );
}

#[test]
fn parses_function_with_mixed_number_and_percentage_args() {
    assert_eq!(parse_value("hsl(180, 100%, 50%)").unwrap(),
        CssValue::Function {
            function: CssFunction::Hsl,
            args: vec![CssValue::Number(180.0), CssValue::Percentage(100.0), CssValue::Percentage(50.0)],
        }
    );
}
