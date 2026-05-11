use lui_css::{parse_value, CssFunction, CssValue};

fn f(name: &str, args: Vec<CssValue>) -> CssValue {
    CssValue::Function { function: CssFunction::from_name(name).unwrap(), args }
}
fn n(v: f64) -> CssValue { CssValue::Number(v) }
fn p(v: f64) -> CssValue { CssValue::Percentage(v) }
fn d(v: f64, u: &str) -> CssValue { CssValue::Dimension { value: v, unit: u.into() } }
fn s(v: &str) -> CssValue { CssValue::String(v.into()) }

#[test]
fn parses_rgb_with_three_comma_separated_arguments() {
    assert_eq!(
        parse_value("rgb(255, 0, 128)").unwrap(),
        f("rgb", vec![n(255.0), n(0.0), n(128.0)]),
    );
}

#[test]
fn parses_rgba_with_percentage_arg() {
    assert_eq!(
        parse_value("rgba(255, 0, 128, 50%)").unwrap(),
        f("rgba", vec![n(255.0), n(0.0), n(128.0), p(50.0)]),
    );
}

#[test]
fn parses_hsl_with_mixed_number_and_percentage_args() {
    assert_eq!(
        parse_value("hsl(180, 100%, 50%)").unwrap(),
        f("hsl", vec![n(180.0), p(100.0), p(50.0)]),
    );
}

#[test]
fn parses_function_with_dimension_args() {
    assert_eq!(
        parse_value("blur(5px)").unwrap(),
        f("blur", vec![d(5.0, "px")]),
    );
}

#[test]
fn parses_function_with_string_arg() {
    assert_eq!(
        parse_value("url(\"image.png\")").unwrap(),
        f("url", vec![s("image.png")]),
    );
}

#[test]
fn parses_function_with_mixed_dimension_args() {
    assert_eq!(
        parse_value("drop-shadow(10px, 5px, 2px)").unwrap(),
        f("drop-shadow", vec![d(10.0, "px"), d(5.0, "px"), d(2.0, "px")]),
    );
}

#[test]
fn parses_function_surrounded_by_whitespace() {
    assert_eq!(
        parse_value("  acos (  1  )  ").unwrap(),
        f("acos", vec![n(1.0)]),
    );
}
