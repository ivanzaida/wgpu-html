use crate::*;
use super::helpers::*;
// ---------------------------------------------------------------------------
// box-sizing
// ---------------------------------------------------------------------------

#[test]
fn box_sizing_content_box_is_default() {
  // `width` is the content-box width; padding is added on the outside.
  let tree = make(r#"<body style="margin: 0; width: 100px; padding: 10px; height: 50px;"></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.content_rect.w, 100.0);
  assert_eq!(root.border_rect.w, 120.0); // 100 + 10*2 padding
}

#[test]
fn box_sizing_border_box_subtracts_padding() {
  // `width` is the border-box width; padding eats into the content.
  let tree =
    make(r#"<body style="margin: 0; box-sizing: border-box; width: 100px; padding: 10px; height: 50px;"></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.border_rect.w, 100.0);
  assert_eq!(root.content_rect.w, 80.0); // 100 - 10*2 padding
}

#[test]
fn box_sizing_border_box_subtracts_padding_from_height() {
  let tree =
    make(r#"<body style="margin: 0; box-sizing: border-box; width: 100px; height: 100px; padding: 10px;"></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.border_rect.h, 100.0);
  assert_eq!(root.content_rect.h, 80.0);
}

// ---------------------------------------------------------------------------
// box-sizing border-box overflow regression
// ---------------------------------------------------------------------------

#[test]
fn box_sizing_border_box_with_full_width_fits_within_container() {
  // Reproduces the original `width: 100% + padding` overflow bug:
  // with border-box the body now stays inside the viewport.
  let tree = make(r#"<body style="margin: 0; box-sizing: border-box; width: 100%; padding: 32px;"></body>"#);
  let root = layout(&tree, 1024.0, 768.0).unwrap();
  assert_eq!(root.border_rect.w, 1024.0);
  assert_eq!(root.content_rect.x, 32.0);
  assert_eq!(root.content_rect.w, 960.0); // 1024 - 32*2
}
