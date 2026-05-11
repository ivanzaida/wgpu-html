use lui_css::{parse_value, CssFunction, CssValue};

fn f(name: &str, args: Vec<CssValue>) -> CssValue {
    CssValue::Function { function: CssFunction::from_name(name).unwrap(), args }
}
fn n(v: f64) -> CssValue { CssValue::Number(v) }

#[test]
fn parses_nested_function_call_with_one_level() {
    assert_eq!(
        parse_value("abs(acos(0.5))").unwrap(),
        f("abs", vec![f("acos", vec![n(0.5)])]),
    );
}

#[test]
fn parses_deeply_nested_function_calls_with_three_levels() {
    assert_eq!(
        parse_value("abs(acos(sin(0.3)))").unwrap(),
        f("abs", vec![f("acos", vec![f("sin", vec![n(0.3)])])]),
    );
}

#[test]
fn parses_function_with_negative_argument() {
    assert_eq!(
        parse_value("abs(-5)").unwrap(),
        f("abs", vec![n(-5.0)]),
    );
}
