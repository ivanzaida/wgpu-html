use super::helpers::*;
use crate::*;
// ---------------------------------------------------------------------------
// overflow propagation
// ---------------------------------------------------------------------------

#[test]
fn overflow_field_propagates_from_style() {
  let tree = make(r#"<body style="overflow: hidden; width: 100px; height: 50px;"></body>"#);
  let body = layout(&tree, 800.0, 600.0).unwrap();
  use lui_models::common::css_enums::Overflow;
  assert_eq!(body.overflow.x, Overflow::Hidden);
  assert_eq!(body.overflow.y, Overflow::Hidden);
}

#[test]
fn overflow_visible_is_default() {
  let tree = make(r#"<body style="width: 100px; height: 50px;"></body>"#);
  let body = layout(&tree, 800.0, 600.0).unwrap();
  use lui_models::common::css_enums::Overflow;
  assert_eq!(body.overflow.x, Overflow::Visible);
  assert_eq!(body.overflow.y, Overflow::Visible);
}

#[test]
fn overflow_axis_longhand_wins_over_shorthand() {
  let tree = make(r#"<body style="overflow: scroll; overflow-y: clip; width: 100px; height: 50px;"></body>"#);
  let body = layout(&tree, 800.0, 600.0).unwrap();
  use lui_models::common::css_enums::Overflow;
  assert_eq!(body.overflow.x, Overflow::Scroll);
  assert_eq!(body.overflow.y, Overflow::Hidden);
}

#[test]
fn overflow_shorthand_two_values_sets_axes() {
  let tree = make(r#"<body style="overflow: clip visible; width: 100px; height: 50px;"></body>"#);
  let body = layout(&tree, 800.0, 600.0).unwrap();
  use lui_models::common::css_enums::Overflow;
  assert_eq!(body.overflow.x, Overflow::Clip);
  assert_eq!(body.overflow.y, Overflow::Visible);
}

#[test]
fn overflow_visible_computes_to_auto_against_scrollable_axis() {
  let tree = make(r#"<body style="overflow: hidden visible; width: 100px; height: 50px;"></body>"#);
  let body = layout(&tree, 800.0, 600.0).unwrap();
  use lui_models::common::css_enums::Overflow;
  assert_eq!(body.overflow.x, Overflow::Hidden);
  assert_eq!(body.overflow.y, Overflow::Auto);
}

#[test]
fn overflow_hidden_blocks_child_hit_outside_padding_box() {
  let tree = make(
    r#"<body style="margin: 0; width: 300px; height: 300px;">
            <div style="width: 100px; height: 100px; overflow: hidden;">
                <div style="width: 200px; height: 200px;"></div>
            </div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.hit_path((120.0, 20.0)).unwrap(), Vec::<usize>::new());
}

#[test]
fn overflow_visible_allows_child_hit_outside_parent() {
  let tree = make(
    r#"<body style="margin: 0; width: 300px; height: 300px;">
            <div style="width: 100px; height: 100px; overflow: visible;">
                <div style="width: 200px; height: 200px;"></div>
            </div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.hit_path((120.0, 20.0)).unwrap(), vec![0, 0]);
}

// ---------------------------------------------------------------------------
// pointer-events
// ---------------------------------------------------------------------------

#[test]
fn pointer_events_none_skips_self_in_hit_test() {
  let tree = make(
    r#"<body style="margin: 0; width: 300px; height: 300px;">
            <div style="width: 100px; height: 100px; pointer-events: none;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // Click inside the div — should pass through to body (empty path).
  let path = body.hit_path((50.0, 50.0)).unwrap();
  assert_eq!(path, Vec::<usize>::new(), "pointer-events:none div is transparent");
}

#[test]
fn pointer_events_none_children_still_hittable() {
  let tree = make(
    r#"<body style="margin: 0; width: 300px; height: 300px;">
            <div style="width: 200px; height: 200px; pointer-events: none;">
                <div style="width: 100px; height: 100px; pointer-events: auto;"></div>
            </div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  // Click inside the inner div — child has pointer-events: auto.
  let path = body.hit_path((50.0, 50.0)).unwrap();
  assert_eq!(path, vec![0, 0], "child with auto is hittable through none parent");
  // Click inside the outer div but outside the child.
  let path = body.hit_path((150.0, 150.0)).unwrap();
  assert_eq!(path, Vec::<usize>::new(), "outer none div still transparent");
}

// ---------------------------------------------------------------------------
// user-select
// ---------------------------------------------------------------------------

#[test]
fn user_select_none_blocks_text_cursor_hit() {
  let mut lay = synthetic_text_layout();
  assert!(lay.hit_text_cursor((15.0, 24.0)).is_some(), "baseline: cursor works");
  lay.user_select = UserSelect::None;
  assert!(
    lay.hit_text_cursor((15.0, 24.0)).is_none(),
    "user-select:none blocks cursor"
  );
}

#[test]
fn user_select_none_inherits_through_cascade() {
  let tree = make(
    r#"<body style="margin: 0; width: 300px; height: 300px; user-select: none;">
            <div style="width: 100px; height: 100px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.user_select, UserSelect::None);
  assert_eq!(
    body.children[0].user_select,
    UserSelect::None,
    "user-select:none inherited to child"
  );
}
