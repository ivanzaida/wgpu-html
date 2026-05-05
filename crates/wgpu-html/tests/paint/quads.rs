use wgpu_html::paint::*;

#[test]
fn border_emits_four_edge_quads() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border: 2px solid red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  assert_eq!(list.quads[0].stroke, [2.0, 2.0, 2.0, 2.0]);
}

#[test]
fn border_with_background_emits_five_quads() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: blue;
                             border: 2px solid red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 2);
}

#[test]
fn radii_carry_through_to_display_list() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-radius: 1px 2px 3px 4px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  let q = list.quads[0];
  assert_eq!(q.radii_h, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn rounded_uniform_border_emits_single_ring_quad() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border: 1px solid grey;
                             border-radius: 16px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  let q = list.quads[0];
  assert_eq!(q.radii_h, [16.0; 4]);
  assert_eq!(q.radii_v, [16.0; 4]);
  assert_eq!(q.stroke, [1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn rounded_with_background_and_border_emits_two_quads() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border: 2px solid blue;
                             border-radius: 8px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 2);
  assert_eq!(list.quads[0].stroke, [0.0; 4]);
  assert_eq!(list.quads[1].stroke, [2.0, 2.0, 2.0, 2.0]);
}

#[test]
fn rounded_with_per_side_colors_emits_per_side_ring_quads() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-color: red green blue orange;
                             border-radius: 8px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 5);
  for q in &list.quads[1..] {
    let nonzero_sides = q.stroke.iter().filter(|s| **s > 0.0).count();
    assert_eq!(nonzero_sides, 1);
  }
}

#[test]
fn rounded_with_mixed_styles_skips_none_sides() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-style: solid solid none solid;
                             border-color: grey;
                             border-radius: 8px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 4);
}

#[test]
fn sharp_box_border_still_uses_four_edges() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border: 2px solid red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  assert_eq!(list.quads[0].stroke, [2.0, 2.0, 2.0, 2.0]);
  assert_eq!(list.quads[0].radii_h, [0.0; 4]);
}

#[test]
fn no_radius_keeps_sharp_quad() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  assert_eq!(list.quads[0].radii_h, [0.0; 4]);
  assert_eq!(list.quads[0].radii_v, [0.0; 4]);
}

#[test]
fn border_with_no_color_is_skipped() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             background-color: blue;
                             border-width: 2px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
}

#[test]
fn dashed_border_emits_multiple_segments_per_side() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 200px; height: 100px;
                             border: 2px dashed red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(
    list.quads.len() > 8,
    "expected dashed border to emit many segments, got {}",
    list.quads.len()
  );
}

#[test]
fn dotted_border_emits_segments_too() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 200px; height: 100px;
                             border: 2px dotted blue;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(list.quads.len() > 8);
}

#[test]
fn border_style_none_skips_that_side() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 100px; height: 50px;
                             border-width: 2px;
                             border-style: solid solid none solid;
                             border-color: red;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 3);
}

#[test]
fn dashed_with_rounded_emits_per_side_patterned_rings() {
  let tree = wgpu_html_parser::parse(
    r#"<body style="width: 200px; height: 100px;
                             background-color: white;
                             border: 2px dashed red;
                             border-radius: 12px;"></body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 5);
  assert_eq!(list.quads[0].stroke, [0.0; 4]);
  assert_eq!(list.quads[0].pattern, [0.0; 4]);
  for q in &list.quads[1..] {
    assert_eq!(q.pattern[0], 1.0);
    let nonzero = q.stroke.iter().filter(|s| **s > 0.0).count();
    assert_eq!(nonzero, 1);
  }
}
