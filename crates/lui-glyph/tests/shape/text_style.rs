use lui_glyph::TextStyle;

#[test]
fn default_font_size_is_16() {
    let ts = TextStyle::default();
    assert_eq!(ts.font_size, 16.0);
}

#[test]
fn default_line_height_is_19_2() {
    let ts = TextStyle::default();
    assert_eq!(ts.line_height, 19.2);
}

#[test]
fn default_font_family_is_sans_serif() {
    let ts = TextStyle::default();
    assert_eq!(ts.font_family, "sans-serif");
}

#[test]
fn default_weight_is_400() {
    let ts = TextStyle::default();
    assert_eq!(ts.weight, 400);
}

#[test]
fn custom_values_are_preserved() {
    let ts = TextStyle {
        font_size: 24.0,
        line_height: 32.0,
        font_family: "monospace",
        weight: 700,
    };
    assert_eq!(ts.font_size, 24.0);
    assert_eq!(ts.line_height, 32.0);
    assert_eq!(ts.font_family, "monospace");
    assert_eq!(ts.weight, 700);
}
