use std::sync::Arc;

use wgpu_html_tree::{FontFace, FontHandle, FontRegistry, FontStyleAxis};

fn data(b: &[u8]) -> Arc<[u8]> {
  Arc::from(b.to_vec().into_boxed_slice())
}

fn face(family: &str, weight: u16, style: FontStyleAxis, marker: u8) -> FontFace {
  FontFace {
    family: family.to_string(),
    weight,
    style,
    data: data(&[marker]),
  }
}

#[test]
fn register_returns_sequential_handles() {
  let mut r = FontRegistry::new();
  let a = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let b = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
  assert_eq!(a, FontHandle(0));
  assert_eq!(b, FontHandle(1));
  assert_eq!(r.len(), 2);
}

#[test]
fn find_resolves_exact_match() {
  let mut r = FontRegistry::new();
  let regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
  assert_eq!(r.find("Inter", 400, FontStyleAxis::Normal), Some(regular));
  assert_eq!(r.find("Inter", 700, FontStyleAxis::Normal), Some(bold));
  assert_eq!(r.get(regular).unwrap().data[0], 1);
  assert_eq!(r.get(bold).unwrap().data[0], 2);
}

#[test]
fn find_is_case_insensitive_on_family() {
  let mut r = FontRegistry::new();
  r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  assert!(r.find("INTER", 400, FontStyleAxis::Normal).is_some());
  assert!(r.find("inter", 400, FontStyleAxis::Normal).is_some());
  assert!(r.find("Roboto", 400, FontStyleAxis::Normal).is_none());
}

#[test]
fn find_picks_closer_weight() {
  let mut r = FontRegistry::new();
  let regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
  assert_eq!(r.find("Inter", 600, FontStyleAxis::Normal), Some(bold));
  assert_eq!(r.find("Inter", 450, FontStyleAxis::Normal), Some(regular));
}

#[test]
fn find_prefers_lighter_for_sub_400_targets() {
  let mut r = FontRegistry::new();
  let regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let _bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
  assert_eq!(r.find("Inter", 300, FontStyleAxis::Normal), Some(regular));
}

#[test]
fn find_prefers_heavier_for_super_500_targets() {
  let mut r = FontRegistry::new();
  let _light = r.register(face("Inter", 300, FontStyleAxis::Normal, 1));
  let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
  assert_eq!(r.find("Inter", 600, FontStyleAxis::Normal), Some(bold));
}

#[test]
fn style_axis_exact_beats_swap_beats_normal() {
  let mut r = FontRegistry::new();
  let normal = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let italic = r.register(face("Inter", 400, FontStyleAxis::Italic, 2));
  let oblique = r.register(face("Inter", 400, FontStyleAxis::Oblique, 3));
  assert_eq!(r.find("Inter", 400, FontStyleAxis::Italic), Some(italic));
  assert_eq!(r.find("Inter", 400, FontStyleAxis::Oblique), Some(oblique));
  assert_eq!(r.find("Inter", 400, FontStyleAxis::Normal), Some(normal));
}

#[test]
fn italic_swaps_to_oblique_when_no_italic() {
  let mut r = FontRegistry::new();
  let normal = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let oblique = r.register(face("Inter", 400, FontStyleAxis::Oblique, 2));
  assert_eq!(r.find("Inter", 400, FontStyleAxis::Italic), Some(oblique));
  assert_eq!(r.find("Inter", 400, FontStyleAxis::Normal), Some(normal));
}

#[test]
fn re_register_overrides_on_ties() {
  let mut r = FontRegistry::new();
  let _first = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let second = r.register(face("Inter", 400, FontStyleAxis::Normal, 9));
  let h = r.find("Inter", 400, FontStyleAxis::Normal).unwrap();
  assert_eq!(h, second);
  assert_eq!(r.get(h).unwrap().data[0], 9);
}

#[test]
fn find_first_walks_family_list() {
  let mut r = FontRegistry::new();
  let inter = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let _other = r.register(face("Helvetica", 400, FontStyleAxis::Normal, 2));
  assert_eq!(
    r.find_first(&["Roboto", "Inter", "Helvetica"], 400, FontStyleAxis::Normal),
    Some(inter)
  );
  assert!(r.find_first(&["Garamond"], 400, FontStyleAxis::Normal).is_none());
}

#[test]
fn find_first_falls_back_on_generic_family() {
  let mut r = FontRegistry::new();
  let inter = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let _bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));

  assert_eq!(r.find_first(&["sans-serif"], 400, FontStyleAxis::Normal), Some(inter));
  assert_eq!(
    r.find_first(&["Garamond", "sans-serif"], 400, FontStyleAxis::Normal),
    Some(inter)
  );
  assert!(r.find_first(&["Garamond"], 400, FontStyleAxis::Normal).is_none());
}

#[test]
fn find_first_generic_fallback_respects_weight() {
  let mut r = FontRegistry::new();
  let _regular = r.register(face("Inter", 400, FontStyleAxis::Normal, 1));
  let bold = r.register(face("Inter", 700, FontStyleAxis::Normal, 2));
  assert_eq!(r.find_first(&["sans-serif"], 700, FontStyleAxis::Normal), Some(bold));
}

#[test]
fn empty_registry_returns_none() {
  let r = FontRegistry::new();
  assert!(r.is_empty());
  assert!(r.find("Inter", 400, FontStyleAxis::Normal).is_none());
  assert!(r.find_first(&["Inter"], 400, FontStyleAxis::Normal).is_none());
}
