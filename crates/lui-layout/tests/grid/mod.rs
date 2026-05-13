use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::{BoxKind, engine::layout_tree};
use crate::helpers::*;

// ============================================================================
// 23. Grid layout tests
// ============================================================================

#[test]
fn grid_container_detected_as_grid_kind() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid"><div>A</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(grid.kind, BoxKind::GridContainer);
}

#[test]
fn grid_fixed_columns() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:100px 200px; width:400px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(grid.children.len(), 2);
    // A should be 100px wide, B should be 200px wide
    assert!((grid.children[0].content.width - 100.0).abs() < 1.0,
        "first col 100px, got {}", grid.children[0].content.width);
    assert!((grid.children[1].content.width - 200.0).abs() < 1.0,
        "second col 200px, got {}", grid.children[1].content.width);
}

#[test]
fn grid_fr_columns_distribute_space() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 2fr; width:300px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // 1fr=100, 2fr=200
    let ratio = grid.children[1].content.width / grid.children[0].content.width;
    assert!((ratio - 2.0).abs() < 0.1, "2fr should be 2x wider than 1fr, ratio={}", ratio);
}

#[test]
fn grid_mixed_px_and_fr() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:100px 1fr; width:400px">
        <div style="height:50px">fixed</div><div style="height:50px">flex</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "fixed 100px");
    assert!((grid.children[1].content.width - 300.0).abs() < 1.0,
        "1fr takes remaining 300px, got {}", grid.children[1].content.width);
}

#[test]
fn grid_auto_placement_wraps_rows() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr; width:200px">
        <div style="height:40px">A</div><div style="height:40px">B</div>
        <div style="height:40px">C</div><div style="height:40px">D</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(grid.children.len(), 4);
    // A and B on row 0, C and D on row 1
    assert!(grid.children[2].content.y > grid.children[0].content.y,
        "C should be below A");
    // A and C in col 0, B and D in col 1
    assert!((grid.children[0].content.x - grid.children[2].content.x).abs() < 1.0,
        "A and C same column");
    assert!((grid.children[1].content.x - grid.children[3].content.x).abs() < 1.0,
        "B and D same column");
}

#[test]
fn grid_column_gap() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:100px 100px; column-gap:20px; width:300px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
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
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr; row-gap:15px; width:200px">
        <div style="height:40px">A</div><div style="height:40px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
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
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr; grid-template-rows:60px 80px; width:200px">
        <div>A</div><div>B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Row heights from template
    let total_h = grid.content.height;
    assert!((total_h - 140.0).abs() < 1.0, "60+80=140px, got {}", total_h);
}

#[test]
fn grid_three_columns() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; width:300px">
        <div style="height:30px">A</div><div style="height:30px">B</div><div style="height:30px">C</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Each column 100px
    assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "col0=100");
    assert!((grid.children[1].content.width - 100.0).abs() < 1.0, "col1=100");
    assert!((grid.children[2].content.width - 100.0).abs() < 1.0, "col2=100");
    // All on same row
    assert!((grid.children[0].content.y - grid.children[2].content.y).abs() < 1.0, "same row");
}

#[test]
fn grid_no_children_does_not_panic() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr; width:200px"></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(grid.kind, BoxKind::GridContainer);
}

#[test]
fn grid_single_column_stacks_vertically() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; width:200px">
        <div style="height:40px">A</div><div style="height:40px">B</div><div style="height:40px">C</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // No grid-template-columns → single implicit 1fr column → items stack vertically
    assert!(grid.children[1].content.y > grid.children[0].content.y, "B below A");
    assert!(grid.children[2].content.y > grid.children[1].content.y, "C below B");
}

#[test]
fn grid_gap_shorthand() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr; gap:10px; width:200px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
        <div style="height:30px">C</div><div style="height:30px">D</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
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
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:repeat(3, 1fr); width:300px">
        <div style="height:30px">A</div><div style="height:30px">B</div><div style="height:30px">C</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(grid.children.len(), 3);
    assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "repeat(3,1fr): 100px each, got {}", grid.children[0].content.width);
    assert!((grid.children[0].content.y - grid.children[2].content.y).abs() < 1.0, "all on same row");
}

