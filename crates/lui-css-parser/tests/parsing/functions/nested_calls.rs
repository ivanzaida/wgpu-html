use lui_css_parser::{parse_value, CssFunction, CssValue};

#[test]
fn parses_double_nested_function_call() {
    assert_eq!(parse_value("abs(acos(0.5))").unwrap(),
        CssValue::Function {
            function: CssFunction::Abs,
            args: vec![CssValue::Function {
                function: CssFunction::Acos,
                args: vec![CssValue::Number(0.5)],
            }],
        }
    );
}

#[test]
fn parses_triple_nested_function_call() {
    assert_eq!(parse_value("abs(acos(sin(0.3)))").unwrap(),
        CssValue::Function {
            function: CssFunction::Abs,
            args: vec![CssValue::Function {
                function: CssFunction::Acos,
                args: vec![CssValue::Function {
                    function: CssFunction::Sin,
                    args: vec![CssValue::Number(0.3)],
                }],
            }],
        }
    );
}
