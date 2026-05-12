use lui_css_old::{CssParser, CssRule};

#[test]
fn parses_basic_supports_rule() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@supports (display: grid) { .grid { display: grid; } }");
  match &sheet.rules[0] {
    CssRule::Supports(rule) => {
      assert_eq!(&*rule.condition, "(display: grid)");
      assert_eq!(rule.rules.len(), 1);
    }
    _ => panic!("expected supports rule"),
  }
}

#[test]
fn parses_supports_with_complex_condition() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@supports (display: grid) and (gap: 1em) { .grid { gap: 1em; } }");
  match &sheet.rules[0] {
    CssRule::Supports(rule) => {
      assert!(rule.condition.contains("display: grid"));
      assert!(rule.condition.contains("gap: 1em"));
    }
    _ => panic!("expected supports rule"),
  }
}

#[test]
fn parses_supports_not() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@supports not (display: grid) { .fallback { display: flex; } }");
  match &sheet.rules[0] {
    CssRule::Supports(rule) => {
      assert!(rule.condition.starts_with("not"));
    }
    _ => panic!("expected supports rule"),
  }
}

#[test]
fn parses_supports_with_multiple_rules() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @supports (backdrop-filter: blur(10px)) {
            .glass { backdrop-filter: blur(10px); }
            .overlay { background: transparent; }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Supports(rule) => {
      assert_eq!(rule.rules.len(), 2);
    }
    _ => panic!("expected supports rule"),
  }
}

#[test]
fn empty_supports_condition_is_rejected() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@supports { .a { color: red; } }");
  assert!(matches!(&sheet.rules[0], CssRule::Unknown(_)));
}