#[test]
fn grid_repeat_mixed_tracks() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:repeat(2, 100px 1fr); width:500px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
        <div style="height:30px">C</div><div style="height:30px">D</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // repeat(2, 100px 1fr) → 100px 1fr 100px 1fr → 4 columns
    assert_eq!(grid.children.len(), 4);
    assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "col0=100px");
    assert!((grid.children[2].content.width - 100.0).abs() < 1.0, "col2=100px");
    // fr columns share remaining 300px equally
    assert!((grid.children[1].content.width - 150.0).abs() < 1.0,
        "col1=1fr=150px, got {}", grid.children[1].content.width);
}

#[test]
fn grid_minmax_tracks() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:minmax(100px, 1fr) minmax(50px, 200px); width:400px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // minmax(100px, 1fr): min 100, takes remaining via fr
    // minmax(50px, 200px): fixed at 200px
    assert!(grid.children[0].content.width >= 99.0, "col0 >= 100px min, got {}", grid.children[0].content.width);
    assert!((grid.children[1].content.width - 200.0).abs() < 1.0, "col1=200px, got {}", grid.children[1].content.width);
}

#[test]
fn grid_explicit_placement() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; width:300px">
        <div style="height:30px; grid-column-start:2">placed</div>
        <div style="height:30px">auto</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // "placed" should be in column 2 (index 1), not column 1
    let placed_x = grid.children[0].content.x - grid.content.x;
    assert!(placed_x > 90.0, "placed at col 2 should be offset ~100px, got {}", placed_x);
}

#[test]
fn grid_span_two_columns() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr 1fr; width:300px">
        <div style="height:30px; grid-column-end:span 2">wide</div>
        <div style="height:30px">normal</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // "wide" spans 2 columns → should be ~200px
    assert!(grid.children[0].content.width > 190.0,
        "span 2 should be ~200px, got {}", grid.children[0].content.width);
}

#[test]
fn grid_span_two_rows() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr 1fr; width:200px">
        <div style="height:30px; grid-row-end:span 2">tall</div>
        <div style="height:30px">B</div>
        <div style="height:30px">C</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // "tall" should span 2 rows, B and C beside/below it
    assert!(grid.children.len() >= 2);
}

#[test]
fn grid_align_items_center() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr; grid-template-rows:100px; align-items:center; width:200px">
        <div style="height:30px">centered</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child = &grid.children[0];
    let child_center = child.content.y + child.content.height / 2.0;
    let cell_center = grid.content.y + 50.0; // 100px row, center at 50
    assert!((child_center - cell_center).abs() < 2.0,
        "align-items:center, child_center={}, cell_center={}", child_center, cell_center);
}

#[test]
fn grid_justify_items_center() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:200px; justify-items:center; width:300px">
        <div style="width:80px; height:30px">centered</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child = &grid.children[0];
    let child_center = child.content.x + child.content.width / 2.0;
    let cell_center = grid.content.x + 100.0; // 200px column, center at 100
    assert!((child_center - cell_center).abs() < 2.0,
        "justify-items:center, child_center={}, cell_center={}", child_center, cell_center);
}

#[test]
fn grid_auto_rows_sets_implicit_row_height() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:1fr; grid-auto-rows:60px; width:200px">
        <div>A</div><div>B</div><div>C</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // 3 rows at 60px each = 180px total
    assert!((grid.content.height - 180.0).abs() < 1.0,
        "3 rows * 60px = 180, got {}", grid.content.height);
}

#[test]
fn grid_percentage_tracks() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid; grid-template-columns:25% 75%; width:400px">
        <div style="height:30px">A</div><div style="height:30px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((grid.children[0].content.width - 100.0).abs() < 1.0, "25% of 400=100, got {}", grid.children[0].content.width);
    assert!((grid.children[1].content.width - 300.0).abs() < 1.0, "75% of 400=300, got {}", grid.children[1].content.width);
}
