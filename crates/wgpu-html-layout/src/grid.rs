//! Grid formatting context.
//!
//! Implements CSS-Grid-Layout-1 §6–§11 at a level adequate for most
//! everyday grid layouts:
//!
//! - `display: grid` / `display: inline-grid`
//! - `grid-template-columns` / `grid-template-rows` with `<length>` /
//!   `<percent>` / `auto` / `<flex>` (`fr`) tracks plus `repeat(<int>,
//!   <list>)` expansion
//! - `grid-auto-rows` / `grid-auto-columns` for implicit tracks
//! - `grid-auto-flow: row | column` (the dense variants are accepted
//!   but lay out the same as their non-dense counterparts for now)
//! - Explicit placement via `grid-column-start/end`,
//!   `grid-row-start/end` and the `grid-column` / `grid-row`
//!   shorthands (line numbers and `span <n>`)
//! - Auto-placement of items with no explicit placement, in source
//!   order
//! - `gap` / `row-gap` / `column-gap`
//! - `justify-items` / `align-items` (default cell anchoring),
//!   `justify-self` / `align-self` (per-item override),
//!   `justify-content` / `align-content` (track block distribution
//!   when sized below the container)
//!
//! Deferred (see `spec/grid.md` for the full list): `minmax()`
//! two-bound clamping, `min-content` / `max-content` / `fit-content`
//! sizing, `repeat(auto-fill | auto-fit, …)`, `dense` packing,
//! `grid-template-areas`, named grid lines, negative line numbers,
//! subgrid, baseline alignment.

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::{
    AlignContent, AlignItems, AlignSelf, CssLength, GridAutoFlow, GridLine, GridTrackSize,
    JustifyContent, JustifyItems, JustifySelf,
};
use wgpu_html_style::CascadedNode;

use crate::{
    BlockOverrides, Ctx, LayoutBox, layout_block_at_with, length, translate_box_x_in_place,
    translate_box_y_in_place,
};

