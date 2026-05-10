use super::helpers::*;
use crate::*;

const RESET: &str = "border-spacing: 0; border: none; padding: 0;";

fn cell(h: u32) -> String {
  format!(r#"<td style="height: {h}px; border: none; padding: 0;"></td>"#)
}

fn cell_w(w: u32, h: u32) -> String {
  format!(r#"<td style="width: {w}px; height: {h}px; border: none; padding: 0;"></td>"#)
}

// ---------------------------------------------------------------------------
// Table layout: basic structure, column sizing, colspan, rowspan, spacing
// ---------------------------------------------------------------------------

#[test]
fn table_three_cells_in_one_row_share_width_equally() {
  let c = cell(40);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET} display: table; width: 300px;">
        <tr style="display: table-row;">{c}{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert_eq!(table.children.len(), 3);
  for cell in &table.children {
    assert!((cell.margin_rect.w - 100.0).abs() < 1.0,
      "expected ~100px, got {}", cell.margin_rect.w);
  }
}

#[test]
fn table_cells_in_single_row_are_horizontally_adjacent() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="{RESET} display: table; width: 200px;">
        <tr style="display: table-row;">{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let xs: Vec<f32> = table.children.iter().map(|c| c.margin_rect.x).collect();
  assert!(xs[1] > xs[0], "second cell should be right of first");
  assert!((xs[1] - 100.0).abs() < 1.0,
    "second cell should start at 100px, got {}", xs[1]);
}

#[test]
fn table_two_rows_stack_vertically() {
  let c40 = cell(40);
  let c60 = cell(60);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="{RESET} display: table; width: 200px;">
        <tr style="display: table-row;">{c40}</tr>
        <tr style="display: table-row;">{c60}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let ys: Vec<f32> = table.children.iter().map(|c| c.margin_rect.y).collect();
  assert!(ys[1] > ys[0], "second row cell should be below first");
  assert!((ys[1] - 40.0).abs() < 1.0,
    "second row should start at 40px, got {}", ys[1]);
}

#[test]
fn table_row_height_is_max_of_cells() {
  let c30 = cell(30);
  let c60 = cell(60);
  let c20 = cell(20);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="{RESET} display: table; width: 200px;">
        <tr style="display: table-row;">{c30}{c60}</tr>
        <tr style="display: table-row;">{c20}{c20}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let row2_ys: Vec<f32> = table.children.iter()
    .filter(|c| c.margin_rect.y > 1.0)
    .map(|c| c.margin_rect.y)
    .collect();
  assert!(!row2_ys.is_empty());
  for y in &row2_ys {
    assert!((*y - 60.0).abs() < 1.0,
      "row 2 should start at 60px (max of row 1 cells), got {y}");
  }
}

#[test]
fn table_explicit_cell_width_is_respected() {
  let c_auto = cell(30);
  let c150 = cell_w(150, 30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 400px;">
      <table style="{RESET} display: table; width: 400px;">
        <tr style="display: table-row;">{c150}{c_auto}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let first = &table.children[0];
  assert!((first.margin_rect.w - 150.0).abs() < 1.0,
    "explicit width cell should be ~150px, got {}", first.margin_rect.w);
  let second = &table.children[1];
  assert!((second.margin_rect.w - 250.0).abs() < 1.0,
    "auto cell should get remaining ~250px, got {}", second.margin_rect.w);
}

// ---------------------------------------------------------------------------
// Spacing
// ---------------------------------------------------------------------------

#[test]
fn table_border_spacing_separates_cells() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 220px;">
      <table style="display: table; width: 220px; border-spacing: 10px; border: none;">
        <tr style="display: table-row;">{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let first = &table.children[0];
  let second = &table.children[1];
  let gap = second.margin_rect.x - (first.margin_rect.x + first.margin_rect.w);
  assert!((gap - 10.0).abs() < 1.0,
    "border-spacing between cells should be ~10px, got {gap}");
}

#[test]
fn table_border_spacing_between_rows() {
  let c = cell(40);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="display: table; width: 200px; border-spacing: 8px; border: none;">
        <tr style="display: table-row;">{c}</tr>
        <tr style="display: table-row;">{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let first_y = table.children[0].margin_rect.y;
  let second_y = table.children[1].margin_rect.y;
  let row_gap = second_y - (first_y + 40.0);
  assert!((row_gap - 8.0).abs() < 1.0,
    "row spacing should be ~8px, got {row_gap}");
}

#[test]
fn table_default_border_spacing_from_ua_stylesheet() {
  let tree = make(
    r#"<body style="margin: 0; width: 200px;">
      <table>
        <tr>
          <td style="height: 30px; border: none;"></td>
          <td style="height: 30px; border: none;"></td>
        </tr>
      </table>
    </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let first = &table.children[0];
  let second = &table.children[1];
  let gap = second.margin_rect.x - (first.margin_rect.x + first.margin_rect.w);
  assert!((gap - 2.0).abs() < 0.5,
    "UA default border-spacing should be 2px, got {gap}");
}

// ---------------------------------------------------------------------------
// Colspan
// ---------------------------------------------------------------------------

#[test]
fn table_colspan_spans_multiple_columns() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <tr>
          <td colspan="2" style="height: 30px; border: none; padding: 0;"></td>
          {c}
        </tr>
        <tr>{c}{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let spanning_cell = &table.children[0];
  assert!(spanning_cell.margin_rect.w > 150.0,
    "colspan=2 cell should span ~200px, got {}", spanning_cell.margin_rect.w);
}

#[test]
fn table_colspan_full_width() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <tr>
          <td colspan="3" style="height: 30px; border: none; padding: 0;"></td>
        </tr>
        <tr>{c}{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let spanning_cell = &table.children[0];
  let table_content_w = table.content_rect.w;
  assert!((spanning_cell.margin_rect.w - table_content_w).abs() < 2.0,
    "colspan=3 cell should span full table width ({table_content_w}), got {}",
    spanning_cell.margin_rect.w);
}

// ---------------------------------------------------------------------------
// Rowspan
// ---------------------------------------------------------------------------

#[test]
fn table_rowspan_cell_height_covers_all_spanned_rows() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="{RESET}">
        <tr>
          <td style="height: 30px; border: none; padding: 0;"></td>
          <td rowspan="2" style="border: none; padding: 0;"></td>
        </tr>
        <tr>{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  // The rowspan=2 cell is the second child (after the first row's first cell).
  let rowspan_cell = table.children.iter()
    .find(|c| c.margin_rect.h > 31.0)
    .expect("should find a cell taller than one row");
  assert!((rowspan_cell.margin_rect.h - 60.0).abs() < 1.0,
    "rowspan=2 cell should be 60px tall (2 × 30px rows), got {}",
    rowspan_cell.margin_rect.h);
}

#[test]
fn table_rowspan_cell_spans_multiple_rows() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="{RESET}">
        <tr>
          <td rowspan="2" style="height: 80px; border: none; padding: 0;"></td>
          {c}
        </tr>
        <tr>{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert!(table.margin_rect.h >= 79.0,
    "table should be at least 80px tall for rowspan cell, got {}", table.margin_rect.h);
}

#[test]
fn table_rowspan_distributes_extra_height() {
  let c = cell(20);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 200px;">
      <table style="{RESET}">
        <tr>
          <td rowspan="2" style="height: 100px; border: none; padding: 0;"></td>
          {c}
        </tr>
        <tr>{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert!(table.margin_rect.h >= 99.0,
    "table should accommodate the 100px rowspan cell, got {}", table.margin_rect.h);
}

// ---------------------------------------------------------------------------
// Row groups (thead / tbody / tfoot)
// ---------------------------------------------------------------------------

#[test]
fn table_thead_tbody_tfoot_rows_are_collected() {
  let c = cell(25);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <thead><tr>{c}{c}</tr></thead>
        <tbody><tr>{c}{c}</tr></tbody>
        <tfoot><tr>{c}{c}</tr></tfoot>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert!(table.children.len() >= 6,
    "expected at least 6 cells from thead+tbody+tfoot, got {}", table.children.len());
  assert!(table.margin_rect.h >= 74.0,
    "table should be at least 75px (25×3), got {}", table.margin_rect.h);
}

// ---------------------------------------------------------------------------
// Caption
// ---------------------------------------------------------------------------

#[test]
fn table_caption_is_laid_out_above_rows() {
  let c = cell(40);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <caption style="height: 20px;">Title</caption>
        <tr>{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let caption = &table.children[0];
  let cell = &table.children[1];
  assert!(cell.margin_rect.y > caption.margin_rect.y,
    "cell should be below caption");
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn table_with_no_cells_produces_no_children() {
  let tree = make(
    r#"<body style="margin: 0;">
      <table style="display: table; width: 200px;"></table>
    </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert!(table.children.is_empty());
}

#[test]
fn table_without_explicit_width_fills_container() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 500px;">
      <table style="display: table; border: none;">
        <tr style="display: table-row;">{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert!((table.margin_rect.w - 500.0).abs() < 1.0,
    "table should fill container width (500px), got {}", table.margin_rect.w);
}

#[test]
fn two_tables_stack_vertically() {
  let c = cell(50);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET} display: table; width: 300px;">
        <tr style="display: table-row;">{c}</tr>
      </table>
      <table style="{RESET} display: table; width: 300px;">
        <tr style="display: table-row;">{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table1 = &body.children[0];
  let table2 = &body.children[1];
  assert!(table2.margin_rect.y >= table1.margin_rect.y + table1.margin_rect.h - 1.0,
    "second table should be below first");
}

// ---------------------------------------------------------------------------
// HTML table elements (UA stylesheet provides display types)
// ---------------------------------------------------------------------------

#[test]
fn html_table_elements_get_table_layout() {
  let c = cell(40);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <tr>{c}{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let xs: Vec<f32> = table.children.iter().map(|c| c.margin_rect.x).collect();
  assert!(xs.len() >= 3, "expected at least 3 cells");
  assert!(xs[1] > xs[0], "cells should be horizontally adjacent");
  assert!(xs[2] > xs[1], "cells should be horizontally adjacent");
}

#[test]
fn html_table_with_thead_and_tbody() {
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <thead>
          <tr><th style="height: 30px; border: none; padding: 0;">A</th>
              <th style="height: 30px; border: none; padding: 0;">B</th></tr>
        </thead>
        <tbody>
          <tr><td style="height: 25px; border: none; padding: 0;">1</td>
              <td style="height: 25px; border: none; padding: 0;">2</td></tr>
          <tr><td style="height: 25px; border: none; padding: 0;">3</td>
              <td style="height: 25px; border: none; padding: 0;">4</td></tr>
        </tbody>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  assert!(table.children.len() >= 6,
    "expected at least 6 cells, got {}", table.children.len());
  assert!(table.margin_rect.h >= 79.0,
    "table height should be >= 80px (30+25+25), got {}", table.margin_rect.h);
}

#[test]
fn html_table_colspan_attribute() {
  let c = cell(30);
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET}">
        <tr>
          <td colspan="3" style="height: 30px; border: none; padding: 0;">Wide</td>
        </tr>
        <tr>{c}{c}{c}</tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let wide_cell = &table.children[0];
  let table_content_w = table.content_rect.w;
  assert!((wide_cell.margin_rect.w - table_content_w).abs() < 2.0,
    "colspan=3 cell should span full width ({table_content_w}), got {}",
    wide_cell.margin_rect.w);
}

#[test]
fn table_cell_padding_is_applied() {
  let tree = make(&format!(
    r#"<body style="margin: 0; width: 300px;">
      <table style="{RESET} display: table; width: 300px;">
        <tr style="display: table-row;">
          <td style="display: table-cell; padding: 10px; height: 20px; border: none;"></td>
        </tr>
      </table>
    </body>"#));
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let cell = &table.children[0];
  assert!(cell.content_rect.w < cell.border_rect.w,
    "content should be narrower than border rect due to padding");
  let h_padding = cell.border_rect.w - cell.content_rect.w;
  assert!((h_padding - 20.0).abs() < 1.0,
    "horizontal padding should be 20px (10+10), got {h_padding}");
}

// ---------------------------------------------------------------------------
// Cells must not overflow the table content rect
// ---------------------------------------------------------------------------

#[test]
fn table_cells_stay_within_table_content_rect() {
  let tree = make(
    r#"<body style="margin: 0; width: 300px;">
      <table>
        <tr>
          <td colspan="2" style="height: 30px;">Name</td>
          <td style="height: 30px;">Score</td>
        </tr>
        <tr>
          <td style="height: 30px;">First</td>
          <td style="height: 30px;">Last</td>
          <td rowspan="2" style="height: 60px;">95</td>
        </tr>
        <tr>
          <td style="height: 30px;">Jane</td>
          <td style="height: 30px;">Doe</td>
        </tr>
      </table>
    </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let table_right = table.content_rect.x + table.content_rect.w;
  for (i, child) in table.children.iter().enumerate() {
    let child_right = child.margin_rect.x + child.margin_rect.w;
    assert!(child_right <= table_right + 1.0,
      "cell {i} right edge ({child_right}) exceeds table content right ({table_right})");
  }
}

// ---------------------------------------------------------------------------
// UA default border rendering
// ---------------------------------------------------------------------------

#[test]
fn html_table_cells_have_default_border() {
  let tree = make(
    r#"<body style="margin: 0; width: 200px;">
      <table style="border-spacing: 0;">
        <tr><td style="height: 30px;">cell</td></tr>
      </table>
    </body>"#,
  );
  let body = layout(&tree, 800.0, 600.0).unwrap();
  let table = &body.children[0];
  let cell = &table.children[0];
  let total_border = cell.border.top + cell.border.bottom;
  assert!(total_border >= 1.5,
    "UA stylesheet should give td a 1px inset border, got total vertical border {total_border}");
}
