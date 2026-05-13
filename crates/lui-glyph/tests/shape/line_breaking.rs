use lui_glyph::{FontContext, TextStyle};

#[test]
fn break_into_lines_single_line_if_fits() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("Hi", &style, 1000.0);

    assert_eq!(lines.len(), 1);
    assert!(!lines[0].glyphs.is_empty());
}

#[test]
fn break_into_lines_wraps_long_text() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let text = "The quick brown fox jumps over the lazy dog and keeps running far away";
    let lines = ctx.break_into_lines(text, &style, 100.0);

    assert!(lines.len() > 1, "expected multiple lines, got {}", lines.len());
}

#[test]
fn break_into_lines_each_line_fits_max_width() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let max_width = 120.0;
    let text = "This is a longer sentence that should wrap into multiple lines";
    let lines = ctx.break_into_lines(text, &style, max_width);

    for (i, line) in lines.iter().enumerate() {
        assert!(
            line.width <= max_width + 1.0,
            "line {} width {} exceeds max_width {}",
            i, line.width, max_width,
        );
    }
}

#[test]
fn break_into_lines_empty_string() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("", &style, 100.0);

    assert!(lines.is_empty() || lines.iter().all(|l| l.glyphs.is_empty()));
}

#[test]
fn break_into_lines_preserves_line_height() {
    let mut ctx = FontContext::new();
    let style = TextStyle {
        font_size: 16.0,
        line_height: 24.0,
        ..TextStyle::default()
    };
    let lines = ctx.break_into_lines("Some text here", &style, 1000.0);

    for line in &lines {
        assert_eq!(line.height, 24.0);
    }
}

#[test]
fn break_into_lines_narrower_width_produces_more_lines() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let text = "The quick brown fox jumps over the lazy dog";

    let wide = ctx.break_into_lines(text, &style, 500.0);
    let narrow = ctx.break_into_lines(text, &style, 80.0);

    assert!(narrow.len() >= wide.len());
}

// ── Explicit newlines ─────────────────────────────────────────────────

#[test]
fn explicit_newline_produces_multiple_lines() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("aaa\nbbb", &style, 1000.0);

    assert_eq!(lines.len(), 2, "\\n should split into two lines");
}

#[test]
fn multiple_newlines_produce_more_lines_than_single() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let one_line = ctx.break_into_lines("abcd", &style, 1000.0);
    let with_newlines = ctx.break_into_lines("a\nb\nc\nd", &style, 1000.0);

    assert!(
        with_newlines.len() > one_line.len(),
        "newlines should produce more lines ({}) than no newlines ({})",
        with_newlines.len(), one_line.len(),
    );
}

#[test]
fn trailing_newline_adds_empty_line() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let without = ctx.break_into_lines("hello", &style, 1000.0);
    let with = ctx.break_into_lines("hello\n", &style, 1000.0);

    assert!(with.len() >= without.len());
}

// ── Word-boundary wrapping ────────────────────────────────────────────

#[test]
fn wraps_between_words_not_mid_word() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("aaa bbb ccc", &style, 60.0);

    // Each line's glyphs should start near x=0, confirming wrap happened
    // at a boundary rather than splitting a glyph arbitrarily
    for (i, line) in lines.iter().enumerate() {
        if line.glyphs.is_empty() { continue; }
        let first_x = line.glyphs[0].x;
        assert!(
            first_x < 5.0,
            "line {} first glyph x={} — expected near 0 for a clean word wrap",
            i, first_x,
        );
    }
}

#[test]
fn total_glyphs_preserved_across_lines() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let text = "one two three four five";

    let single = ctx.shape(text, &style);
    let lines = ctx.break_into_lines(text, &style, 80.0);
    let total_line_glyphs: usize = lines.iter().map(|l| l.glyphs.len()).sum();

    // Line-broken output should have at least as many glyphs as the
    // single-run shape (spaces may be trimmed at line edges)
    assert!(
        total_line_glyphs >= single.glyphs.len() - lines.len(),
        "total glyphs across lines ({}) should be close to single-run ({})",
        total_line_glyphs, single.glyphs.len(),
    );
}

// ── Very narrow / very wide ───────────────────────────────────────────

#[test]
fn very_narrow_width_still_produces_output() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("Hello World", &style, 1.0);

    assert!(!lines.is_empty(), "even at 1px width, should produce lines");
}

#[test]
fn very_wide_width_keeps_text_on_one_line() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("Short text", &style, 10000.0);

    assert_eq!(lines.len(), 1);
}

// ── Whitespace edge cases ─────────────────────────────────────────────

#[test]
fn only_spaces() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("     ", &style, 100.0);

    // Should not panic; may produce 0 or 1 lines depending on shaper
    assert!(lines.len() <= 1);
}

#[test]
fn leading_and_trailing_spaces_do_not_panic() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("  hello  world  ", &style, 80.0);

    assert!(!lines.is_empty());
}

// ── Font size affects wrapping ────────────────────────────────────────

#[test]
fn larger_font_wraps_sooner() {
    let mut ctx = FontContext::new();
    let text = "The quick brown fox";
    let max_width = 120.0;

    let small = TextStyle { font_size: 10.0, line_height: 12.0, ..TextStyle::default() };
    let large = TextStyle { font_size: 24.0, line_height: 28.0, ..TextStyle::default() };

    let small_lines = ctx.break_into_lines(text, &small, max_width);
    let large_lines = ctx.break_into_lines(text, &large, max_width);

    assert!(
        large_lines.len() >= small_lines.len(),
        "larger font ({} lines) should wrap at least as much as smaller ({} lines)",
        large_lines.len(), small_lines.len(),
    );
}

// ── Line dimensions ───────────────────────────────────────────────────

#[test]
fn each_line_has_positive_width_when_non_empty() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let lines = ctx.break_into_lines("alpha bravo charlie delta echo", &style, 90.0);

    for (i, line) in lines.iter().enumerate() {
        if !line.glyphs.is_empty() {
            assert!(line.width > 0.0, "line {} has glyphs but width=0", i);
        }
    }
}

#[test]
fn line_widths_do_not_exceed_max_with_various_widths() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let text = "Lorem ipsum dolor sit amet consectetur adipiscing elit";

    for max_w in [50.0, 100.0, 200.0, 300.0] {
        let lines = ctx.break_into_lines(text, &style, max_w);
        for (i, line) in lines.iter().enumerate() {
            assert!(
                line.width <= max_w + 1.0,
                "max_width={}: line {} width {} exceeds limit",
                max_w, i, line.width,
            );
        }
    }
}