const EPS: f32 = 0.001;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Lay grid children out and return them positioned at absolute
/// pixel coordinates plus the container's used inline / block size.
pub(crate) fn layout_grid_children(
    parent: &CascadedNode,
    parent_style: &Style,
    content_x: f32,
    content_y: f32,
    inner_width: f32,
    inner_height_explicit: Option<f32>,
    ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
    let auto_flow = parent_style
        .grid_auto_flow
        .clone()
        .unwrap_or(GridAutoFlow::Row);
    let column_first = matches!(
        auto_flow,
        GridAutoFlow::Column | GridAutoFlow::ColumnDense
    );

    // Gaps. Per-axis longhands win over `gap`; otherwise default to 0.
    let gap_col = length::resolve(
        parent_style
            .column_gap
            .as_ref()
            .or(parent_style.gap.as_ref()),
        inner_width,
        ctx,
    )
    .unwrap_or(0.0);
    let gap_row = length::resolve(
        parent_style.row_gap.as_ref().or(parent_style.gap.as_ref()),
        inner_width,
        ctx,
    )
    .unwrap_or(0.0);

    // Default cell-axis alignment, falling through to stretch.
    let default_align_items = parent_style
        .align_items
        .clone()
        .unwrap_or(AlignItems::Stretch);
    let default_justify_items = parent_style
        .justify_items
        .clone()
        .unwrap_or(JustifyItems::Stretch);

    // Container distribution along each axis.
    let justify_content = parent_style
        .justify_content
        .clone()
        .unwrap_or(JustifyContent::Start);
    let align_content = parent_style
        .align_content
        .clone()
        .unwrap_or(AlignContent::Start);

    // -----------------------------------------------------------------
    // Phase 1: build raw items.
    // -----------------------------------------------------------------
    let mut items: Vec<GridItem> = Vec::with_capacity(parent.children.len());
    for (idx, child) in parent.children.iter().enumerate() {
        if matches!(
            child.style.display,
            Some(wgpu_html_models::common::css_enums::Display::None)
        ) {
            continue;
        }
        items.push(GridItem::from_node(child, idx));
    }
    if items.is_empty() {
        return (Vec::new(), 0.0, 0.0);
    }

    // -----------------------------------------------------------------
    // Phase 2: explicit grid templates.
    // -----------------------------------------------------------------
    let explicit_cols: Vec<GridTrackSize> = parent_style
        .grid_template_columns
        .clone()
        .unwrap_or_default();
    let explicit_rows: Vec<GridTrackSize> = parent_style
        .grid_template_rows
        .clone()
        .unwrap_or_default();
    let auto_col_size = parent_style
        .grid_auto_columns
        .clone()
        .unwrap_or(GridTrackSize::Auto);
    let auto_row_size = parent_style
        .grid_auto_rows
        .clone()
        .unwrap_or(GridTrackSize::Auto);

    let mut tracks_x: Vec<GridTrackSize> = explicit_cols.clone();
    let mut tracks_y: Vec<GridTrackSize> = explicit_rows.clone();

    // -----------------------------------------------------------------
    // Phase 3: auto-place items into a 2D grid.
    // -----------------------------------------------------------------
    place_items(
        &mut items,
        &mut tracks_x,
        &mut tracks_y,
        &auto_col_size,
        &auto_row_size,
        column_first,
    );

    // Ensure at least one track each axis (an empty grid is degenerate
    // but harmless).
    if tracks_x.is_empty() {
        tracks_x.push(auto_col_size.clone());
    }
    if tracks_y.is_empty() {
        tracks_y.push(auto_row_size.clone());
    }

    // -----------------------------------------------------------------
    // Phase 4: resolve column widths.
    // -----------------------------------------------------------------
    let col_widths = resolve_track_sizes(
        &tracks_x,
        &items,
        Axis::Inline,
        inner_width,
        gap_col,
        ctx,
        parent,
    );

    // -----------------------------------------------------------------
    // Phase 5: lay each item out to discover its block-axis extent.
    //
    // Items that will stretch across the inline cell extent get
    // `BlockOverrides { width: Some(span_w), … }` so their content
    // wraps at the cell width. Items with their own `width` (or with
    // `justify-self` other than stretch) lay out at their natural
    // size, so phase 7 can position them inside the cell.
    // -----------------------------------------------------------------
    for item in items.iter_mut() {
        let span_w = sum_track_extent(&col_widths, item.col_start, item.col_end, gap_col);
        let stretch_inline = !item.has_explicit_inline_size
            && matches!(
                resolve_justify_self(item.justify_self.as_ref(), &default_justify_items),
                ResolvedAxis::Stretch
            );
        let overrides = if stretch_inline {
            BlockOverrides {
                width: Some(span_w),
                height: None,
            }
        } else {
            BlockOverrides::default()
        };
        let laid = layout_block_at_with(
            item.node,
            0.0,
            0.0,
            inner_width,
            inner_height_explicit.unwrap_or(0.0),
            overrides,
            ctx,
        );
        item.measured_h = laid.margin_rect.h;
        item.measured_w = laid.margin_rect.w;
        item.box_ = Some(laid);
    }

    // -----------------------------------------------------------------
    // Phase 6: resolve row heights using the measurements above.
    // -----------------------------------------------------------------
    let block_axis_extent = inner_height_explicit.unwrap_or(0.0);
    let row_heights = resolve_track_sizes(
        &tracks_y,
        &items,
        Axis::Block,
        block_axis_extent,
        gap_row,
        ctx,
        parent,
    );

    // -----------------------------------------------------------------
    // Track positions (cumulative sums + gaps + axis-content offset).
    // -----------------------------------------------------------------
    let total_col_w =
        col_widths.iter().sum::<f32>() + gap_col * (col_widths.len() as f32 - 1.0).max(0.0);
    let total_row_h =
        row_heights.iter().sum::<f32>() + gap_row * (row_heights.len() as f32 - 1.0).max(0.0);

    // Phase 8 baked in: distribute remaining inline / block space via
    // justify-content / align-content. (Only meaningful when the
    // axis is definite and there's free space; otherwise these are
    // no-ops.)
    let inline_free = (inner_width - total_col_w).max(0.0);
    let (inline_start_offset, inline_extra_between) =
        track_distribution(&justify_content, inline_free, col_widths.len() as f32);
    let block_free = match inner_height_explicit {
        Some(h) => (h - total_row_h).max(0.0),
        None => 0.0,
    };
    let (block_start_offset, block_extra_between) =
        align_content_distribution(&align_content, block_free, row_heights.len() as f32);

    let col_positions = cumulative_track_positions(&col_widths, gap_col, inline_extra_between);
    let row_positions = cumulative_track_positions(&row_heights, gap_row, block_extra_between);

    // -----------------------------------------------------------------
    // Phase 7: place items into cells.
    // -----------------------------------------------------------------
    let mut placed: Vec<(usize, LayoutBox)> = Vec::with_capacity(items.len());
    for item in items.into_iter() {
        let cell_x = content_x + inline_start_offset + col_positions[item.col_start];
        let cell_y = content_y + block_start_offset + row_positions[item.row_start];
        let cell_w = sum_track_extent(&col_widths, item.col_start, item.col_end, gap_col);
        let cell_h = sum_track_extent(&row_heights, item.row_start, item.row_end, gap_row);

        let justify = resolve_justify_self(
            item.justify_self.as_ref(),
            &default_justify_items,
        );
        let align = resolve_align_self(item.align_self.as_ref(), &default_align_items);

        let stretch_x = matches!(justify, ResolvedAxis::Stretch)
            && !item.has_explicit_inline_size;
        let stretch_y =
            matches!(align, ResolvedAxis::Stretch) && !item.has_explicit_block_size;

        // Re-lay the item if the placement requires stretching to the
        // cell on either axis. Otherwise reuse the box from phase 5.
        let mut child_box = if stretch_x || stretch_y {
            let target_w = if stretch_x { Some(cell_w) } else { Some(item.measured_w) };
            let target_h = if stretch_y { Some(cell_h) } else { None };
            layout_block_at_with(
                item.node,
                0.0,
                0.0,
                inner_width,
                inner_height_explicit.unwrap_or(cell_h),
                BlockOverrides {
                    width: target_w,
                    height: target_h,
                },
                ctx,
            )
        } else {
            item.box_.expect("item box was laid out in phase 5")
        };

        let used_w = child_box.margin_rect.w;
        let used_h = child_box.margin_rect.h;
        let dx_within_cell = match justify {
            ResolvedAxis::Start => 0.0,
            ResolvedAxis::Center => (cell_w - used_w) * 0.5,
            ResolvedAxis::End => (cell_w - used_w).max(0.0),
            ResolvedAxis::Stretch => 0.0,
        };
        let dy_within_cell = match align {
            ResolvedAxis::Start => 0.0,
            ResolvedAxis::Center => (cell_h - used_h) * 0.5,
            ResolvedAxis::End => (cell_h - used_h).max(0.0),
            ResolvedAxis::Stretch => 0.0,
        };

        let dx = (cell_x + dx_within_cell) - child_box.margin_rect.x;
        let dy = (cell_y + dy_within_cell) - child_box.margin_rect.y;
        if dx != 0.0 {
            translate_box_x_in_place(&mut child_box, dx);
        }
        if dy != 0.0 {
            translate_box_y_in_place(&mut child_box, dy);
        }
        placed.push((item.source_index, child_box));
    }

    // Restore source order so hit-testing matches the DOM.
    placed.sort_by_key(|(idx, _)| *idx);
    let final_boxes: Vec<LayoutBox> = placed.into_iter().map(|(_, b)| b).collect();

    let used_inline = total_col_w;
    let used_block = total_row_h;
    (final_boxes, used_inline, used_block)
}

