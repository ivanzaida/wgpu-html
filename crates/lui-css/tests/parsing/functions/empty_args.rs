use lui_css::{parse_value, CssFunction, CssValue};

#[test]
fn parses_function_call_with_no_arguments() {
    assert_eq!(parse_value("var()").unwrap(),
        CssValue::Function { function: CssFunction::Var, args: vec![] }
    );
}
