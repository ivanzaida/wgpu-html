use lui_css_old::{CssParser, CssRule, KeyframeSelector};

#[test]
fn parses_basic_keyframes() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }");
  match &sheet.rules[0] {
    CssRule::Keyframes(rule) => {
      assert_eq!(&*rule.name, "fadeIn");
      assert_eq!(rule.keyframes.len(), 2);
      assert_eq!(rule.keyframes[0].selectors, vec![KeyframeSelector::From]);
      assert_eq!(rule.keyframes[1].selectors, vec![KeyframeSelector::To]);
    }
    _ => panic!("expected keyframes rule"),
  }
}

#[test]
fn parses_percentage_keyframe_selectors() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @keyframes slide {
            0% { transform: translateX(0); }
            50% { transform: translateX(50%); }
            100% { transform: translateX(100%); }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Keyframes(rule) => {
      assert_eq!(rule.keyframes.len(), 3);
      assert_eq!(rule.keyframes[0].selectors, vec![KeyframeSelector::Percentage(0.0)]);
      assert_eq!(rule.keyframes[1].selectors, vec![KeyframeSelector::Percentage(50.0)]);
      assert_eq!(rule.keyframes[2].selectors, vec![KeyframeSelector::Percentage(100.0)]);
    }
    _ => panic!("expected keyframes rule"),
  }
}

#[test]
fn parses_keyframes_with_multiple_selectors_per_block() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Keyframes(rule) => {
      assert_eq!(rule.keyframes.len(), 2);
      assert_eq!(
        rule.keyframes[0].selectors,
        vec![KeyframeSelector::Percentage(0.0), KeyframeSelector::Percentage(100.0),]
      );
    }
    _ => panic!("expected keyframes rule"),
  }
}

#[test]
fn parses_quoted_keyframes_name() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("@keyframes \"my-animation\" { from { opacity: 0; } to { opacity: 1; } }");
  match &sheet.rules[0] {
    CssRule::Keyframes(rule) => {
      assert_eq!(&*rule.name, "my-animation");
    }
    _ => panic!("expected keyframes rule"),
  }
}

#[test]
fn parses_keyframe_with_multiple_properties() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @keyframes complex {
            from {
                opacity: 0;
                transform: scale(0.5) rotate(0deg);
                background-color: red;
            }
            to {
                opacity: 1;
                transform: scale(1) rotate(360deg);
                background-color: blue;
            }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Keyframes(rule) => {
      assert_eq!(rule.keyframes[0].declarations.len(), 3);
      assert_eq!(rule.keyframes[1].declarations.len(), 3);
    }
    _ => panic!("expected keyframes rule"),
  }
}

#[test]
fn parses_vendor_prefixed_keyframes() {
  let parser = CssParser::new();
  let sheet =
    parser.parse_stylesheet("@keyframes spin { from { transform: rotate(0); } to { transform: rotate(360deg); } }");
  assert_eq!(sheet.rules.len(), 1);
  assert!(matches!(&sheet.rules[0], CssRule::Keyframes(_)));
}

#[test]
fn parses_many_keyframe_stops() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @keyframes rainbow {
            0% { color: red; }
            16% { color: orange; }
            33% { color: yellow; }
            50% { color: green; }
            66% { color: blue; }
            83% { color: indigo; }
            100% { color: violet; }
        }
    "#,
  );
  match &sheet.rules[0] {
    CssRule::Keyframes(rule) => {
      assert_eq!(rule.keyframes.len(), 7);
    }
    _ => panic!("expected keyframes rule"),
  }
}
