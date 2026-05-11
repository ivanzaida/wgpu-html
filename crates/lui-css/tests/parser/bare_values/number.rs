use lui_css::{parse_value, CssValue};

#[test]
fn parses_bare_positive_number() {
    assert_eq!(parse_value("42").unwrap(), CssValue::Number(42.0));
}

#[test]
fn parses_bare_negative_number() {
    assert_eq!(parse_value("-3.14").unwrap(), CssValue::Number(-3.14));
}
