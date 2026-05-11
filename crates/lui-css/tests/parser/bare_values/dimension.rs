use lui_css::{parse_value, CssValue};

#[test]
fn parses_bare_dimension_value_with_float_and_unit() {
    assert_eq!(
        parse_value("1.5em").unwrap(),
        CssValue::Dimension { value: 1.5, unit: "em".into() }
    );
}
