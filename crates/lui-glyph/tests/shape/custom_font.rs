use lui_glyph::{FontContext, FontHandle, FontStyleAxis};

#[test]
fn shape_with_handle_returns_none_for_unregistered_handle() {
  let mut ctx = FontContext::new();
  let result = ctx.shape_with_handle("Test", FontHandle(999), 16.0, 19.2, 400, FontStyleAxis::Normal);
  assert!(result.is_none());
}

#[test]
fn measure_with_handle_returns_none_for_unregistered_handle() {
  let mut ctx = FontContext::new();
  let result = ctx.measure_with_handle("Test", FontHandle(42), 16.0, 19.2, 400, FontStyleAxis::Normal);
  assert!(result.is_none());
}

#[test]
fn shape_with_families_falls_back_to_system_font() {
  let mut ctx = FontContext::new();
  let run = ctx.shape_with_families(
    "Hello",
    &["NonExistentFont", "sans-serif"],
    16.0,
    19.2,
    400,
    FontStyleAxis::Normal,
  );
  assert!(!run.glyphs.is_empty());
}

#[test]
fn shape_with_families_empty_list_uses_sans_serif() {
  let mut ctx = FontContext::new();
  let run = ctx.shape_with_families("Hello", &[], 16.0, 19.2, 400, FontStyleAxis::Normal);
  assert!(!run.glyphs.is_empty());
}

#[test]
fn measure_with_families_returns_positive_dimensions() {
  let mut ctx = FontContext::new();
  let metrics = ctx.measure_with_families("Test", &["sans-serif"], 16.0, 19.2, 400, FontStyleAxis::Normal);
  assert!(metrics.width > 0.0);
  assert!(metrics.height > 0.0);
}

#[test]
fn shape_with_families_preserves_text_field() {
  let mut ctx = FontContext::new();
  let run = ctx.shape_with_families("abc", &["serif"], 16.0, 19.2, 400, FontStyleAxis::Normal);
  assert_eq!(run.text, "abc");
}

#[test]
fn pick_font_returns_none_with_no_registered_fonts() {
  let ctx = FontContext::new();
  let result = ctx.pick_font(&["CustomFont"], 400, FontStyleAxis::Normal);
  assert!(result.is_none());
}

#[test]
fn fontdb_id_returns_none_for_unregistered_handle() {
  let ctx = FontContext::new();
  assert!(ctx.fontdb_id(FontHandle(0)).is_none());
}

#[test]
fn resolve_family_returns_none_for_unknown_family() {
  let mut ctx = FontContext::new();
  let result = ctx.resolve_family(&["TotallyFakeFont"], 400, FontStyleAxis::Normal);
  assert!(result.is_none());
}

#[test]
fn registry_is_empty_on_new_context() {
  let ctx = FontContext::new();
  assert!(ctx.registry().is_empty());
}
