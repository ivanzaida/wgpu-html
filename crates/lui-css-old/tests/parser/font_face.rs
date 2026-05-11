use lui_css_old::{CssParser, CssRule};

#[test]
fn parses_basic_font_face() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@font-face { font-family: 'MyFont'; src: url('myfont.woff2'); }");
  match &sheet.rules[0] {
    CssRule::FontFace(rule) => {
      assert_eq!(rule.descriptors.len(), 2);
      assert_eq!(&*rule.descriptors[0].name, "font-family");
      assert_eq!(&*rule.descriptors[0].value, "'MyFont'");
      assert_eq!(&*rule.descriptors[1].name, "src");
    }
    _ => panic!("expected font-face rule"),
  }
}

#[test]
fn parses_font_face_with_weight_and_style() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @font-face {
            font-family: 'Roboto';
            src: url('roboto-bold-italic.woff2') format('woff2');
            font-weight: bold;
            font-style: italic;
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::FontFace(rule) => {
      assert_eq!(rule.descriptors.len(), 4);
      let names: Vec<_> = rule.descriptors.iter().map(|d| &*d.name).collect();
      assert!(names.contains(&"font-weight"));
      assert!(names.contains(&"font-style"));
    }
    _ => panic!("expected font-face rule"),
  }
}

#[test]
fn parses_font_face_with_unicode_range() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @font-face {
            font-family: 'Noto';
            src: url('noto-latin.woff2');
            unicode-range: U+0000-00FF, U+0131, U+0152-0153;
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::FontFace(rule) => {
      assert_eq!(rule.descriptors.len(), 3);
      let unicode = rule.descriptors.iter().find(|d| &*d.name == "unicode-range").unwrap();
      assert!(unicode.value.contains("U+0000"));
    }
    _ => panic!("expected font-face rule"),
  }
}

#[test]
fn parses_multiple_font_face_rules() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @font-face { font-family: 'A'; src: url('a.woff2'); font-weight: 400; }
        @font-face { font-family: 'A'; src: url('a-bold.woff2'); font-weight: 700; }
    "#,
  );
  assert_eq!(sheet.rules.len(), 2);
  assert!(matches!(&sheet.rules[0], CssRule::FontFace(_)));
  assert!(matches!(&sheet.rules[1], CssRule::FontFace(_)));
}

#[test]
fn font_face_with_prelude_is_rejected() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@font-face something { font-family: 'X'; }");
  assert!(matches!(&sheet.rules[0], CssRule::Unknown(_)));
}

#[test]
fn parses_font_face_multiple_src() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @font-face {
            font-family: 'Inter';
            src: url('inter.woff2') format('woff2'),
                 url('inter.woff') format('woff');
            font-display: swap;
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::FontFace(rule) => {
      assert_eq!(rule.descriptors.len(), 3);
    }
    _ => panic!("expected font-face rule"),
  }
}
