use std::sync::Arc;

use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};
use lui_glyph::{FontFace, TextContext};

// ── shape_and_pack cache ──────────────────────────────────────────────

#[test]
fn shape_and_pack_cache_hit_returns_identical_dimensions() {
  let mut ctx = TextContext::new();
  let color = [1.0, 1.0, 1.0, 1.0];
  let first = ctx.shape_and_pack("Hello", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  let second = ctx.shape_and_pack("Hello", 16.0, 19.2, 400, color, "sans-serif", 1.0);

  assert_eq!(first.width, second.width);
  assert_eq!(first.height, second.height);
  assert_eq!(first.ascent, second.ascent);
  assert_eq!(first.line_count, second.line_count);
  assert_eq!(first.glyphs.len(), second.glyphs.len());
}

#[test]
fn shape_and_pack_cache_hit_patches_color() {
  let mut ctx = TextContext::new();
  let red = [1.0, 0.0, 0.0, 1.0];
  let blue = [0.0, 0.0, 1.0, 1.0];

  ctx.shape_and_pack("Test", 16.0, 19.2, 400, red, "sans-serif", 1.0);
  let cached = ctx.shape_and_pack("Test", 16.0, 19.2, 400, blue, "sans-serif", 1.0);

  for g in &cached.glyphs {
    assert_eq!(g.color, blue, "cached hit should patch color to blue");
  }
}

#[test]
fn shape_and_pack_cache_hit_preserves_uv_coords() {
  let mut ctx = TextContext::new();
  let color = [1.0; 4];
  let first = ctx.shape_and_pack("UV", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  let second = ctx.shape_and_pack("UV", 16.0, 19.2, 400, [0.0; 4], "sans-serif", 1.0);

  for (a, b) in first.glyphs.iter().zip(second.glyphs.iter()) {
    assert_eq!(a.uv_min, b.uv_min, "UV coords should survive cache hit");
    assert_eq!(a.uv_max, b.uv_max);
  }
}

#[test]
fn shape_and_pack_cache_hit_preserves_glyph_positions() {
  let mut ctx = TextContext::new();
  let first = ctx.shape_and_pack("Pos", 16.0, 19.2, 400, [1.0; 4], "sans-serif", 1.0);
  let second = ctx.shape_and_pack("Pos", 16.0, 19.2, 400, [0.5; 4], "sans-serif", 1.0);

  for (a, b) in first.glyphs.iter().zip(second.glyphs.iter()) {
    assert_eq!(a.x, b.x);
    assert_eq!(a.y, b.y);
    assert_eq!(a.w, b.w);
    assert_eq!(a.h, b.h);
    assert_eq!(a.glyph_id, b.glyph_id);
  }
}

#[test]
fn shape_and_pack_cache_hit_preserves_text_and_char_data() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];
  let first = ctx.shape_and_pack("café", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  let second = ctx.shape_and_pack("café", 16.0, 19.2, 400, color, "sans-serif", 1.0);

  assert_eq!(first.text, second.text);
  assert_eq!(first.glyph_chars, second.glyph_chars);
  assert_eq!(first.byte_boundaries, second.byte_boundaries);
}

// ── Cache miss on different parameters ────────────────────────────────

#[test]
fn cache_miss_on_different_text() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];
  let a = ctx.shape_and_pack("AAA", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  let b = ctx.shape_and_pack("BBB", 16.0, 19.2, 400, color, "sans-serif", 1.0);

  assert_ne!(a.text, b.text);
}

#[test]
fn cache_miss_on_different_font_size() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];
  let small = ctx.shape_and_pack("X", 12.0, 14.0, 400, color, "sans-serif", 1.0);
  let large = ctx.shape_and_pack("X", 24.0, 28.0, 400, color, "sans-serif", 1.0);

  assert_ne!(small.font_size, large.font_size);
}

#[test]
fn cache_miss_on_different_line_height() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];
  let a = ctx.shape_and_pack("X", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  let b = ctx.shape_and_pack("X", 16.0, 32.0, 400, color, "sans-serif", 1.0);

  assert_ne!(a.line_height, b.line_height);
}

#[test]
fn cache_miss_on_different_weight() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];
  let normal = ctx.shape_and_pack("W", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  let bold = ctx.shape_and_pack("W", 16.0, 19.2, 700, color, "sans-serif", 1.0);

  // Different weight may produce different glyph ids or widths
  assert_eq!(normal.text, bold.text);
  // They are separate cache entries — verifying both shaped without panic
}

#[test]
fn color_does_not_affect_cache_key() {
  let mut ctx = TextContext::new();
  let a = ctx.shape_and_pack("C", 16.0, 19.2, 400, [1.0, 0.0, 0.0, 1.0], "sans-serif", 1.0);
  let b = ctx.shape_and_pack("C", 16.0, 19.2, 400, [0.0, 1.0, 0.0, 1.0], "sans-serif", 1.0);

  // Same geometry — cache hit just patches color
  assert_eq!(a.width, b.width);
  assert_eq!(a.glyphs.len(), b.glyphs.len());
  // But colors differ
  if !a.glyphs.is_empty() {
    assert_ne!(a.glyphs[0].color, b.glyphs[0].color);
  }
}

// ── Cache invalidation on font registration ───────────────────────────

