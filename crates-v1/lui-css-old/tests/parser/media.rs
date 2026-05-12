use lui_css_old::{
  stylesheet::{MediaFeature, MediaType},
  CssParser, CssRule,
};

#[test]
fn parses_basic_media_rule() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@media screen and (min-width: 768px) { .container { width: 100%; } }");
  assert_eq!(sheet.rules.len(), 1);
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      assert_eq!(rule.query.queries.len(), 1);
      let q = &rule.query.queries[0];
      assert!(!q.not);
      assert_eq!(q.media_type, MediaType::Screen);
      assert_eq!(q.features.len(), 1);
      assert_eq!(q.features[0], MediaFeature::MinWidth(768.0));
      assert_eq!(rule.rules.len(), 1);
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_media_with_multiple_features() {
  let parser = CssParser::new();
  let sheet =
    parser.parse_stylesheet("@media screen and (min-width: 768px) and (max-width: 1024px) { div { color: red; } }");
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      let q = &rule.query.queries[0];
      assert_eq!(q.features.len(), 2);
      assert_eq!(q.features[0], MediaFeature::MinWidth(768.0));
      assert_eq!(q.features[1], MediaFeature::MaxWidth(1024.0));
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_comma_separated_media_queries() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@media screen and (max-width: 600px), print { body { font-size: 12pt; } }");
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      assert_eq!(rule.query.queries.len(), 2);
      assert_eq!(rule.query.queries[0].media_type, MediaType::Screen);
      assert_eq!(rule.query.queries[1].media_type, MediaType::Print);
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_negated_media_query() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@media not print { body { background: white; } }");
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      assert!(rule.query.queries[0].not);
      assert_eq!(rule.query.queries[0].media_type, MediaType::Print);
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_only_keyword_in_media() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@media only screen and (min-width: 320px) { div { width: 100%; } }");
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      assert_eq!(rule.query.queries[0].media_type, MediaType::Screen);
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_media_with_multiple_rules_inside() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @media (max-width: 600px) {
            .header { display: none; }
            .sidebar { width: 100%; }
            .content { padding: 10px; }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      assert_eq!(rule.rules.len(), 3);
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_nested_media_rules() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @media screen {
            @media (min-width: 768px) {
                .wide { width: 960px; }
            }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Media(outer) => {
      assert_eq!(outer.rules.len(), 1);
      match &outer.rules[0] {
        CssRule::Media(inner) => {
          assert_eq!(inner.rules.len(), 1);
        }
        _ => panic!("expected nested media rule"),
      }
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_media_feature_only_query() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@media (orientation: landscape) { body { max-width: 120ch; } }");
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      let q = &rule.query.queries[0];
      assert_eq!(q.media_type, MediaType::All);
      assert_eq!(q.features[0], MediaFeature::OrientationLandscape);
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn parses_media_height_features() {
  let parser = CssParser::new();
  let sheet =
    parser.parse_stylesheet("@media (min-height: 400px) and (max-height: 800px) { div { overflow: scroll; } }");
  match &sheet.rules[0] {
    CssRule::Media(rule) => {
      let q = &rule.query.queries[0];
      assert_eq!(q.features.len(), 2);
      assert_eq!(q.features[0], MediaFeature::MinHeight(400.0));
      assert_eq!(q.features[1], MediaFeature::MaxHeight(800.0));
    }
    _ => panic!("expected media rule"),
  }
}
