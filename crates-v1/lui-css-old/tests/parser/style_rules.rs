use lui_css_old::{CssParser, CssRule, Importance};

#[test]
fn parses_single_style_rule() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("div { color: red; font-size: 14px; }");
  assert_eq!(sheet.rules.len(), 1);
  match &sheet.rules[0] {
    CssRule::Style(rule) => {
      assert_eq!(&*rule.selector_text, "div");
      assert_eq!(rule.declarations.len(), 2);
    }
    _ => panic!("expected style rule"),
  }
}

#[test]
fn preserves_complex_selector() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("#main > .item:nth-child(2n+1) { opacity: 0.5; }");
  match &sheet.rules[0] {
    CssRule::Style(rule) => {
      assert_eq!(&*rule.selector_text, "#main > .item:nth-child(2n+1)");
    }
    _ => panic!("expected style rule"),
  }
}

#[test]
fn important_declarations_through_parser() {
  let parser = CssParser::new();
  let block = parser.parse_declarations("color: red !important; font-size: 12px;");
  assert_eq!(block.declarations[0].importance, Importance::Important);
  assert_eq!(block.declarations[1].importance, Importance::Normal);
}

#[test]
fn multiple_selectors_preserved() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("h1, h2, h3, h4, h5, h6 { margin-top: 0; }");
  match &sheet.rules[0] {
    CssRule::Style(rule) => {
      assert!(rule.selector_text.contains("h1"));
      assert!(rule.selector_text.contains("h6"));
    }
    _ => panic!("expected style rule"),
  }
}

#[test]
fn parses_pseudo_element_selector() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("p::first-line { text-transform: uppercase; }");
  match &sheet.rules[0] {
    CssRule::Style(rule) => {
      assert_eq!(&*rule.selector_text, "p::first-line");
    }
    _ => panic!("expected style rule"),
  }
}

#[test]
fn parses_declaration_with_url_containing_colon() {
  let parser = CssParser::new();
  let block = parser.parse_declarations("background: url(https://example.com/img.png);");
  assert_eq!(block.len(), 1);
  assert!(block.declarations[0].value.contains("https://example.com"));
}

#[test]
fn parses_attribute_selector_unquoted_with_case_flag() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("ol[type=a s] { list-style-type: lower-alpha; }");
  assert_eq!(sheet.rules.len(), 1);
  match &sheet.rules[0] {
    CssRule::Style(rule) => {
      assert_eq!(&*rule.selector_text, "ol[type=a s]");
      assert_eq!(rule.declarations.len(), 1);
      assert_eq!(&*rule.declarations.declarations[0].property, "list-style-type");
      assert_eq!(&*rule.declarations.declarations[0].value, "lower-alpha");
    }
    _ => panic!("expected style rule"),
  }
}

#[test]
fn parses_attribute_selector_with_comma_list() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(r#"ol[type="1"], li[type="1"] { list-style-type: decimal; }"#);
  assert_eq!(sheet.rules.len(), 1);
  match &sheet.rules[0] {
    CssRule::Style(rule) => {
      assert_eq!(&*rule.selector_text, r#"ol[type="1"], li[type="1"]"#);
      assert_eq!(rule.declarations.len(), 1);
      assert_eq!(&*rule.declarations.declarations[0].property, "list-style-type");
      assert_eq!(&*rule.declarations.declarations[0].value, "decimal");
    }
    _ => panic!("expected style rule"),
  }
}

#[test]
fn parses_custom_properties() {
  let parser = CssParser::new();
  let block = parser.parse_declarations("--primary-color: #ff0000; --spacing: 16px;");
  assert_eq!(block.len(), 2);
  assert_eq!(&*block.declarations[0].property, "--primary-color");
  assert_eq!(&*block.declarations[0].value, "#ff0000");
}
