use lui_css_parser::{parse_value, CssUnit, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn parses_bare_dimension_value_with_em_unit() {
    assert_eq!(parse_value("1.5em").unwrap(),
        CssValue::Dimension { value: 1.5, unit: CssUnit::Em }
    );
}

#[test]
fn parses_bare_negative_dimension_value_with_px_unit() {
    assert_eq!(parse_value("-10px").unwrap(),
        CssValue::Dimension { value: -10.0, unit: CssUnit::Px }
    );
}