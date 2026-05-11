use lui_css::{parse_stylesheet, CssAtRule, CssColor, CssProperty, CssValue};

#[test]
fn parses_style_rule_before_at_rule() {
    let sheet = parse_stylesheet(
        "h1 { color: red; } @media print { p { font-size: 12px; } }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(sheet.at_rules.len(), 1);

    let rule = &sheet.rules[0];
    let decl = &rule.declarations[0];
    assert_eq!(decl.property, CssProperty::Color);
    assert_eq!(decl.value, CssValue::Color(CssColor::Named("red".into())));

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "print");
}

#[test]
fn parses_at_rule_before_style_rule() {
    let sheet = parse_stylesheet(
        "@import url(\"fonts.css\"); h1 { color: blue; }"
    ).unwrap();

    assert_eq!(sheet.at_rules.len(), 1);
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Import);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn parses_interleaved_rules_and_at_rules() {
    let sheet = parse_stylesheet(
        "h1 { color: red; } @media print { p { font-size: 12px; } } div { margin: 0; }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 2);
    assert_eq!(sheet.at_rules.len(), 1);
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Media);
}

#[test]
fn parses_multiple_block_at_rules_interleaved_with_style_rules() {
    let sheet = parse_stylesheet(
        "body { margin: 0; } @media screen { .sidebar { width: 200px; } } @supports (display: grid) { .grid { display: grid; } } p { line-height: 1.5; }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 2);
    assert_eq!(sheet.at_rules.len(), 2);

    // First at-rule
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Media);
    assert_eq!(sheet.at_rules[0].prelude, "screen");
    assert_eq!(sheet.at_rules[0].rules.len(), 1);

    // Second at-rule
    assert_eq!(sheet.at_rules[1].at_rule, CssAtRule::Supports);
    assert_eq!(sheet.at_rules[1].prelude, "(display: grid)");
    assert_eq!(sheet.at_rules[1].rules.len(), 1);

    // Style rules
    assert_eq!(sheet.rules[0].declarations[0].property, CssProperty::Margin);
    assert_eq!(sheet.rules[1].declarations[0].property, CssProperty::LineHeight);
}

#[test]
fn parses_at_rule_with_regular_rules_inside_and_outside() {
    let css = r#"
h1 { color: red; }
@media (min-width: 600px) {
    .wide { font-size: 18px; }
    .wide p { line-height: 1.6; }
}
footer { margin-top: 20px; }
"#;

    let sheet = parse_stylesheet(css).unwrap();

    assert_eq!(sheet.rules.len(), 2); // h1 and footer
    assert_eq!(sheet.at_rules.len(), 1);

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.rules.len(), 2); // .wide and .wide p

    // Verify outer style rules
    let h1_decl = &sheet.rules[0].declarations[0];
    assert_eq!(h1_decl.property, CssProperty::Color);

    let footer_decl = &sheet.rules[1].declarations[0];
    assert_eq!(footer_decl.property, CssProperty::MarginTop);
}

#[test]
fn parses_only_at_rules_with_no_regular_style_rules() {
    let sheet = parse_stylesheet(
        "@import url(\"base.css\"); @media print { .hidden { display: none; } }"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 2);
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Import);
    assert_eq!(sheet.at_rules[1].at_rule, CssAtRule::Media);
}
