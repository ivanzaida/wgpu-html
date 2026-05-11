use lui_css_parser::{parse_stylesheet, CssAtRule, CssProperty, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn parses_supports_with_single_nested_rule() {
    let sheet = parse_stylesheet(
        "@supports (display: grid) { div { display: grid; } }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Supports);
    assert_eq!(at.prelude, "(display: grid)");
    assert_eq!(at.rules.len(), 1);
    assert_eq!(at.at_rules.len(), 0);

    let nested = &at.rules[0];
    let decl = &nested.declarations[0];
    assert_eq!(decl.property, CssProperty::Display);
    assert_eq!(decl.value, CssValue::String(ArcStr::from("grid")));
}

#[test]
fn parses_supports_with_not_condition() {
    let sheet = parse_stylesheet(
        "@supports not (display: flex) { .fallback { display: block; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Supports);
    assert_eq!(at.prelude, "not (display: flex)");
    assert_eq!(at.rules.len(), 1);
}

#[test]
fn parses_supports_with_and_condition() {
    let sheet = parse_stylesheet(
        "@supports (display: flex) and (gap: 10px) { .layout { display: flex; gap: 10px; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Supports);
    assert_eq!(at.prelude, "(display: flex) and (gap: 10px)");
    assert_eq!(at.rules.len(), 1);
}

#[test]
fn parses_supports_with_or_condition() {
    let sheet = parse_stylesheet(
        "@supports (display: grid) or (display: flex) { .container { display: grid; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Supports);
    assert_eq!(at.prelude, "(display: grid) or (display: flex)");
    assert_eq!(at.rules.len(), 1);
}

#[test]
fn parses_supports_with_empty_body() {
    let sheet = parse_stylesheet(
        "@supports (transform: scale(1)) {}"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Supports);
    assert_eq!(at.prelude, "(transform: scale(1))");
    assert_eq!(at.rules.len(), 0);
    assert_eq!(at.at_rules.len(), 0);
}

#[test]
fn parses_supports_with_multiple_nested_rules() {
    let sheet = parse_stylesheet(
        "@supports (display: flex) { .a { color: red; } .b { color: blue; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Supports);
    assert_eq!(at.rules.len(), 2);
}