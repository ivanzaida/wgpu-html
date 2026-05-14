use lui_glyph::{FontContext, TextStyle};

#[test]
fn shape_returns_glyphs_for_ascii_text() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("Hello", &style);

  assert!(!run.glyphs.is_empty());
  assert!(run.width > 0.0);
  assert!(run.height > 0.0);
  assert!(run.line_count >= 1);
}

#[test]
fn shape_empty_string_returns_zero_glyphs() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("", &style);

  assert!(run.glyphs.is_empty());
}

#[test]
fn shape_preserves_font_size_and_line_height() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_size: 24.0,
    line_height: 32.0,
    ..TextStyle::default()
  };
  let run = ctx.shape("Test", &style);

  assert_eq!(run.font_size, 24.0);
  assert_eq!(run.line_height, 32.0);
}

#[test]
fn shape_longer_text_produces_more_glyphs() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let short = ctx.shape("Hi", &style);
  let long = ctx.shape("Hello World", &style);

  assert!(long.glyphs.len() > short.glyphs.len());
}

#[test]
fn shape_larger_font_produces_taller_run() {
  let mut ctx = FontContext::new();

  let small = TextStyle {
    font_size: 12.0,
    line_height: 14.0,
    ..TextStyle::default()
  };
  let large = TextStyle {
    font_size: 24.0,
    line_height: 28.0,
    ..TextStyle::default()
  };

  let small_run = ctx.shape("Test", &small);
  let large_run = ctx.shape("Test", &large);

  assert!(large_run.height > small_run.height);
}

#[test]
fn shape_height_equals_line_count_times_line_height() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_size: 16.0,
    line_height: 20.0,
    ..TextStyle::default()
  };
  let run = ctx.shape("Single line", &style);

  assert_eq!(run.height, run.line_count as f32 * run.line_height);
}

#[test]
fn shape_glyph_positions_are_non_negative() {
  let mut ctx = FontContext::new();
  let style = TextStyle::default();
  let run = ctx.shape("Hello World", &style);

  for g in &run.glyphs {
    assert!(g.x >= 0.0, "glyph x={} should be non-negative", g.x);
    assert!(g.w > 0.0, "glyph w={} should be positive", g.w);
  }
}

#[test]
fn shape_with_serif_family() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_family: "serif",
    ..TextStyle::default()
  };
  let run = ctx.shape("Test", &style);
  assert!(!run.glyphs.is_empty());
}

#[test]
fn shape_with_monospace_family() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    font_family: "monospace",
    ..TextStyle::default()
  };
  let run = ctx.shape("Test", &style);
  assert!(!run.glyphs.is_empty());
}

#[test]
fn shape_with_bold_weight() {
  let mut ctx = FontContext::new();
  let style = TextStyle {
    weight: 700,
    ..TextStyle::default()
  };
  let run = ctx.shape("Bold text", &style);
  assert!(!run.glyphs.is_empty());
}
