use lui_models::common::css_enums::{CssColor, CssLength};

use super::*;

#[test]
fn inherited_block_is_marked() {
  assert!(is_inherited("color"));
  assert!(is_inherited("font-family"));
  assert!(is_inherited("line-height"));
  assert!(is_inherited("visibility"));
  assert!(is_inherited("cursor"));
  // Non-inherited reference checks
  assert!(!is_inherited("background-color"));
  assert!(!is_inherited("margin"));
  assert!(!is_inherited("display"));
  assert!(!is_inherited("z-index"));
}

#[test]
fn clear_value_for_unsets_named_field() {
  let mut s = Style::default();
  s.color = Some(CssColor::Named("red".into()));
  s.width = Some(CssLength::Px(10.0));
  clear_value_for("color", &mut s);
  assert!(s.color.is_none());
  assert!(s.width.is_some());
}

#[test]
fn clear_value_for_background_clears_supported_longhands() {
  let mut s = Style::default();
  s.background = Some("#123456".into());
  s.background_color = Some(CssColor::Hex("#123456".into()));
  s.background_position = Some("center".into());
  clear_value_for("background", &mut s);
  assert!(s.background.is_none());
  assert!(s.background_color.is_none());
  assert!(s.background_position.is_none());
}

#[test]
fn apply_inherit_uses_parent_value() {
  let mut child = Style::default();
  let mut parent = Style::default();
  parent.color = Some(CssColor::Named("white".into()));
  apply_keyword(&mut child, Some(&parent), "color", CssWideKeyword::Inherit);
  assert!(matches!(
      child.color.as_ref().unwrap(),
      CssColor::Named(ref s) if &**s == "white"
  ));
}

#[test]
fn apply_initial_clears_field() {
  let mut child = Style::default();
  child.color = Some(CssColor::Named("red".into()));
  let parent = Style::default();
  apply_keyword(&mut child, Some(&parent), "color", CssWideKeyword::Initial);
  assert!(child.color.is_none());
}

#[test]
fn apply_unset_inherits_for_inherited_props() {
  let mut child = Style::default();
  child.color = Some(CssColor::Named("red".into()));
  let mut parent = Style::default();
  parent.color = Some(CssColor::Named("blue".into()));
  apply_keyword(&mut child, Some(&parent), "color", CssWideKeyword::Unset);
  assert!(matches!(
      child.color.as_ref().unwrap(),
      CssColor::Named(ref s) if &**s == "blue"
  ));
}

#[test]
fn apply_unset_clears_for_non_inherited_props() {
  let mut child = Style::default();
  child.background_color = Some(CssColor::Named("red".into()));
  let mut parent = Style::default();
  parent.background_color = Some(CssColor::Named("blue".into()));
  apply_keyword(&mut child, Some(&parent), "background-color", CssWideKeyword::Unset);
  assert!(child.background_color.is_none());
}

#[test]
fn apply_background_inherit_copies_supported_longhands() {
  let mut child = Style::default();
  child.background_color = Some(CssColor::Named("red".into()));
  let mut parent = Style::default();
  parent.background = Some("#1b1d22".into());
  parent.background_color = Some(CssColor::Hex("#1b1d22".into()));
  parent.background_position = Some("center".into());
  apply_keyword(&mut child, Some(&parent), "background", CssWideKeyword::Inherit);
  assert!(matches!(child.background_color, Some(CssColor::Hex(ref s)) if &**s == "#1b1d22"));
  assert_eq!(child.background.as_deref(), Some("#1b1d22"));
  assert_eq!(child.background_position.as_deref(), Some("center"));
}

#[test]
fn merge_values_clears_keywords_for_touched_fields() {
  let mut dst = Style::default();
  let mut kw: HashMap<lui_models::ArcStr, CssWideKeyword> = HashMap::new();
  kw.insert("color".into(), CssWideKeyword::Inherit);
  kw.insert("width".into(), CssWideKeyword::Initial);
  let mut src = Style::default();
  src.color = Some(CssColor::Named("red".into()));
  merge_values_clearing_keywords(&mut dst, &mut kw, &src);
  assert!(dst.color.is_some());
  assert!(!kw.contains_key("color"));
  assert!(kw.contains_key("width"));
}

#[test]
fn background_shorthand_merge_clears_related_keywords_and_values() {
  let mut dst = Style::default();
  dst.background_image = Some(lui_models::common::css_enums::CssImage::Url("a.png".into()));
  let mut kw: HashMap<lui_models::ArcStr, CssWideKeyword> = HashMap::new();
  kw.insert("background-color".into(), CssWideKeyword::Inherit);
  kw.insert("background-repeat".into(), CssWideKeyword::Initial);

  let mut src = Style::default();
  src.background = Some("#1b1d22".into());
  src.background_color = Some(CssColor::Hex("#1b1d22".into()));

  merge_values_clearing_keywords(&mut dst, &mut kw, &src);
  assert!(matches!(dst.background_color, Some(CssColor::Hex(ref s)) if &**s == "#1b1d22"));
  assert!(dst.background_image.is_none());
  assert!(!kw.contains_key("background-color"));
  assert!(!kw.contains_key("background-repeat"));
}

#[test]
fn root_inherit_with_no_parent_clears() {
  let mut child = Style::default();
  child.color = Some(CssColor::Named("red".into()));
  apply_keyword(&mut child, None, "color", CssWideKeyword::Inherit);
  assert!(child.color.is_none());
}
