use lui_glyph::{FontRegistry, FontStyleAxis};

use crate::helpers::dummy_face;

#[test]
fn find_first_returns_first_matching_family() {
  let mut reg = FontRegistry::new();
  reg.register(dummy_face("Fallback"));
  let h = reg.register(dummy_face("Primary"));

  let result = reg.find_first(&["Primary", "Fallback"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_skips_unregistered_families() {
  let mut reg = FontRegistry::new();
  let h = reg.register(dummy_face("Exists"));

  let result = reg.find_first(&["Missing", "Exists"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_returns_none_when_no_match_and_no_generic() {
  let mut reg = FontRegistry::new();
  reg.register(dummy_face("Registered"));

  let result = reg.find_first(&["Unknown1", "Unknown2"], 400, FontStyleAxis::Normal);
  assert_eq!(result, None);
}

#[test]
fn find_first_falls_back_to_any_face_for_sans_serif() {
  let mut reg = FontRegistry::new();
  let h = reg.register(dummy_face("OnlyFont"));

  let result = reg.find_first(&["Missing", "sans-serif"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_falls_back_for_serif_generic() {
  let mut reg = FontRegistry::new();
  let h = reg.register(dummy_face("SomeFont"));

  let result = reg.find_first(&["serif"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_falls_back_for_monospace_generic() {
  let mut reg = FontRegistry::new();
  let h = reg.register(dummy_face("MyMono"));

  let result = reg.find_first(&["monospace"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_falls_back_for_system_ui_generic() {
  let mut reg = FontRegistry::new();
  let h = reg.register(dummy_face("SysFont"));

  let result = reg.find_first(&["system-ui"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_generic_keyword_is_case_insensitive() {
  let mut reg = FontRegistry::new();
  let h = reg.register(dummy_face("AnyFont"));

  let result = reg.find_first(&["SANS-SERIF"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}

#[test]
fn find_first_returns_none_for_empty_list() {
  let mut reg = FontRegistry::new();
  reg.register(dummy_face("Exists"));

  let result = reg.find_first(&[], 400, FontStyleAxis::Normal);
  assert_eq!(result, None);
}

#[test]
fn find_first_returns_none_for_empty_registry_even_with_generic() {
  let reg = FontRegistry::new();
  let result = reg.find_first(&["sans-serif"], 400, FontStyleAxis::Normal);
  assert_eq!(result, None);
}

#[test]
fn find_first_prefers_exact_match_over_generic_fallback() {
  let mut reg = FontRegistry::new();
  reg.register(dummy_face("Other"));
  let h = reg.register(dummy_face("Target"));

  let result = reg.find_first(&["Target", "sans-serif"], 400, FontStyleAxis::Normal);
  assert_eq!(result, Some(h));
}
