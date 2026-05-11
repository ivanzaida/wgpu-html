use lui_css_parser::{parse_declaration, CssColor, CssProperty};

#[test]
fn parses_valid_property_with_string_value() {
    assert_eq!(
        parse_declaration("color", "red").unwrap(),
        (CssProperty::Color, lui_css_parser::CssValue::Color(CssColor::Named("red".into())))
    );
}
