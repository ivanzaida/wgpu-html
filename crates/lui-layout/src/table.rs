//! Table formatting context.
//!
//! Implements CSS table layout (CSS 2.1 §17) at a practical level:
//!
//! - `display: table` wrapper — establishes table formatting context
//! - `display: table-row-group` (thead/tbody/tfoot), `table-row`, `table-cell`
//! - `display: table-caption` — laid out as a block above the table grid
//! - Column width distribution: measures min/max content per column, distributes remaining space proportionally
//! - `colspan` / `rowspan` attribute support
//! - `gap` / `border-spacing` between cells
//! - Nested tables (cells delegate back to `layout_block_at_with`)

use lui_models::{Style, common::css_enums::Display};
use lui_style::CascadedNode;

use crate::{
  BlockOverrides, Ctx, LayoutBox, layout_block_at_with, length, translate_box_x_in_place, translate_box_y_in_place,
};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn layout_table_children(
  parent: &CascadedNode,
  parent_style: &Style,
  content_x: f32,
  content_y: f32,
  inner_width: f32,
  _inner_height_explicit: Option<f32>,
  ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
  let spacing = resolve_spacing(parent_style, inner_width, ctx);

  let mut captions: Vec<&CascadedNode> = Vec::new();
  let mut rows: Vec<&CascadedNode> = Vec::new();

  collect_rows(parent, &mut captions, &mut rows);

  if rows.is_empty() && captions.is_empty() {
    return (Vec::new(), 0.0, 0.0);
  }

  // Build the cell grid with colspan/rowspan tracking.
  let grid = build_cell_grid(&rows);
  let num_cols = grid.num_cols;
  let num_rows = grid.cells.len();

  if num_cols == 0 {
    // Caption-only table.
    let mut result = Vec::new();
    let mut cursor_y = 0.0_f32;
    for cap in &captions {
      let b = layout_block_at_with(
        cap,
        content_x,
        content_y + cursor_y,
        inner_width,
        0.0,
        BlockOverrides::default(),
        ctx,
      );
      cursor_y += b.margin_rect.h;
      result.push(b);
    }
    return (result, inner_width, cursor_y);
  }

  // --- Column width resolution ---
  let col_widths = resolve_column_widths(&grid, num_cols, inner_width, spacing, ctx);

  // --- Row layout ---
  let mut all_boxes: Vec<LayoutBox> = Vec::new();
  let mut cursor_y = 0.0_f32;

  // Captions above.
  for cap in &captions {
    let b = layout_block_at_with(
      cap,
      content_x,
      content_y + cursor_y,
      inner_width,
      0.0,
      BlockOverrides::default(),
      ctx,
    );
    cursor_y += b.margin_rect.h;
    all_boxes.push(b);
  }

  // Lay out cells row by row, then vertically align them once row
  // height is known.
  let mut row_heights: Vec<f32> = vec![0.0; num_rows];
  let mut cell_boxes: Vec<Vec<Option<LayoutBox>>> = Vec::with_capacity(num_rows);

  for (ri, row_cells) in grid.cells.iter().enumerate() {
    let mut row_box_list: Vec<Option<LayoutBox>> = vec![None; num_cols];

    for ci in 0..num_cols {
      let cell_info = &row_cells[ci];
      if cell_info.origin_row != ri || cell_info.origin_col != ci {
        continue; // spanned-over slot
      }
      let Some(cell_node) = cell_info.node else { continue };

      let colspan = cell_info.colspan;
      let cell_w = cell_width(&col_widths, ci, colspan, spacing);

      let b = layout_block_at_with(cell_node, 0.0, 0.0, cell_w, 0.0, BlockOverrides::default(), ctx);

      let rowspan = cell_info.rowspan;
      if rowspan == 1 {
        row_heights[ri] = row_heights[ri].max(b.margin_rect.h);
      }
      // For rowspan > 1, we distribute height after measuring all rows.

      row_box_list[ci] = Some(b);
    }
    cell_boxes.push(row_box_list);
  }

  // Handle rowspan height distribution: if a spanning cell is taller
  // than the sum of its spanned rows, distribute the extra equally.
  for (ri, row_cells) in grid.cells.iter().enumerate() {
    for ci in 0..num_cols {
      let cell_info = &row_cells[ci];
      if cell_info.origin_row != ri || cell_info.origin_col != ci {
        continue;
      }
      let rowspan = cell_info.rowspan;
      if rowspan <= 1 {
        continue;
      }

      let b = match &cell_boxes[ri][ci] {
        Some(b) => b,
        None => continue,
      };

      let span_end = (ri + rowspan).min(num_rows);
      let spanned_h: f32 =
        (ri..span_end).map(|r| row_heights[r]).sum::<f32>() + spacing * (span_end - ri - 1).max(0) as f32;

      if b.margin_rect.h > spanned_h {
        let extra = b.margin_rect.h - spanned_h;
        let per_row = extra / (span_end - ri) as f32;
        for r in ri..span_end {
          row_heights[r] += per_row;
        }
      }
    }
  }

  // Compute cumulative row Y offsets for positioning.
  let mut row_y_offsets: Vec<f32> = Vec::with_capacity(num_rows);
  {
    let mut y = 0.0_f32;
    for ri in 0..num_rows {
      row_y_offsets.push(y);
      y += row_heights[ri] + spacing;
    }
  }

  // Position cells at their final coordinates.
  for ri in 0..num_rows {
    for ci in 0..num_cols {
      let cell_info = &grid.cells[ri][ci];
      if cell_info.origin_row != ri || cell_info.origin_col != ci {
        continue;
      }
      let Some(mut b) = cell_boxes[ri][ci].take() else {
        continue;
      };

      let cell_x = col_x_offset(&col_widths, ci, spacing);
      translate_box_x_in_place(&mut b, content_x + cell_x);
      translate_box_y_in_place(&mut b, content_y + cursor_y + row_y_offsets[ri]);

      // Stretch rowspan cells to cover all spanned rows.
      let rowspan = cell_info.rowspan;
      if rowspan > 1 {
        let span_end = (ri + rowspan).min(num_rows);
        let spanned_h: f32 =
          (ri..span_end).map(|r| row_heights[r]).sum::<f32>() + spacing * (span_end - ri).saturating_sub(1) as f32;
        let dh = spanned_h - b.margin_rect.h;
        if dh > 0.0 {
          b.margin_rect.h = spanned_h;
          b.border_rect.h += dh;
          b.content_rect.h += dh;
          b.background_rect.h += dh;
        }
      }

      all_boxes.push(b);
    }
  }

  let total_rows_h = row_y_offsets.last().copied().unwrap_or(0.0) + row_heights.last().copied().unwrap_or(0.0);
  cursor_y += total_rows_h;

  // Remove trailing spacing.
  if num_rows > 0 {
    cursor_y -= spacing;
  }

  (all_boxes, inner_width, cursor_y)
}

