use lui_css_old::{CssParser, CssRule};

#[test]
fn parses_double_quoted_import() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@import \"reset.css\";");
  match &sheet.rules[0] {
    CssRule::Import(rule) => {
      assert_eq!(&*rule.url, "reset.css");
      assert!(rule.media.is_none());
    }
    _ => panic!("expected import rule"),
  }
}

#[test]
fn parses_single_quoted_import() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@import 'base.css';");
  match &sheet.rules[0] {
    CssRule::Import(rule) => {
      assert_eq!(&*rule.url, "base.css");
    }
    _ => panic!("expected import rule"),
  }
}

#[test]
fn parses_url_function_import() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@import url(\"theme.css\");");
  match &sheet.rules[0] {
    CssRule::Import(rule) => {
      assert_eq!(&*rule.url, "theme.css");
    }
    _ => panic!("expected import rule"),
  }
}

#[test]
fn parses_url_function_import_unquoted() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@import url(theme.css);");
  match &sheet.rules[0] {
    CssRule::Import(rule) => {
      assert_eq!(&*rule.url, "theme.css");
    }
    _ => panic!("expected import rule"),
  }
}

#[test]
fn parses_import_with_media_query() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@import \"print.css\" print;");
  match &sheet.rules[0] {
    CssRule::Import(rule) => {
      assert_eq!(&*rule.url, "print.css");
      assert_eq!(rule.media.as_deref(), Some("print"));
    }
    _ => panic!("expected import rule"),
  }
}

#[test]
fn parses_url_import_with_media_query() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@import url(\"mobile.css\") screen and (max-width: 600px);");
  match &sheet.rules[0] {
    CssRule::Import(rule) => {
      assert_eq!(&*rule.url, "mobile.css");
      assert!(rule.media.as_deref().unwrap().contains("screen"));
    }
    _ => panic!("expected import rule"),
  }
}

#[test]
fn multiple_imports() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @import "reset.css";
        @import "typography.css";
        @import "layout.css";
    "#,
  );
  assert_eq!(sheet.rules.len(), 3);
  for rule in &sheet.rules {
    assert!(matches!(rule, CssRule::Import(_)));
  }
}
