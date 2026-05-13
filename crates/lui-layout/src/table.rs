//! Table layout — `display: table / table-row / table-cell`.
//!
//! Implements: auto & fixed column sizing, border-spacing, border-collapse,
//! colspan/rowspan, caption, row groups (thead/tbody/tfoot).

use lui_core::{CssUnit, CssValue, Rect};
use lui_parse::HtmlNode;

use crate::box_tree::{BoxKind, LayoutBox};
use crate::context::LayoutContext;
use crate::geometry::Point;
use crate::sides;
use crate::sizes;
use crate::text::TextContext;

fn css_str(v: Option<&CssValue>) -> &str {
    match v {
        Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
        _ => "",
    }
}

struct Spacing {
    h: f32,
    v: f32,
}

fn resolve_border_spacing(style: &lui_cascade::ComputedStyle) -> Spacing {
    match style.border_spacing {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => {
            Spacing { h: *value as f32, v: *value as f32 }
        }
        Some(CssValue::Number(n)) => {
            Spacing { h: *n as f32, v: *n as f32 }
        }
        Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => {
            let parts: Vec<&str> = s.split_whitespace().collect();
            let parse_px = |s: &str| -> f32 {
                s.trim_end_matches("px").parse::<f32>().unwrap_or(0.0)
            };
            match parts.len() {
                1 => { let v = parse_px(parts[0]); Spacing { h: v, v } }
                2 => Spacing { h: parse_px(parts[0]), v: parse_px(parts[1]) },
                _ => Spacing { h: 0.0, v: 0.0 },
            }
        }
        _ => Spacing { h: 0.0, v: 0.0 },
    }
}

fn is_collapsed(style: &lui_cascade::ComputedStyle) -> bool {
    css_str(style.border_collapse) == "collapse"
}

fn is_fixed_layout(style: &lui_cascade::ComputedStyle) -> bool {
    css_str(style.table_layout) == "fixed"
}

fn get_colspan(node: &HtmlNode) -> usize {
    node.attr("colspan")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1)
}

fn get_rowspan(node: &HtmlNode) -> usize {
    node.attr("rowspan")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(1)
        .max(1)
}

#[derive(Clone, Copy)]
struct RowPath {
    child_idx: usize,
    sub_idx: Option<usize>,
}

struct RowInfo {
    path: RowPath,
    num_cells: usize,
    cell_colspans: Vec<usize>,
    cell_rowspans: Vec<usize>,
}

fn collect_row_info(b: &LayoutBox) -> Vec<RowInfo> {
    let mut rows = Vec::new();
    for (ci, child) in b.children.iter().enumerate() {
        match child.kind {
            BoxKind::TableRow => {
                let mut colspans = Vec::new();
                let mut rowspans = Vec::new();
                for cell in &child.children {
                    colspans.push(get_colspan(cell.node));
                    rowspans.push(get_rowspan(cell.node));
                }
                rows.push(RowInfo {
                    path: RowPath { child_idx: ci, sub_idx: None },
                    num_cells: child.children.len(),
                    cell_colspans: colspans,
                    cell_rowspans: rowspans,
                });
            }
            BoxKind::TableRowGroup => {
                for (si, grandchild) in child.children.iter().enumerate() {
                    if grandchild.kind == BoxKind::TableRow {
                        let mut colspans = Vec::new();
                        let mut rowspans = Vec::new();
                        for cell in &grandchild.children {
                            colspans.push(get_colspan(cell.node));
                            rowspans.push(get_rowspan(cell.node));
                        }
                        rows.push(RowInfo {
                            path: RowPath { child_idx: ci, sub_idx: Some(si) },
                            num_cells: grandchild.children.len(),
                            cell_colspans: colspans,
                            cell_rowspans: rowspans,
                        });
                    }
                }
            }
            _ => {}
        }
    }
    rows
}

fn get_row<'a>(b: &'a LayoutBox<'a>, path: RowPath) -> &'a LayoutBox<'a> {
    let child = &b.children[path.child_idx];
    match path.sub_idx {
        Some(si) => &child.children[si],
        None => child,
    }
}

fn get_row_mut<'a, 'b>(b: &'b mut LayoutBox<'a>, path: RowPath) -> &'b mut LayoutBox<'a> {
    let child = &mut b.children[path.child_idx];
    match path.sub_idx {
        Some(si) => &mut child.children[si],
        None => child,
    }
}

