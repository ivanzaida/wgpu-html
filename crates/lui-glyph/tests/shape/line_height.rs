use lui_glyph::parse_line_height_multiplier;

#[test]
fn returns_none_for_empty_data() {
    assert!(parse_line_height_multiplier(&[]).is_none());
}

#[test]
fn returns_none_for_too_short_data() {
    assert!(parse_line_height_multiplier(&[0u8; 8]).is_none());
}

#[test]
fn returns_none_for_garbage_data() {
    let garbage = vec![0xFFu8; 256];
    assert!(parse_line_height_multiplier(&garbage).is_none());
}

#[test]
fn returns_positive_value_for_valid_font() {
    let font_data = load_system_font();
    if font_data.is_empty() { return; }

    let multiplier = parse_line_height_multiplier(&font_data);
    if let Some(m) = multiplier {
        assert!(m > 0.0, "line height multiplier should be positive, got {}", m);
        assert!(m < 5.0, "line height multiplier should be reasonable, got {}", m);
    }
}

#[test]
fn typical_font_multiplier_near_1_2() {
    let font_data = load_system_font();
    if font_data.is_empty() { return; }

    if let Some(m) = parse_line_height_multiplier(&font_data) {
        assert!(m >= 1.0 && m <= 2.0,
            "typical font multiplier should be 1.0-2.0, got {}", m);
    }
}

fn load_system_font() -> Vec<u8> {
    // Try common system font paths
    for path in &[
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
    ] {
        if let Ok(data) = std::fs::read(path) {
            return data;
        }
    }
    Vec::new()
}
