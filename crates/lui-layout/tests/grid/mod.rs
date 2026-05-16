use lui_cascade::{cascade::InteractionState, media::MediaContext};
use lui_layout::{BoxKind, engine::layout_tree};

use crate::helpers::*;

// ============================================================================
// 23. Grid layout tests
// ============================================================================

#[test]
fn grid_container_detected_as_grid_kind() {
  let (doc, ctx) = flex_lt(r#"<div style="display:grid"><div>A</div></div>"#, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.kind, BoxKind::GridContainer);
}

#[test]
fn grid_fixed_columns() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:100px 200px; width:400px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 2);
  // A should be 100px wide, B should be 200px wide
  assert!(
    (grid.children[0].content.width - 100.0).abs() < 1.0,
    "first col 100px, got {}",
    grid.children[0].content.width
  );
  assert!(
    (grid.children[1].content.width - 200.0).abs() < 1.0,
    "second col 200px, got {}",
    grid.children[1].content.width
  );
}

#[test]
fn grid_fr_columns_distribute_space() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 2fr; width:300px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // 1fr=100, 2fr=200
  let ratio = grid.children[1].content.width / grid.children[0].content.width;
  assert!(
    (ratio - 2.0).abs() < 0.1,
    "2fr should be 2x wider than 1fr, ratio={}",
    ratio
  );
}

#[test]
fn grid_mixed_px_and_fr() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:100px 1fr; width:400px">
        <div style="height:50px">fixed</div><div style="height:50px">flex</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "fixed 100px");
  assert!(
    (grid.children[1].content.width - 300.0).abs() < 1.0,
    "1fr takes remaining 300px, got {}",
    grid.children[1].content.width
  );
}

#[test]
fn grid_auto_placement_wraps_rows() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr; width:200px">
        <div style="height:40px">A</div><div style="height:40px">B</div>
        <div style="height:40px">C</div><div style="height:40px">D</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 4);
  // A and B on row 0, C and D on row 1
  assert!(
    grid.children[2].content.y > grid.children[0].content.y,
    "C should be below A"
  );
  // A and C in col 0, B and D in col 1
  assert!(
    (grid.children[0].content.x - grid.children[2].content.x).abs() < 1.0,
    "A and C same column"
  );
  assert!(
    (grid.children[1].content.x - grid.children[3].content.x).abs() < 1.0,
    "B and D same column"
  );
}

#[test]
fn grid_column_gap() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:100px 100px; column-gap:20px; width:300px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a_end = grid.children[0].content.x + grid.children[0].content.width;
  let b_start = grid.children[1].content.x;
  let gap = b_start - a_end;
  assert!((gap - 20.0).abs() < 1.0, "column-gap:20px, got {}", gap);
}

#[test]
fn grid_row_gap() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr; row-gap:15px; width:200px">
        <div style="height:40px">A</div><div style="height:40px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a_bottom = grid.children[0].content.y + grid.children[0].content.height;
  let b_top = grid.children[1].content.y;
  let gap = b_top - a_bottom;
  assert!((gap - 15.0).abs() < 1.0, "row-gap:15px, got {}", gap);
}

#[test]
fn grid_explicit_row_heights() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr; grid-template-rows:60px 80px; width:200px">
        <div>A</div><div>B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Row heights from template
  let total_h = grid.content.height;
  assert!((total_h - 140.0).abs() < 1.0, "60+80=140px, got {}", total_h);
}

#[test]
fn grid_three_columns() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; width:300px">
        <div style="height:30px">A</div><div style="height:30px">B</div><div style="height:30px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Each column 100px
  assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "col0=100");
  assert!((grid.children[1].content.width - 100.0).abs() < 1.0, "col1=100");
  assert!((grid.children[2].content.width - 100.0).abs() < 1.0, "col2=100");
  // All on same row
  assert!(
    (grid.children[0].content.y - grid.children[2].content.y).abs() < 1.0,
    "same row"
  );
}

#[test]
fn grid_no_children_does_not_panic() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr; width:200px"></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.kind, BoxKind::GridContainer);
}

#[test]
fn grid_single_column_stacks_vertically() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; width:200px">
        <div style="height:40px">A</div><div style="height:40px">B</div><div style="height:40px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // No grid-template-columns → single implicit 1fr column → items stack vertically
  assert!(grid.children[1].content.y > grid.children[0].content.y, "B below A");
  assert!(grid.children[2].content.y > grid.children[1].content.y, "C below B");
}

