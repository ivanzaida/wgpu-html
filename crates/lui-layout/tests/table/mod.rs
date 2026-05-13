use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::{BoxKind, LayoutBox, engine::layout_tree};
use crate::helpers::*;

fn find_caption<'a>(t: &'a LayoutBox<'a>) -> &'a LayoutBox<'a> {
    t.children.iter().find(|c| c.kind == BoxKind::TableCaption).unwrap()
}

fn find_last_row<'a>(t: &'a LayoutBox<'a>) -> &'a LayoutBox<'a> {
    t.children.iter().rev()
        .find(|c| matches!(c.kind, BoxKind::TableRow | BoxKind::TableRowGroup))
        .unwrap()
}

macro_rules! table_test {
    ($name:ident, $html:expr, $vw:expr, |$tbl:ident| $body:block) => {
        #[test]
        fn $name() {
            let (doc, ctx) = flex_lt($html, $vw);
            let media = MediaContext::default();
            let interaction = InteractionState::default();
            let styled = ctx.cascade(&doc.root, &media, &interaction);
            let lt = layout_tree(&styled, $vw, 600.0);
            let $tbl = find_by_tag(&lt.root, "table").unwrap();
            $body
        }
    };
    ($name:ident, $html:expr, $vw:expr, |$lt:ident, $tbl:ident| $body:block) => {
        #[test]
        fn $name() {
            let (doc, ctx) = flex_lt($html, $vw);
            let media = MediaContext::default();
            let interaction = InteractionState::default();
            let styled = ctx.cascade(&doc.root, &media, &interaction);
            let $lt = layout_tree(&styled, $vw, 600.0);
            let $tbl = find_by_tag(&$lt.root, "table").unwrap();
            $body
        }
    };
}

// ============================================================================
// Basic table recognition
// ============================================================================

table_test!(table_element_gets_table_kind,
    r#"<table><tr><td>A</td></tr></table>"#, 800.0, |t| {
    assert_eq!(t.kind, BoxKind::Table);
});

#[test]
fn display_table_gets_table_kind() {
    let html = r#"<div style="display:table"><div style="display:table-row"><div style="display:table-cell">A</div></div></div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    assert_eq!(body.children[0].kind, BoxKind::Table);
}

table_test!(tr_gets_table_row_kind,
    r#"<table><tr><td>A</td></tr></table>"#, 800.0, |t| {
    assert_eq!(t.children[0].kind, BoxKind::TableRow);
});

table_test!(td_gets_table_cell_kind,
    r#"<table><tr><td>A</td></tr></table>"#, 800.0, |t| {
    let row = &t.children[0];
    assert_eq!(row.children[0].kind, BoxKind::TableCell);
});

table_test!(thead_gets_row_group_kind,
    r#"<table><thead><tr><td>A</td></tr></thead></table>"#, 800.0, |t| {
    assert_eq!(t.children[0].kind, BoxKind::TableRowGroup);
});

table_test!(tbody_gets_row_group_kind,
    r#"<table><tbody><tr><td>A</td></tr></tbody></table>"#, 800.0, |t| {
    assert_eq!(t.children[0].kind, BoxKind::TableRowGroup);
});

table_test!(tfoot_gets_row_group_kind,
    r#"<table><tfoot><tr><td>A</td></tr></tfoot></table>"#, 800.0, |t| {
    assert_eq!(t.children[0].kind, BoxKind::TableRowGroup);
});

table_test!(caption_gets_caption_kind,
    r#"<table><caption>Title</caption><tr><td>A</td></tr></table>"#, 800.0, |t| {
    assert_eq!(t.children[0].kind, BoxKind::TableCaption);
});

// ============================================================================
// Equal column sizing (basic, no explicit widths)
// ============================================================================

