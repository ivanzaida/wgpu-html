use lui_css_old::{CssParser, CssRule};

#[test]
fn parses_mixed_rule_types() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @import "base.css";
        body { margin: 0; }
        @media (max-width: 600px) {
            body { padding: 10px; }
        }
        @keyframes spin {
            from { transform: rotate(0deg); }
            to { transform: rotate(360deg); }
        }
    "#,
  );
  assert_eq!(sheet.rules.len(), 4);
  assert!(matches!(&sheet.rules[0], CssRule::Import(_)));
  assert!(matches!(&sheet.rules[1], CssRule::Style(_)));
  assert!(matches!(&sheet.rules[2], CssRule::Media(_)));
  assert!(matches!(&sheet.rules[3], CssRule::Keyframes(_)));
}

#[test]
fn parses_real_world_reset_stylesheet() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(r#"
        *, *::before, *::after { box-sizing: border-box; }
        html { line-height: 1.15; -webkit-text-size-adjust: 100%; }
        body { margin: 0; }
        main { display: block; }
        h1 { font-size: 2em; margin: 0.67em 0; }
        hr { box-sizing: content-box; height: 0; overflow: visible; }
        pre { font-family: monospace, monospace; font-size: 1em; }
        a { background-color: transparent; }
        abbr[title] { border-bottom: none; text-decoration: underline dotted; }
        b, strong { font-weight: bolder; }
        code, kbd, samp { font-family: monospace, monospace; font-size: 1em; }
        small { font-size: 80%; }
        img { border-style: none; }
        button, input, optgroup, select, textarea { font-family: inherit; font-size: 100%; line-height: 1.15; margin: 0; }
    "#);
  assert!(sheet.rules.len() >= 12);
  for rule in &sheet.rules {
    assert!(matches!(rule, CssRule::Style(_)));
  }
}

#[test]
fn parses_real_world_responsive_layout() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @import url("https://fonts.googleapis.com/css2?family=Inter&display=swap");

        :root {
            --primary: #3b82f6;
            --spacing: 16px;
            --radius: 8px;
        }

        body {
            font-family: 'Inter', sans-serif;
            margin: 0;
            padding: 0;
            background-color: #f9fafb;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 0 var(--spacing);
        }

        @media (max-width: 768px) {
            .container { padding: 0 8px; }
            .grid { grid-template-columns: 1fr; }
        }

        @media (min-width: 769px) and (max-width: 1024px) {
            .grid { grid-template-columns: repeat(2, 1fr); }
        }

        @media (min-width: 1025px) {
            .grid { grid-template-columns: repeat(3, 1fr); }
        }

        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(20px); }
            to { opacity: 1; transform: translateY(0); }
        }

        .card {
            background: white;
            border-radius: var(--radius);
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            animation: fadeIn 0.3s ease-out;
        }

        @font-face {
            font-family: 'Icons';
            src: url('icons.woff2') format('woff2');
            font-display: swap;
        }
    "#,
  );

  let mut import_count = 0;
  let mut style_count = 0;
  let mut media_count = 0;
  let mut keyframes_count = 0;
  let mut font_face_count = 0;

  for rule in &sheet.rules {
    match rule {
      CssRule::Import(_) => import_count += 1,
      CssRule::Style(_) => style_count += 1,
      CssRule::Media(_) => media_count += 1,
      CssRule::Keyframes(_) => keyframes_count += 1,
      CssRule::FontFace(_) => font_face_count += 1,
      _ => {}
    }
  }

  assert_eq!(import_count, 1);
  assert!(style_count >= 3);
  assert_eq!(media_count, 3);
  assert_eq!(keyframes_count, 1);
  assert_eq!(font_face_count, 1);
}

#[test]
fn parses_bootstrap_like_grid_system() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        .row { display: flex; flex-wrap: wrap; margin-right: -15px; margin-left: -15px; }
        .col { flex-basis: 0; flex-grow: 1; max-width: 100%; padding-right: 15px; padding-left: 15px; }
        .col-6 { flex: 0 0 50%; max-width: 50%; }
        .col-4 { flex: 0 0 33.333333%; max-width: 33.333333%; }
        .col-3 { flex: 0 0 25%; max-width: 25%; }
        @media (min-width: 576px) {
            .col-sm-6 { flex: 0 0 50%; max-width: 50%; }
            .col-sm-4 { flex: 0 0 33.333333%; max-width: 33.333333%; }
        }
        @media (min-width: 768px) {
            .col-md-6 { flex: 0 0 50%; max-width: 50%; }
            .col-md-4 { flex: 0 0 33.333333%; max-width: 33.333333%; }
            .col-md-3 { flex: 0 0 25%; max-width: 25%; }
        }
    "#,
  );
  assert_eq!(sheet.rules.len(), 7);
}

#[test]
fn parses_empty_stylesheet() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("");
  assert!(sheet.rules.is_empty());
}

#[test]
fn parses_stylesheet_with_only_comments() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet("/* this is a comment */ /* another */");
  assert!(sheet.rules.is_empty());
}

#[test]
fn handles_deeply_nested_media_with_keyframes() {
  let parser = CssParser::new();
  let sheet = parser.parse_stylesheet(
    r#"
        @media screen {
            @supports (animation: fadeIn) {
                @keyframes fadeIn {
                    from { opacity: 0; }
                    to { opacity: 1; }
                }
                .animated { animation: fadeIn 1s; }
            }
        }
    "#,
  );
  assert_eq!(sheet.rules.len(), 1);
  match &sheet.rules[0] {
    CssRule::Media(media) => {
      assert_eq!(media.rules.len(), 1);
      match &media.rules[0] {
        CssRule::Supports(supports) => {
          assert_eq!(supports.rules.len(), 2);
          assert!(matches!(&supports.rules[0], CssRule::Keyframes(_)));
          assert!(matches!(&supports.rules[1], CssRule::Style(_)));
        }
        _ => panic!("expected supports inside media"),
      }
    }
    _ => panic!("expected media rule"),
  }
}

#[test]
fn stylesheet_append_combines_rules() {
  let parser = CssParser::new();
  let mut sheet1 = parser.parse_stylesheet("a { color: red; }");
  let sheet2 = parser.parse_stylesheet("b { color: blue; }");
  sheet1.append(sheet2);
  assert_eq!(sheet1.rules.len(), 2);
}