// ---------------------------------------------------------------------------
// Per-item state
// ---------------------------------------------------------------------------

/// One grid item: cascaded node + its placement coordinates and
/// measured sizes through the algorithm. Coordinates are 0-based
/// half-open ranges over the resolved track lists (`col_end` is
/// exclusive, so `colspan` 1 → `col_end - col_start = 1`).
struct GridItem<'a> {
    node: &'a CascadedNode,
    source_index: usize,
    /// Original placement directives (still in their CSS form) so the
    /// auto-placement pass can normalize them.
    raw: RawPlacement,
    col_start: usize,
    col_end: usize,
    row_start: usize,
    row_end: usize,
    measured_w: f32,
    measured_h: f32,
    has_explicit_inline_size: bool,
    has_explicit_block_size: bool,
    justify_self: Option<JustifySelf>,
    align_self: Option<AlignSelf>,
    box_: Option<LayoutBox>,
}

#[derive(Clone)]
struct RawPlacement {
    col_start: GridLine,
    col_end: GridLine,
    row_start: GridLine,
    row_end: GridLine,
}

impl<'a> GridItem<'a> {
    fn from_node(node: &'a CascadedNode, source_index: usize) -> Self {
        let s = &node.style;
        let raw = RawPlacement {
            col_start: s.grid_column_start.clone().unwrap_or(GridLine::Auto),
            col_end: s.grid_column_end.clone().unwrap_or(GridLine::Auto),
            row_start: s.grid_row_start.clone().unwrap_or(GridLine::Auto),
            row_end: s.grid_row_end.clone().unwrap_or(GridLine::Auto),
        };
        let has_explicit_inline_size = is_definite_length(s.width.as_ref());
        let has_explicit_block_size = is_definite_length(s.height.as_ref());
        Self {
            node,
            source_index,
            raw,
            col_start: 0,
            col_end: 1,
            row_start: 0,
            row_end: 1,
            measured_w: 0.0,
            measured_h: 0.0,
            has_explicit_inline_size,
            has_explicit_block_size,
            justify_self: s.justify_self.clone(),
            align_self: s.align_self.clone(),
            box_: None,
        }
    }
}

