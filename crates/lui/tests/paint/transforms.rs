use lui::paint::*;

fn approx(a: f32, b: f32) -> bool {
  (a - b).abs() < 0.5
}

// ── translate ─────────────────────────────────────────────────────────

#[test]
fn translate_shifts_quad_position() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:translate(30px,20px);width:100px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  assert!(approx(q.rect.x, 30.0), "x should be shifted by 30px, got {}", q.rect.x);
  assert!(approx(q.rect.y, 20.0), "y should be shifted by 20px, got {}", q.rect.y);
  assert!(approx(q.rect.w, 100.0), "width unchanged");
  assert!(approx(q.rect.h, 50.0), "height unchanged");
}

#[test]
fn translate_percentage_shifts_by_own_size() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:translate(-50%,-50%);width:200px;height:100px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  assert!(approx(q.rect.x, -100.0), "x = -50% of 200 = -100, got {}", q.rect.x);
  assert!(approx(q.rect.y, -50.0), "y = -50% of 100 = -50, got {}", q.rect.y);
}

// ── scale ─────────────────────────────────────────────────────────────

#[test]
fn scale_doubles_quad_size_around_center() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:scale(2);width:100px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  assert!(approx(q.rect.w, 200.0), "width should double: got {}", q.rect.w);
  assert!(approx(q.rect.h, 100.0), "height should double: got {}", q.rect.h);
  // origin is center (50, 25), so the quad expands outward:
  // new_x = 50 - 100 = -50,  new_y = 25 - 50 = -25
  assert!(approx(q.rect.x, -50.0), "x should shift to -50: got {}", q.rect.x);
  assert!(approx(q.rect.y, -25.0), "y should shift to -25: got {}", q.rect.y);
}

#[test]
fn scale_half_shrinks_quad() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:scale(0.5);width:100px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  assert!(approx(q.rect.w, 50.0), "width should halve: got {}", q.rect.w);
  assert!(approx(q.rect.h, 25.0), "height should halve: got {}", q.rect.h);
  // origin is center (50, 25), scale 0.5:
  // new_x = 50 - 25 = 25,  new_y = 25 - 12.5 = 12.5
  assert!(approx(q.rect.x, 25.0), "x should shift to 25: got {}", q.rect.x);
  assert!(approx(q.rect.y, 12.5), "y should shift to 12.5: got {}", q.rect.y);
}

#[test]
fn scale_with_origin_top_left() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:scale(2);transform-origin:left top;width:100px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  // origin top-left: position stays at (0,0), size doubles
  assert!(approx(q.rect.x, 0.0), "x stays at 0 with top-left origin: got {}", q.rect.x);
  assert!(approx(q.rect.y, 0.0), "y stays at 0 with top-left origin: got {}", q.rect.y);
  assert!(approx(q.rect.w, 200.0), "width doubles: got {}", q.rect.w);
  assert!(approx(q.rect.h, 100.0), "height doubles: got {}", q.rect.h);
}

#[test]
fn scale_xy_non_uniform() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:scale(2,0.5);transform-origin:left top;width:100px;height:100px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  assert!(approx(q.rect.w, 200.0), "width scaled 2x: got {}", q.rect.w);
  assert!(approx(q.rect.h, 50.0), "height scaled 0.5x: got {}", q.rect.h);
}

#[test]
fn scale_affects_child_quads() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:scale(2);transform-origin:left top;width:100px;height:100px">
        <div style="width:50px;height:25px;background:blue"></div>
      </div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  assert!(approx(q.rect.w, 100.0), "child width 50 * scale 2 = 100: got {}", q.rect.w);
  assert!(approx(q.rect.h, 50.0), "child height 25 * scale 2 = 50: got {}", q.rect.h);
}

// ── combined ──────────────────────────────────────────────────────────

#[test]
fn translate_then_scale() {
  let list = paint_tree(
    &lui_parser::parse(r#"<body style="margin:0">
      <div style="transform:translate(10px,10px) scale(2);transform-origin:left top;width:50px;height:50px;background:red"></div>
    </body>"#),
    400.0, 400.0,
  );
  assert_eq!(list.quads.len(), 1);
  let q = &list.quads[0];
  // translate(10,10) then scale(2) from top-left:
  // matrix: a=2, d=2, tx=10, ty=10
  // position = (10, 10), size = (100, 100)
  assert!(approx(q.rect.x, 10.0), "x = 10: got {}", q.rect.x);
  assert!(approx(q.rect.y, 10.0), "y = 10: got {}", q.rect.y);
  assert!(approx(q.rect.w, 100.0), "w = 50*2 = 100: got {}", q.rect.w);
  assert!(approx(q.rect.h, 100.0), "h = 50*2 = 100: got {}", q.rect.h);
}
