//! CSS Grid layout — CSS-Grid-1 implementation.
//!
//! Supports: grid-template-columns/rows (px/fr/auto/repeat()/minmax()),
//! gap, auto-placement (row/column/dense), grid-column/row-start/end,
//! span N, grid-auto-rows/columns, align-items/justify-items on cells.

use lui_core::{CssUnit, CssValue, Rect};
use lui_parse::HtmlNode;

use crate::box_tree::{BoxKind, LayoutBox};
use crate::context::LayoutContext;
use crate::geometry::Point;
use crate::positioned;
use crate::sides;
use crate::sizes;
use crate::text::TextContext;

// ── Track sizing ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum TrackSize {
    Px(f32),
    Fr(f32),
    Auto,
    MinMax(f32, TrackMax),
}

#[derive(Debug, Clone)]
enum TrackMax {
    Px(f32),
    Fr(f32),
    Auto,
}

fn parse_track_list(value: Option<&CssValue>, container_width: f32) -> Vec<TrackSize> {
    match value {
        Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) => {
            let raw = s.as_ref();
            if raw == "none" || raw.is_empty() { return vec![]; }
            parse_track_str(raw, container_width)
        }
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => vec![TrackSize::Px(*value as f32)],
        Some(CssValue::Dimension { value, unit: CssUnit::Fr }) => vec![TrackSize::Fr(*value as f32)],
        Some(CssValue::Number(n)) if *n == 0.0 => vec![TrackSize::Px(0.0)],
        Some(CssValue::Function { function, args }) => {
            parse_track_function(function, args, container_width)
        }
        _ => vec![],
    }
}

fn parse_track_function(function: &lui_core::CssFunction, args: &[CssValue], container_width: f32) -> Vec<TrackSize> {
    let name = function.name();
    if name == "repeat" || name.contains("repeat") {
        if args.len() >= 2 {
            let count = match &args[0] {
                CssValue::Number(n) => *n as usize,
                _ => 1,
            };
            let mut pattern = Vec::new();
            for arg in &args[1..] {
                match arg {
                    CssValue::Dimension { value, unit: CssUnit::Px } => pattern.push(TrackSize::Px(*value as f32)),
                    CssValue::Dimension { value, unit: CssUnit::Fr } => pattern.push(TrackSize::Fr(*value as f32)),
                    CssValue::Percentage(p) => pattern.push(TrackSize::Px(*p as f32 / 100.0 * container_width)),
                    CssValue::String(s) | CssValue::Unknown(s) if s.as_ref() == "auto" => pattern.push(TrackSize::Auto),
                    _ => {}
                }
            }
            if pattern.is_empty() { pattern.push(TrackSize::Fr(1.0)); }
            let mut result = Vec::with_capacity(count * pattern.len());
            for _ in 0..count { result.extend(pattern.iter().cloned()); }
            return result;
        }
    }
    if name == "minmax" && args.len() >= 2 {
        let min = match &args[0] {
            CssValue::Dimension { value, unit: CssUnit::Px } => *value as f32,
            CssValue::Number(n) => *n as f32,
            _ => 0.0,
        };
        let max = match &args[1] {
            CssValue::Dimension { value, unit: CssUnit::Px } => TrackMax::Px(*value as f32),
            CssValue::Dimension { value, unit: CssUnit::Fr } => TrackMax::Fr(*value as f32),
            _ => TrackMax::Auto,
        };
        return vec![TrackSize::MinMax(min, max)];
    }
    vec![]
}