fn is_definite_length(len: Option<&CssLength>) -> bool {
    match len {
        None => false,
        Some(CssLength::Auto) | Some(CssLength::Raw(_)) => false,
        Some(_) => true,
    }
}

// ---------------------------------------------------------------------------
// Auto-placement
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Axis {
    Inline,
    Block,
}

/// Resolve every item's `(col_start, col_end, row_start, row_end)`.
///
/// Strategy (CSS-Grid-Layout-1 §8, simplified):
/// 1. Resolve span sizes for each axis (default span 1).
/// 2. Process items with both axes definite first — they pin the
///    grid's lower bound on each axis.
/// 3. Process items with only one axis definite, placing them at the
///    next free row / column on the opposite axis.
/// 4. Process items with both axes auto using the auto-flow cursor.
///
/// Implicit tracks (beyond the explicit template) are appended on
/// demand and inherit `grid-auto-rows` / `grid-auto-columns`.
fn place_items(
    items: &mut [GridItem],
    tracks_x: &mut Vec<GridTrackSize>,
    tracks_y: &mut Vec<GridTrackSize>,
    auto_col: &GridTrackSize,
    auto_row: &GridTrackSize,
    column_first: bool,
) {
    // Phase 1: compute each item's tentative span on each axis.
    let mut tentative: Vec<TentativePlacement> = items
        .iter()
        .map(|it| TentativePlacement::from_raw(&it.raw))
        .collect();

    // Phase 2: occupancy grid. Grows as we go.
    let mut grid = Occupancy::new();

    // Pass A: place items with both axes definite (line numbers).
    for (i, t) in tentative.iter_mut().enumerate() {
        if let (Some(c), Some(r)) = (t.column_range_if_definite(), t.row_range_if_definite()) {
            grow_tracks(tracks_x, c.1, auto_col);
            grow_tracks(tracks_y, r.1, auto_row);
            grid.mark(c.0..c.1, r.0..r.1);
            items[i].col_start = c.0;
            items[i].col_end = c.1;
            items[i].row_start = r.0;
            items[i].row_end = r.1;
            t.placed = true;
        }
    }

    // Pass B + C: items with one or both axes auto. Use the auto-flow
    // cursor. We walk the cursor row-major or column-major as
    // requested by `grid-auto-flow`.
    let mut cursor_col: usize = 0;
    let mut cursor_row: usize = 0;

    for (i, t) in tentative.iter_mut().enumerate() {
        if t.placed {
            continue;
        }
        let col_span = t.col_span;
        let row_span = t.row_span;

        let placement = if column_first {
            place_column_major(
                t,
                col_span,
                row_span,
                tracks_x.len(),
                tracks_y.len(),
                &mut cursor_col,
                &mut cursor_row,
                &grid,
            )
        } else {
            place_row_major(
                t,
                col_span,
                row_span,
                tracks_x.len(),
                tracks_y.len(),
                &mut cursor_col,
                &mut cursor_row,
                &grid,
            )
        };

        let (cs, ce, rs, re) = placement;
        grow_tracks(tracks_x, ce, auto_col);
        grow_tracks(tracks_y, re, auto_row);
        grid.mark(cs..ce, rs..re);
        items[i].col_start = cs;
        items[i].col_end = ce;
        items[i].row_start = rs;
        items[i].row_end = re;
    }
}

/// Inflate a track list to at least `min_len` entries by appending
/// the supplied implicit-track template.
fn grow_tracks(tracks: &mut Vec<GridTrackSize>, min_len: usize, fill: &GridTrackSize) {
    while tracks.len() < min_len {
        tracks.push(fill.clone());
    }
}

#[derive(Default)]
struct Occupancy {
    /// Bit-per-cell flat storage. Indexed `row * cols + col`.
    bits: Vec<bool>,
    cols: usize,
    rows: usize,
}

impl Occupancy {
    fn new() -> Self {
        Self::default()
    }

