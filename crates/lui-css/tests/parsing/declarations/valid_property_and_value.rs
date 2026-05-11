use lui_css::{parse_declaration, CssProperty, CssValue};

#[test]
fn parses_valid_property_with_string_value() {
    let result = parse_declaration("color", "red").unwrap();
    assert_eq!(result, (CssProperty::Color, CssValue::String("red".into())));
}