/// Build occupancy grid AND the starting column for each cell.
/// Returns (occupancy, cell_start_cols) where cell_start_cols[ri][ci] = column index.
fn build_grid(row_infos: &[RowInfo], num_cols: usize) -> (Vec<Vec<bool>>, Vec<Vec<usize>>) {
    let num_rows = row_infos.len();
    let mut occ = vec![vec![false; num_cols]; num_rows];
    let mut start_cols: Vec<Vec<usize>> = row_infos.iter()
        .map(|info| vec![0; info.num_cells])
        .collect();

    for (ri, info) in row_infos.iter().enumerate() {
        let mut col = 0;
        for ci in 0..info.num_cells {
            while col < num_cols && occ[ri][col] {
                col += 1;
            }
            if col >= num_cols {
                start_cols[ri][ci] = num_cols;
                continue;
            }
            start_cols[ri][ci] = col;
            let cs = info.cell_colspans[ci].min(num_cols - col);
            let rs = info.cell_rowspans[ci].min(num_rows - ri);
            for r in ri..(ri + rs) {
                for c in col..(col + cs) {
                    occ[r][c] = true;
                }
            }
            col += cs;
        }
    }
    (occ, start_cols)
}

fn cell_width(col_widths: &[f32], start_col: usize, colspan: usize, spacing: f32) -> f32 {
    let end = (start_col + colspan).min(col_widths.len());
    if start_col >= col_widths.len() { return 0.0; }
    let sum: f32 = col_widths[start_col..end].iter().sum();
    sum + spacing * (colspan as f32 - 1.0).max(0.0)
}

fn col_x_offset(col_widths: &[f32], col: usize, spacing: f32) -> f32 {
    let mut x = 0.0;
    for i in 0..col.min(col_widths.len()) {
        x += col_widths[i] + spacing;
    }
    x
}

