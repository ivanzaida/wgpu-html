use super::helpers::*;
use crate::*;

// ---------------------------------------------------------------------------
// border
// ---------------------------------------------------------------------------

#[test]
fn border_width_pushes_content_inward() {
  // content-box: width=100 + border-width=4 (each side) → border_rect=108.
  let tree = make(r#"<body style="margin: 0; width: 100px; height: 50px; border-width: 4px;"></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.border.top, 4.0);
  assert_eq!(root.border.left, 4.0);
  assert_eq!(root.border_rect.w, 108.0);
  assert_eq!(root.border_rect.h, 58.0);
  assert_eq!(root.content_rect.x, 4.0);
  assert_eq!(root.content_rect.y, 4.0);
  assert_eq!(root.content_rect.w, 100.0);
}

#[test]
fn border_box_subtracts_border_too() {
  // box-sizing: border-box → 100px = border + padding + content.
  let tree = make(
    r#"<body style="margin: 0; box-sizing: border-box;
                         width: 100px; height: 100px;
                         border-width: 4px; padding: 8px;"></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.border_rect.w, 100.0);
  // 100 - border*2 (8) - padding*2 (16) = 76
  assert_eq!(root.content_rect.w, 76.0);
}

#[test]
fn border_color_resolves_for_paint() {
  let tree = make(
    r#"<body style="margin: 0; width: 50px; height: 50px;
                         border: 2px solid red;"></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  // The shorthand fans red to all four sides.
  let c = root.border_colors.top.expect("top");
  assert!((c[0] - 1.0).abs() < 1e-6);
  assert_eq!(c[1], 0.0);
  assert_eq!(c[2], 0.0);
  assert!(root.border_colors.left.is_some());
  assert!(root.border_colors.right.is_some());
  assert!(root.border_colors.bottom.is_some());
}

#[test]
fn per_side_border_widths_become_per_side_insets() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 50px;
                         border-width: 1px 2px 3px 4px;"></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.border.top, 1.0);
  assert_eq!(root.border.right, 2.0);
  assert_eq!(root.border.bottom, 3.0);
  assert_eq!(root.border.left, 4.0);
  // border_rect = content + horizontal/vertical border.
  assert_eq!(root.border_rect.w, 100.0 + 2.0 + 4.0);
  assert_eq!(root.border_rect.h, 50.0 + 1.0 + 3.0);
  // content offset by left/top borders.
  assert_eq!(root.content_rect.x, 4.0);
  assert_eq!(root.content_rect.y, 1.0);
}

#[test]
fn per_side_border_colors_make_their_way_to_layout() {
  let tree = make(
    r#"<body style="margin: 0; width: 50px; height: 50px;
                         border-width: 2px;
                         border-color: red green blue orange;"></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert!(root.border_colors.top.is_some());
  assert!(root.border_colors.right.is_some());
  assert!(root.border_colors.bottom.is_some());
  assert!(root.border_colors.left.is_some());
  // Different sides should resolve to different values.
  assert_ne!(root.border_colors.top, root.border_colors.right);
}

#[test]
fn border_radius_per_corner_lays_out() {
  let tree = make(
    r#"<body style="margin: 0; width: 50px; height: 50px;
                         border-radius: 1px 2px 3px 4px;"></body>"#,
  );
  let root = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(root.border_radius.top_left, Radius::circle(1.0));
  assert_eq!(root.border_radius.top_right, Radius::circle(2.0));
  assert_eq!(root.border_radius.bottom_right, Radius::circle(3.0));
  assert_eq!(root.border_radius.bottom_left, Radius::circle(4.0));
}

#[test]
fn radii_no_overflow_left_unchanged() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 100px;
                         border-radius: 10px 20px 30px 40px;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
  // 10+20=30 ≤ 100 (top), 30+40=70 ≤ 100 (bottom), 10+40=50 ≤ 100 (left), 20+30=50 ≤ 100 (right)
  assert_eq!(r.top_left, Radius::circle(10.0));
  assert_eq!(r.top_right, Radius::circle(20.0));
  assert_eq!(r.bottom_right, Radius::circle(30.0));
  assert_eq!(r.bottom_left, Radius::circle(40.0));
}

#[test]
fn radii_horizontal_overflow_scales_all_corners() {
  // Top side sum = 60 + 80 = 140 > 100 → scale = 100 / 140.
  // Final radii: each multiplied by 100/140 ≈ 0.7142857.
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 200px;
                         border-radius: 60px 80px 60px 80px;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
  let s = 100.0_f32 / 140.0;
  // Both axes scale uniformly for a circular input radius.
  assert!((r.top_left.h - 60.0 * s).abs() < 1e-3);
  assert!((r.top_right.h - 80.0 * s).abs() < 1e-3);
  assert!((r.bottom_right.h - 60.0 * s).abs() < 1e-3);
  assert!((r.bottom_left.h - 80.0 * s).abs() < 1e-3);
  assert!((r.top_left.h + r.top_right.h - 100.0).abs() < 1e-3);
}

#[test]
fn radii_smallest_factor_wins_when_multiple_sides_overflow() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 200px;
                         border-radius: 80px;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
  let s = 100.0_f32 / 160.0;
  assert!((r.top_left.h - 80.0 * s).abs() < 1e-3);
  assert!((r.bottom_right.h - 80.0 * s).abs() < 1e-3);
}

#[test]
fn background_clip_default_is_border_box() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(r.background_rect, r.border_rect);
}

#[test]
fn background_clip_padding_box_strips_border() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;
                         background-clip: padding-box;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(r.background_rect.x, r.border_rect.x + 4.0);
  assert_eq!(r.background_rect.y, r.border_rect.y + 4.0);
  assert_eq!(r.background_rect.w, r.border_rect.w - 8.0);
  assert_eq!(r.background_rect.h, r.border_rect.h - 8.0);
}

#[test]
fn background_clip_content_box_strips_border_and_padding() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;
                         background-clip: content-box;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(r.background_rect, r.content_rect);
}

#[test]
fn background_clip_padding_box_shrinks_radii() {
  let tree = make(
    r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red;
                         border: 4px solid blue;
                         border-radius: 12px;
                         background-clip: padding-box;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(r.background_radii.top_left, Radius::circle(8.0));
  assert_eq!(r.background_radii.bottom_right, Radius::circle(8.0));
}

#[test]
fn radii_negative_input_clamped_to_zero() {
  // Negative px in the source → resolved to 0 by `.max(0.0)`.
  let tree = make(
    r#"<body style="margin: 0; width: 50px; height: 50px;
                         border-top-left-radius: -8px;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
  assert_eq!(r.top_left, Radius::zero());
}

#[test]
fn elliptical_radius_h_v_split() {
  let tree = make(
    r#"<body style="margin: 0; width: 200px; height: 100px;
                         border-radius: 20px / 10px;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
  assert_eq!(r.top_left.h, 20.0);
  assert_eq!(r.top_left.v, 10.0);
  assert_eq!(r.bottom_right.h, 20.0);
  assert_eq!(r.bottom_right.v, 10.0);
}

#[test]
fn per_corner_h_v_in_longhand() {
  let tree = make(
    r#"<body style="margin: 0; width: 200px; height: 100px;
                         border-top-left-radius: 30px 10px;"></body>"#,
  );
  let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
  assert_eq!(r.top_left.h, 30.0);
  assert_eq!(r.top_left.v, 10.0);
}
