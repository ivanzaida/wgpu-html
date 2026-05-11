use lui_css_parser::{parse_stylesheet, CssAtRule, CssProperty, CssValue};

#[test]
fn parses_media_containing_supports() {
    let css = r#"
@media (min-width: 500px) {
    @supports (display: flex) {
        div { display: flex; }
    }
}
"#;

    let sheet = parse_stylesheet(css).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);

    let media = &sheet.at_rules[0];
    assert_eq!(media.at_rule, CssAtRule::Media);
    assert_eq!(media.prelude, "(min-width: 500px)");
    assert_eq!(media.rules.len(), 0);
    assert_eq!(media.at_rules.len(), 1);

    let supports = &media.at_rules[0];
    assert_eq!(supports.at_rule, CssAtRule::Supports);
    assert_eq!(supports.prelude, "(display: flex)");

    let nested_rule = &supports.rules[0];
    let decl = &nested_rule.declarations[0];
    assert_eq!(decl.property, CssProperty::Display);
    assert_eq!(decl.value, CssValue::String("flex".into()));
}

#[test]
fn parses_media_containing_supports_with_multiple_nested_rules() {
    let css = r#"
@media screen {
    @supports (display: grid) {
        .grid { display: grid; }
        .row { grid-row: 1; }
    }
}
"#;

    let sheet = parse_stylesheet(css).unwrap();
    let media = &sheet.at_rules[0];
    assert_eq!(media.at_rule, CssAtRule::Media);
    assert_eq!(media.prelude, "screen");

    let supports = &media.at_rules[0];
    assert_eq!(supports.at_rule, CssAtRule::Supports);
    assert_eq!(supports.prelude, "(display: grid)");
    assert_eq!(supports.rules.len(), 2);
    assert_eq!(supports.at_rules.len(), 0);
}

#[test]
fn parses_media_containing_nested_media() {
    let css = r#"
@media (min-width: 600px) {
    @media (orientation: landscape) {
        p { font-size: 20px; }
    }
}
"#;

    let sheet = parse_stylesheet(css).unwrap();

    let outer = &sheet.at_rules[0];
    assert_eq!(outer.at_rule, CssAtRule::Media);
    assert_eq!(outer.prelude, "(min-width: 600px)");
    assert_eq!(outer.rules.len(), 0);
    assert_eq!(outer.at_rules.len(), 1);

    let inner = &outer.at_rules[0];
    assert_eq!(inner.at_rule, CssAtRule::Media);
    assert_eq!(inner.prelude, "(orientation: landscape)");
    assert_eq!(inner.rules.len(), 1);
    let nested_decl = &inner.rules[0].declarations[0];
    assert_eq!(nested_decl.property, CssProperty::FontSize);
}

#[test]
fn parses_deeply_nested_at_rules() {
    let css = r#"
@media screen {
    @supports (color: red) {
        @media (min-width: 300px) {
            span { opacity: 0.8; }
        }
    }
}
"#;

    let sheet = parse_stylesheet(css).unwrap();

    let media1 = &sheet.at_rules[0];
    assert_eq!(media1.at_rule, CssAtRule::Media);
    assert_eq!(media1.prelude, "screen");

    let supports = &media1.at_rules[0];
    assert_eq!(supports.at_rule, CssAtRule::Supports);
    assert_eq!(supports.prelude, "(color: red)");

    let media2 = &supports.at_rules[0];
    assert_eq!(media2.at_rule, CssAtRule::Media);
    assert_eq!(media2.prelude, "(min-width: 300px)");

    let deepest = &media2.rules[0].declarations[0];
    assert_eq!(deepest.property, CssProperty::Opacity);
}
