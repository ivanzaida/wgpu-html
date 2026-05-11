use lui::{paint::*, text::TextContext};

fn approx(a: f32, b: f32) -> bool {
  (a - b).abs() < 1.0
}

fn paint_with_fonts(html: &str, w: f32, h: f32) -> lui::renderer::DisplayList {
  let mut tree = lui_parser::parse(html);
  tree.register_system_fonts("DemoSans");
  let mut ctx = TextContext::new(64);
  let mut ic = lui_layout::ImageCache::default();
  paint_tree_with_text(&tree, &mut ctx, &mut ic, w, h, 1.0, 0.0)
}

// ── shadow quads get the parent's transform ───────────────────────────

#[test]
fn shadow_quad_inherits_rotation() {
  let list = paint_tree(
    &lui_parser::parse(
      r#"<body style="margin:0">
      <div style="transform:rotate(45deg);box-shadow:0 4px 8px black;width:100px;height:50px;background:red"></div>
    </body>"#,
    ),
    400.0,
    400.0,
  );
  // shadow + background = 2 quads minimum
  assert!(
    list.quads.len() >= 2,
    "need shadow + bg quads: got {}",
    list.quads.len()
  );
  let cos45 = std::f32::consts::FRAC_1_SQRT_2;
  let shadow = &list.quads[0];
  let bg = &list.quads[1];
  // Both shadow and background should carry the 45deg rotation
  let [a, b, _, _] = shadow.transform;
  assert!((a - cos45).abs() < 0.01, "shadow a ~ cos(45): got {a}");
  assert!((b - cos45).abs() < 0.01, "shadow b ~ sin(45): got {b}");
  let [a, b, _, _] = bg.transform;
  assert!((a - cos45).abs() < 0.01, "bg a ~ cos(45): got {a}");
  assert!((b - cos45).abs() < 0.01, "bg b ~ sin(45): got {b}");
}

#[test]
fn shadow_quad_inherits_scale() {
  let list = paint_tree(
    &lui_parser::parse(
      r#"<body style="margin:0">
      <div style="transform:scale(2);transform-origin:left top;box-shadow:0 0 10px black;width:100px;height:50px;background:red"></div>
    </body>"#,
    ),
    400.0,
    400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  let [a, _, _, d] = shadow.transform;
  assert!(approx(a, 2.0), "shadow scaled 2x: got {a}");
  assert!(approx(d, 2.0), "shadow scaled 2y: got {d}");
}

#[test]
fn shadow_with_sigma_on_rotated_element() {
  let list = paint_tree(
    &lui_parser::parse(
      r#"<body style="margin:0">
      <div style="transform:rotate(30deg);box-shadow:0 4px 16px rgba(0,0,0,0.5);width:100px;height:50px;background:white"></div>
    </body>"#,
    ),
    400.0,
    400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  assert!(shadow.shadow_sigma > 0.0, "shadow should have sigma");
  assert!(shadow.transform != [1.0, 0.0, 0.0, 1.0], "shadow should be rotated");
}

// ── text inside shadowed + transformed container ──────────────────────

#[test]
fn glyphs_in_shadowed_rotated_box_get_transform() {
  let list = paint_with_fonts(
    r#"<body style="margin:0">
      <div style="transform:rotate(15deg);box-shadow:0 2px 8px black;width:200px;height:100px">
        <span>Hello shadow</span>
      </div>
    </body>"#,
    400.0,
    400.0,
  );
  assert!(!list.glyphs.is_empty(), "should have glyphs");
  // Shadow quads should exist
  let shadow_count = list.quads.iter().filter(|q| q.shadow_sigma > 0.0).count();
  assert!(shadow_count >= 1, "should have shadow quad(s)");
  // All glyphs should carry the rotation
  for (i, g) in list.glyphs.iter().enumerate() {
    assert!(
      g.transform != [1.0, 0.0, 0.0, 1.0],
      "glyph {i} should be rotated, got identity"
    );
  }
}

// ── child elements with shadows inside transformed parent ─────────────

#[test]
fn child_shadow_inside_rotated_parent() {
  let list = paint_tree(
    &lui_parser::parse(
      r#"<body style="margin:0">
      <div style="transform:rotate(10deg);width:300px;height:200px">
        <div style="box-shadow:0 2px 6px black;width:100px;height:50px;background:blue"></div>
      </div>
    </body>"#,
    ),
    400.0,
    400.0,
  );
  // Child should have shadow + background, both rotated
  assert!(list.quads.len() >= 2, "need shadow + bg: got {}", list.quads.len());
  for (i, q) in list.quads.iter().enumerate() {
    assert!(
      q.transform != [1.0, 0.0, 0.0, 1.0],
      "quad {i} (sigma={}) should be rotated",
      q.shadow_sigma
    );
  }
}

// ── skewed container with shadow ──────────────────────────────────────

#[test]
fn shadow_on_skewed_element() {
  let list = paint_tree(
    &lui_parser::parse(
      r#"<body style="margin:0">
      <div style="transform:skewX(15deg);box-shadow:0 4px 12px black;width:100px;height:50px;background:red"></div>
    </body>"#,
    ),
    400.0,
    400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  // skewX(15deg) produces c = tan(15deg) ≈ 0.268
  let [_, _, c, _] = shadow.transform;
  assert!(c.abs() > 0.1, "shadow should have skew component: got c={c}");
}

// ── no transform, shadow stays identity ───────────────────────────────

#[test]
fn shadow_without_transform_has_identity() {
  let list = paint_tree(
    &lui_parser::parse(
      r#"<body style="margin:0">
      <div style="box-shadow:0 4px 8px black;width:100px;height:50px;background:red"></div>
    </body>"#,
    ),
    400.0,
    400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  assert_eq!(shadow.transform, [1.0, 0.0, 0.0, 1.0], "no transform = identity");
}
