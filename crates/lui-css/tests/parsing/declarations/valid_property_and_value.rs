use lui_css::{parse_declaration, CssColor, CssProperty};

#[test]
fn parses_valid_property_with_string_value() {
    assert_eq!(
        parse_declaration("color", "red").unwrap(),
        (CssProperty::Color, lui_css::CssValue::Color(CssColor::Named("red".into())))
    );
}