    fn ensure(&mut self, cols: usize, rows: usize) {
        if cols <= self.cols && rows <= self.rows {
            return;
        }
        let new_cols = cols.max(self.cols).max(1);
        let new_rows = rows.max(self.rows).max(1);
        let mut new_bits = vec![false; new_cols * new_rows];
        for r in 0..self.rows {
            for c in 0..self.cols {
                if self.bits[r * self.cols + c] {
                    new_bits[r * new_cols + c] = true;
                }
            }
        }
        self.bits = new_bits;
        self.cols = new_cols;
        self.rows = new_rows;
    }

    fn is_free(&self, c0: usize, c1: usize, r0: usize, r1: usize) -> bool {
        if c1 > self.cols || r1 > self.rows {
            return true;
        }
        for r in r0..r1 {
            for c in c0..c1 {
                if self.bits[r * self.cols + c] {
                    return false;
                }
            }
        }
        true
    }

    fn mark(&mut self, cols: std::ops::Range<usize>, rows: std::ops::Range<usize>) {
        self.ensure(cols.end, rows.end);
        for r in rows.clone() {
            for c in cols.clone() {
                self.bits[r * self.cols + c] = true;
            }
        }
    }
}

#[derive(Clone)]
struct TentativePlacement {
    col_start: GridLine,
    col_end: GridLine,
    row_start: GridLine,
    row_end: GridLine,
    col_span: usize,
    row_span: usize,
    placed: bool,
}

impl TentativePlacement {
    fn from_raw(raw: &RawPlacement) -> Self {
        let col_span = effective_span(&raw.col_start, &raw.col_end);
        let row_span = effective_span(&raw.row_start, &raw.row_end);
        Self {
            col_start: raw.col_start.clone(),
            col_end: raw.col_end.clone(),
            row_start: raw.row_start.clone(),
            row_end: raw.row_end.clone(),
            col_span,
            row_span,
            placed: false,
        }
    }

    /// Resolve the column range when both ends are definite line
    /// numbers (or one is a span anchored to a definite line).
    fn column_range_if_definite(&self) -> Option<(usize, usize)> {
        resolve_axis_range(&self.col_start, &self.col_end)
    }

    fn row_range_if_definite(&self) -> Option<(usize, usize)> {
        resolve_axis_range(&self.row_start, &self.row_end)
    }
}

/// `span N` defaults to 1 when both ends are auto. A `span N` next to
/// a definite line gives N. Two definite lines give `|end - start|`.
fn effective_span(start: &GridLine, end: &GridLine) -> usize {
    match (start, end) {
        (GridLine::Span(n), GridLine::Auto) | (GridLine::Auto, GridLine::Span(n)) => {
            (*n as usize).max(1)
        }
        (GridLine::Span(n), _) | (_, GridLine::Span(n)) => (*n as usize).max(1),
        (GridLine::Line(a), GridLine::Line(b)) => (b - a).unsigned_abs() as usize,
        _ => 1,
    }
}

/// Convert two definite line numbers into a 0-based half-open range
/// over track indices. Returns `None` if either end is auto / span.
fn resolve_axis_range(start: &GridLine, end: &GridLine) -> Option<(usize, usize)> {
    let s = match start {
        GridLine::Line(n) if *n >= 1 => Some((*n as usize) - 1),
        _ => None,
    };
    let e = match end {
        GridLine::Line(n) if *n >= 1 => Some((*n as usize) - 1),
        GridLine::Span(n) => s.map(|s| s + (*n as usize).max(1)),
        _ => None,
    };
    match (s, e) {
        (Some(a), Some(b)) => {
            let lo = a.min(b);
            let hi = a.max(b);
            // Spec: same line → empty span counts as 1.
            let hi = if hi == lo { hi + 1 } else { hi };
            Some((lo, hi))
        }
        _ => None,
    }
}

