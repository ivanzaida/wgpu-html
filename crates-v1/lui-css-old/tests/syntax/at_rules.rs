use lui_css_old::syntax::{parse_raw_rules, RawRule};

#[test]
fn parses_at_rule_with_block() {
  let rules = parse_raw_rules("@media screen { div { color: red; } }");
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::AtRule(r) => {
      assert_eq!(r.name, "media");
      assert_eq!(r.prelude, "screen");
      assert!(r.block.is_some());
    }
    _ => panic!("expected at-rule"),
  }
}

#[test]
fn parses_at_rule_statement() {
  let rules = parse_raw_rules("@import url(\"foo.css\");");
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::AtRule(r) => {
      assert_eq!(r.name, "import");
      assert!(r.prelude.contains("foo.css"));
      assert!(r.block.is_none());
    }
    _ => panic!("expected at-rule"),
  }
}

#[test]
fn parses_charset_statement() {
  let rules = parse_raw_rules("@charset \"UTF-8\";");
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::AtRule(r) => {
      assert_eq!(r.name, "charset");
      assert!(r.block.is_none());
    }
    _ => panic!("expected at-rule"),
  }
}

#[test]
fn at_rule_before_qualified_rule() {
  let rules = parse_raw_rules("@import \"a.css\"; div { color: red; }");
  assert_eq!(rules.len(), 2);
  assert!(matches!(&rules[0], RawRule::AtRule(_)));
  assert!(matches!(&rules[1], RawRule::QualifiedRule(_)));
}

#[test]
fn multiple_at_rules() {
  let rules = parse_raw_rules("@import \"a.css\"; @import \"b.css\";");
  assert_eq!(rules.len(), 2);
}

#[test]
fn at_rule_with_complex_prelude() {
  let rules = parse_raw_rules("@media screen and (min-width: 768px) and (max-width: 1024px) { div { color: red; } }");
  match &rules[0] {
    RawRule::AtRule(r) => {
      assert!(r.prelude.contains("min-width: 768px"));
      assert!(r.prelude.contains("max-width: 1024px"));
    }
    _ => panic!("expected at-rule"),
  }
}

#[test]
fn at_rule_with_hyphenated_name() {
  let rules = parse_raw_rules("@font-face { font-family: 'X'; }");
  match &rules[0] {
    RawRule::AtRule(r) => {
      assert_eq!(r.name, "font-face");
    }
    _ => panic!("expected at-rule"),
  }
}