table_test!(two_cols_equal_width,
    r#"<table style="width:200px"><tr>
        <td style="height:30px">A</td><td style="height:30px">B</td>
    </tr></table>"#, 800.0, |t| {
    let row = &t.children[0];
    assert!((row.children[0].content.width - 100.0).abs() < 1.0,
        "col0 should be 100px, got {}", row.children[0].content.width);
    assert!((row.children[1].content.width - 100.0).abs() < 1.0,
        "col1 should be 100px, got {}", row.children[1].content.width);
});

table_test!(three_cols_equal_width,
    r#"<table style="width:300px"><tr>
        <td style="height:30px">A</td><td style="height:30px">B</td><td style="height:30px">C</td>
    </tr></table>"#, 800.0, |t| {
    let row = &t.children[0];
    for (i, cell) in row.children.iter().enumerate() {
        assert!((cell.content.width - 100.0).abs() < 1.0,
            "col{} should be 100px, got {}", i, cell.content.width);
    }
});

// ============================================================================
// Row height = tallest cell
// ============================================================================

table_test!(row_height_is_tallest_cell,
    r#"<table style="width:200px"><tr>
        <td style="height:30px">A</td><td style="height:60px">B</td>
    </tr></table>"#, 800.0, |t| {
    let row = &t.children[0];
    assert!((row.content.height - 60.0).abs() < 1.0,
        "row height should be 60px, got {}", row.content.height);
});

// ============================================================================
// Multiple rows stack vertically
// ============================================================================

table_test!(two_rows_stack_vertically,
    r#"<table style="width:200px">
        <tr><td style="height:40px">A</td><td style="height:40px">B</td></tr>
        <tr><td style="height:30px">C</td><td style="height:30px">D</td></tr>
    </table>"#, 800.0, |t| {
    let row0 = &t.children[0];
    let row1 = &t.children[1];
    assert!(row1.content.y > row0.content.y, "row1 should be below row0");
    let gap = row1.content.y - row0.content.y;
    assert!((gap - 40.0).abs() < 1.0,
        "row1 should start 40px below row0, got {}", gap);
});

// ============================================================================
// Cells in same column share x position across rows
// ============================================================================

table_test!(cells_aligned_in_columns,
    r#"<table style="width:200px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
        <tr><td style="height:30px">C</td><td style="height:30px">D</td></tr>
    </table>"#, 800.0, |t| {
    let a = &t.children[0].children[0];
    let c = &t.children[1].children[0];
    assert!((a.content.x - c.content.x).abs() < 1.0, "A and C should share x");
    let b = &t.children[0].children[1];
    let d = &t.children[1].children[1];
    assert!((b.content.x - d.content.x).abs() < 1.0, "B and D should share x");
});

// ============================================================================
// Table width fills available when no explicit width
// ============================================================================

table_test!(table_fills_available_width,
    r#"<table><tr><td style="height:30px">A</td></tr></table>"#, 600.0, |t| {
    assert!((t.content.width - 600.0).abs() < 1.0,
        "table should fill 600px viewport, got {}", t.content.width);
});

// ============================================================================
// border-spacing
// ============================================================================

table_test!(border_spacing_horizontal,
    r#"<table style="width:220px; border-spacing:10px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children[0];
    let a = &row.children[0];
    let b = &row.children[1];
    let gap = b.content.x - (a.content.x + a.content.width);
    assert!((gap - 10.0).abs() < 2.0,
        "horizontal spacing between cells should be 10px, got {}", gap);
});

table_test!(border_spacing_vertical,
    r#"<table style="width:200px; border-spacing:10px">
        <tr><td style="height:30px">A</td></tr>
        <tr><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row0 = &t.children[0];
    let row1 = &t.children[1];
    let gap = row1.content.y - (row0.content.y + row0.content.height);
    assert!((gap - 10.0).abs() < 2.0,
        "vertical spacing between rows should be 10px, got {}", gap);
});

table_test!(border_spacing_outer_edges,
    r#"<table style="width:220px; border-spacing:10px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children[0];
    let a = &row.children[0];
    let left_offset = a.content.x - t.content.x;
    assert!((left_offset - 10.0).abs() < 2.0,
        "first cell should be offset by border-spacing from table edge, got {}", left_offset);
});

