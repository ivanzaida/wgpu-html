use lui::paint::*;

fn approx(a: f32, b: f32) -> bool {
  (a - b).abs() < 1.0
}

#[test]
fn box_shadow_emits_quad_before_background() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:0 4px 8px rgba(0,0,0,0.5);width:100px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  // Should have 2 quads: shadow + background
  assert!(list.quads.len() >= 2, "expected shadow + background quads, got {}", list.quads.len());
  let shadow = &list.quads[0];
  let bg = &list.quads[1];
  // Shadow quad should be larger (expanded by blur + spread)
  assert!(shadow.rect.w > bg.rect.w, "shadow wider than background");
  assert!(shadow.rect.h > bg.rect.h, "shadow taller than background");
  // Shadow should have non-zero sigma
  assert!(shadow.shadow_sigma > 0.0, "shadow sigma should be > 0, got {}", shadow.shadow_sigma);
  // Background should have zero sigma
  assert_eq!(bg.shadow_sigma, 0.0, "background sigma should be 0");
}

#[test]
fn box_shadow_offset_shifts_position() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:10px 20px 0 black;width:100px;height:50px;background:white"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  let bg = &list.quads[1];
  // Shadow offset: x+10, y+20 relative to background
  assert!(approx(shadow.rect.x - bg.rect.x, 10.0), "x offset = 10: got {}", shadow.rect.x - bg.rect.x);
  assert!(approx(shadow.rect.y - bg.rect.y, 20.0), "y offset = 20: got {}", shadow.rect.y - bg.rect.y);
}

#[test]
fn box_shadow_spread_expands_rect() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:0 0 0 10px black;width:100px;height:50px;background:white"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  // Spread of 10px expands by 10 on each side
  assert!(approx(shadow.rect.w, 120.0), "w = 100 + 2*10 = 120: got {}", shadow.rect.w);
  assert!(approx(shadow.rect.h, 70.0), "h = 50 + 2*10 = 70: got {}", shadow.rect.h);
}

#[test]
fn box_shadow_blur_expands_and_sets_sigma() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:0 0 20px black;width:100px;height:50px;background:white"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  assert!(approx(shadow.shadow_sigma, 20.0), "sigma = blur = 20: got {}", shadow.shadow_sigma);
  // Rect expanded by blur on each side
  assert!(approx(shadow.rect.w, 140.0), "w = 100 + 2*20 = 140: got {}", shadow.rect.w);
}

#[test]
fn multiple_shadows_emit_multiple_quads() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:0 2px 4px black, 0 0 10px red;width:100px;height:50px;background:white"></div>
    </body>"#),
    400.0, 400.0,
  );
  // 2 shadow quads + 1 background = 3 minimum
  assert!(list.quads.len() >= 3, "expected 3+ quads: got {}", list.quads.len());
  assert!(list.quads[0].shadow_sigma > 0.0, "first shadow has sigma");
  assert!(list.quads[1].shadow_sigma > 0.0, "second shadow has sigma");
}

#[test]
fn no_shadow_no_extra_quads() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="width:100px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1, "just one background quad");
  assert_eq!(list.quads[0].shadow_sigma, 0.0);
}

#[test]
fn shadow_respects_border_radius() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:0 0 10px black;border-radius:8px;width:100px;height:50px;background:white"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert!(list.quads.len() >= 2);
  let shadow = &list.quads[0];
  // Shadow radii should be expanded by the blur amount
  assert!(shadow.radii_h[0] > 8.0, "shadow radius > box radius: got {}", shadow.radii_h[0]);
}

#[test]
fn inset_shadow_not_painted_as_outer() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="box-shadow:inset 0 2px 4px black;width:100px;height:50px;background:white"></div>
    </body>"#),
    400.0, 400.0,
  );
  // Inset shadows are skipped for now — only background quad
  assert_eq!(list.quads.len(), 1, "inset shadow should not emit outer quad");
}