#[test]
fn grid_gap_shorthand() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr; gap:10px; width:200px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
        <div style="height:30px">C</div><div style="height:30px">D</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Column gap between A and B
  let col_gap = grid.children[1].content.x - (grid.children[0].content.x + grid.children[0].content.width);
  assert!((col_gap - 10.0).abs() < 1.0, "column gap:10px, got {}", col_gap);
  // Row gap between A and C
  let row_gap = grid.children[2].content.y - (grid.children[0].content.y + grid.children[0].content.height);
  assert!((row_gap - 10.0).abs() < 1.0, "row gap:10px, got {}", row_gap);
}

// ── Grid advanced features ────────────────────────────────────────────

#[test]
fn grid_repeat_syntax() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:repeat(3, 1fr); width:300px">
        <div style="height:30px">A</div><div style="height:30px">B</div><div style="height:30px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 3);
  assert!(
    (grid.children[0].content.width - 100.0).abs() < 1.0,
    "repeat(3,1fr): 100px each, got {}",
    grid.children[0].content.width
  );
  assert!(
    (grid.children[0].content.y - grid.children[2].content.y).abs() < 1.0,
    "all on same row"
  );
}

#[test]
fn grid_repeat_mixed_tracks() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:repeat(2, 100px 1fr); width:500px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
        <div style="height:30px">C</div><div style="height:30px">D</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // repeat(2, 100px 1fr) → 100px 1fr 100px 1fr → 4 columns
  assert_eq!(grid.children.len(), 4);
  assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "col0=100px");
  assert!((grid.children[2].content.width - 100.0).abs() < 1.0, "col2=100px");
  // fr columns share remaining 300px equally
  assert!(
    (grid.children[1].content.width - 150.0).abs() < 1.0,
    "col1=1fr=150px, got {}",
    grid.children[1].content.width
  );
}

#[test]
fn grid_minmax_tracks() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:minmax(100px, 1fr) minmax(50px, 200px); width:400px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // minmax(100px, 1fr): min 100, takes remaining via fr
  // minmax(50px, 200px): fixed at 200px
  assert!(
    grid.children[0].content.width >= 99.0,
    "col0 >= 100px min, got {}",
    grid.children[0].content.width
  );
  assert!(
    (grid.children[1].content.width - 200.0).abs() < 1.0,
    "col1=200px, got {}",
    grid.children[1].content.width
  );
}

#[test]
fn grid_explicit_placement() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; width:300px">
        <div style="height:30px; grid-column-start:2">placed</div>
        <div style="height:30px">auto</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // "placed" should be in column 2 (index 1), not column 1
  let placed_x = grid.children[0].content.x - grid.content.x;
  assert!(
    placed_x > 90.0,
    "placed at col 2 should be offset ~100px, got {}",
    placed_x
  );
}

#[test]
fn grid_span_two_columns() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; width:300px">
        <div style="height:30px; grid-column-end:span 2">wide</div>
        <div style="height:30px">normal</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // "wide" spans 2 columns → should be ~200px
  assert!(
    grid.children[0].content.width > 190.0,
    "span 2 should be ~200px, got {}",
    grid.children[0].content.width
  );
}

#[test]
fn grid_span_two_rows() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr; width:200px">
        <div style="height:30px; grid-row-end:span 2">tall</div>
        <div style="height:30px">B</div>
        <div style="height:30px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // "tall" should span 2 rows, B and C beside/below it
  assert!(grid.children.len() >= 2);
}

#[test]
fn grid_align_items_center() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr; grid-template-rows:100px; align-items:center; width:200px">
        <div style="height:30px">centered</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child = &grid.children[0];
  let child_center = child.content.y + child.content.height / 2.0;
  let cell_center = grid.content.y + 50.0; // 100px row, center at 50
  assert!(
    (child_center - cell_center).abs() < 2.0,
    "align-items:center, child_center={}, cell_center={}",
    child_center,
    cell_center
  );
}