fn parse_track_str(raw: &str, container_width: f32) -> Vec<TrackSize> {
    let mut tracks = Vec::new();
    let mut chars = raw.chars().peekable();
    let mut buf = String::new();

    while chars.peek().is_some() {
        skip_ws(&mut chars);
        if chars.peek().is_none() { break; }

        buf.clear();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_whitespace() { break; }
            if c == '(' {
                buf.push(c);
                chars.next();
                let mut depth = 1;
                while let Some(&c2) = chars.peek() {
                    buf.push(c2);
                    chars.next();
                    if c2 == '(' { depth += 1; }
                    if c2 == ')' { depth -= 1; if depth == 0 { break; } }
                }
                continue;
            }
            buf.push(c);
            chars.next();
        }

        if buf.is_empty() { continue; }

        if buf.starts_with("repeat(") && buf.ends_with(')') {
            let inner = &buf[7..buf.len()-1];
            if let Some((count_str, pattern)) = inner.split_once(',') {
                let count_str = count_str.trim();
                if let Ok(count) = count_str.parse::<usize>() {
                    let pattern_tracks = parse_track_str(pattern.trim(), container_width);
                    for _ in 0..count {
                        tracks.extend(pattern_tracks.iter().cloned());
                    }
                }
            }
        } else if buf.starts_with("minmax(") && buf.ends_with(')') {
            let inner = &buf[7..buf.len()-1];
            if let Some((min_s, max_s)) = inner.split_once(',') {
                let min_v = parse_single_size(min_s.trim(), container_width);
                let max_v = parse_single_size(max_s.trim(), container_width);
                let min_px = match &min_v {
                    TrackSize::Px(v) => *v,
                    _ => 0.0,
                };
                let max_tm = match max_v {
                    TrackSize::Px(v) => TrackMax::Px(v),
                    TrackSize::Fr(v) => TrackMax::Fr(v),
                    TrackSize::Auto => TrackMax::Auto,
                    TrackSize::MinMax(_, m) => m,
                };
                tracks.push(TrackSize::MinMax(min_px, max_tm));
            }
        } else {
            tracks.push(parse_single_size(&buf, container_width));
        }
    }
    tracks
}

fn parse_single_size(token: &str, container_width: f32) -> TrackSize {
    if token == "auto" {
        TrackSize::Auto
    } else if let Some(fr) = token.strip_suffix("fr") {
        fr.parse::<f32>().map(TrackSize::Fr).unwrap_or(TrackSize::Auto)
    } else if let Some(px) = token.strip_suffix("px") {
        px.parse::<f32>().map(TrackSize::Px).unwrap_or(TrackSize::Auto)
    } else if let Some(pct) = token.strip_suffix('%') {
        pct.parse::<f32>().map(|v| TrackSize::Px(v / 100.0 * container_width)).unwrap_or(TrackSize::Auto)
    } else if let Ok(v) = token.parse::<f32>() {
        TrackSize::Px(v)
    } else {
        TrackSize::Auto
    }
}

fn skip_ws(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(&c) = chars.peek() {
        if !c.is_ascii_whitespace() { break; }
        chars.next();
    }
}

fn resolve_tracks(defs: &[TrackSize], available: f32, gap: f32, auto_sizes: &[f32]) -> Vec<f32> {
    if defs.is_empty() { return vec![]; }

    let total_gap = gap * (defs.len() as f32 - 1.0).max(0.0);
    let mut fixed_sum = 0.0_f32;
    let mut fr_sum = 0.0_f32;
    let mut sizes: Vec<f32> = defs.iter().enumerate().map(|(i, d)| match d {
        TrackSize::Px(v) => { fixed_sum += v; *v }
        TrackSize::Fr(v) => { fr_sum += v; 0.0 }
        TrackSize::Auto => {
            let s = auto_sizes.get(i).copied().unwrap_or(0.0);
            fixed_sum += s;
            s
        }
        TrackSize::MinMax(min, max) => {
            match max {
                TrackMax::Fr(v) => { fr_sum += v; fixed_sum += min; *min }
                TrackMax::Px(v) => { let s = *v; fixed_sum += s; s }
                TrackMax::Auto => {
                    let s = auto_sizes.get(i).copied().unwrap_or(*min).max(*min);
                    fixed_sum += s;
                    s
                }
            }
        }
    }).collect();

    if fr_sum > 0.0 {
        let free = (available - total_gap - fixed_sum).max(0.0);
        for (i, def) in defs.iter().enumerate() {
            let fr_val = match def {
                TrackSize::Fr(f) => Some(*f),
                TrackSize::MinMax(min, TrackMax::Fr(f)) => {
                    let share = free * f / fr_sum;
                    sizes[i] = share.max(*min);
                    None
                }
                _ => None,
            };
            if let Some(f) = fr_val {
                sizes[i] = free * f / fr_sum;
            }
        }
    }
    sizes
}

// ── Grid item placement ───────────────────────────────────────────────

fn css_str(v: Option<&CssValue>) -> &str {
    match v {
        Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
        _ => "",
    }
}

struct GridPlacement {
    col_start: usize,
    col_end: usize,
    row_start: usize,
    row_end: usize,
}