/// Row-major auto-placement: scan rows top-to-bottom, columns
/// left-to-right, looking for the first free `(col_span × row_span)`
/// rectangle.
#[allow(clippy::too_many_arguments)]
fn place_row_major(
    t: &TentativePlacement,
    col_span: usize,
    row_span: usize,
    explicit_cols: usize,
    _explicit_rows: usize,
    cursor_col: &mut usize,
    cursor_row: &mut usize,
    grid: &Occupancy,
) -> (usize, usize, usize, usize) {
    let span_c = col_span.max(1);
    let span_r = row_span.max(1);

    // Locked-column case: column is definite, row is auto.
    if let Some((cs, ce)) = t.column_range_if_definite() {
        let mut r = *cursor_row;
        loop {
            if grid.is_free(cs, ce, r, r + span_r) {
                *cursor_row = r;
                return (cs, ce, r, r + span_r);
            }
            r += 1;
        }
    }

    // Locked-row case: row is definite, column is auto.
    if let Some((rs, re)) = t.row_range_if_definite() {
        let mut c = 0usize;
        let cols = explicit_cols.max(span_c);
        loop {
            let ce = c + span_c;
            if ce <= cols && grid.is_free(c, ce, rs, re) {
                return (c, ce, rs, re);
            }
            c += 1;
            if c + span_c > cols {
                // Implicit columns — keep extending.
                return (c, c + span_c, rs, re);
            }
        }
    }

    // Both auto: row-major sweep starting at the cursor.
    let mut r = *cursor_row;
    let mut c = *cursor_col;
    loop {
        let cols = explicit_cols.max(span_c);
        if c + span_c > cols {
            // Wrap to next row.
            r += 1;
            c = 0;
        }
        if grid.is_free(c, c + span_c, r, r + span_r) {
            *cursor_col = c + span_c;
            *cursor_row = r;
            return (c, c + span_c, r, r + span_r);
        }
        c += 1;
    }
}

/// Column-major sweep, mirror of `place_row_major`.
#[allow(clippy::too_many_arguments)]
fn place_column_major(
    t: &TentativePlacement,
    col_span: usize,
    row_span: usize,
    _explicit_cols: usize,
    explicit_rows: usize,
    cursor_col: &mut usize,
    cursor_row: &mut usize,
    grid: &Occupancy,
) -> (usize, usize, usize, usize) {
    let span_c = col_span.max(1);
    let span_r = row_span.max(1);

    if let Some((rs, re)) = t.row_range_if_definite() {
        let mut c = *cursor_col;
        loop {
            if grid.is_free(c, c + span_c, rs, re) {
                *cursor_col = c;
                return (c, c + span_c, rs, re);
            }
            c += 1;
        }
    }
    if let Some((cs, ce)) = t.column_range_if_definite() {
        let mut r = 0usize;
        let rows = explicit_rows.max(span_r);
        loop {
            let re = r + span_r;
            if re <= rows && grid.is_free(cs, ce, r, re) {
                return (cs, ce, r, re);
            }
            r += 1;
            if r + span_r > rows {
                return (cs, ce, r, r + span_r);
            }
        }
    }

    let mut c = *cursor_col;
    let mut r = *cursor_row;
    loop {
        let rows = explicit_rows.max(span_r);
        if r + span_r > rows {
            c += 1;
            r = 0;
        }
        if grid.is_free(c, c + span_c, r, r + span_r) {
            *cursor_col = c;
            *cursor_row = r + span_r;
            return (c, c + span_c, r, r + span_r);
        }
        r += 1;
    }
}

// ---------------------------------------------------------------------------
// Track sizing
// ---------------------------------------------------------------------------

