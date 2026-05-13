use lui_glyph::{FontStyleAxis, ParagraphSpan, TextContext};

fn white() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }
fn red() -> [f32; 4] { [1.0, 0.0, 0.0, 1.0] }

fn span<'a>(text: &'a str, leaf_id: u32) -> ParagraphSpan<'a> {
    ParagraphSpan {
        text,
        family: "sans-serif",
        weight: 400,
        style: FontStyleAxis::Normal,
        size_px: 16.0,
        line_height_px: 19.2,
        color: white(),
        leaf_id,
    }
}

#[test]
fn empty_spans_returns_none() {
    let mut ctx = TextContext::new();
    assert!(ctx.shape_paragraph(&[], None).is_none());
}

#[test]
fn single_span_produces_layout() {
    let mut ctx = TextContext::new();
    let spans = [span("Hello World", 0)];
    let layout = ctx.shape_paragraph(&spans, None);

    assert!(layout.is_some());
    let layout = layout.unwrap();
    assert!(!layout.lines.is_empty());
    assert!(layout.width > 0.0);
    assert!(layout.height > 0.0);
}

#[test]
fn multi_span_produces_glyphs() {
    let mut ctx = TextContext::new();
    let spans = [span("Hello ", 0), span("World", 1)];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    assert!(!layout.glyphs.is_empty());
}

#[test]
fn leaf_segments_track_span_positions() {
    let mut ctx = TextContext::new();
    let spans = [span("aaa ", 10), span("bbb", 20)];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    // Both leaf_ids should appear in leaf_segments
    assert!(layout.leaf_segments.contains_key(&10), "leaf_id 10 missing");
    assert!(layout.leaf_segments.contains_key(&20), "leaf_id 20 missing");
}

#[test]
fn leaf_segment_x_ranges_are_positive() {
    let mut ctx = TextContext::new();
    let spans = [span("Hello ", 1), span("World", 2)];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    for (_, segments) in &layout.leaf_segments {
        for seg in segments {
            assert!(seg.x_end >= seg.x_start,
                "x_end {} should be >= x_start {}", seg.x_end, seg.x_start);
        }
    }
}

#[test]
fn paragraph_line_glyph_ranges_are_valid() {
    let mut ctx = TextContext::new();
    let spans = [span("Some text here", 0)];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    for line in &layout.lines {
        assert!(line.glyph_range.0 <= line.glyph_range.1);
        assert!(line.glyph_range.1 <= layout.glyphs.len());
    }
}

#[test]
fn paragraph_with_max_width_wraps() {
    let mut ctx = TextContext::new();
    let text = "The quick brown fox jumps over the lazy dog and keeps running";
    let spans = [span(text, 0)];

    let no_wrap = ctx.shape_paragraph(&spans, None).unwrap();
    let wrapped = ctx.shape_paragraph(&spans, Some(100.0)).unwrap();

    assert!(wrapped.lines.len() >= no_wrap.lines.len(),
        "wrapped ({} lines) should have >= lines than unwrapped ({})",
        wrapped.lines.len(), no_wrap.lines.len());
}

#[test]
fn paragraph_lines_have_positive_dimensions() {
    let mut ctx = TextContext::new();
    let spans = [span("Line one", 0)];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    for line in &layout.lines {
        assert!(line.height > 0.0);
        assert!(line.line_width >= 0.0);
    }
}

#[test]
fn paragraph_first_line_ascent_is_positive() {
    let mut ctx = TextContext::new();
    let spans = [span("Test", 0)];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    assert!(layout.first_line_ascent >= 0.0);
}

#[test]
fn paragraph_with_different_colors_per_span() {
    let mut ctx = TextContext::new();
    let spans = [
        ParagraphSpan { color: red(), ..span("Red ", 0) },
        ParagraphSpan { color: white(), ..span("White", 1) },
    ];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    assert!(!layout.glyphs.is_empty());
}

#[test]
fn paragraph_with_different_sizes_per_span() {
    let mut ctx = TextContext::new();
    let spans = [
        ParagraphSpan { size_px: 12.0, line_height_px: 14.0, ..span("Small ", 0) },
        ParagraphSpan { size_px: 24.0, line_height_px: 28.0, ..span("Large", 1) },
    ];
    let layout = ctx.shape_paragraph(&spans, None).unwrap();

    assert!(!layout.glyphs.is_empty());
    assert!(layout.height > 0.0);
}

#[test]
fn paragraph_wrapping_creates_multiple_leaf_segments() {
    let mut ctx = TextContext::new();
    let long_text = "This is a very long span that should definitely wrap across multiple lines when constrained";
    let spans = [span(long_text, 42)];
    let layout = ctx.shape_paragraph(&spans, Some(80.0)).unwrap();

    if layout.lines.len() > 1 {
        let segs = layout.leaf_segments.get(&42);
        assert!(segs.is_some());
        assert!(segs.unwrap().len() > 1,
            "a wrapping span should produce multiple leaf segments");
    }
}