table_test!(border_spacing_two_values,
    r#"<table style="width:200px; border-spacing:5px 15px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
        <tr><td style="height:30px">C</td><td style="height:30px">D</td></tr>
    </table>"#, 800.0, |t| {
    let row0 = &t.children[0];
    let row1 = &t.children[1];
    let a = &row0.children[0];
    let b = &row0.children[1];
    let h_gap = b.content.x - (a.content.x + a.content.width);
    assert!((h_gap - 5.0).abs() < 2.0,
        "horizontal spacing should be 5px, got {}", h_gap);
    let v_gap = row1.content.y - (row0.content.y + row0.content.height);
    assert!((v_gap - 15.0).abs() < 2.0,
        "vertical spacing should be 15px, got {}", v_gap);
});

// ============================================================================
// border-collapse
// ============================================================================

table_test!(border_collapse_no_spacing,
    r#"<table style="width:200px; border-collapse:collapse; border-spacing:10px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children[0];
    let a = &row.children[0];
    let b = &row.children[1];
    let gap = b.content.x - (a.content.x + a.content.width);
    assert!(gap.abs() < 2.0,
        "collapsed borders should have no spacing, got gap={}", gap);
});

// ============================================================================
// Fixed table layout
// ============================================================================

table_test!(fixed_layout_uses_first_row_widths,
    r#"<table style="width:300px; table-layout:fixed">
        <tr><td style="width:100px; height:30px">A</td><td style="height:30px">B</td></tr>
        <tr><td style="height:30px">C</td><td style="height:30px">D</td></tr>
    </table>"#, 800.0, |t| {
    let row0 = &t.children[0];
    assert!((row0.children[0].content.width - 100.0).abs() < 1.0,
        "first col should be 100px, got {}", row0.children[0].content.width);
    assert!((row0.children[1].content.width - 200.0).abs() < 1.0,
        "second col should get remaining 200px, got {}", row0.children[1].content.width);
    let row1 = &t.children[1];
    assert!((row1.children[0].content.width - 100.0).abs() < 1.0,
        "second row first col should also be 100px");
    assert!((row1.children[1].content.width - 200.0).abs() < 1.0,
        "second row second col should also be 200px");
});

// ============================================================================
// colspan
// ============================================================================

table_test!(colspan_spans_multiple_columns,
    r#"<table style="width:300px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td><td style="height:30px">C</td></tr>
        <tr><td colspan="2" style="height:30px">AB</td><td style="height:30px">C2</td></tr>
    </table>"#, 800.0, |t| {
    let row1 = &t.children[1];
    let spanned = &row1.children[0];
    assert!((spanned.content.width - 200.0).abs() < 2.0,
        "colspan=2 cell should be ~200px, got {}", spanned.content.width);
});

table_test!(colspan_cell_position_correct,
    r#"<table style="width:300px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td><td style="height:30px">C</td></tr>
        <tr><td colspan="2" style="height:30px">AB</td><td style="height:30px">C2</td></tr>
    </table>"#, 800.0, |t| {
    let row0 = &t.children[0];
    let row1 = &t.children[1];
    let c_top = &row0.children[2];
    let c_bot = &row1.children[1];
    assert!((c_top.content.x - c_bot.content.x).abs() < 2.0,
        "C and C2 should share x position");
});

// ============================================================================
// rowspan
// ============================================================================

table_test!(rowspan_spans_multiple_rows,
    r#"<table style="width:200px">
        <tr><td rowspan="2" style="height:60px">A</td><td style="height:30px">B</td></tr>
        <tr><td style="height:30px">C</td></tr>
    </table>"#, 800.0, |t| {
    let row0 = &t.children[0];
    let a = &row0.children[0];
    assert!((a.content.height - 60.0).abs() < 2.0,
        "rowspan=2 cell height should be 60px, got {}", a.content.height);
    let row1 = &t.children[1];
    let c = &row1.children[0];
    let b = &row0.children[1];
    assert!((c.content.x - b.content.x).abs() < 2.0,
        "C should be in same column as B (col 1)");
});

