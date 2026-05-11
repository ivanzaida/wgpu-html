use lui_css_parser::{parse_declaration, CssProperty, CssValue};

#[test]
fn unknown_property_produces_unknown_variant() {
    let (prop, val) = parse_declaration("bogus-prop", "auto").unwrap();
    assert_eq!(prop, CssProperty::Unknown("bogus-prop".into()));
    assert_eq!(val, CssValue::String("auto".into()));
}
