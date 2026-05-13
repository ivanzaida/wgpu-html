use lui_glyph::FontStyleAxis;
use lui_glyph::FontRegistry;
use crate::helpers::{dummy_face, dummy_face_weighted, dummy_face_styled};

#[test]
fn find_exact_family_match() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face("Roboto"));
    assert_eq!(reg.find("Roboto", 400, FontStyleAxis::Normal), Some(h));
}

#[test]
fn find_returns_none_for_unknown_family() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face("Roboto"));
    assert_eq!(reg.find("Arial", 400, FontStyleAxis::Normal), None);
}

#[test]
fn find_is_case_insensitive() {
    let mut reg = FontRegistry::new();
    let h = reg.register(dummy_face("Roboto"));
    assert_eq!(reg.find("roboto", 400, FontStyleAxis::Normal), Some(h));
    assert_eq!(reg.find("ROBOTO", 400, FontStyleAxis::Normal), Some(h));
    assert_eq!(reg.find("rObOtO", 400, FontStyleAxis::Normal), Some(h));
}

#[test]
fn find_returns_none_for_empty_registry() {
    let reg = FontRegistry::new();
    assert_eq!(reg.find("Anything", 400, FontStyleAxis::Normal), None);
}

#[test]
fn find_prefers_exact_weight() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face_weighted("Fam", 300));
    let h_exact = reg.register(dummy_face_weighted("Fam", 400));
    reg.register(dummy_face_weighted("Fam", 700));

    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Normal), Some(h_exact));
}

#[test]
fn find_prefers_exact_style_match() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Italic));
    let h_normal = reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Normal));

    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Normal), Some(h_normal));
}

#[test]
fn find_italic_falls_back_to_oblique_over_normal() {
    let mut reg = FontRegistry::new();
    let h_oblique = reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Oblique));
    reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Normal));

    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Italic), Some(h_oblique));
}

#[test]
fn find_oblique_falls_back_to_italic_over_normal() {
    let mut reg = FontRegistry::new();
    let h_italic = reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Italic));
    reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Normal));

    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Oblique), Some(h_italic));
}

#[test]
fn find_with_weight_above_500_prefers_heavier() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face_weighted("Fam", 600));
    let h_heavier = reg.register(dummy_face_weighted("Fam", 800));

    assert_eq!(reg.find("Fam", 700, FontStyleAxis::Normal), Some(h_heavier));
}

#[test]
fn find_with_weight_below_400_prefers_lighter() {
    let mut reg = FontRegistry::new();
    let h_lighter = reg.register(dummy_face_weighted("Fam", 200));
    reg.register(dummy_face_weighted("Fam", 500));

    assert_eq!(reg.find("Fam", 300, FontStyleAxis::Normal), Some(h_lighter));
}

#[test]
fn find_with_weight_in_neutral_zone_picks_closest() {
    let mut reg = FontRegistry::new();
    reg.register(dummy_face_weighted("Fam", 200));
    let h_closest = reg.register(dummy_face_weighted("Fam", 420));

    // Target 400 is in the neutral zone (400..=500), no directional bias
    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Normal), Some(h_closest));
}

#[test]
fn find_style_match_trumps_weight_match() {
    let mut reg = FontRegistry::new();
    // Close weight but wrong style
    reg.register(dummy_face_styled("Fam", 400, FontStyleAxis::Italic));
    // Farther weight but correct style
    let h = reg.register(dummy_face_styled("Fam", 700, FontStyleAxis::Normal));

    assert_eq!(reg.find("Fam", 400, FontStyleAxis::Normal), Some(h));
}