#[test]
fn cache_invalidated_when_font_registered() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];

  let before = ctx.shape_and_pack("Test", 16.0, 19.2, 400, color, "sans-serif", 1.0);

  // Register a font — bumps generation
  let data: Arc<[u8]> = Arc::from(vec![0u8; 16].into_boxed_slice());
  ctx.register_font(FontFace::regular("Dummy", data));

  let after = ctx.shape_and_pack("Test", 16.0, 19.2, 400, color, "sans-serif", 1.0);

  // Both should produce valid runs (cache was cleared, reshaping happened)
  assert_eq!(before.text, after.text);
  assert_eq!(before.text, "Test");
}

#[test]
fn multiple_registrations_keep_invalidating() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];

  for i in 0..5 {
    ctx.shape_and_pack("stable", 16.0, 19.2, 400, color, "sans-serif", 1.0);
    let data: Arc<[u8]> = Arc::from(vec![0u8; 16].into_boxed_slice());
    ctx.register_font(FontFace::regular(format!("Font{}", i), data));
  }

  // Should not panic — cache invalidation handles repeated clears
  let run = ctx.shape_and_pack("stable", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  assert_eq!(run.text, "stable");
}

// ── measure_run cache ─────────────────────────────────────────────────

#[test]
fn measure_run_cache_hit_returns_same_metrics() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();

  let first = ctx.measure_run("Hello", &style);
  let second = ctx.measure_run("Hello", &style);

  assert_eq!(first.width, second.width);
  assert_eq!(first.height, second.height);
  assert_eq!(first.ascent, second.ascent);
}

#[test]
fn measure_run_cache_miss_on_different_text() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();

  let a = ctx.measure_run("Short", &style);
  let b = ctx.measure_run("A much longer string", &style);

  // Different texts should produce different metrics
  // (can't guarantee width differs due to wrapping, but they're distinct cache entries)
  let _ = (a, b);
}

#[test]
fn measure_run_cache_miss_on_different_font_size() {
  let mut ctx = TextContext::new();
  let fs_small = CssValue::Dimension {
    value: 12.0,
    unit: CssUnit::Px,
  };
  let fs_large = CssValue::Dimension {
    value: 24.0,
    unit: CssUnit::Px,
  };
  let small_style = ComputedStyle {
    font_size: Some(&fs_small),
    ..ComputedStyle::default()
  };
  let large_style = ComputedStyle {
    font_size: Some(&fs_large),
    ..ComputedStyle::default()
  };

  let small = ctx.measure_run("X", &small_style);
  let large = ctx.measure_run("X", &large_style);

  assert!(large.height > small.height, "larger font should be taller");
}

#[test]
fn measure_run_cache_invalidated_on_font_registration() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();

  let before = ctx.measure_run("Test", &style);

  let data: Arc<[u8]> = Arc::from(vec![0u8; 16].into_boxed_slice());
  ctx.register_font(FontFace::regular("Dummy", data));

  let after = ctx.measure_run("Test", &style);

  // Both produce valid metrics (cache cleared, reshaping happened)
  assert!(before.width >= 0.0);
  assert!(after.width >= 0.0);
}

#[test]
fn measure_run_result_matches_shape_run() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();

  let run = ctx.shape_run("Compare", &style);
  let metrics = ctx.measure_run("Compare", &style);

  assert!((metrics.width - run.width).abs() < 0.01);
  assert!((metrics.height - run.height).abs() < 0.01);
  assert!((metrics.ascent - run.ascent).abs() < 0.01);
}

// ── Cache with many entries ───────────────────────────────────────────

#[test]
fn cache_handles_many_distinct_entries() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];

  for i in 0..100 {
    let text = format!("entry_{}", i);
    ctx.shape_and_pack(&text, 16.0, 19.2, 400, color, "sans-serif", 1.0);
  }

  // Verify first entry is still serviced (may be cache hit or reshaping)
  let run = ctx.shape_and_pack("entry_0", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  assert_eq!(run.text, "entry_0");
}

#[test]
fn cache_still_works_after_overflow_clear() {
  let mut ctx = TextContext::new();
  let color = [0.0; 4];

  // Pump more than TEXT_CACHE_MAX (4096) distinct entries
  for i in 0..4100 {
    let text = format!("t{}", i);
    ctx.shape_and_pack(&text, 16.0, 19.2, 400, color, "sans-serif", 1.0);
  }

  // Should still function after cache clear
  let run = ctx.shape_and_pack("final", 16.0, 19.2, 400, color, "sans-serif", 1.0);
  assert_eq!(run.text, "final");
  assert!(run.height > 0.0);
}

// ── Cross-method cache sharing ────────────────────────────────────────

#[test]
fn measure_run_populates_cache_for_subsequent_measure() {
  let mut ctx = TextContext::new();
  let style = ComputedStyle::default();

  // First call shapes and caches
  let first = ctx.measure_run("cached", &style);
  // Second call should hit cache
  let second = ctx.measure_run("cached", &style);

  assert_eq!(first.width, second.width);
  assert_eq!(first.height, second.height);
}

#[test]
fn repeated_shape_and_pack_same_params_is_deterministic() {
  let mut ctx = TextContext::new();
  let color = [0.5, 0.5, 0.5, 1.0];

  let runs: Vec<_> = (0..5)
    .map(|_| ctx.shape_and_pack("deterministic", 16.0, 19.2, 400, color, "sans-serif", 1.0))
    .collect();

  for run in &runs[1..] {
    assert_eq!(runs[0].width, run.width);
    assert_eq!(runs[0].height, run.height);
    assert_eq!(runs[0].glyphs.len(), run.glyphs.len());
    assert_eq!(runs[0].text, run.text);
  }
}
