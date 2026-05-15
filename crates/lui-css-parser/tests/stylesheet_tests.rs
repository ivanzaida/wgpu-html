use lui_css_parser::{parse_stylesheet, CssColor, CssProperty, CssPseudo, CssValue};
use lui_css_parser::ArcStr;

#[test]
fn parses_simple_rule() {
    let sheet = parse_stylesheet("h1 { color: red; }").unwrap();
    assert_eq!(sheet.rules.len(), 1);
    let rule = &sheet.rules[0];
    let decl = &rule.declarations[0];
    assert_eq!(decl.property, CssProperty::Color);
    assert_eq!(decl.value, CssValue::Color(CssColor::Named(ArcStr::from("red"))));
    assert!(!decl.important);
}

#[test]
fn parses_multiple_rules() {
    let sheet = parse_stylesheet("h1 { color: red; } p { font-size: 14px; }").unwrap();
    assert_eq!(sheet.rules.len(), 2);
}

#[test]
fn parses_important() {
    let sheet = parse_stylesheet("h1 { color: red !important; }").unwrap();
    assert!(sheet.rules[0].declarations[0].important);
}

#[test]
fn parses_complex_selector_rule() {
    let sheet = parse_stylesheet("div.foo > span { display: none; }").unwrap();
    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(sheet.rules[0].specificity, (0, 1, 2)); // 1 class, 2 tags
}

#[test]
fn expands_nested_ampersand_hover_after_pseudo_element() {
    let sheet = parse_stylesheet(
        r#"
        *::lui-scrollbar-thumb {
            &:hover {
                background-color: red;
            }
        }
        "#,
    )
    .unwrap();

    assert_eq!(sheet.rules.len(), 1);
    let rule = &sheet.rules[0];
    assert_eq!(rule.declarations[0].property, CssProperty::BackgroundColor);

    let compound = &rule.selector.0[0].compounds[0];
    assert_eq!(compound.tag.as_deref(), Some("*"));
    assert_eq!(compound.pseudos.len(), 2);
    assert_eq!(compound.pseudos[0].pseudo, CssPseudo::LuiScrollbarThumb);
    assert_eq!(compound.pseudos[1].pseudo, CssPseudo::Hover);
}
