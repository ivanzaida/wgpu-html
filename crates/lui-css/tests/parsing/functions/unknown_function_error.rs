use lui_css::{parse_value, CssFunction, CssValue};

#[test]
fn unknown_function_produces_unknown_variant() {
    let result = parse_value("bogus(1)").unwrap();
    assert_eq!(result, CssValue::Function {
        function: CssFunction::Unknown("bogus".into()),
        args: vec![CssValue::Number(1.0)],
    });
}