#[test]
fn grid_justify_items_center() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:200px; justify-items:center; width:300px">
        <div style="width:80px; height:30px">centered</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child = &grid.children[0];
  let child_center = child.content.x + child.content.width / 2.0;
  let cell_center = grid.content.x + 100.0; // 200px column, center at 100
  assert!(
    (child_center - cell_center).abs() < 2.0,
    "justify-items:center, child_center={}, cell_center={}",
    child_center,
    cell_center
  );
}

#[test]
fn grid_auto_rows_sets_implicit_row_height() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr; grid-auto-rows:60px; width:200px">
        <div>A</div><div>B</div><div>C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // 3 rows at 60px each = 180px total
  assert!(
    (grid.content.height - 180.0).abs() < 1.0,
    "3 rows * 60px = 180, got {}",
    grid.content.height
  );
}

#[test]
fn grid_percentage_tracks() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:25% 75%; width:400px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    (grid.children[0].content.width - 100.0).abs() < 1.0,
    "25% of 400=100, got {}",
    grid.children[0].content.width
  );
  assert!(
    (grid.children[1].content.width - 300.0).abs() < 1.0,
    "75% of 400=300, got {}",
    grid.children[1].content.width
  );
}

// ── grid-auto-flow: dense ────────────────────────────────────────────

#[test]
fn grid_auto_flow_dense_fills_gaps() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; grid-auto-flow:dense; width:300px">
        <div style="height:30px; grid-column-start:2">B</div>
        <div style="height:30px">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // B is explicitly placed at col 2. With dense, A should fill col 1 (the gap).
  let b_x = grid.children[0].content.x - grid.content.x;
  let a_x = grid.children[1].content.x - grid.content.x;
  assert!(
    a_x < b_x,
    "dense: A should fill gap at col 1 (a_x={}, b_x={})",
    a_x,
    b_x
  );
  assert!(a_x < 1.0, "dense: A should be at col 0, got offset {}", a_x);
}

// ── auto-fill / auto-fit ─────────────────────────────────────────────

#[test]
fn grid_repeat_auto_fill_computes_track_count() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:repeat(auto-fill, 100px); width:350px">
        <div style="height:30px">A</div><div style="height:30px">B</div><div style="height:30px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // 350px / 100px = 3 columns. All three items on same row.
  assert!(
    (grid.children[0].content.width - 100.0).abs() < 1.0,
    "auto-fill 100px tracks, got {}",
    grid.children[0].content.width
  );
  assert!(
    (grid.children[0].content.y - grid.children[2].content.y).abs() < 1.0,
    "all items should be on the same row"
  );
}

#[test]
fn grid_repeat_auto_fill_wraps_when_needed() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:repeat(auto-fill, 100px); width:250px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
        <div style="height:30px">C</div><div style="height:30px">D</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // 250px / 100px = 2 columns. A,B on row 0; C,D on row 1.
  assert!(
    grid.children[2].content.y > grid.children[0].content.y,
    "C should wrap to next row"
  );
}

#[test]
fn grid_repeat_auto_fit_collapses_empty_tracks() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:repeat(auto-fit, 100px); width:400px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // 400px / 100px = 4 tracks. Only 2 items. auto-fit collapses empty tracks 3 and 4.
  // With fr-like behavior on collapsed tracks, A and B should each get 100px.
  assert!(
    (grid.children[0].content.width - 100.0).abs() < 1.0,
    "auto-fit items should be 100px, got {}",
    grid.children[0].content.width
  );
}

// ── grid-auto-flow: column ───────────────────────────────────────────

#[test]
fn grid_auto_flow_column_places_items_column_first() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-rows:50px 50px; grid-auto-flow:column; grid-auto-columns:100px; width:400px">
        <div>A</div><div>B</div><div>C</div><div>D</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Column flow: A→row0/col0, B→row1/col0, C→row0/col1, D→row1/col1
  assert!(
    (grid.children[0].content.y - grid.children[2].content.y).abs() < 1.0,
    "A and C should be on same row (row 0)"
  );
  assert!(
    grid.children[2].content.x > grid.children[0].content.x,
    "C should be right of A (different column)"
  );
}

#[test]
fn grid_auto_columns_size_used_in_column_flow() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-rows:50px; grid-auto-flow:column; grid-auto-columns:80px; width:400px">
        <div>A</div><div>B</div><div>C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // All items in row 0, implicit columns at 80px each
  assert!(
    (grid.children[1].content.width - 80.0).abs() < 1.0,
    "auto-columns:80px, got {}",
    grid.children[1].content.width
  );
}