// ---------------------------------------------------------------------------
// Cell grid construction
// ---------------------------------------------------------------------------

struct CellInfo<'a> {
  node: Option<&'a CascadedNode>,
  colspan: usize,
  rowspan: usize,
  origin_row: usize,
  origin_col: usize,
}

struct CellGrid<'a> {
  cells: Vec<Vec<CellInfo<'a>>>,
  num_cols: usize,
}

fn collect_rows<'a>(parent: &'a CascadedNode, captions: &mut Vec<&'a CascadedNode>, rows: &mut Vec<&'a CascadedNode>) {
  for child in &parent.children {
    let d = child.style.display.as_ref();
    match d {
      Some(Display::TableCaption) => captions.push(child),
      Some(Display::TableRow) => rows.push(child),
      Some(Display::TableHeaderGroup | Display::TableRowGroup | Display::TableFooterGroup) => {
        for grandchild in &child.children {
          if matches!(grandchild.style.display.as_ref(), Some(Display::TableRow)) {
            rows.push(grandchild);
          } else {
            // Treat non-row children of row groups as implicit rows.
            rows.push(grandchild);
          }
        }
      }
      _ => {
        // Non-table children — treat as implicit single-cell row.
        rows.push(child);
      }
    }
  }
}

fn build_cell_grid<'a>(rows: &[&'a CascadedNode]) -> CellGrid<'a> {
  let num_rows = rows.len();

  // First pass: determine column count.
  let mut max_cols = 0_usize;
  for row in rows {
    let mut col_count = 0_usize;
    let cells = row_cells(row);
    for cell in &cells {
      col_count += cell_colspan(cell);
    }
    max_cols = max_cols.max(col_count);
  }

  // Second pass: populate grid with spanning.
  let mut grid: Vec<Vec<CellInfo<'a>>> = Vec::with_capacity(num_rows);
  for _ in 0..num_rows {
    let mut row_vec = Vec::with_capacity(max_cols);
    for c in 0..max_cols {
      row_vec.push(CellInfo {
        node: None,
        colspan: 1,
        rowspan: 1,
        origin_row: 0,
        origin_col: c,
      });
    }
    grid.push(row_vec);
  }

  for (ri, row) in rows.iter().enumerate() {
    let cells = row_cells(row);
    let mut ci = 0_usize;
    for cell in &cells {
      // Skip slots already occupied by rowspan from above.
      while ci < max_cols && grid[ri][ci].node.is_some() {
        ci += 1;
      }
      if ci >= max_cols {
        break;
      }

      let colspan = cell_colspan(cell).max(1);
      let rowspan = cell_rowspan(cell).max(1);

      // Fill the grid slots this cell spans.
      for dr in 0..rowspan {
        let r = ri + dr;
        if r >= num_rows {
          break;
        }
        for dc in 0..colspan {
          let c = ci + dc;
          if c >= max_cols {
            break;
          }
          grid[r][c] = CellInfo {
            node: if dr == 0 && dc == 0 { Some(cell) } else { None },
            colspan,
            rowspan,
            origin_row: ri,
            origin_col: ci,
          };
        }
      }

      // Place the actual node reference on the origin cell.
      grid[ri][ci].node = Some(cell);

      ci += colspan;
    }
  }

  CellGrid {
    cells: grid,
    num_cols: max_cols,
  }
}

