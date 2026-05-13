use lui_glyph::{FontContext, TextStyle};

#[test]
fn measure_only_returns_positive_dimensions_for_text() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let metrics = ctx.measure_only("Hello", &style);

    assert!(metrics.width > 0.0);
    assert!(metrics.height > 0.0);
}

#[test]
fn measure_only_matches_shape_dimensions() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let run = ctx.shape("Hello World", &style);
    let metrics = ctx.measure_only("Hello World", &style);

    assert!((metrics.width - run.width).abs() < 0.01);
    assert!((metrics.height - run.height).abs() < 0.01);
    assert!((metrics.ascent - run.ascent).abs() < 0.01);
}

#[test]
fn measure_empty_text() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let metrics = ctx.measure_only("", &style);

    assert_eq!(metrics.width, 0.0);
}

#[test]
fn measure_longer_text_is_taller_due_to_wrapping() {
    let mut ctx = FontContext::new();
    let style = TextStyle::default();
    let short = ctx.measure_only("A", &style);
    let long = ctx.measure_only("Hello World this is a long sentence", &style);

    assert!(long.height >= short.height);
}