// ── Named grid lines ─────────────────────────────────────────────────

#[test]
fn grid_named_lines_placement() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:[start] 100px [mid] 200px [end]; width:400px">
        <div style="height:30px; grid-column-start:mid">B</div>
        <div style="height:30px">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let b_x = grid.children[0].content.x - grid.content.x;
  assert!(
    b_x >= 99.0,
    "grid-column-start:mid should place at col 1 (100px offset), got {}",
    b_x
  );
}

// ── grid-template-areas ──────────────────────────────────────────────

#[test]
fn grid_template_areas_basic() {
  let (doc, ctx) = flex_lt(
    r#"<div style='display:grid; grid-template-areas:"header header" "sidebar main"; grid-template-columns:100px 200px; grid-template-rows:40px 60px; width:400px'>
        <div style="grid-area:header; height:40px">H</div>
        <div style="grid-area:sidebar; height:60px">S</div>
        <div style="grid-area:main; height:60px">M</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let header = &grid.children[0];
  let sidebar = &grid.children[1];
  let main = &grid.children[2];
  // Header should span both columns (300px total)
  assert!(
    header.content.width >= 290.0,
    "header should span 2 cols (300px), got {}",
    header.content.width
  );
  // Sidebar at col 0, row 1
  let sidebar_x = sidebar.content.x - grid.content.x;
  assert!(sidebar_x < 1.0, "sidebar at col 0, got offset {}", sidebar_x);
  let sidebar_y = sidebar.content.y - grid.content.y;
  assert!(sidebar_y >= 39.0, "sidebar at row 1 (40px offset), got {}", sidebar_y);
  // Main at col 1, row 1
  let main_x = main.content.x - grid.content.x;
  assert!(main_x >= 99.0, "main at col 1 (100px offset), got {}", main_x);
}

// ── grid-area shorthand ──────────────────────────────────────────────

#[test]
fn grid_area_shorthand_four_values() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; grid-template-rows:50px 50px; width:300px">
        <div style="grid-area:1/2/3/4; height:30px">span</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let item = &grid.children[0];
  // grid-area: 1/2/3/4 → row 0-1, col 1-2 (line numbers are 1-based)
  let x_offset = item.content.x - grid.content.x;
  assert!(x_offset >= 99.0, "should start at col 1 (100px), got {}", x_offset);
  assert!(
    item.content.width >= 190.0,
    "should span 2 cols (200px), got {}",
    item.content.width
  );
}

#[test]
fn grid_explicit_row_heights_applied_to_items() {
  // grid-template-rows: 48px 72px — items should stretch to fill row height
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:90px 120px 160px; grid-template-rows:48px 72px; gap:8px">
      <div>A</div><div>B</div><div>C</div>
      <div>D</div><div>E</div><div>F</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 6);
  // Row 1 items (indices 0,1,2) should be 48px outer height
  for i in 0..3 {
    assert!(
      (grid.children[i].outer_height() - 48.0).abs() < 1.0,
      "row 1 item {} outer height should be 48, got {}",
      i, grid.children[i].outer_height(),
    );
  }
  // Row 2 items (indices 3,4,5) should be 72px outer height
  for i in 3..6 {
    assert!(
      (grid.children[i].outer_height() - 72.0).abs() < 1.0,
      "row 2 item {} outer height should be 72, got {}",
      i, grid.children[i].outer_height(),
    );
  }
}

#[test]
fn grid_minmax_fr_columns_distribute_proportionally() {
  // minmax(80px,1fr) minmax(100px,2fr) 120px in a 500px container
  // Available for fr = 500 - 120 - 2*8(gap) = 364
  // 1fr = 364/3 ≈ 121.3, 2fr = 364*2/3 ≈ 242.7
  // Both exceed their minimums (80, 100), so fr sizing applies.
  // Col2 should be ~2x col1.
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:minmax(80px,1fr) minmax(100px,2fr) 120px; gap:8px; width:500px">
      <div>A</div><div>B</div><div>C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 3);

  let w0 = grid.children[0].content.width;
  let w1 = grid.children[1].content.width;
  let w2 = grid.children[2].content.width;

  // Col3 should be exactly 120px
  assert!(
    (w2 - 120.0).abs() < 1.0,
    "fixed col should be 120, got {}", w2,
  );
  // Col2 should be roughly 2x col1
  let ratio = w1 / w0;
  assert!(
    (ratio - 2.0).abs() < 0.3,
    "col2/col1 ratio should be ~2.0, got {} (w0={}, w1={})",
    ratio, w0, w1,
  );
}

