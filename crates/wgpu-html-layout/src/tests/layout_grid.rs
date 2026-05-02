use crate::*;
use super::helpers::*;
// ---------------------------------------------------------------------------
// CSS Grid: tracks, placement, alignment, gaps, distribution
// ---------------------------------------------------------------------------

#[test]
fn grid_two_by_two_fixed_columns() {
  // 2×2 fixed grid: columns 100px / 100px, rows 50px / 50px.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px;
                          grid-template-rows: 50px 50px;">
            <div></div>
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert_eq!(kids[0].margin_rect.x, 0.0);
  assert_eq!(kids[0].margin_rect.y, 0.0);
  assert_eq!(kids[1].margin_rect.x, 100.0);
  assert_eq!(kids[1].margin_rect.y, 0.0);
  assert_eq!(kids[2].margin_rect.x, 0.0);
  assert_eq!(kids[2].margin_rect.y, 50.0);
  assert_eq!(kids[3].margin_rect.x, 100.0);
  assert_eq!(kids[3].margin_rect.y, 50.0);
}

#[test]
fn grid_fr_distributes_remaining_width() {
  // 1fr 2fr inside a 300px container → 100 / 200.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 1fr 2fr;
                          width: 300px;">
            <div style="height: 50px;"></div>
            <div style="height: 50px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert!((kids[0].margin_rect.w - 100.0).abs() < 0.05);
  assert!((kids[1].margin_rect.w - 200.0).abs() < 0.05);
  assert!((kids[1].margin_rect.x - 100.0).abs() < 0.05);
}

#[test]
fn grid_repeat_expands_track_list() {
  // `repeat(3, 80px)` → three identical 80px columns.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: repeat(3, 80px);">
            <div style="height: 40px;"></div>
            <div style="height: 40px;"></div>
            <div style="height: 40px;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![0.0, 80.0, 160.0]);
  for c in &body.children {
    assert_eq!(c.margin_rect.w, 80.0);
  }
}

#[test]
fn grid_explicit_placement_via_grid_column() {
  // `grid-column: 2 / 4` covers cols 2 and 3 (sum = 200px).
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px 100px;
                          grid-template-rows: 60px;">
            <div style="grid-column: 2 / 4;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let item = &body.children[0];
  assert_eq!(item.margin_rect.x, 100.0);
  assert!((item.margin_rect.w - 200.0).abs() < 0.05);
}

#[test]
fn grid_span_shorthand() {
  // `grid-column: span 2` from auto-placement at col 0 covers cols
  // 0 and 1.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px 100px;
                          grid-template-rows: 60px;">
            <div style="grid-column: span 2;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let item = &body.children[0];
  assert_eq!(item.margin_rect.x, 0.0);
  assert!((item.margin_rect.w - 200.0).abs() < 0.05);
}

#[test]
fn grid_auto_flow_row_packs_in_source_order() {
  // Three items on a 3-column track → all on row 1.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 80px 80px 80px;
                          grid-auto-rows: 50px;">
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let ys: Vec<f32> = body.children.iter().map(|c| c.margin_rect.y).collect();
  assert_eq!(ys, vec![0.0, 0.0, 0.0]);
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![0.0, 80.0, 160.0]);
}

#[test]
fn grid_auto_flow_column_packs_vertically() {
  // Three items on a 3-row track in column-major flow → all in col 1.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-auto-flow: column;
                          grid-template-rows: 50px 50px 50px;
                          grid-auto-columns: 80px;">
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
  assert_eq!(xs, vec![0.0, 0.0, 0.0]);
  let ys: Vec<f32> = body.children.iter().map(|c| c.margin_rect.y).collect();
  assert_eq!(ys, vec![0.0, 50.0, 100.0]);
}

#[test]
fn grid_implicit_rows_use_grid_auto_rows() {
  // Two items on a single explicit row → second wraps to an
  // implicit row sized at `grid-auto-rows: 70px`.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 50px;
                          grid-auto-rows: 70px;">
            <div></div>
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 0.0);
  // Second item lands in the implicit row at y = first row height (50).
  assert_eq!(body.children[1].margin_rect.y, 50.0);
}

#[test]
fn grid_row_gap_and_column_gap_separate_cells() {
  // `column-gap: 10px; row-gap: 5px;` with 100px tracks → second
  // column at x=110, second row at y=55.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px;
                          grid-template-rows: 50px 50px;
                          column-gap: 10px;
                          row-gap: 5px;">
            <div></div>
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let kids = &body.children;
  assert_eq!(kids[1].margin_rect.x, 110.0);
  assert_eq!(kids[2].margin_rect.y, 55.0);
}

#[test]
fn grid_align_self_end_anchors_item_to_cell_bottom() {
  // Cell is 50px tall, item is 20px → align-self: end pushes the
  // item to y=30.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 50px;">
            <div style="height: 20px; align-self: end;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.y, 30.0);
}

#[test]
fn grid_justify_self_center_horizontally() {
  // Cell 100px, item 40px → centered at x=30.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 50px;">
            <div style="width: 40px; height: 20px; justify-self: center;"></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.x, 30.0);
}

#[test]
fn grid_align_items_stretch_default_fills_cell_vertically() {
  // No explicit height → default `align-items: stretch` makes the
  // child fill the cell's row height.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 80px;">
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].border_rect.h, 80.0);
}

#[test]
fn grid_justify_content_center_centers_track_block() {
  // Two 80px columns in a 400px container with `justify-content:
  // center` → blocks start at (400 − 160) / 2 = 120.
  let tree = make(
    r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 80px 80px;
                          grid-template-rows: 40px;
                          justify-content: center;
                          width: 400px;">
            <div></div>
            <div></div>
        </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  assert_eq!(body.children[0].margin_rect.x, 120.0);
  assert_eq!(body.children[1].margin_rect.x, 200.0);
}