pub fn layout_table<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) {
    let margin = sides::resolve_margin_against(b.style, ctx.containing_width);
    let border = sides::resolve_border(b.style);
    let padding = sides::resolve_padding_against(b.style, ctx.containing_width);
    b.margin = margin.edges;
    b.border = border;
    b.padding = padding;

    let frame_h = border.horizontal() + padding.horizontal();
    let available = ctx.containing_width - margin.edges.horizontal() - frame_h;
    let table_width = sizes::resolve_length(b.style.width, ctx.containing_width)
        .unwrap_or(available.max(0.0));
    b.content.width = table_width;
    b.content.x = pos.x + margin.edges.left + border.left + padding.left;
    b.content.y = pos.y + margin.edges.top + border.top + padding.top;
    let table_x = b.content.x;

    let collapsed = is_collapsed(b.style);
    let spacing = if collapsed {
        Spacing { h: 0.0, v: 0.0 }
    } else {
        resolve_border_spacing(b.style)
    };
    let fixed = is_fixed_layout(b.style);
    let sp_h = spacing.h;
    let sp_v = spacing.v;

    let mut cursor_y = b.content.y;

    // Phase 1: Layout top captions
    for child in b.children.iter_mut() {
        if child.kind != BoxKind::TableCaption { continue; }
        let side = css_str(child.style.caption_side);
        if side == "bottom" { continue; }
        let cap_ctx = LayoutContext { containing_width: table_width, ..*ctx };
        crate::block::layout_block(child, &cap_ctx, Point::new(table_x, cursor_y), text_ctx, rects);
        cursor_y += child.outer_height();
    }

    // Phase 2: Collect row metadata
    let row_infos = collect_row_info(b);
    let num_rows = row_infos.len();
    if num_rows == 0 {
        layout_bottom_captions(b, table_x, table_width, ctx, &mut cursor_y, text_ctx, rects);
        b.content.height = sizes::resolve_length(b.style.height, ctx.containing_height)
            .unwrap_or((cursor_y - b.content.y).max(0.0));
        return;
    }

    // Determine column count from occupancy
    let mut num_cols = 0usize;
    for info in &row_infos {
        let mut c = 0;
        for ci in 0..info.num_cells {
            c += info.cell_colspans[ci];
        }
        num_cols = num_cols.max(c);
    }
    if num_cols == 0 { num_cols = 1; }

    let (occ, start_cols) = build_grid(&row_infos, num_cols);
    // Recount from occupancy in case rowspans expanded the grid
    for row in &occ {
        let c = row.iter().rposition(|&v| v).map(|i| i + 1).unwrap_or(0);
        num_cols = num_cols.max(c);
    }

    // Phase 3: Compute column widths
    let col_widths = if fixed {
        compute_column_widths_fixed(b, &row_infos, &start_cols, num_cols, table_width, &spacing)
    } else {
        compute_column_widths_auto(b, &row_infos, &start_cols, num_cols, table_width, &spacing)
    };

    // Phase 4: Compute row heights — first pass (non-rowspan cells)
    let mut row_heights = vec![0.0_f32; num_rows];
    for (ri, info) in row_infos.iter().enumerate() {
        let row = get_row(b, info.path);
        for ci in 0..info.num_cells {
            let col = start_cols[ri][ci];
            if col >= num_cols { continue; }
            let cs = info.cell_colspans[ci].min(num_cols - col);
            let rs = info.cell_rowspans[ci];
            if rs == 1 {
                let cw = cell_width(&col_widths, col, cs, sp_h);
                let h = estimate_cell_height(&row.children[ci], cw, ctx, text_ctx);
                row_heights[ri] = row_heights[ri].max(h);
            }
        }
    }

    // Second pass: distribute rowspan heights
    for (ri, info) in row_infos.iter().enumerate() {
        let row = get_row(b, info.path);
        for ci in 0..info.num_cells {
            let col = start_cols[ri][ci];
            if col >= num_cols { continue; }
            let cs = info.cell_colspans[ci].min(num_cols - col);
            let rs = info.cell_rowspans[ci].min(num_rows - ri);
            if rs > 1 {
                let cw = cell_width(&col_widths, col, cs, sp_h);
                let needed = estimate_cell_height(&row.children[ci], cw, ctx, text_ctx);
                let spanned: f32 = (ri..ri + rs).map(|r| row_heights[r]).sum::<f32>()
                    + sp_v * (rs as f32 - 1.0);
                if needed > spanned {
                    let extra = (needed - spanned) / rs as f32;
                    for r in ri..ri + rs {
                        row_heights[r] += extra;
                    }
                }
            }
        }
    }

    // Phase 5: Compute row Y positions
    cursor_y += sp_v;
    let mut row_ys = vec![0.0_f32; num_rows];
    {
        let mut y = cursor_y;
        for ri in 0..num_rows {
            row_ys[ri] = y;
            y += row_heights[ri] + sp_v;
        }
        cursor_y = y;
    }

    // Phase 6: Layout cells and position rows
    for (ri, info) in row_infos.iter().enumerate() {
        let row = get_row_mut(b, info.path);
        row.content.x = table_x;
        row.content.y = row_ys[ri];
        row.content.height = row_heights[ri];
        row.content.width = table_width;

        for ci in 0..info.num_cells {
            let col = start_cols[ri][ci];
            if col >= num_cols { continue; }
            let cs = info.cell_colspans[ci].min(num_cols - col);
            let rs = info.cell_rowspans[ci].min(num_rows - ri);
            let cw = cell_width(&col_widths, col, cs, sp_h);
            let cell_x = table_x + sp_h + col_x_offset(&col_widths, col, sp_h);
            let cell_ctx = LayoutContext { containing_width: cw, ..*ctx };

            let cell = &mut row.children[ci];
            crate::block::layout_block(cell, &cell_ctx, Point::new(cell_x, row_ys[ri]), text_ctx, rects);
            cell.content.width = cw;
            if rs > 1 {
                let cell_h: f32 = (ri..ri + rs).map(|r| row_heights[r]).sum::<f32>()
                    + sp_v * (rs as f32 - 1.0);
                cell.content.height = cell_h;
            }
        }
    }

    // Position row groups
    for child in b.children.iter_mut() {
        if child.kind == BoxKind::TableRowGroup {
            let mut y_min = f32::MAX;
            let mut y_max = 0.0_f32;
            for sub in &child.children {
                if sub.kind == BoxKind::TableRow {
                    y_min = y_min.min(sub.content.y);
                    y_max = y_max.max(sub.content.y + sub.content.height);
                }
            }
            child.content.x = table_x;
            child.content.width = table_width;
            if y_min < f32::MAX {
                child.content.y = y_min;
                child.content.height = y_max - y_min;
            }
        }
    }

    // Phase 7: Layout bottom captions
    layout_bottom_captions(b, table_x, table_width, ctx, &mut cursor_y, text_ctx, rects);

    b.content.height = sizes::resolve_length(b.style.height, ctx.containing_height)
        .unwrap_or((cursor_y - b.content.y).max(0.0));
}

fn layout_bottom_captions<'a>(
    b: &mut LayoutBox<'a>,
    table_x: f32,
    table_width: f32,
    ctx: &LayoutContext,
    cursor_y: &mut f32,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) {
    for child in b.children.iter_mut() {
        if child.kind != BoxKind::TableCaption { continue; }
        let side = css_str(child.style.caption_side);
        if side != "bottom" { continue; }
        let cap_ctx = LayoutContext { containing_width: table_width, ..*ctx };
        crate::block::layout_block(child, &cap_ctx, Point::new(table_x, *cursor_y), text_ctx, rects);
        *cursor_y += child.outer_height();
    }
}

