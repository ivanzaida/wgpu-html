use lui_css::{CssParser, CssRule};

#[test]
fn unknown_at_rule_with_block_is_preserved() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@layer base { .a { color: red; } }");
  match &sheet.rules[0] {
    CssRule::Unknown(rule) => {
      assert_eq!(&*rule.name, "layer");
      assert_eq!(&*rule.prelude, "base");
      assert!(rule.block.is_some());
    }
    _ => panic!("expected unknown at-rule"),
  }
}

#[test]
fn unknown_at_rule_statement_is_preserved() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@namespace svg \"http://www.w3.org/2000/svg\";");
  match &sheet.rules[0] {
    CssRule::Unknown(rule) => {
      assert_eq!(&*rule.name, "namespace");
      assert!(rule.block.is_none());
    }
    _ => panic!("expected unknown at-rule"),
  }
}

#[test]
fn unknown_at_rule_does_not_break_subsequent_rules() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @layer utilities { .sr-only { display: none; } }
        div { color: red; }
    "#,
  );
  assert_eq!(sheet.rules.len(), 2);
  assert!(matches!(&sheet.rules[0], CssRule::Unknown(_)));
  assert!(matches!(&sheet.rules[1], CssRule::Style(_)));
}

#[test]
fn unknown_at_rule_counter_style() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@counter-style thumbs { system: cyclic; symbols: '\\1F44D'; }");
  assert!(matches!(&sheet.rules[0], CssRule::Unknown(r) if &*r.name == "counter-style"));
}

#[test]
fn unknown_at_rule_page() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@page { margin: 1cm; }");
  assert!(matches!(&sheet.rules[0], CssRule::Unknown(r) if &*r.name == "page"));
}
