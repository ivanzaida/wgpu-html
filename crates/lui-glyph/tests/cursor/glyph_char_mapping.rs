use lui_glyph::{FontContext, ShapedRun, TextStyle};

#[test]
fn glyph_chars_populated_for_shaped_text() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("Hello", &style);

    assert_eq!(run.glyph_chars.len(), run.glyphs.len());
}

#[test]
fn byte_boundaries_populated_for_shaped_text() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("Hello", &style);

    assert_eq!(run.byte_boundaries.len(), "Hello".chars().count() + 1);
    assert_eq!(run.byte_boundaries, vec![0, 1, 2, 3, 4, 5]);
}

#[test]
fn text_field_matches_input() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("Test input", &style);

    assert_eq!(run.text, "Test input");
}

#[test]
fn char_count_matches_text_chars() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("abc", &style);

    assert_eq!(run.char_count(), 3);
}

#[test]
fn char_count_empty_text() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("", &style);

    assert_eq!(run.char_count(), 0);
}

#[test]
fn glyph_to_char_index_identity_fallback() {
    // A synthetic run with empty glyph_chars falls back to identity
    let run = ShapedRun {
        glyphs: vec![], glyph_chars: vec![],
        text: String::new(), byte_boundaries: vec![],
        width: 0.0, height: 0.0, ascent: 0.0,
        line_height: 0.0, font_size: 0.0, line_count: 0,
    };
    assert_eq!(run.glyph_to_char_index(0), 0);
    assert_eq!(run.glyph_to_char_index(5), 5);
}

#[test]
fn char_to_glyph_index_identity_fallback() {
    let run = ShapedRun {
        glyphs: vec![], glyph_chars: vec![],
        text: String::new(), byte_boundaries: vec![],
        width: 0.0, height: 0.0, ascent: 0.0,
        line_height: 0.0, font_size: 0.0, line_count: 0,
    };
    assert_eq!(run.char_to_glyph_index(0), 0);
    assert_eq!(run.char_to_glyph_index(100), 0);
}

#[test]
fn glyph_to_char_past_end_returns_next_char() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("ab", &style);

    if !run.glyph_chars.is_empty() {
        let past_end = run.glyph_to_char_index(run.glyphs.len() + 10);
        let last_char = *run.glyph_chars.last().unwrap();
        assert_eq!(past_end, last_char + 1);
    }
}

#[test]
fn char_to_glyph_past_end_returns_glyphs_len() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("abc", &style);

    let idx = run.char_to_glyph_index(999);
    assert_eq!(idx, run.glyphs.len());
}

#[test]
fn glyph_chars_are_monotonically_non_decreasing() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("Hello World", &style);

    for window in run.glyph_chars.windows(2) {
        assert!(window[1] >= window[0], "glyph_chars should be non-decreasing");
    }
}

#[test]
fn byte_offset_for_boundary_at_zero() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("test", &style);

    assert_eq!(run.byte_offset_for_boundary(0), 0);
}

#[test]
fn byte_offset_for_boundary_clamps_to_last() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("test", &style);

    let last = *run.byte_boundaries.last().unwrap();
    assert_eq!(run.byte_offset_for_boundary(999), last);
}

#[test]
fn byte_offset_for_boundary_empty_returns_zero() {
    let run = ShapedRun {
        glyphs: vec![], glyph_chars: vec![],
        text: String::new(), byte_boundaries: vec![],
        width: 0.0, height: 0.0, ascent: 0.0,
        line_height: 0.0, font_size: 0.0, line_count: 0,
    };
    assert_eq!(run.byte_offset_for_boundary(0), 0);
}

#[test]
fn roundtrip_char_to_glyph_to_char() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("abcde", &style);

    if run.glyph_chars.is_empty() { return; }

    for char_idx in 0..run.char_count() {
        let glyph_idx = run.char_to_glyph_index(char_idx);
        if glyph_idx < run.glyphs.len() {
            let back = run.glyph_to_char_index(glyph_idx);
            assert!(back <= char_idx, "roundtrip: char {} -> glyph {} -> char {}", char_idx, glyph_idx, back);
        }
    }
}
