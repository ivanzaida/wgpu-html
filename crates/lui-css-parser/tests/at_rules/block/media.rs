use lui_css_parser::{parse_stylesheet, CssAtRule, CssColor, CssProperty, CssValue};

#[test]
fn parses_media_with_single_nested_rule() {
    let sheet = parse_stylesheet(
        "@media (max-width: 600px) { h1 { color: red; } }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "(max-width: 600px)");
    assert_eq!(at.rules.len(), 1);
    assert_eq!(at.at_rules.len(), 0);

    let nested = &at.rules[0];
    let decl = &nested.declarations[0];
    assert_eq!(decl.property, CssProperty::Color);
    assert_eq!(decl.value, CssValue::Color(CssColor::Named("red".into())));
}

#[test]
fn parses_media_with_multiple_nested_rules() {
    let sheet = parse_stylesheet(
        "@media screen and (min-width: 768px) { h1 { color: blue; } p { font-size: 16px; } }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "screen and (min-width: 768px)");
    assert_eq!(at.rules.len(), 2);
    assert_eq!(at.at_rules.len(), 0);
}

#[test]
fn parses_media_with_print_keyword() {
    let sheet = parse_stylesheet(
        "@media print { body { margin: 0; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "print");
    assert_eq!(at.rules.len(), 1);
}

#[test]
fn parses_media_with_empty_body() {
    let sheet = parse_stylesheet(
        "@media (max-width: 400px) {}"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "(max-width: 400px)");
    assert_eq!(at.rules.len(), 0);
    assert_eq!(at.at_rules.len(), 0);
}