// ============================================================================
// Row groups (thead, tbody, tfoot)
// ============================================================================

table_test!(row_groups_contain_rows,
    r#"<table style="width:200px">
        <thead><tr><td style="height:30px">H</td></tr></thead>
        <tbody><tr><td style="height:30px">B</td></tr></tbody>
        <tfoot><tr><td style="height:30px">F</td></tr></tfoot>
    </table>"#, 800.0, |t| {
    assert_eq!(t.children.len(), 3, "table should have 3 row groups");
    assert_eq!(t.children[0].kind, BoxKind::TableRowGroup);
    assert_eq!(t.children[0].children[0].kind, BoxKind::TableRow);
});

table_test!(row_groups_stack_vertically,
    r#"<table style="width:200px">
        <thead><tr><td style="height:30px">H</td></tr></thead>
        <tbody><tr><td style="height:40px">B</td></tr></tbody>
    </table>"#, 800.0, |t| {
    let thead = &t.children[0];
    let tbody = &t.children[1];
    assert!(tbody.content.y > thead.content.y, "tbody should be below thead");
});

// ============================================================================
// Caption
// ============================================================================

table_test!(caption_above_table_content,
    r#"<table style="width:200px">
        <caption style="height:25px">Title</caption>
        <tr><td style="height:30px">A</td></tr>
    </table>"#, 800.0, |t| {
    let caption = &t.children[0];
    assert_eq!(caption.kind, BoxKind::TableCaption);
    let first_row_y = t.children[1].content.y;
    assert!(first_row_y >= caption.content.y + caption.content.height - 1.0,
        "first row should be below caption");
});

table_test!(caption_bottom,
    r#"<table style="width:200px">
        <caption style="height:25px; caption-side:bottom">Title</caption>
        <tr><td style="height:30px">A</td></tr>
    </table>"#, 800.0, |t| {
    let caption = find_caption(t);
    let last_row = find_last_row(t);
    assert!(caption.content.y >= last_row.content.y + last_row.content.height - 1.0,
        "bottom caption should be below rows");
});

// ============================================================================
// Table total height
// ============================================================================

table_test!(table_total_height_includes_rows,
    r#"<table style="width:200px">
        <tr><td style="height:40px">A</td></tr>
        <tr><td style="height:60px">B</td></tr>
    </table>"#, 800.0, |t| {
    assert!((t.content.height - 100.0).abs() < 2.0,
        "table height should be 100px (40+60), got {}", t.content.height);
});

// ============================================================================
// Cell padding
// ============================================================================

table_test!(cell_padding_reduces_inner_content,
    r#"<table style="width:200px"><tr>
        <td style="height:30px; padding:5px">A</td>
        <td style="height:30px">B</td>
    </tr></table>"#, 800.0, |t| {
    let row = &t.children[0];
    let a = &row.children[0];
    assert!((a.padding.left - 5.0).abs() < 1.0, "cell should have 5px padding");
});

// ============================================================================
// Explicit cell widths in auto layout
// ============================================================================

table_test!(explicit_cell_width_in_auto_layout,
    r#"<table style="width:300px">
        <tr><td style="width:150px; height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children[0];
    assert!((row.children[0].content.width - 150.0).abs() < 2.0,
        "explicit width cell should be 150px, got {}", row.children[0].content.width);
    assert!((row.children[1].content.width - 150.0).abs() < 2.0,
        "remaining cell should get 150px, got {}", row.children[1].content.width);
});

// ============================================================================
// Table margin
// ============================================================================

