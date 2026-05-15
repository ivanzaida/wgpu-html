use crate::support::{TEST_HEIGHT, TEST_WIDTH, test_lui};

fn approx(a: f32, b: f32) -> bool {
  (a - b).abs() < 0.01
}

#[test]
fn rotate_produces_non_identity_transform() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="width:50px; height:50px; background:red; transform: rotate(45deg)"></div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[0] > 0.9 && q.color[1] < 0.1 && q.color[2] < 0.1)
    .collect();
  assert!(!red_quads.is_empty(), "expected red quad");

  let q = red_quads[0];
  assert!(
    q.transform != [1.0, 0.0, 0.0, 1.0],
    "transform should not be identity for rotate(45deg), got {:?}",
    q.transform
  );
  // rotate(45deg): a=cos(45°)≈0.707, b=sin(45°)≈0.707
  assert!(approx(q.transform[0], 0.707), "a ≈ cos(45°)");
  assert!(approx(q.transform[1], 0.707), "b ≈ sin(45°)");
}

#[test]
fn scale_produces_scaled_transform() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="width:50px; height:50px; background:blue; transform: scale(2)"></div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let blue_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[2] > 0.9 && q.color[0] < 0.1 && q.color[1] < 0.1)
    .collect();
  assert!(!blue_quads.is_empty(), "expected blue quad");

  let q = blue_quads[0];
  assert!(approx(q.transform[0], 2.0), "a = sx = 2.0, got {}", q.transform[0]);
  assert!(approx(q.transform[3], 2.0), "d = sy = 2.0, got {}", q.transform[3]);
}

#[test]
fn no_transform_keeps_identity() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="width:50px; height:50px; background:#00ff00"></div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let green_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[1] > 0.9 && q.color[0] < 0.1 && q.color[2] < 0.1)
    .collect();
  assert!(!green_quads.is_empty(), "expected green quad");
  assert_eq!(green_quads[0].transform, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn chained_transforms_compose() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="width:50px; height:50px; background:red; transform: scale(2) rotate(90deg)"></div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[0] > 0.9 && q.color[1] < 0.1 && q.color[2] < 0.1)
    .collect();
  assert!(!red_quads.is_empty(), "expected red quad");

  let q = red_quads[0];
  assert!(
    q.transform != [1.0, 0.0, 0.0, 1.0],
    "chained transform should not be identity"
  );
  // scale(2) then rotate(90deg): a≈0, b≈2, c≈-2, d≈0
  assert!(approx(q.transform[0], 0.0), "a ≈ 0");
  assert!(approx(q.transform[1], 2.0), "b ≈ 2");
  assert!(approx(q.transform[2], -2.0), "c ≈ -2");
  assert!(approx(q.transform[3], 0.0), "d ≈ 0");
}

#[test]
fn transform_origin_changes_origin_point() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="width:50px; height:50px; background:red; transform: rotate(45deg); transform-origin: left top"></div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[0] > 0.9 && q.color[1] < 0.1 && q.color[2] < 0.1)
    .collect();
  assert!(!red_quads.is_empty(), "expected red quad");

  let q = red_quads[0];
  assert!(approx(q.transform_origin[0], 0.0), "ox should be 0 (left)");
  assert!(approx(q.transform_origin[1], 0.0), "oy should be 0 (top)");
}

#[test]
fn transform_none_keeps_identity() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="width:50px; height:50px; background:red; transform: none"></div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  let red_quads: Vec<_> = list
    .quads
    .iter()
    .filter(|q| q.color[0] > 0.9 && q.color[1] < 0.1 && q.color[2] < 0.1)
    .collect();
  assert!(!red_quads.is_empty(), "expected red quad");
  assert_eq!(red_quads[0].transform, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn text_glyphs_get_parent_transform() {
  let (mut lui, mut spy) = test_lui(
    r#"<html><body>
      <div style="font-size: 20px; transform: scale(2)">Hello</div>
    </body></html>"#,
  );
  lui.render_frame(&mut spy, TEST_WIDTH, TEST_HEIGHT, 1.0);
  let list = spy.take_last_list();

  if !list.glyphs.is_empty() {
    let g = &list.glyphs[0];
    assert!(approx(g.transform[0], 2.0), "glyph should have scale(2) transform");
    assert!(approx(g.transform[3], 2.0), "glyph should have scale(2) transform");
  }
}
