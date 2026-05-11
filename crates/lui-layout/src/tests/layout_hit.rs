use super::helpers::synthetic_text_layout;
use crate::{hit_test::collect_hit_path, *};

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

const HIT_HTML: &str = r#"<body style="margin: 0; width: 800px; height: 600px;">
         <div style="width: 200px; height: 100px;">
           <div style="width: 50px; height: 40px;
                        margin: 10px 0 0 10px;"></div>
         </div>
       </body>"#;

fn hit_setup() -> (lui_tree::Tree, LayoutBox) {
  let tree = lui_parser::parse(HIT_HTML);
  let cascaded = lui_style::cascade(&tree);
  let lay = layout(&cascaded, 800.0, 600.0).unwrap();
  (tree, lay)
}

fn element_kind(n: &lui_tree::Node) -> &'static str {
  use lui_tree::Element;
  match &n.element {
    Element::Body(_) => "body",
    Element::Div(_) => "div",
    Element::Text(_) => "text",
    _ => "other",
  }
}

#[test]
fn hit_path_outside_is_none() {
  let (_tree, lay) = hit_setup();
  assert!(lay.hit_path((10_000.0, 10_000.0)).is_none());
}

#[test]
fn hit_path_drills_to_inner_div() {
  let (_tree, lay) = hit_setup();
  // (20, 20) lives inside the inner div: outer (idx 0) → inner (idx 0).
  let path = lay.hit_path((20.0, 20.0)).unwrap();
  assert_eq!(path, vec![0, 0]);
}

#[test]
fn find_element_outside_returns_none() {
  let (mut tree, lay) = hit_setup();
  assert!(lay.find_element_from_point(&mut tree, (10_000.0, 10_000.0)).is_none());
}

#[test]
fn find_element_returns_deepest_node() {
  let (mut tree, lay) = hit_setup();
  let node = lay.find_element_from_point(&mut tree, (20.0, 20.0)).unwrap();
  assert_eq!(element_kind(node), "div");
  assert!(node.children.is_empty()); // it's the inner div
}

#[test]
fn find_element_lets_caller_mutate_style() {
  let (mut tree, lay) = hit_setup();
  {
    let node = lay.find_element_from_point(&mut tree, (20.0, 20.0)).unwrap();
    // The whole point of returning &mut Node: mutate the source
    // element's style attribute, then re-cascade and re-layout.
    if let lui_tree::Element::Div(div) = &mut node.element {
      div.style = Some("width: 123px; height: 40px; margin: 10px 0 0 10px;".into());
    } else {
      panic!("expected a Div at the hit point");
    }
  }
  let cascaded = lui_style::cascade(&tree);
  let lay2 = layout(&cascaded, 800.0, 600.0).unwrap();
  let inner = &lay2.children[0].children[0];
  assert_eq!(inner.border_rect.w, 123.0);
}

#[test]
fn find_element_falls_back_to_root_when_no_descendant_hit() {
  let (mut tree, lay) = hit_setup();
  // (300, 50) is inside body but past the outer div (only 200 wide).
  let node = lay.find_element_from_point(&mut tree, (300.0, 50.0)).unwrap();
  assert_eq!(element_kind(node), "body");
}

#[test]
fn find_elements_orders_child_to_parent() {
  let (mut tree, lay) = hit_setup();
  let chain = lay.find_elements_from_point(&mut tree, (20.0, 20.0));
  assert_eq!(chain.len(), 3);
  // Deepest first: inner div, outer div, body.
  assert_eq!(element_kind(chain[0]), "div");
  assert!(chain[0].children.is_empty());
  assert_eq!(element_kind(chain[1]), "div");
  assert_eq!(chain[1].children.len(), 1);
  assert_eq!(element_kind(chain[2]), "body");
}

#[test]
fn find_elements_outside_is_empty() {
  let (mut tree, lay) = hit_setup();
  assert!(lay.find_elements_from_point(&mut tree, (-1.0, -1.0)).is_empty());
}

#[test]
fn hit_text_cursor_maps_point_to_glyph_boundary() {
  let lay = synthetic_text_layout();
  let c0 = lay.hit_text_cursor((11.0, 24.0)).expect("cursor");
  let c1 = lay.hit_text_cursor((26.0, 24.0)).expect("cursor");
  let c2 = lay.hit_text_cursor((39.0, 24.0)).expect("cursor");
  assert_eq!(c0.glyph_index, 0);
  assert_eq!(c1.glyph_index, 2);
  assert_eq!(c2.glyph_index, 3);
}

#[test]
fn hit_text_cursor_outside_returns_none() {
  let lay = synthetic_text_layout();
  assert!(lay.hit_text_cursor((200.0, 24.0)).is_none());
}

// ── Transform-aware hit-testing ─────────────────────────────────────

#[test]
fn hit_path_uses_transform_inverse() {
  // A 100×50 div at (200, 100), rotated 90° clockwise.
  // After 90° rotation around its top-left, the visual box spans
  // from x=200 to x=250 (the original Y becomes X), and y=100 down.
  let r = Rect::new(200.0, 100.0, 100.0, 50.0);
  let rot = crate::transform::Transform2D::rotate(std::f32::consts::FRAC_PI_2);
  let bx = LayoutBox {
    margin_rect: r,
    border_rect: r,
    content_rect: r,
    transform: Some(rot),
    transform_origin: (0.0, 0.0),
    ..synthetic_text_layout()
  };

  // The center of the original rect is (250, 125).
  // After 90° CW rotation around top-left (200, 100):
  // (50, 25) in local → (-25, 50) + (200, 100) = (175, 150)
  // A point at (175, 150) should be inside the rotated rect.
  let path = collect_hit_path(&bx, 175.0, 150.0, None);
  assert!(path.is_some(), "point inside rotated rect should hit");

  // A point far outside (e.g. the original untransformed rect center
  // but NOT in the rotated area) should NOT hit.
  let path = collect_hit_path(&bx, 250.0, 125.0, None);
  assert!(
    path.is_none(),
    "point at original center but outside rotated area should miss"
  );
}
