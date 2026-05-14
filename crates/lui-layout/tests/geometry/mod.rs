use lui_core::Rect;
use lui_layout::{Point, RectEdges, Size, geometry::RectExt};

#[test]
fn point_new_sets_x_and_y() {
  let p = Point::new(1.0, 2.0);
  assert_eq!(p.x, 1.0, "x should be 1.0");
  assert_eq!(p.y, 2.0, "y should be 2.0");
}

#[test]
fn point_default_is_zero() {
  let p = Point::default();
  assert_eq!(p.x, 0.0);
  assert_eq!(p.y, 0.0);
}

#[test]
fn size_default_is_zero() {
  let s = Size::default();
  assert_eq!(s.width, 0.0, "width should be 0");
  assert_eq!(s.height, 0.0, "height should be 0");
}

#[test]
fn rect_edges_new_sets_all_fields() {
  let edges: RectEdges<i32> = RectEdges::new(1, 2, 3, 4);
  assert_eq!(edges.top, 1);
  assert_eq!(edges.right, 2);
  assert_eq!(edges.bottom, 3);
  assert_eq!(edges.left, 4);
}

#[test]
fn rect_edges_default_is_all_zero() {
  let edges: RectEdges<f32> = RectEdges::default();
  assert_eq!(edges.top, 0.0);
  assert_eq!(edges.right, 0.0);
  assert_eq!(edges.bottom, 0.0);
  assert_eq!(edges.left, 0.0);
}

#[test]
fn rect_edges_horizontal_sums_left_and_right() {
  let edges = RectEdges::<f32>::new(5.0, 10.0, 3.0, 2.0);
  assert_eq!(edges.horizontal(), 12.0, "horizontal = left + right = 2 + 10");
}

#[test]
fn rect_edges_vertical_sums_top_and_bottom() {
  let edges = RectEdges::<f32>::new(5.0, 10.0, 3.0, 2.0);
  assert_eq!(edges.vertical(), 8.0, "vertical = top + bottom = 5 + 3");
}

#[test]
fn rect_ext_max_x_is_x_plus_width() {
  let r = Rect::new(10.0, 20.0, 100.0, 50.0);
  assert_eq!(r.max_x(), 110.0);
}

#[test]
fn rect_ext_max_y_is_y_plus_height() {
  let r = Rect::new(10.0, 20.0, 100.0, 50.0);
  assert_eq!(r.max_y(), 70.0);
}

#[test]
fn rect_ext_contains_point_inside() {
  let r = Rect::new(10.0, 20.0, 100.0, 50.0);
  assert!(r.contains(50.0, 40.0), "point inside rect should be contained");
}

#[test]
fn rect_ext_contains_top_left_corner() {
  let r = Rect::new(10.0, 20.0, 100.0, 50.0);
  assert!(
    r.contains(10.0, 20.0),
    "top-left corner is on the edge, should be contained"
  );
}

#[test]
fn rect_ext_contains_bottom_right_corner() {
  let r = Rect::new(10.0, 20.0, 100.0, 50.0);
  assert!(
    r.contains(110.0, 70.0),
    "bottom-right corner is on the edge, should be contained"
  );
}

#[test]
fn rect_ext_does_not_contain_point_outside() {
  let r = Rect::new(10.0, 20.0, 100.0, 50.0);
  assert!(!r.contains(5.0, 40.0), "point left of rect should not be contained");
  assert!(!r.contains(120.0, 40.0), "point right of rect should not be contained");
  assert!(!r.contains(50.0, 10.0), "point above rect should not be contained");
  assert!(!r.contains(50.0, 80.0), "point below rect should not be contained");
}
