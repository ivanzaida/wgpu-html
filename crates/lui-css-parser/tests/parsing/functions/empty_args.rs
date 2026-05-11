use lui_css_parser::{parse_value, CssFunction, CssValue};

#[test]
fn parses_function_call_with_no_arguments() {
    assert_eq!(parse_value("translate()").unwrap(),
        CssValue::Function { function: CssFunction::Translate, args: vec![] }
    );
}
