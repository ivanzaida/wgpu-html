use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};
use lui_glyph::TextContext;

#[test]
fn shape_run_with_default_style_produces_glyphs() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let run = ctx.shape_run("Hello World", &style);

  assert!(!run.glyphs.is_empty());
  assert!(run.width > 0.0);
  assert!(run.height > 0.0);
}

#[test]
fn shape_run_with_empty_text() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let run = ctx.shape_run("", &style);

  assert!(run.glyphs.is_empty());
}

#[test]
fn shape_run_preserves_text_field() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let run = ctx.shape_run("café", &style);

  assert_eq!(run.text, "café");
}

#[test]
fn shape_run_populates_glyph_chars() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let run = ctx.shape_run("abc", &style);

  assert_eq!(run.glyph_chars.len(), run.glyphs.len());
}

#[test]
fn shape_run_with_custom_font_size() {
  let mut ctx = TextContext::new();
  let fs = CssValue::Dimension {
    value: 32.0,
    unit: CssUnit::Px,
  };
  let style = ComputedStyle {
    font_size: Some(&fs),
    ..ComputedStyle::default()
  };
  let run = ctx.shape_run("Test", &style);

  assert_eq!(run.font_size, 32.0);
}

#[test]
fn shape_run_with_custom_weight() {
  let mut ctx = TextContext::new();
  let fw = CssValue::Number(700.0);
  let style = ComputedStyle {
    font_weight: Some(&fw),
    ..ComputedStyle::default()
  };
  let run = ctx.shape_run("Bold", &style);

  assert!(!run.glyphs.is_empty());
}

#[test]
fn shape_run_with_custom_family() {
  let mut ctx = TextContext::new();
  let ff = CssValue::String("monospace".into());
  let style = ComputedStyle {
    font_family: Some(&ff),
    ..ComputedStyle::default()
  };
  let run = ctx.shape_run("code", &style);

  assert!(!run.glyphs.is_empty());
}

#[test]
fn measure_run_returns_positive_dimensions() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let metrics = ctx.measure_run("Hello", &style);

  assert!(metrics.width > 0.0);
  assert!(metrics.height > 0.0);
}

#[test]
fn measure_run_matches_shape_run_dimensions() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let run = ctx.shape_run("Test", &style);
  let metrics = ctx.measure_run("Test", &style);

  assert!((metrics.width - run.width).abs() < 0.01);
  assert!((metrics.height - run.height).abs() < 0.01);
}

#[test]
fn measure_run_empty_text() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();
  let metrics = ctx.measure_run("", &style);

  assert_eq!(metrics.width, 0.0);
}

#[test]
fn pick_font_delegation_returns_none_without_registered_fonts() {
  let ctx = TextContext::new();
  assert!(
    ctx
      .pick_font(&["CustomFont"], 400, lui_glyph::FontStyleAxis::Normal)
      .is_none()
  );
}

#[test]
fn normal_line_height_multiplier_returns_none_for_invalid_handle() {
  let ctx = TextContext::new();
  assert!(ctx.normal_line_height_multiplier(lui_glyph::FontHandle(999)).is_none());
}
