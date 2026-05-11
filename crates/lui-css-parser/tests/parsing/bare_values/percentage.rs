use lui_css_parser::{parse_value, CssValue};

#[test]
fn parses_bare_percentage_value() {
    assert_eq!(parse_value("75%").unwrap(), CssValue::Percentage(75.0));
}
