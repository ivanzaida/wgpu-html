use lui_glyph::{FontRegistry, FontStyleAxis};
use crate::helpers::{dummy_face, dummy_face_weighted, dummy_face_styled};

#[test]
fn find_with_empty_family_string() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("Roboto"));
    assert!(reg.find("", 400, FontStyleAxis::Normal).is_none());
}

#[test]
fn find_with_weight_zero() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face_weighted("Fam", 100));
    // Weight 0 should still find the closest match
    let result = reg.find("Fam", 0, FontStyleAxis::Normal);
    assert_eq!(result, Some(h));
}

#[test]
fn find_with_max_weight() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face_weighted("Fam", 900));
    let result = reg.find("Fam", u16::MAX, FontStyleAxis::Normal);
    assert_eq!(result, Some(h));
}

#[test]
fn find_first_with_duplicate_families() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face("Dup"));
    let result = reg.find_first(&["Dup", "Dup", "Dup"], 400, FontStyleAxis::Normal);
    assert_eq!(result, Some(h));
}

#[test]
fn register_same_family_twice_creates_two_entries() {
    let mut reg = FontRegistry::new();
    let h1 = reg.register(dummy_face("Same"));
    let h2 = reg.register(dummy_face("Same"));
    assert_ne!(h1, h2);
    assert_eq!(reg.len(), 2);
}

#[test]
fn find_with_unicode_family_name() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face("日本語フォント"));
    assert_eq!(reg.find("日本語フォント", 400, FontStyleAxis::Normal), Some(h));
}

#[test]
fn find_with_special_characters_in_family() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face("Font-Name_v2.0"));
    assert_eq!(reg.find("Font-Name_v2.0", 400, FontStyleAxis::Normal), Some(h));
}

#[test]
fn find_boundary_weight_400() {
    let mut reg = FontRegistry::new();
    let h300 = reg.register(dummy_face_weighted("Fam", 300));
    let h500 = reg.register(dummy_face_weighted("Fam", 500));

    // At 400, no directional preference; both are equidistant (100 away)
    let result = reg.find("Fam", 400, FontStyleAxis::Normal);
    assert!(result == Some(h300) || result == Some(h500));
}

#[test]
fn find_boundary_weight_500() {
    let mut reg = FontRegistry::new();
    let h400 = reg.register(dummy_face_weighted("Fam", 400));
    let h600 = reg.register(dummy_face_weighted("Fam", 600));

    // At 500, no directional preference
    let result = reg.find("Fam", 500, FontStyleAxis::Normal);
    assert!(result == Some(h400) || result == Some(h600));
}

#[test]
fn find_prefers_closer_weight_at_700() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face_weighted("Fam", 400));
    let h800 = reg.register(dummy_face_weighted("Fam", 800));

    // At 700 (>500), prefers heavier → 800 is closer in preferred direction
    let result = reg.find("Fam", 700, FontStyleAxis::Normal);
    assert_eq!(result, Some(h800));
}

#[test]
fn find_with_all_three_styles_available() {
    let mut reg = FontRegistry::new();
    let h_normal = reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Normal));
    let h_italic = reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Italic));
    let h_oblique = reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Oblique));

    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Normal), Some(h_normal));
    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Italic), Some(h_italic));
    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Oblique), Some(h_oblique));
}

#[test]
fn find_first_recognizes_all_generic_families() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("AnyFont"));

    for generic in &["sans-serif", "serif", "monospace", "cursive", "fantasy",
                      "system-ui", "ui-sans-serif", "ui-serif", "ui-monospace",
                      "ui-rounded", "math", "emoji", "fangsong",
                      "-apple-system", "blinkmacsystemfont"] {
        let result = reg.find_first(&[generic], 400, FontStyleAxis::Normal);
        assert!(result.is_some(), "generic '{}' should trigger fallback", generic);
    }
}

#[test]
fn find_first_non_generic_unknown_family_returns_none() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("Registered"));

    let result = reg.find_first(&["not-a-generic-keyword"], 400, FontStyleAxis::Normal);
    assert!(result.is_none());
}
