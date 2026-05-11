use lui_css_parser::{parse_value, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn parses_bare_number_value() {
    assert_eq!(parse_value("42").unwrap(), CssValue::Number(42.0));
}

#[test]
fn parses_bare_negative_number_value() {
    assert_eq!(parse_value("-3.14").unwrap(), CssValue::Number(-3.14));
}