fn row_cells<'a>(row: &'a CascadedNode) -> Vec<&'a CascadedNode> {
  let d = row.style.display.as_ref();
  if matches!(d, Some(Display::TableRow)) {
    row.children.iter().collect()
  } else {
    // Implicit row: the node itself is the single cell.
    vec![row]
  }
}

fn cell_colspan(cell: &CascadedNode) -> usize {
  cell
    .element
    .attr("colspan")
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(1)
    .max(1)
}

fn cell_rowspan(cell: &CascadedNode) -> usize {
  cell
    .element
    .attr("rowspan")
    .and_then(|v| v.parse::<usize>().ok())
    .unwrap_or(1)
    .max(1)
}

// ---------------------------------------------------------------------------
// Column width resolution
// ---------------------------------------------------------------------------

fn resolve_column_widths(grid: &CellGrid, num_cols: usize, table_width: f32, spacing: f32, ctx: &mut Ctx) -> Vec<f32> {
  let total_spacing = spacing * (num_cols.saturating_sub(1)) as f32;
  let available = (table_width - total_spacing).max(0.0);

  // Measure min content width for each column.
  let mut min_widths = vec![0.0_f32; num_cols];
  let mut has_explicit = vec![false; num_cols];
  let mut explicit_widths = vec![0.0_f32; num_cols];

  for row_cells in &grid.cells {
    for ci in 0..num_cols {
      let info = &row_cells[ci];
      if info.origin_col != ci || info.colspan != 1 {
        continue; // only single-span cells contribute to column min
      }
      let Some(cell) = info.node else { continue };

      // Check for explicit width on the cell.
      if let Some(w) = length::resolve(cell.style.width.as_ref(), available, ctx) {
        if w > explicit_widths[ci] {
          explicit_widths[ci] = w;
          has_explicit[ci] = true;
        }
      }

      // Measure intrinsic minimum.
      let probe = layout_block_at_with(cell, 0.0, 0.0, 0.0, 0.0, BlockOverrides::default(), ctx);
      min_widths[ci] = min_widths[ci].max(probe.margin_rect.w);
    }
  }

  // Start with the larger of explicit width or min content.
  let mut widths: Vec<f32> = (0..num_cols)
    .map(|ci| {
      if has_explicit[ci] {
        explicit_widths[ci].max(min_widths[ci])
      } else {
        min_widths[ci]
      }
    })
    .collect();

  // Distribute remaining space to columns without explicit widths.
  let used: f32 = widths.iter().sum();
  let remaining = available - used;
  if remaining > 0.0 {
    let auto_cols: Vec<usize> = (0..num_cols).filter(|&ci| !has_explicit[ci]).collect();
    if !auto_cols.is_empty() {
      let share = remaining / auto_cols.len() as f32;
      for ci in auto_cols {
        widths[ci] += share;
      }
    } else {
      // All columns explicit — distribute proportionally.
      if used > 0.0 {
        let factor = available / used;
        for w in &mut widths {
          *w *= factor;
        }
      }
    }
  }

  widths
}

fn cell_width(col_widths: &[f32], start_col: usize, colspan: usize, spacing: f32) -> f32 {
  let end = (start_col + colspan).min(col_widths.len());
  let w: f32 = col_widths[start_col..end].iter().sum();
  w + spacing * (end - start_col).saturating_sub(1) as f32
}

fn col_x_offset(col_widths: &[f32], col: usize, spacing: f32) -> f32 {
  let mut x = 0.0_f32;
  for ci in 0..col {
    x += col_widths[ci] + spacing;
  }
  x
}

// ---------------------------------------------------------------------------
// Spacing
// ---------------------------------------------------------------------------

fn resolve_spacing(style: &Style, container_w: f32, ctx: &Ctx) -> f32 {
  // border-spacing from deferred_longhands, then fall back to gap.
  if let Some(raw) = style.deferred_longhands.get("border-spacing") {
    if let Some(v) = parse_spacing_value(raw, container_w, ctx) {
      return v;
    }
  }
  length::resolve(
    style
      .gap
      .as_ref()
      .or(style.row_gap.as_ref())
      .or(style.column_gap.as_ref()),
    container_w,
    ctx,
  )
  .unwrap_or(0.0)
}

fn parse_spacing_value(raw: &str, _container_w: f32, ctx: &Ctx) -> Option<f32> {
  let token = raw.trim().split_whitespace().next()?;
  if let Some(stripped) = token.strip_suffix("px") {
    return stripped.parse::<f32>().ok().map(|v| v * ctx.scale);
  }
  token.parse::<f32>().ok().map(|v| v * ctx.scale)
}