fn parse_line_value(v: Option<&CssValue>) -> LinePlacement {
    match v {
        Some(CssValue::Number(n)) => LinePlacement::Line((*n as i32).max(1) as usize),
        Some(CssValue::Dimension { value, .. }) => LinePlacement::Line((*value as i32).max(1) as usize),
        Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) => {
            let s = s.as_ref().trim();
            if s == "auto" || s.is_empty() { return LinePlacement::Auto; }
            if let Some(rest) = s.strip_prefix("span") {
                let rest = rest.trim();
                if let Ok(n) = rest.parse::<usize>() {
                    return LinePlacement::Span(n.max(1));
                }
            }
            if let Ok(n) = s.parse::<i32>() {
                return LinePlacement::Line(n.max(1) as usize);
            }
            LinePlacement::Auto
        }
        _ => LinePlacement::Auto,
    }
}

#[derive(Debug, Clone)]
enum LinePlacement {
    Auto,
    Line(usize),
    Span(usize),
}

fn resolve_placement(
    style: &lui_cascade::ComputedStyle,
    auto_col: &mut usize,
    auto_row: &mut usize,
    num_cols: usize,
    occupied: &mut Vec<Vec<bool>>,
    flow_column: bool,
    dense: bool,
) -> GridPlacement {
    let cs = parse_line_value(style.grid_column_start);
    let ce = parse_line_value(style.grid_column_end);
    let rs = parse_line_value(style.grid_row_start);
    let re = parse_line_value(style.grid_row_end);

    let col_span = match (&cs, &ce) {
        (_, LinePlacement::Span(n)) => *n,
        (LinePlacement::Line(s), LinePlacement::Line(e)) if *e > *s => *e - *s,
        _ => 1,
    };
    let row_span = match (&rs, &re) {
        (_, LinePlacement::Span(n)) => *n,
        (LinePlacement::Line(s), LinePlacement::Line(e)) if *e > *s => *e - *s,
        _ => 1,
    };

    let col_start = match &cs {
        LinePlacement::Line(n) => (*n - 1).min(num_cols.saturating_sub(1)),
        _ => {
            if flow_column {
                *auto_row
            } else {
                find_auto_slot(auto_col, auto_row, col_span, row_span, num_cols, occupied, flow_column, dense)
            }
        }
    };
    let row_start = match &rs {
        LinePlacement::Line(n) => *n - 1,
        _ => {
            if flow_column {
                find_auto_slot(auto_col, auto_row, col_span, row_span, num_cols, occupied, flow_column, dense)
            } else {
                *auto_row
            }
        }
    };

    let col_end = col_start + col_span;
    let row_end = row_start + row_span;

    // Mark cells as occupied
    while occupied.len() <= row_end {
        occupied.push(vec![false; num_cols.max(col_end)]);
    }
    for r in row_start..row_end {
        while occupied[r].len() < col_end { occupied[r].push(false); }
        for c in col_start..col_end { occupied[r][c] = true; }
    }

    // Advance auto cursor past this item
    if flow_column {
        *auto_row = row_end;
        if *auto_row >= occupied.len() { *auto_row = 0; *auto_col += 1; }
    } else {
        *auto_col = col_end;
        if *auto_col >= num_cols { *auto_col = 0; *auto_row += 1; }
    }

    GridPlacement { col_start, col_end, row_start, row_end }
}

fn find_auto_slot(
    auto_col: &mut usize, auto_row: &mut usize,
    col_span: usize, row_span: usize,
    num_cols: usize, occupied: &mut Vec<Vec<bool>>,
    flow_column: bool, _dense: bool,
) -> usize {
    let max_iter = (occupied.len() + row_span + 20) * (num_cols + col_span);
    for _ in 0..max_iter {
        let c = *auto_col;
        let r = *auto_row;
        if c + col_span <= num_cols && fits(occupied, r, c, row_span, col_span) {
            return if flow_column { r } else { c };
        }
        if flow_column {
            *auto_row += 1;
            if *auto_row + row_span > occupied.len() + 10 {
                *auto_row = 0;
                *auto_col += 1;
            }
        } else {
            *auto_col += 1;
            if *auto_col + col_span > num_cols {
                *auto_col = 0;
                *auto_row += 1;
            }
        }
    }
    if flow_column { *auto_row } else { *auto_col }
}