table_test!(table_margin_offsets_position,
    r#"<table style="width:200px; margin:20px">
        <tr><td style="height:30px">A</td></tr>
    </table>"#, 800.0, |t| {
    assert!((t.margin.top - 20.0).abs() < 1.0, "margin-top should be 20px");
    assert!((t.margin.left - 20.0).abs() < 1.0, "margin-left should be 20px");
});

// ============================================================================
// Empty table
// ============================================================================

table_test!(empty_table_has_zero_height,
    r#"<table style="width:200px"></table>"#, 800.0, |t| {
    assert!((t.content.height).abs() < 1.0,
        "empty table should have ~0 height, got {}", t.content.height);
});

// ============================================================================
// <colgroup> / <col> column width hints
// ============================================================================

table_test!(col_sets_column_width,
    r#"<table style="width:300px">
        <col style="width:100px"><col style="width:200px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children.iter()
        .find(|c| c.kind == BoxKind::TableRow).unwrap();
    assert!((row.children[0].content.width - 100.0).abs() < 2.0,
        "col 0 should be 100px from <col>, got {}", row.children[0].content.width);
    assert!((row.children[1].content.width - 200.0).abs() < 2.0,
        "col 1 should be 200px from <col>, got {}", row.children[1].content.width);
});

table_test!(colgroup_with_col_children,
    r#"<table style="width:300px">
        <colgroup><col style="width:100px"><col style="width:200px"></colgroup>
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children.iter()
        .find(|c| c.kind == BoxKind::TableRow).unwrap();
    assert!((row.children[0].content.width - 100.0).abs() < 2.0,
        "col 0 should be 100px, got {}", row.children[0].content.width);
    assert!((row.children[1].content.width - 200.0).abs() < 2.0,
        "col 1 should be 200px, got {}", row.children[1].content.width);
});

table_test!(colgroup_width_without_col_children,
    r#"<table style="width:200px">
        <colgroup span="2" style="width:100px"></colgroup>
        <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children.iter()
        .find(|c| c.kind == BoxKind::TableRow).unwrap();
    assert!((row.children[0].content.width - 100.0).abs() < 2.0,
        "col 0 should be 100px from colgroup, got {}", row.children[0].content.width);
    assert!((row.children[1].content.width - 100.0).abs() < 2.0,
        "col 1 should be 100px from colgroup, got {}", row.children[1].content.width);
});

table_test!(col_span_attribute,
    r#"<table style="width:300px">
        <col span="2" style="width:100px"><col style="width:100px">
        <tr><td style="height:30px">A</td><td style="height:30px">B</td><td style="height:30px">C</td></tr>
    </table>"#, 800.0, |t| {
    let row = &t.children.iter()
        .find(|c| c.kind == BoxKind::TableRow).unwrap();
    assert!((row.children[0].content.width - 100.0).abs() < 2.0,
        "col 0 should be 100px, got {}", row.children[0].content.width);
    assert!((row.children[1].content.width - 100.0).abs() < 2.0,
        "col 1 should be 100px (span=2), got {}", row.children[1].content.width);
    assert!((row.children[2].content.width - 100.0).abs() < 2.0,
        "col 2 should be 100px, got {}", row.children[2].content.width);
});

// ============================================================================
// Anonymous table wrappers
// ============================================================================

#[test]
fn orphaned_td_gets_anonymous_table_wrapper() {
    let html = r#"<div><td style="height:30px">A</td></div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // The td should be wrapped in anonymous row → anonymous table
    let anon_table = &div.children[0];
    assert_eq!(anon_table.kind, BoxKind::Table,
        "orphaned td should be wrapped in anonymous table, got {:?}", anon_table.kind);
}

#[test]
fn orphaned_tr_gets_anonymous_table_wrapper() {
    let html = r#"<div><tr><td style="height:30px">A</td></tr></div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let anon_table = &div.children[0];
    assert_eq!(anon_table.kind, BoxKind::Table,
        "orphaned tr should be wrapped in anonymous table, got {:?}", anon_table.kind);
}