/// Resolve a track list to concrete pixel sizes. Fixed lengths
/// (`px`/`%`/`em`/etc.) resolve straight away, `auto` tracks pick up
/// the max-content of items spanning *only* auto tracks (multi-track
/// auto items distribute their measured size across spanned auto
/// tracks), and `<flex>` (`fr`) tracks divide whatever inline /
/// block extent remains.
fn resolve_track_sizes(
    tracks: &[GridTrackSize],
    items: &[GridItem],
    axis: Axis,
    container_extent: f32,
    gap: f32,
    ctx: &mut Ctx,
    parent: &CascadedNode,
) -> Vec<f32> {
    let n = tracks.len();
    let mut sizes = vec![0.0_f32; n];
    let mut is_fr = vec![false; n];
    let mut fr_factor = vec![0.0_f32; n];
    let mut is_auto = vec![false; n];

    // Pass 1: fixed lengths and tagging.
    for (i, t) in tracks.iter().enumerate() {
        match t {
            GridTrackSize::Length(len) => {
                sizes[i] = length::resolve(Some(len), container_extent, ctx).unwrap_or(0.0);
            }
            GridTrackSize::Auto => {
                is_auto[i] = true;
            }
            GridTrackSize::Fr(f) => {
                is_fr[i] = true;
                fr_factor[i] = (*f).max(0.0);
            }
        }
    }

    // Pass 2: auto track contributions from items.
    if is_auto.iter().any(|&b| b) {
        let measurements = measure_items(items, axis, ctx, parent);
        for (idx, item) in items.iter().enumerate() {
            let (start, end) = match axis {
                Axis::Inline => (item.col_start, item.col_end),
                Axis::Block => (item.row_start, item.row_end),
            };
            // Track which spanned tracks are auto-typed.
            let auto_indices: Vec<usize> = (start..end).filter(|&i| is_auto[i]).collect();
            if auto_indices.is_empty() {
                continue;
            }
            let measure = measurements[idx];
            // Subtract fixed track contributions and gaps from the
            // item's measurement: the auto tracks need to cover only
            // what's left.
            let span_len = end - start;
            let fixed_sum: f32 = (start..end).filter(|&i| !is_auto[i]).map(|i| sizes[i]).sum();
            let gaps = gap * (span_len as f32 - 1.0).max(0.0);
            let needed = (measure - fixed_sum - gaps).max(0.0);
            // Distribute proportionally to current auto track sizes,
            // or uniformly if all are still 0.
            let current_total: f32 = auto_indices.iter().map(|&i| sizes[i]).sum();
            if current_total <= EPS {
                let per = needed / auto_indices.len() as f32;
                for &i in &auto_indices {
                    if per > sizes[i] {
                        sizes[i] = per;
                    }
                }
            } else {
                for &i in &auto_indices {
                    let share = needed * (sizes[i] / current_total);
                    if share > sizes[i] {
                        sizes[i] = share;
                    }
                }
            }
        }
    }

    // Pass 3: distribute remainder to `fr` tracks.
    if is_fr.iter().any(|&b| b) {
        let used: f32 =
            sizes.iter().sum::<f32>() + gap * (n as f32 - 1.0).max(0.0);
        let free = (container_extent - used).max(0.0);
        let total_factor: f32 = fr_factor.iter().sum();
        if total_factor > EPS && free > 0.0 {
            for i in 0..n {
                if is_fr[i] {
                    sizes[i] = free * (fr_factor[i] / total_factor);
                }
            }
        }
        // No fr factors? Leave at zero (matches the "0fr" CSS edge).
    }

    sizes
}

/// Measure each item's natural extent along `axis` (inline = max
/// content width when laid out unconstrained; block = content height
/// when laid out at the inline cell width).
fn measure_items(
    items: &[GridItem],
    axis: Axis,
    ctx: &mut Ctx,
    _parent: &CascadedNode,
) -> Vec<f32> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match axis {
            Axis::Inline => {
                // Unconstrained-width measurement: lay out at the
                // current ctx viewport so the engine has *some*
                // budget without forcing a width.
                let measure = layout_block_at_with(
                    item.node,
                    0.0,
                    0.0,
                    ctx.viewport_w,
                    ctx.viewport_h,
                    BlockOverrides::default(),
                    ctx,
                );
                out.push(measure.margin_rect.w);
            }
            Axis::Block => {
                // Block axis: items have already been laid out in
                // phase 5 at their assigned column widths. Reuse that
                // measurement.
                out.push(item.measured_h);
            }
        }
    }
    out
}

/// Sum of track sizes from `start..end` plus intervening gaps.
fn sum_track_extent(tracks: &[f32], start: usize, end: usize, gap: f32) -> f32 {
    if start >= end {
        return 0.0;
    }
    let span = (end - start) as f32;
    let total: f32 = tracks[start..end].iter().sum();
    total + gap * (span - 1.0).max(0.0)
}

/// Build cumulative offsets for each track, including gaps and any
/// extra distribution between tracks (e.g. from `space-between`).
fn cumulative_track_positions(
    sizes: &[f32],
    gap: f32,
    extra_between: f32,
) -> Vec<f32> {
    let mut out = Vec::with_capacity(sizes.len() + 1);
    let mut cursor = 0.0_f32;
    for (i, s) in sizes.iter().enumerate() {
        out.push(cursor);
        cursor += s;
        if i + 1 < sizes.len() {
            cursor += gap + extra_between;
        }
    }
    out.push(cursor);
    out
}

// ---------------------------------------------------------------------------
// Self / cell alignment
// ---------------------------------------------------------------------------

enum ResolvedAxis {
    Start,
    Center,
    End,
    Stretch,
}

