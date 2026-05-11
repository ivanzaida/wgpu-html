use lui_css_old::syntax::{parse_raw_rules, RawRule};

#[test]
fn parses_single_qualified_rule() {
  let rules = parse_raw_rules("div { color: red; }");
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert_eq!(r.prelude, "div");
      assert!(r.block.contains("color: red"));
    }
    _ => panic!("expected qualified rule"),
  }
}

#[test]
fn parses_multiple_qualified_rules() {
  let rules = parse_raw_rules("h1 { color: red; } p { margin: 0; }");
  assert_eq!(rules.len(), 2);
}

#[test]
fn preserves_complex_selector_in_prelude() {
  let rules = parse_raw_rules("div > p.active:hover { color: blue; }");
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert_eq!(r.prelude, "div > p.active:hover");
    }
    _ => panic!("expected qualified rule"),
  }
}

#[test]
fn preserves_attribute_selector_with_brackets() {
  let rules = parse_raw_rules("input[type=\"text\"] { border: 1px; }");
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert!(r.prelude.contains("[type=\"text\"]"));
    }
    _ => panic!("expected qualified rule"),
  }
}

#[test]
fn preserves_comma_separated_selectors() {
  let rules = parse_raw_rules("h1, h2, h3 { font-weight: bold; }");
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert_eq!(r.prelude, "h1, h2, h3");
    }
    _ => panic!("expected qualified rule"),
  }
}

#[test]
fn empty_block_is_valid() {
  let rules = parse_raw_rules("div { }");
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert!(r.block.trim().is_empty());
    }
    _ => panic!("expected qualified rule"),
  }
}

#[test]
fn skips_rule_with_empty_prelude() {
  let rules = parse_raw_rules("{ color: red; }");
  assert_eq!(rules.len(), 0);
}

#[test]
fn handles_strings_with_braces_in_content() {
  let rules = parse_raw_rules(r#"div::before { content: "{hello}"; }"#);
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert!(r.block.contains("{hello}"));
    }
    _ => panic!("expected qualified rule"),
  }
}