#[test]
fn grid_auto_columns_sized_by_content() {
  // auto 1fr auto — auto columns should size to content, 1fr gets remaining space
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:auto 1fr auto; gap:8px; width:500px">
      <div>Label</div><div>Middle fills</div><div>End</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 3);

  let w0 = grid.children[0].outer_width();
  let w1 = grid.children[1].outer_width();
  let w2 = grid.children[2].outer_width();

  // Auto columns should have non-trivial width (content-based)
  assert!(
    w0 > 10.0,
    "first auto column should be content-sized, got {}",
    w0,
  );
  assert!(
    w2 > 10.0,
    "last auto column should be content-sized, got {}",
    w2,
  );
  // 1fr column should be the widest
  assert!(
    w1 > w0 && w1 > w2,
    "1fr column ({}) should be wider than auto columns ({}, {})",
    w1, w0, w2,
  );
}

#[test]
fn grid_justify_content_center() {
  // 3 fixed 80px columns in a 420px container — justify-content:center should
  // center the track group horizontally.
  // Total tracks = 80*3 + 8*2(gap) = 256. Free = 420 - 256 = 164. Offset = 82.
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:80px 80px 80px; gap:8px; width:420px; justify-content:center">
      <div>A</div><div>B</div><div>C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let first = &grid.children[0];
  let last = &grid.children[2];
  let left_offset = first.content.x - grid.content.x;
  let right_edge = last.content.x + last.content.width;
  let right_offset = (grid.content.x + grid.content.width) - right_edge;
  assert!(
    (left_offset - right_offset).abs() < 2.0,
    "justify-content:center — left ({}) and right ({}) offsets should be equal",
    left_offset, right_offset,
  );
  assert!(
    left_offset > 10.0,
    "justify-content:center — should have offset from left, got {}",
    left_offset,
  );
}

#[test]
fn grid_justify_content_space_between() {
  // 3 fixed 80px columns in 420px — space-between puts first at start, last at end
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:80px 80px 80px; gap:0px; width:420px; justify-content:space-between">
      <div>A</div><div>B</div><div>C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let first = &grid.children[0];
  let last = &grid.children[2];
  let left_offset = first.content.x - grid.content.x;
  let right_edge = last.content.x + last.content.width;
  let right_offset = (grid.content.x + grid.content.width) - right_edge;
  assert!(
    left_offset.abs() < 1.0,
    "space-between — first item should be at start, offset={}",
    left_offset,
  );
  assert!(
    right_offset.abs() < 1.0,
    "space-between — last item should be at end, offset={}",
    right_offset,
  );
}

#[test]
fn grid_repeat_auto_fill_column_count() {
  // repeat(auto-fill, 96px) in 500px with gap:8px
  // Columns: (500+8)/(96+8) = 4.88 → 4 columns
  // 9 items → 4 cols × 3 rows, items wrap to row 2 at index 4
  let (doc, ctx) = flex_lt(
    r#"<div style="display:grid; grid-template-columns:repeat(auto-fill,96px); grid-auto-rows:50px; gap:8px; width:500px">
      <div>1</div><div>2</div><div>3</div><div>4</div>
      <div>5</div><div>6</div><div>7</div><div>8</div><div>9</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(grid.children.len(), 9);
  // Item 5 (index 4) should be on row 2 — its y should be below item 1
  let row1_y = grid.children[0].content.y;
  let row2_y = grid.children[4].content.y;
  assert!(
    row2_y > row1_y + 40.0,
    "item 5 should be on row 2 (y={}) below row 1 (y={})",
    row2_y, row1_y,
  );
  // Item 5 should be in the first column (same x as item 1)
  let col1_x = grid.children[0].content.x;
  let item5_x = grid.children[4].content.x;
  assert!(
    (item5_x - col1_x).abs() < 1.0,
    "item 5 should be in col 1 (x={}), but is at x={}",
    col1_x, item5_x,
  );
}
