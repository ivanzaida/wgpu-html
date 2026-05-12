use lui_css_old::syntax::{parse_raw_rules, RawRule};

#[test]
fn nested_at_rule_inside_media() {
  let input = r#"
        @media screen {
            @media (min-width: 768px) {
                div { color: red; }
            }
        }
    "#;
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::AtRule(r) => {
      assert_eq!(r.name, "media");
      let block = r.block.as_deref().unwrap();
      assert!(block.contains("@media"));
      assert!(block.contains("min-width"));
    }
    _ => panic!("expected at-rule"),
  }
}

#[test]
fn multiple_rules_inside_media_block() {
  let input = "@media print { h1 { color: black; } p { font-size: 12pt; } }";
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::AtRule(r) => {
      let block = r.block.as_deref().unwrap();
      assert!(block.contains("h1"));
      assert!(block.contains("p"));
    }
    _ => panic!("expected at-rule"),
  }
}

#[test]
fn deeply_nested_braces() {
  let input = "@supports (display: grid) { @media screen { .a { color: red; } } }";
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 1);
}

#[test]
fn string_containing_braces_does_not_break_nesting() {
  let input = r#"div::after { content: "{ }"; color: red; }"#;
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::QualifiedRule(r) => {
      assert!(r.block.contains("color: red"));
    }
    _ => panic!("expected qualified rule"),
  }
}

#[test]
fn single_quoted_string_with_braces() {
  let input = "div::before { content: '{ nested }'; }";
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 1);
}

#[test]
fn real_world_media_with_multiple_rules_and_import() {
  let input = r#"
        @import "base.css";
        body { margin: 0; padding: 0; }
        @media (max-width: 600px) {
            body { padding: 10px; }
            .sidebar { display: none; }
        }
        footer { text-align: center; }
    "#;
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 4);
  assert!(matches!(&rules[0], RawRule::AtRule(r) if r.name == "import"));
  assert!(matches!(&rules[1], RawRule::QualifiedRule(_)));
  assert!(matches!(&rules[2], RawRule::AtRule(r) if r.name == "media"));
  assert!(matches!(&rules[3], RawRule::QualifiedRule(_)));
}

#[test]
fn comments_stripped_from_nested_blocks() {
  let input = "@media screen { /* comment */ div { color: red; } }";
  let rules = parse_raw_rules(input);
  assert_eq!(rules.len(), 1);
  match &rules[0] {
    RawRule::AtRule(r) => {
      let block = r.block.as_deref().unwrap();
      assert!(!block.contains("/*"));
    }
    _ => panic!("expected at-rule"),
  }
}