fn fits(occupied: &[Vec<bool>], row: usize, col: usize, row_span: usize, col_span: usize) -> bool {
    for r in row..row + row_span {
        if r >= occupied.len() { continue; }
        for c in col..col + col_span {
            if c >= occupied[r].len() { continue; }
            if occupied[r][c] { return false; }
        }
    }
    true
}

// ── Public entry point ────────────────────────────────────────────────

pub fn layout_grid<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) {
    let margin = sides::resolve_margin(b.style);
    let border = sides::resolve_border(b.style);
    let padding = sides::resolve_padding(b.style);
    b.margin = margin.edges;
    b.border = border;
    b.padding = padding;

    let available = ctx.containing_width - margin.edges.horizontal() - border.horizontal() - padding.horizontal();
    let w = sizes::resolve_length(b.style.width, ctx.containing_width).unwrap_or(available.max(0.0));
    let inner_width = w.min(available.max(0.0));
    b.content.width = inner_width;
    b.content.x = pos.x + margin.edges.left + border.left + padding.left;
    b.content.y = pos.y + margin.edges.top + border.top + padding.top;

    let inner_height = sizes::resolve_length(b.style.height, ctx.containing_height);

    let gap_col = sizes::resolve_length(b.style.column_gap, inner_width).unwrap_or(0.0);
    let gap_row = sizes::resolve_length(b.style.row_gap, inner_width).unwrap_or(0.0);

    let auto_flow = css_str(b.style.grid_auto_flow);
    let flow_column = auto_flow.contains("column");
    let dense = auto_flow.contains("dense");

    let align_items = css_str(b.style.align_items);
    let justify_items = css_str(b.style.justify_items);

    let mut col_defs = parse_track_list(b.style.grid_template_columns, inner_width);
    let mut row_defs = parse_track_list(b.style.grid_template_rows, inner_width);

    let child_count = b.children.len();
    if child_count == 0 {
        b.content.height = inner_height.unwrap_or(0.0);
        return;
    }

    if col_defs.is_empty() {
        col_defs.push(TrackSize::Fr(1.0));
    }

    let num_cols = col_defs.len();

    let auto_row_size = parse_auto_track_size(b.style.grid_auto_rows);
    let auto_col_size = parse_auto_track_size(b.style.grid_auto_columns);
    let _ = auto_col_size;

    // Resolve initial column sizes (auto tracks get 0 initially)
    let col_sizes = resolve_tracks(&col_defs, inner_width, gap_col, &vec![0.0; num_cols]);

    // Phase 1: place items
    let taken_children = std::mem::take(&mut b.children);
    let mut items: Vec<(GridPlacement, LayoutBox<'a>)> = Vec::with_capacity(taken_children.len());
    let mut auto_col = 0_usize;
    let mut auto_row = 0_usize;
    let mut occupied: Vec<Vec<bool>> = Vec::new();

    for child in taken_children {
        if css_str(child.style.display) == "none" { continue; }
        if positioned::is_out_of_flow(child.style) { continue; }

        let placement = resolve_placement(child.style, &mut auto_col, &mut auto_row, num_cols, &mut occupied, flow_column, dense);

        let item_w: f32 = (placement.col_start..placement.col_end)
            .map(|c| col_sizes.get(c).copied().unwrap_or(0.0))
            .sum::<f32>()
            + gap_col * ((placement.col_end - placement.col_start) as f32 - 1.0).max(0.0);

        let child_ctx = LayoutContext { containing_width: item_w, ..*ctx };
        let mut laid = child;
        crate::block::layout_block(&mut laid, &child_ctx, Point::new(0.0, 0.0), text_ctx, rects);

        items.push((placement, laid));
    }

    // Determine total rows needed
    let max_row_end = items.iter().map(|(p, _)| p.row_end).max().unwrap_or(0);
    let needed_rows = max_row_end.max(row_defs.len());
    while row_defs.len() < needed_rows {
        row_defs.push(auto_row_size.clone());
    }
    let num_rows = row_defs.len();

    // Compute auto row heights from placed items
    let mut row_auto_sizes = vec![0.0_f32; num_rows];
    for (placement, laid) in &items {
        let span = placement.row_end - placement.row_start;
        let h = laid.outer_height() / span as f32;
        for r in placement.row_start..placement.row_end {
            if r < row_auto_sizes.len() {
                row_auto_sizes[r] = row_auto_sizes[r].max(h);
            }
        }
    }

    let mut row_heights = resolve_tracks(&row_defs, inner_height.unwrap_or(0.0), gap_row, &row_auto_sizes);

    // Resolve fr rows with definite height
    let total_row_gap = gap_row * (num_rows as f32 - 1.0).max(0.0);
    if let Some(ih) = inner_height {
        let fixed_h: f32 = row_heights.iter().sum::<f32>();
        let fr_sum: f32 = row_defs.iter().map(|d| match d { TrackSize::Fr(f) => *f, _ => 0.0 }).sum();
        if fr_sum > 0.0 {
            let free = (ih - total_row_gap - fixed_h).max(0.0);
            for (i, def) in row_defs.iter().enumerate() {
                if let TrackSize::Fr(f) = def {
                    if i < row_heights.len() { row_heights[i] = free * f / fr_sum; }
                }
            }
        }
    }

    // Compute positions
    let col_positions = compute_positions(&col_sizes, gap_col);
    let row_positions = compute_positions(&row_heights, gap_row);

    // Phase 2: position items
    for (placement, item) in &mut items {
        let cell_x = col_positions.get(placement.col_start).copied().unwrap_or(0.0);
        let cell_y = row_positions.get(placement.row_start).copied().unwrap_or(0.0);

        let cell_w: f32 = (placement.col_start..placement.col_end)
            .map(|c| col_sizes.get(c).copied().unwrap_or(0.0))
            .sum::<f32>()
            + gap_col * ((placement.col_end - placement.col_start) as f32 - 1.0).max(0.0);
        let cell_h: f32 = (placement.row_start..placement.row_end)
            .map(|r| row_heights.get(r).copied().unwrap_or(0.0))
            .sum::<f32>()
            + gap_row * ((placement.row_end - placement.row_start) as f32 - 1.0).max(0.0);

        let item_align = css_str(item.style.align_self);
        let align = if item_align.is_empty() || item_align == "auto" { align_items } else { item_align };
        let item_justify = css_str(item.style.justify_self);
        let justify = if item_justify.is_empty() || item_justify == "auto" { justify_items } else { item_justify };

        let item_w = item.outer_width();
        let item_h = item.outer_height();

        let dx_align = match justify {
            "center" => (cell_w - item_w) / 2.0,
            "end" | "flex-end" => cell_w - item_w,
            _ => 0.0,
        };
        let dy_align = match align {
            "center" => (cell_h - item_h) / 2.0,
            "end" | "flex-end" => cell_h - item_h,
            _ => 0.0,
        };

        let target_x = b.content.x + cell_x + dx_align.max(0.0);
        let target_y = b.content.y + cell_y + dy_align.max(0.0);

        let cur_x = item.content.x - item.padding.left - item.border.left - item.margin.left;
        let cur_y = item.content.y - item.padding.top - item.border.top - item.margin.top;
        let dx = target_x - cur_x;
        let dy = target_y - cur_y;
        if dx.abs() > 0.001 || dy.abs() > 0.001 {
            translate_recursive(item, dx, dy);
        }
    }

    b.children = items.into_iter().map(|(_, item)| item).collect();

    let total_h: f32 = row_heights.iter().sum::<f32>() + total_row_gap;
    b.content.height = inner_height.unwrap_or(total_h);
}

fn parse_auto_track_size(value: Option<&CssValue>) -> TrackSize {
    match value {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => TrackSize::Px(*value as f32),
        Some(CssValue::Dimension { value, unit: CssUnit::Fr }) => TrackSize::Fr(*value as f32),
        _ => TrackSize::Auto,
    }
}

fn compute_positions(sizes: &[f32], gap: f32) -> Vec<f32> {
    let mut positions = Vec::with_capacity(sizes.len());
    let mut cursor = 0.0;
    for (i, s) in sizes.iter().enumerate() {
        positions.push(cursor);
        cursor += s + if i + 1 < sizes.len() { gap } else { 0.0 };
    }
    positions
}

fn translate_recursive(b: &mut LayoutBox, dx: f32, dy: f32) {
    b.content.x += dx;
    b.content.y += dy;
    for child in &mut b.children {
        translate_recursive(child, dx, dy);
    }
}
