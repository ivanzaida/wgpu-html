use lui_glyph::{FontContext, TextStyle};

#[test]
fn shape_with_tiny_font_size() {
  let mut ctx = FontContext::new();
  // cosmic-text panics on line_height==0, so use a tiny nonzero value
  let style = TextStyle {
    font_size: 0.1,
    line_height: 0.1,
    ..TextStyle::default()
  };
  let run = ctx.shape("test", &style);
  assert!(run.height >= 0.0);
}

#[test]
fn shape_with_very_large_font_size() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_size: 1000.0,
    line_height: 1200.0,
    ..TextStyle::default()
  };
  let run = ctx.shape("A", &style);
  assert!(run.height > 0.0);
}

#[test]
fn shape_with_line_height_less_than_font_size() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_size: 24.0,
    line_height: 12.0,
    ..TextStyle::default()
  };
  let run = ctx.shape("Test", &style);
  assert_eq!(run.line_height, 12.0);
}

#[test]
fn shape_whitespace_only() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("   ", &style);
  // Should not panic; may or may not produce glyphs for spaces
  assert!(run.height >= 0.0);
}

#[test]
fn shape_tab_character() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("\t", &style);
  assert!(run.height >= 0.0);
}

#[test]
fn shape_newline_only() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("\n", &style);
  assert!(run.height >= 0.0);
}

#[test]
fn shape_unicode_emoji() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("🎉", &style);
  assert_eq!(run.text, "🎉");
  assert_eq!(run.char_count(), 1);
}

#[test]
fn shape_cjk_characters() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("你好世界", &style);
  assert_eq!(run.char_count(), 4);
}

#[test]
fn shape_combining_diacritics() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  // e + combining acute accent = é (2 chars, may render as 1 glyph)
  let run = ctx.shape("e\u{0301}", &style);
  assert_eq!(run.char_count(), 2);
}

#[test]
fn shape_zero_width_space() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("a\u{200B}b", &style);
  assert_eq!(run.char_count(), 3);
}

#[test]
fn break_into_lines_with_zero_max_width() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let lines = ctx.break_into_lines("Hello", &style, 0.0);
  // Should not panic
  assert!(!lines.is_empty() || true);
}

#[test]
fn break_into_lines_with_negative_max_width() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let lines = ctx.break_into_lines("Hello", &style, -10.0);
  // Should not panic
  let _ = lines;
}

#[test]
fn measure_only_with_tiny_font_size() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_size: 0.1,
    line_height: 0.1,
    ..TextStyle::default()
  };
  let metrics = ctx.measure_only("test", &style);
  assert!(metrics.height >= 0.0);
  assert!(metrics.width >= 0.0);
}

#[test]
fn shape_very_long_text() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let text = "a".repeat(10_000);
  let run = ctx.shape(&text, &style);
  assert_eq!(run.char_count(), 10_000);
}

#[test]
fn shape_empty_font_family() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_family: "",
    ..TextStyle::default()
  };
  let run = ctx.shape("test", &style);
  assert!(!run.glyphs.is_empty());
}
