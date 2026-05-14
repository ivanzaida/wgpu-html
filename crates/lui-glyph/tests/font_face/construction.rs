use std::sync::Arc;

use lui_glyph::{FontFace, FontStyleAxis};

use crate::helpers::dummy_font_data;

#[test]
fn regular_sets_weight_400_and_normal_style() {
  let face = FontFace::regular("TestFont", dummy_font_data());
  assert_eq!(face.family, "TestFont");
  assert_eq!(face.weight, 400);
  assert_eq!(face.style, FontStyleAxis::Normal);
}

#[test]
fn bold_sets_weight_700_and_normal_style() {
  let face = FontFace::bold("TestFont", dummy_font_data());
  assert_eq!(face.weight, 700);
  assert_eq!(face.style, FontStyleAxis::Normal);
}

#[test]
fn italic_sets_weight_400_and_italic_style() {
  let face = FontFace::italic("TestFont", dummy_font_data());
  assert_eq!(face.weight, 400);
  assert_eq!(face.style, FontStyleAxis::Italic);
}

#[test]
fn bold_italic_sets_weight_700_and_italic_style() {
  let face = FontFace::bold_italic("TestFont", dummy_font_data());
  assert_eq!(face.weight, 700);
  assert_eq!(face.style, FontStyleAxis::Italic);
}

#[test]
fn new_with_custom_weight_and_style() {
  let face = FontFace::new("Custom", 250, FontStyleAxis::Oblique, dummy_font_data());
  assert_eq!(face.family, "Custom");
  assert_eq!(face.weight, 250);
  assert_eq!(face.style, FontStyleAxis::Oblique);
}

#[test]
fn family_accepts_string_and_str() {
  let data = dummy_font_data();
  let from_str = FontFace::regular("Foo", data.clone());
  let from_string = FontFace::regular(String::from("Foo"), data);
  assert_eq!(from_str.family, from_string.family);
}

#[test]
fn data_is_shared_via_arc() {
  let data = dummy_font_data();
  let face1 = FontFace::regular("A", data.clone());
  let face2 = FontFace::regular("B", data.clone());
  assert!(Arc::ptr_eq(&face1.data, &face2.data));
}

#[test]
fn default_font_style_axis_is_normal() {
  assert_eq!(FontStyleAxis::default(), FontStyleAxis::Normal);
}