fn resolve_align_self(item: Option<&AlignSelf>, parent: &AlignItems) -> ResolvedAxis {
    let effective = match item {
        Some(AlignSelf::Auto) | None => AlignItems_to_AlignSelf(parent),
        Some(other) => other.clone(),
    };
    match effective {
        AlignSelf::FlexStart | AlignSelf::Start => ResolvedAxis::Start,
        AlignSelf::FlexEnd | AlignSelf::End => ResolvedAxis::End,
        AlignSelf::Center => ResolvedAxis::Center,
        AlignSelf::Stretch | AlignSelf::Normal | AlignSelf::Auto => ResolvedAxis::Stretch,
        // Baseline isn't tracked yet; degrade to start.
        AlignSelf::Baseline => ResolvedAxis::Start,
    }
}

fn resolve_justify_self(
    item: Option<&JustifySelf>,
    parent: &JustifyItems,
) -> ResolvedAxis {
    let effective = match item {
        Some(JustifySelf::Auto) | None => JustifyItems_to_JustifySelf(parent),
        Some(other) => other.clone(),
    };
    match effective {
        JustifySelf::FlexStart | JustifySelf::Start | JustifySelf::Left => ResolvedAxis::Start,
        JustifySelf::FlexEnd | JustifySelf::End | JustifySelf::Right => ResolvedAxis::End,
        JustifySelf::Center => ResolvedAxis::Center,
        JustifySelf::Stretch | JustifySelf::Normal | JustifySelf::Auto => ResolvedAxis::Stretch,
        JustifySelf::Baseline => ResolvedAxis::Start,
    }
}

#[allow(non_snake_case)]
fn AlignItems_to_AlignSelf(p: &AlignItems) -> AlignSelf {
    match p {
        AlignItems::Normal => AlignSelf::Normal,
        AlignItems::Stretch => AlignSelf::Stretch,
        AlignItems::Center => AlignSelf::Center,
        AlignItems::Start => AlignSelf::Start,
        AlignItems::End => AlignSelf::End,
        AlignItems::FlexStart => AlignSelf::FlexStart,
        AlignItems::FlexEnd => AlignSelf::FlexEnd,
        AlignItems::Baseline => AlignSelf::Baseline,
    }
}

#[allow(non_snake_case)]
fn JustifyItems_to_JustifySelf(p: &JustifyItems) -> JustifySelf {
    match p {
        JustifyItems::Normal => JustifySelf::Normal,
        JustifyItems::Stretch => JustifySelf::Stretch,
        JustifyItems::Center => JustifySelf::Center,
        JustifyItems::Start => JustifySelf::Start,
        JustifyItems::End => JustifySelf::End,
        JustifyItems::FlexStart => JustifySelf::FlexStart,
        JustifyItems::FlexEnd => JustifySelf::FlexEnd,
        JustifyItems::Left => JustifySelf::Left,
        JustifyItems::Right => JustifySelf::Right,
        JustifyItems::Baseline => JustifySelf::Baseline,
    }
}

// ---------------------------------------------------------------------------
// Container distribution helpers (like flex's `distribution`)
// ---------------------------------------------------------------------------

/// Returns `(start_offset, between_extra)` for `justify-content` over
/// `n` tracks with `free` pixels left over.
fn track_distribution(justify: &JustifyContent, free: f32, n: f32) -> (f32, f32) {
    use JustifyContent::*;
    if free <= 0.0 || n <= 0.0 {
        return (0.0, 0.0);
    }
    match justify {
        Start | FlexStart | Left => (0.0, 0.0),
        End | FlexEnd | Right => (free, 0.0),
        Center => (free * 0.5, 0.0),
        SpaceBetween => {
            if n > 1.0 {
                (0.0, free / (n - 1.0))
            } else {
                (0.0, 0.0)
            }
        }
        SpaceAround => {
            let s = free / n;
            (s * 0.5, s)
        }
        SpaceEvenly => {
            let s = free / (n + 1.0);
            (s, s)
        }
    }
}

/// Same as `track_distribution` but accepts `align-content`.
fn align_content_distribution(align: &AlignContent, free: f32, n: f32) -> (f32, f32) {
    use AlignContent::*;
    if free <= 0.0 || n <= 0.0 {
        return (0.0, 0.0);
    }
    match align {
        Start | FlexStart | Normal | Stretch => (0.0, 0.0),
        End | FlexEnd => (free, 0.0),
        Center => (free * 0.5, 0.0),
        SpaceBetween => {
            if n > 1.0 {
                (0.0, free / (n - 1.0))
            } else {
                (0.0, 0.0)
            }
        }
        SpaceAround => {
            let s = free / n;
            (s * 0.5, s)
        }
        SpaceEvenly => {
            let s = free / (n + 1.0);
            (s, s)
        }
    }
}