fn estimate_cell_height(
    cell: &LayoutBox,
    cell_w: f32,
    ctx: &LayoutContext,
    text_ctx: &mut TextContext,
) -> f32 {
    if let Some(h) = sizes::resolve_length(cell.style.height, ctx.containing_height) {
        let border = sides::resolve_border(cell.style);
        let padding = sides::resolve_padding(cell.style);
        return h + border.vertical() + padding.vertical();
    }
    let border = sides::resolve_border(cell.style);
    let padding = sides::resolve_padding(cell.style);
    let frame = border.vertical() + padding.vertical();
    let inner_w = (cell_w - border.horizontal() - padding.horizontal()).max(0.0);

    let mut h = 0.0_f32;
    for child in &cell.children {
        if let lui_core::HtmlElement::Text(ref content) = child.node.element {
            let style = crate::text::text_style_from_cascade(child.style);
            let lines = text_ctx.font_ctx.break_into_lines(content, &style, inner_w);
            h += lines.iter().map(|l| l.height).sum::<f32>();
        } else if let Some(ch) = sizes::resolve_length(child.style.height, ctx.containing_height) {
            h += ch;
        }
    }
    h + frame
}

fn compute_column_widths_fixed(
    b: &LayoutBox,
    row_infos: &[RowInfo],
    start_cols: &[Vec<usize>],
    num_cols: usize,
    table_width: f32,
    spacing: &Spacing,
) -> Vec<f32> {
    let total_spacing = spacing.h * (num_cols as f32 + 1.0);
    let available = (table_width - total_spacing).max(0.0);

    let mut col_widths: Vec<Option<f32>> = vec![None; num_cols];

    if let Some(first) = row_infos.first() {
        let row = get_row(b, first.path);
        for ci in 0..first.num_cells {
            let col = start_cols[0][ci];
            if col >= num_cols { continue; }
            let cs = first.cell_colspans[ci];
            if cs == 1 {
                if let Some(w) = sizes::resolve_length(row.children[ci].style.width, available) {
                    col_widths[col] = Some(w);
                }
            }
        }
    }

    let assigned: f32 = col_widths.iter().filter_map(|w| *w).sum();
    let unassigned = col_widths.iter().filter(|w| w.is_none()).count();
    let default_w = if unassigned > 0 {
        ((available - assigned) / unassigned as f32).max(0.0)
    } else {
        0.0
    };

    col_widths.iter().map(|w| w.unwrap_or(default_w)).collect()
}

fn compute_column_widths_auto(
    b: &LayoutBox,
    row_infos: &[RowInfo],
    start_cols: &[Vec<usize>],
    num_cols: usize,
    table_width: f32,
    spacing: &Spacing,
) -> Vec<f32> {
    let total_spacing = spacing.h * (num_cols as f32 + 1.0);
    let available = (table_width - total_spacing).max(0.0);

    let mut col_widths = vec![0.0_f32; num_cols];
    let mut col_has_explicit = vec![false; num_cols];

    for (ri, info) in row_infos.iter().enumerate() {
        let row = get_row(b, info.path);
        for ci in 0..info.num_cells {
            let col = start_cols[ri][ci];
            if col >= num_cols { continue; }
            let cs = info.cell_colspans[ci];
            if cs == 1 {
                if let Some(w) = sizes::resolve_length(row.children[ci].style.width, available) {
                    col_widths[col] = col_widths[col].max(w);
                    col_has_explicit[col] = true;
                }
            }
        }
    }

    let explicit_sum: f32 = col_widths.iter().sum();
    let auto_count = col_has_explicit.iter().filter(|e| !**e).count();

    if auto_count > 0 && explicit_sum < available {
        let share = (available - explicit_sum) / auto_count as f32;
        for (i, w) in col_widths.iter_mut().enumerate() {
            if !col_has_explicit[i] {
                *w = share;
            }
        }
    } else if auto_count == 0 && explicit_sum < available && num_cols > 0 {
        let extra = (available - explicit_sum) / num_cols as f32;
        for w in col_widths.iter_mut() {
            *w += extra;
        }
    } else if explicit_sum > available && num_cols > 0 {
        let scale = available / explicit_sum;
        for w in col_widths.iter_mut() {
            *w *= scale;
        }
    }

    col_widths
}
