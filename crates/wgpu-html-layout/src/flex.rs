//! Flex formatting context.
//!
//! Implements CSS-Flexbox-1 (§9 "Flex Layout Algorithm") at the level
//! of fidelity needed for typical web pages:
//!
//! - `flex-direction` (row / row-reverse / column / column-reverse)
//! - `flex-wrap` (nowrap / wrap / wrap-reverse), multi-line lines
//! - `flex-grow` / `flex-shrink` / `flex-basis` (with min/max clamping
//!   and the canonical iterative freeze loop)
//! - `flex` shorthand (already expanded by the parser into the three
//!   longhands)
//! - `justify-content` (flex-start / flex-end / center / space-between
//!   / space-around / space-evenly, plus the start/end/left/right
//!   aliases)
//! - `align-items` and per-item `align-self`
//! - `align-content` for multi-line containers
//! - `order` (stable sort by order, then source index)
//! - `gap` / `row-gap` / `column-gap`
//! - `min-width` / `max-width` / `min-height` / `max-height` clamping
//!   on flex items (via the shared `clamp_axis` helper in `lib.rs`)
//! - `margin: auto` on flex items: absorbs free space on the main axis
//!   (split across all auto sides) and consumes leftover line cross
//!   space on the cross axis
//! - Baseline alignment falls back to `flex-start` for now (we can
//!   honour first-line baselines once a richer ascent/descent path is
//!   wired through block layout — only the inline pass currently
//!   tracks them).

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::{
    AlignContent, AlignItems, AlignSelf, BoxSizing, CssLength, FlexDirection, FlexWrap,
    JustifyContent,
};
use wgpu_html_style::CascadedNode;
use wgpu_html_tree::Element;

use crate::{
    BlockOverrides, Ctx, LayoutBox, is_auto_margin, layout_block_at_with, length,
    measure_text_leaf, translate_box_x_in_place, translate_box_y_in_place,
};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Lay flex children out and return them positioned at absolute pixel
/// coordinates, plus the total main-axis size used and the
/// container's used cross-axis size — both based on margin boxes.
pub(crate) fn layout_flex_children(
    parent: &CascadedNode,
    parent_style: &Style,
    content_x: f32,
    content_y: f32,
    inner_width: f32,
    inner_height_explicit: Option<f32>,
    ctx: &mut Ctx,
) -> (Vec<LayoutBox>, f32, f32) {
    let direction = parent_style
        .flex_direction
        .clone()
        .unwrap_or(FlexDirection::Row);
    let wrap = parent_style.flex_wrap.clone().unwrap_or(FlexWrap::Nowrap);
    let justify = parent_style
        .justify_content
        .clone()
        .unwrap_or(JustifyContent::FlexStart);
    let align_items = parent_style
        .align_items
        .clone()
        .unwrap_or(AlignItems::Stretch);
    let align_content = parent_style
        .align_content
        .clone()
        .unwrap_or(AlignContent::Normal);

    let is_row = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
    let is_dir_reverse = matches!(
        direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );
    let is_wrap_reverse = matches!(wrap, FlexWrap::WrapReverse);

    // Container main / cross sizes. Cross is only definite when an
    // explicit height (row) or width (column) is set; an indefinite
    // cross size disables `align-content` distribution and falls back
    // to a sum of line cross sizes.
    let main_axis_size = if is_row {
        Some(inner_width)
    } else {
        inner_height_explicit
    };
    let cross_axis_size: Option<f32> = if is_row {
        inner_height_explicit
    } else {
        Some(inner_width)
    };

    // Per-axis gaps. CSS-Flex-1 §10.5: when both `gap` and
    // `row-gap`/`column-gap` are set, the longhand wins; otherwise
    // the shorthand fills both axes.
    let gap_main_len = if is_row {
        parent_style
            .column_gap
            .as_ref()
            .or(parent_style.gap.as_ref())
    } else {
        parent_style.row_gap.as_ref().or(parent_style.gap.as_ref())
    };
    let gap_cross_len = if is_row {
        parent_style.row_gap.as_ref().or(parent_style.gap.as_ref())
    } else {
        parent_style
            .column_gap
            .as_ref()
            .or(parent_style.gap.as_ref())
    };
    let gap_main = length::resolve(gap_main_len, inner_width, ctx).unwrap_or(0.0);
    let gap_cross = length::resolve(gap_cross_len, inner_width, ctx).unwrap_or(0.0);

    // ----------------------------------------------------------------
    // Phase 1: build flex items.
    //
    // We collect each non-`display: none` child into a `FlexItem`
    // carrying its hypothetical sizes, frame insets, auto-margin
    // flags, and source index. `order` then re-sorts them stably.
    // ----------------------------------------------------------------
    let mut items: Vec<FlexItem> = Vec::with_capacity(parent.children.len());
    for (idx, child) in parent.children.iter().enumerate() {
        if matches!(
            child.style.display,
            Some(wgpu_html_models::common::css_enums::Display::None)
        ) {
            continue;
        }
        let item = build_item(child, idx, is_row, main_axis_size, cross_axis_size, ctx);
        items.push(item);
    }
    if items.is_empty() {
        return (Vec::new(), 0.0, 0.0);
    }
    items.sort_by_key(|i| (i.order, i.source_index));

    // ----------------------------------------------------------------
    // Phase 2: collect items into flex lines.
    //
    // `nowrap` forces a single line. `wrap`/`wrap-reverse` greedy-fill
    // by hypothetical main size + gap; the line break happens when the
    // *next* item would push the running total past `main_axis_size`.
    // ----------------------------------------------------------------
    let mut lines: Vec<Vec<usize>> = Vec::new();
    if matches!(wrap, FlexWrap::Nowrap) || main_axis_size.is_none() {
        lines.push((0..items.len()).collect());
    } else {
        let main_axis_size = main_axis_size.unwrap_or(0.0);
        let mut current: Vec<usize> = Vec::new();
        let mut running = 0.0_f32;
        for (i, item) in items.iter().enumerate() {
            let outer = item.hypothetical_outer_main();
            let prospective = if current.is_empty() {
                outer
            } else {
                running + gap_main + outer
            };
            if !current.is_empty() && prospective > main_axis_size + EPS {
                lines.push(std::mem::take(&mut current));
                running = outer;
                current.push(i);
            } else {
                running = prospective;
                current.push(i);
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }

    // ----------------------------------------------------------------
    // Phase 3: resolve flexible lengths per line.
    //
    // Sets each item's `resolved_main` (the inner / content-box main
    // size after grow / shrink and clamping). Items with no flex
    // factors keep their hypothetical main size.
    // ----------------------------------------------------------------
    if let Some(main_axis_size) = main_axis_size {
        for line in &lines {
            resolve_flexible_lengths(&mut items, line, main_axis_size, gap_main);
        }
    }

    // ----------------------------------------------------------------
    // Phase 4: per-item layout at the resolved main size to learn the
    // cross extent that emerges from the recursive block layout.
    //
    // We feed `BlockOverrides` so the recursive call respects the
    // resolved main size verbatim, instead of going back to the
    // cascade's `width`/`height`. Any explicit cross-size style on the
    // item (or stretch later) is honored by the block layout itself.
    // ----------------------------------------------------------------
    for item in &mut items {
        let overrides = if is_row {
            BlockOverrides {
                width: Some(item.resolved_main),
                height: None,
                ignore_style_width: false,
                ignore_style_height: item.ignore_unresolved_percent_cross_size,
            }
        } else {
            BlockOverrides {
                width: None,
                height: Some(item.resolved_main),
                ignore_style_width: item.ignore_unresolved_percent_cross_size,
                ignore_style_height: false,
            }
        };
        let laid = layout_block_at_with(
            item.node,
            0.0,
            0.0,
            inner_width,
            cross_axis_size.unwrap_or(0.0),
            overrides,
            ctx,
        );
        item.measured_cross_inner = if is_row {
            laid.content_rect.h
        } else {
            laid.content_rect.w
        };
        item.box_ = Some(laid);
    }

    // ----------------------------------------------------------------
    // Phase 5: line cross sizes.
    //
    // Each line's cross size is the max of its items' margin-box
    // cross sizes. (Baseline-aligned items would feed in via their
    // first-line baseline; we don't track per-block baselines yet so
    // they fall back to `flex-start`.)
    // ----------------------------------------------------------------
    let mut line_cross_sizes: Vec<f32> = Vec::with_capacity(lines.len());
    for line in &lines {
        let mut max_cross = 0.0_f32;
        for &i in line {
            let outer = items[i].outer_cross();
            if outer > max_cross {
                max_cross = outer;
            }
        }
        line_cross_sizes.push(max_cross);
    }

    // CSS-Flex-1 §9.4 step 15: for a single-line flex container with a
    // definite cross size, the line's cross size is *clamped to* the
    // container's inner cross size — the line fills the container,
    // regardless of `align-content`. (For multi-line, free cross space
    // is handed to `align-content` below.)
    let single_line = lines.len() == 1;
    let mut cross_start_offset = 0.0_f32;
    let mut cross_between = 0.0_f32;

    if single_line {
        if let Some(c) = cross_axis_size {
            line_cross_sizes[0] = c;
        }
    } else {
        // ------------------------------------------------------------
        // Phase 6 (multi-line): distribute free cross space across
        // lines via `align-content`. Only meaningful when the
        // container has a definite cross size; with an indefinite
        // cross size lines stay at their max-of-items size.
        // ------------------------------------------------------------
        let total_lines_cross: f32 =
            line_cross_sizes.iter().sum::<f32>() + gap_cross * (lines.len() as f32 - 1.0).max(0.0);
        let cross_box = cross_axis_size.unwrap_or(total_lines_cross);
        let lines_free_cross = (cross_box - total_lines_cross).max(0.0);
        let (start, between, stretch_extra) = align_content_distribution(
            &align_content,
            lines_free_cross,
            gap_cross,
            lines.len() as f32,
            cross_axis_size.is_some(),
        );
        cross_start_offset = start;
        cross_between = between;
        if stretch_extra > 0.0 {
            for s in &mut line_cross_sizes {
                *s += stretch_extra;
            }
        }
    }

    // Used cross-box extent (for the wrap-reverse mirror below).
    let cross_box = cross_axis_size.unwrap_or_else(|| {
        line_cross_sizes.iter().sum::<f32>() + gap_cross * (lines.len() as f32 - 1.0).max(0.0)
    });

    // Compute each line's cross-axis position. `cross_between` is the
    // extra spacing per gap added by `align-content` (e.g. for
    // space-between); `gap_cross` is CSS `row-gap` / `column-gap`
    // applied unconditionally between lines.
    let mut line_cross_positions: Vec<f32> = Vec::with_capacity(lines.len());
    {
        let mut cursor = cross_start_offset;
        for s in &line_cross_sizes {
            line_cross_positions.push(cursor);
            cursor += s + cross_between + gap_cross;
        }
    }
    if is_wrap_reverse {
        for (idx, pos) in line_cross_positions.iter_mut().enumerate() {
            let line_size = line_cross_sizes[idx];
            *pos = (cross_box - *pos - line_size).max(0.0);
        }
    }

    // ----------------------------------------------------------------
    // Phase 7: per-line main-axis distribution + cross alignment +
    // final translation into absolute coordinates.
    // ----------------------------------------------------------------
    let mut final_boxes: Vec<(usize, LayoutBox)> = Vec::with_capacity(items.len());
    for (line_idx, line) in lines.iter().enumerate() {
        let line_cross_size = line_cross_sizes[line_idx];
        let line_cross_pos = line_cross_positions[line_idx];

        // Items' total outer main + gaps used on this line.
        let total_main: f32 = line.iter().map(|&i| items[i].outer_main()).sum::<f32>()
            + gap_main * (line.len() as f32 - 1.0).max(0.0);
        let mut free_main = main_axis_size
            .map(|m| (m - total_main).max(0.0))
            .unwrap_or(0.0);

        // Auto main-axis margins absorb free space first; whatever
        // is left flows into `justify-content`.
        let auto_main_count: usize = line
            .iter()
            .map(|&i| items[i].auto_main_starts() as usize + items[i].auto_main_ends() as usize)
            .sum();
        let auto_main_each = if auto_main_count > 0 && free_main > 0.0 {
            let each = free_main / auto_main_count as f32;
            free_main = 0.0;
            each
        } else {
            0.0
        };

        let (start_main, between_extra) = distribution(&justify, free_main, line.len() as f32);

        // Walk the line, placing each item at its main + cross
        // position. `*-reverse` direction flips the main axis after
        // positioning forward, mirroring around the container's main
        // extent.
        let mut cursor_main = start_main;
        for &i in line {
            let item = &items[i];
            let outer_main = item.outer_main();
            // Auto margin contributions on each side.
            let auto_pre = if item.auto_main_starts() {
                auto_main_each
            } else {
                0.0
            };
            let auto_post = if item.auto_main_ends() {
                auto_main_each
            } else {
                0.0
            };

            let item_main_pos = cursor_main + auto_pre;
            cursor_main = item_main_pos + outer_main + auto_post + gap_main + between_extra;

            // Cross position within the line.
            let mut item_cross_pos = line_cross_pos;
            let auto_cross_pre = item.auto_cross_starts();
            let auto_cross_post = item.auto_cross_ends();
            let line_free_cross = (line_cross_size - item.outer_cross()).max(0.0);
            let align = resolve_align_self(&item.align_self, &align_items);
            let stretched = matches!(align, ResolvedAlign::Stretch)
                && !item.has_explicit_cross_size
                && !auto_cross_pre
                && !auto_cross_post;
            let auto_cross_count = auto_cross_pre as u32 + auto_cross_post as u32;
            if auto_cross_count > 0 && line_free_cross > 0.0 {
                let each = line_free_cross / auto_cross_count as f32;
                if auto_cross_pre {
                    item_cross_pos += each;
                }
                // `auto_cross_post` is implicitly handled by not advancing further;
                // cross position only matters for the start edge.
            } else if !stretched {
                match align {
                    ResolvedAlign::FlexStart => {}
                    ResolvedAlign::FlexEnd => item_cross_pos += line_free_cross,
                    ResolvedAlign::Center => item_cross_pos += line_free_cross * 0.5,
                    ResolvedAlign::Stretch => {}
                }
            }

            // If we need to stretch, re-lay the item with the line's
            // cross extent as the cross dimension. Only items with no
            // explicit cross style and no auto cross margins stretch.
            let mut child_box = if stretched {
                let stretch_target = (line_cross_size - item.margin_cross_outer_known()).max(0.0);
                let overrides = if is_row {
                    BlockOverrides {
                        width: Some(item.resolved_main),
                        height: Some(stretch_target),
                        ignore_style_width: false,
                        ignore_style_height: false,
                    }
                } else {
                    BlockOverrides {
                        width: Some(stretch_target),
                        height: Some(item.resolved_main),
                        ignore_style_width: false,
                        ignore_style_height: false,
                    }
                };
                layout_block_at_with(
                    item.node,
                    0.0,
                    0.0,
                    inner_width,
                    cross_axis_size.unwrap_or(stretch_target),
                    overrides,
                    ctx,
                )
            } else {
                item.box_.clone().expect("item box was laid out in phase 4")
            };

            // Translate from temporary (0, 0) origin to final absolute
            // position. Item position is at the *margin-box* corner.
            let (final_main_origin, final_cross_origin) = if is_row {
                (content_x + item_main_pos, content_y + item_cross_pos)
            } else {
                (content_x + item_cross_pos, content_y + item_main_pos)
            };
            let dx = final_main_origin - child_box.margin_rect.x;
            let dy = final_cross_origin - child_box.margin_rect.y;
            if dx != 0.0 {
                translate_box_x_in_place(&mut child_box, dx);
            }
            if dy != 0.0 {
                translate_box_y_in_place(&mut child_box, dy);
            }

            final_boxes.push((item.source_index, child_box));
        }
    }

    // ----------------------------------------------------------------
    // Phase 8: handle row-reverse / column-reverse by mirroring main
    // positions inside the container's main extent.
    // ----------------------------------------------------------------
    if is_dir_reverse && main_axis_size.is_some_and(|m| m > 0.0) {
        let main_axis_size = main_axis_size.unwrap_or(0.0);
        for (_, b) in final_boxes.iter_mut() {
            let main_size_box = if is_row {
                b.margin_rect.w
            } else {
                b.margin_rect.h
            };
            if is_row {
                let cur_main = b.margin_rect.x - content_x;
                let new_main = (main_axis_size - cur_main - main_size_box).max(0.0);
                let dx = (content_x + new_main) - b.margin_rect.x;
                if dx != 0.0 {
                    translate_box_x_in_place(b, dx);
                }
            } else {
                let cur_main = b.margin_rect.y - content_y;
                let new_main = (main_axis_size - cur_main - main_size_box).max(0.0);
                let dy = (content_y + new_main) - b.margin_rect.y;
                if dy != 0.0 {
                    translate_box_y_in_place(b, dy);
                }
            }
        }
    }

    // ----------------------------------------------------------------
    // Phase 9: restore source order so the layout child indices stay
    // aligned with the cascaded tree (hit-testing relies on this).
    // Insert empty placeholder boxes for `display: none` children so
    // the layout child list has the same length and indexing as the
    // DOM child list. Without this, every item after a skipped child
    // has a shifted index, and hit-test paths resolve to the wrong
    // DOM node.
    // ----------------------------------------------------------------
    final_boxes.sort_by_key(|(idx, _)| *idx);
    let total_children = parent.children.len();
    let mut final_positions: Vec<LayoutBox> = Vec::with_capacity(total_children);
    let mut fb_iter = final_boxes.into_iter().peekable();
    for i in 0..total_children {
        if fb_iter.peek().is_some_and(|(idx, _)| *idx == i) {
            final_positions.push(fb_iter.next().unwrap().1);
        } else {
            // `display: none` child — insert a zero-size placeholder
            // so the index stays aligned with the DOM.
            final_positions.push(crate::empty_box(content_x, content_y));
        }
    }

    // Container's used main / cross sizes for the parent's content
    // box. Free space distributed by `justify-content` is *not*
    // counted as content (matches browsers).
    let used_main = lines
        .iter()
        .map(|line| {
            line.iter().map(|&i| items[i].outer_main()).sum::<f32>()
                + gap_main * (line.len() as f32 - 1.0).max(0.0)
        })
        .fold(0.0_f32, f32::max);
    let used_cross =
        line_cross_sizes.iter().sum::<f32>() + gap_cross * (lines.len() as f32 - 1.0).max(0.0);
    let (content_w_used, content_h_used) = if is_row {
        (used_main, used_cross)
    } else {
        (used_cross, used_main)
    };

    (final_positions, content_w_used, content_h_used)
}

// ---------------------------------------------------------------------------
// Per-item state
// ---------------------------------------------------------------------------

const EPS: f32 = 0.001;

/// Per-item bookkeeping carried through the algorithm. Sizes are in
/// content-box pixels; `margin_*` and `frame_*` describe the inset
/// stack so we can convert to outer (margin) box on either axis.
struct FlexItem<'a> {
    node: &'a CascadedNode,
    source_index: usize,
    order: i32,
    align_self: AlignSelf,

    /// Resolved flex factors and base size (already clamped into
    /// content-box pixels on the main axis).
    flex_grow: f32,
    flex_shrink: f32,
    base_size: f32,
    /// The hypothetical main size *before* flexing — base size clamped
    /// by `min`/`max` on the main axis.
    hypothetical_main: f32,
    /// The post-flex resolved main size (content-box).
    resolved_main: f32,
    /// Min/max clamps on the main axis, in content-box px (resolved at
    /// build time so the flexing loop doesn't re-resolve them).
    main_min: f32,
    main_max: f32,

    /// Frame insets on each axis (border + padding).
    frame_main: f32,
    frame_cross: f32,

    /// Margin pixels on each axis (auto sides resolve to 0 here; the
    /// `auto_*` flags below tell us which sides actually wanted auto).
    margin_main_start: f32,
    margin_main_end: f32,
    margin_cross_start: f32,
    margin_cross_end: f32,
    auto_main_start: bool,
    auto_main_end: bool,
    auto_cross_start: bool,
    auto_cross_end: bool,

    /// True when the item's cascade has an explicit cross-axis size
    /// (`width` for column, `height` for row), disabling the
    /// `align-self: stretch` re-layout.
    has_explicit_cross_size: bool,
    /// A percentage cross size with an indefinite flex-container
    /// cross size disables stretch, but it cannot resolve for the
    /// item's own layout. Treat that style size as auto for the
    /// recursive block layout.
    ignore_unresolved_percent_cross_size: bool,

    /// Set after phase 4: the cross-axis content-box size emerged
    /// from laying the item out at its resolved main size.
    measured_cross_inner: f32,

    /// The laid-out box (phase 4); replaced by phase 7 for stretched
    /// items. `Some` after phase 4.
    box_: Option<LayoutBox>,
}

impl FlexItem<'_> {
    fn outer_main(&self) -> f32 {
        self.resolved_main + self.frame_main + self.margin_main_start + self.margin_main_end
    }

    fn outer_cross(&self) -> f32 {
        self.measured_cross_inner
            + self.frame_cross
            + self.margin_cross_start
            + self.margin_cross_end
    }

    /// Like `outer_cross` but doesn't add the inner content size — used
    /// when computing the stretch target so we can subtract margin /
    /// border / padding from the line's cross extent.
    fn margin_cross_outer_known(&self) -> f32 {
        self.frame_cross + self.margin_cross_start + self.margin_cross_end
    }

    fn hypothetical_outer_main(&self) -> f32 {
        self.hypothetical_main + self.frame_main + self.margin_main_start + self.margin_main_end
    }

    fn auto_main_starts(&self) -> bool {
        self.auto_main_start
    }
    fn auto_main_ends(&self) -> bool {
        self.auto_main_end
    }
    fn auto_cross_starts(&self) -> bool {
        self.auto_cross_start
    }
    fn auto_cross_ends(&self) -> bool {
        self.auto_cross_end
    }
}

fn build_item<'a>(
    node: &'a CascadedNode,
    source_index: usize,
    is_row: bool,
    parent_inner_main: Option<f32>,
    parent_inner_cross: Option<f32>,
    ctx: &mut Ctx,
) -> FlexItem<'a> {
    let style = &node.style;
    let order = style.order.unwrap_or(0);
    let flex_grow = style.flex_grow.unwrap_or(0.0).max(0.0);
    let flex_shrink = style.flex_shrink.unwrap_or(1.0).max(0.0);
    let align_self = style.align_self.clone().unwrap_or(AlignSelf::Auto);

    let box_sizing = style.box_sizing.clone().unwrap_or(BoxSizing::ContentBox);

    // Resolve insets. We keep the four sides separate so auto-margin
    // bookkeeping per axis is exact.
    let margin_top = side_margin(&style.margin_top, &style.margin, parent_inner_main, ctx);
    let margin_right = side_margin(&style.margin_right, &style.margin, parent_inner_main, ctx);
    let margin_bottom = side_margin(&style.margin_bottom, &style.margin, parent_inner_main, ctx);
    let margin_left = side_margin(&style.margin_left, &style.margin, parent_inner_main, ctx);

    let auto_top = is_auto_margin(&style.margin_top, &style.margin);
    let auto_right = is_auto_margin(&style.margin_right, &style.margin);
    let auto_bottom = is_auto_margin(&style.margin_bottom, &style.margin);
    let auto_left = is_auto_margin(&style.margin_left, &style.margin);

    let border_top =
        resolve_axis_length(style.border_top_width.as_ref(), parent_inner_main, ctx).unwrap_or(0.0);
    let border_right =
        resolve_axis_length(style.border_right_width.as_ref(), parent_inner_main, ctx)
            .unwrap_or(0.0);
    let border_bottom =
        resolve_axis_length(style.border_bottom_width.as_ref(), parent_inner_main, ctx)
            .unwrap_or(0.0);
    let border_left = resolve_axis_length(style.border_left_width.as_ref(), parent_inner_main, ctx)
        .unwrap_or(0.0);

    let pad_top = side_pad(&style.padding_top, &style.padding, parent_inner_main, ctx);
    let pad_right = side_pad(&style.padding_right, &style.padding, parent_inner_main, ctx);
    let pad_bottom = side_pad(
        &style.padding_bottom,
        &style.padding,
        parent_inner_main,
        ctx,
    );
    let pad_left = side_pad(&style.padding_left, &style.padding, parent_inner_main, ctx);

    let frame_h = border_left + border_right + pad_left + pad_right;
    let frame_v = border_top + border_bottom + pad_top + pad_bottom;
    let (frame_main, frame_cross) = if is_row {
        (frame_h, frame_v)
    } else {
        (frame_v, frame_h)
    };

    let (margin_main_start, margin_main_end, margin_cross_start, margin_cross_end) = if is_row {
        (margin_left, margin_right, margin_top, margin_bottom)
    } else {
        (margin_top, margin_bottom, margin_left, margin_right)
    };
    let (auto_main_start, auto_main_end, auto_cross_start, auto_cross_end) = if is_row {
        (auto_left, auto_right, auto_top, auto_bottom)
    } else {
        (auto_top, auto_bottom, auto_left, auto_right)
    };

    // --- Flex base size (CSS-Flex-1 §9.2, simplified) -----------------
    //
    // - `flex-basis: <length>` → use it.
    // - `flex-basis: auto` (default) → fall back to the main-axis
    //   size property (`width` for row, `height` for column).
    // - Neither set → the item's intrinsic content size. For replaced
    //   elements (`<img>`) we use the image's HTML width/height
    //   attributes, falling back to its decoded dimensions; for
    //   non-replaced items the approximation is still 0.
    let main_size_prop = if is_row {
        style.width.as_ref()
    } else {
        style.height.as_ref()
    };
    // Explicit basis: from `flex-basis` (when not `auto`) or the
    // matching main-size property. This value comes from CSS and is
    // interpreted via `box-sizing`.
    let basis_explicit = match style.flex_basis.as_ref() {
        Some(CssLength::Auto) | None => resolve_axis_length(main_size_prop, parent_inner_main, ctx),
        other => resolve_axis_length(other, parent_inner_main, ctx),
    };
    // Intrinsic basis: max-content of the item's content. Already
    // content-box (no padding/border), so it bypasses the box-sizing
    // conversion and is used as-is when no explicit basis is set.
    let intrinsic_main = replaced_intrinsic_main(node, is_row, ctx.images)
        .or_else(|| text_intrinsic_main(node, is_row, ctx));
    let mut base_size = match basis_explicit {
        Some(v) => match box_sizing {
            BoxSizing::ContentBox => v,
            BoxSizing::BorderBox => (v - frame_main).max(0.0),
        },
        None => intrinsic_main.unwrap_or(0.0).max(0.0),
    };

    // Min/max on main axis, resolved into content-box pixels.
    let (min_prop, max_prop) = if is_row {
        (style.min_width.as_ref(), style.max_width.as_ref())
    } else {
        (style.min_height.as_ref(), style.max_height.as_ref())
    };
    let main_min = resolve_axis_length(min_prop, parent_inner_main, ctx)
        .map(|v| match box_sizing {
            BoxSizing::ContentBox => v,
            BoxSizing::BorderBox => (v - frame_main).max(0.0),
        })
        .unwrap_or(0.0);
    let main_max = resolve_axis_length(max_prop, parent_inner_main, ctx)
        .map(|v| match box_sizing {
            BoxSizing::ContentBox => v,
            BoxSizing::BorderBox => (v - frame_main).max(0.0),
        })
        .unwrap_or(f32::INFINITY);

    base_size = base_size.max(0.0);
    let hypothetical_main = base_size.clamp(main_min, main_max);

    let cross_size_prop = if is_row {
        style.height.as_ref()
    } else {
        style.width.as_ref()
    };
    let ignore_unresolved_percent_cross_size =
        parent_inner_cross.is_none() && matches!(cross_size_prop, Some(CssLength::Percent(_)));
    let has_explicit_cross_size =
        matches!(cross_size_prop, Some(v) if !matches!(v, CssLength::Auto));

    FlexItem {
        node,
        source_index,
        order,
        align_self,
        flex_grow,
        flex_shrink,
        base_size,
        hypothetical_main,
        resolved_main: hypothetical_main,
        main_min,
        main_max,
        frame_main,
        frame_cross,
        margin_main_start,
        margin_main_end,
        margin_cross_start,
        margin_cross_end,
        auto_main_start,
        auto_main_end,
        auto_cross_start,
        auto_cross_end,
        has_explicit_cross_size,
        ignore_unresolved_percent_cross_size,
        measured_cross_inner: 0.0,
        box_: None,
    }
}

/// Intrinsic main-axis size (in physical pixels) for replaced flex
/// items. Currently covers `<img>` only — HTML `width`/`height`
/// attributes are preferred over decoded dimensions, mirroring
/// [`crate::load_image`]. Returns `None` for non-replaced elements
/// and for images whose fetch hasn't completed yet, in which case the
/// flex algorithm falls back to its previous "0 base size" behaviour.
fn replaced_intrinsic_main(
    node: &CascadedNode,
    is_row: bool,
    images: &mut crate::ImageCache,
) -> Option<f32> {
    match &node.element {
        Element::Img(img) => {
            let loaded = images.load(img);
            let w = img.width.or_else(|| loaded.as_ref().map(|d| d.width));
            let h = img.height.or_else(|| loaded.as_ref().map(|d| d.height));
            let main = if is_row { w } else { h };
            main.map(|v| v as f32)
        }
        _ => None,
    }
}

/// Approximate max-content main-axis size for a flex item by walking
/// its text descendants.
///
/// CSS-Sizing-3 §5.2 defines max-content as the size required to lay
/// out the box's content on a single line. For replaced items
/// `replaced_intrinsic_main` covers `<img>`; this function covers the
/// remaining "non-replaced wrapper around text" case
/// (`<button>Submit</button>`, `<a>link text</a>`, `<span>foo</span>`,
/// …) that would otherwise fall through to `base_size = 0` and
/// collapse the item to padding-only width.
///
/// For row-direction containers it sums each text descendant's shaped
/// one-line width (closest spec-correct max-content for inline-flowing
/// content). For column-direction it returns the maximum descendant
/// text height (a stacked-block approximation).
///
/// Padding / border on the item itself is *not* added here — the
/// caller treats the result as content-box, and the flex layout adds
/// the frame separately when it builds the item's box.
fn text_intrinsic_main(node: &CascadedNode, is_row: bool, ctx: &mut Ctx) -> Option<f32> {
    if let Element::Text(text) = &node.element {
        let (w, h) = measure_text_leaf(text, &node.style, ctx);
        return Some(if is_row { w } else { h });
    }
    // Scan children first (buttons have text children whose
    // intrinsic width must be measured).
    let mut sum_main = 0.0_f32;
    let mut max_main = 0.0_f32;
    let mut found = false;
    for child in &node.children {
        if let Some(v) = text_intrinsic_main(child, is_row, ctx) {
            sum_main += v;
            max_main = max_main.max(v);
            found = true;
        }
    }
    if found {
        return Some(if is_row { sum_main } else { max_main });
    }
    // Form controls (`<input>`, `<textarea>`, `<select>`, `<button>`)
    // have an intrinsic height of one line even when they have no DOM
    // children. Without this, a column-direction flex container gives
    // them 0 content height because the recursive child scan finds
    // nothing, and the `form_control_default_line_height` fallback in
    // `layout_block` is bypassed by the flex height override.
    // Only applies on the cross axis (height in column flex) — for
    // the main axis (width in row flex) we return None to let the
    // flex algorithm use 0 / auto sizing.
    if !is_row && crate::form_control_default_line_height(node) {
        let font_size = crate::font_size_px(&node.style).unwrap_or(16.0);
        let line_h = crate::line_height_px(&node.style, font_size);
        return Some(line_h);
    }
    None
}

// ---------------------------------------------------------------------------
// Inset helpers (with auto handling)
// ---------------------------------------------------------------------------

fn side_margin(
    specific: &Option<CssLength>,
    shorthand: &Option<CssLength>,
    container: Option<f32>,
    ctx: &mut Ctx,
) -> f32 {
    resolve_axis_length(specific.as_ref(), container, ctx)
        .or_else(|| resolve_axis_length(shorthand.as_ref(), container, ctx))
        .unwrap_or(0.0)
}

fn side_pad(
    specific: &Option<CssLength>,
    shorthand: &Option<CssLength>,
    container: Option<f32>,
    ctx: &mut Ctx,
) -> f32 {
    resolve_axis_length(specific.as_ref(), container, ctx)
        .or_else(|| resolve_axis_length(shorthand.as_ref(), container, ctx))
        .unwrap_or(0.0)
}

fn resolve_axis_length(
    len: Option<&CssLength>,
    container: Option<f32>,
    ctx: &mut Ctx,
) -> Option<f32> {
    let len = len?;
    match len {
        CssLength::Percent(_) if container.is_none() => None,
        _ => length::resolve(Some(len), container.unwrap_or(0.0), ctx),
    }
}

// ---------------------------------------------------------------------------
// Flex factor resolution
// ---------------------------------------------------------------------------

/// Resolve flexible lengths per line. Implements the iterative freeze
/// loop from CSS-Flex-1 §9.7. Each iteration computes the remaining
/// free space (container main − sum of frozen items' outer main − sum
/// of unfrozen items' *base* outer main − gaps), distributes it among
/// unfrozen items proportional to `flex-grow` (growing) or
/// `flex-shrink × base_size` (shrinking), then clamps to min/max and
/// freezes any item that hit a clamp. The loop is bounded by the
/// number of items because each iteration either freezes at least one
/// item or makes no progress.
fn resolve_flexible_lengths(
    items: &mut [FlexItem<'_>],
    line: &[usize],
    main_axis_size: f32,
    gap_main: f32,
) {
    if line.is_empty() {
        return;
    }

    // Initial: every item sized at its hypothetical main.
    for &i in line {
        items[i].resolved_main = items[i].hypothetical_main;
    }

    // Initial free space (from hypothetical sizes) decides whether
    // we're in grow mode or shrink mode for the rest of the loop.
    let initial_outer_total: f32 = line
        .iter()
        .map(|&i| items[i].hypothetical_outer_main())
        .sum::<f32>()
        + gap_main * (line.len() as f32 - 1.0).max(0.0);
    let initial_free = main_axis_size - initial_outer_total;
    if initial_free.abs() <= EPS {
        return;
    }
    let growing = initial_free > 0.0;

    // Items with no relevant flex factor (or shrinking with base 0)
    // can't take part in distribution and are frozen up front.
    let mut frozen = vec![false; line.len()];
    for (k, &i) in line.iter().enumerate() {
        let factor = if growing {
            items[i].flex_grow
        } else {
            items[i].flex_shrink
        };
        if factor <= 0.0 || (!growing && items[i].base_size <= 0.0) {
            frozen[k] = true;
        }
    }
    if frozen.iter().all(|&f| f) {
        return;
    }

    let frame_outer =
        |it: &FlexItem| -> f32 { it.frame_main + it.margin_main_start + it.margin_main_end };
    let gap_total = gap_main * (line.len() as f32 - 1.0).max(0.0);

    for _ in 0..(line.len() + 1) {
        // a) Free space at this iteration: subtract frozen items'
        //    *resolved* outer sizes and unfrozen items' *base* outer
        //    sizes (plus gaps) from the container.
        let mut consumed = gap_total;
        let mut sum_factor = 0.0_f32;
        let mut sum_scaled_shrink = 0.0_f32;
        for (k, &i) in line.iter().enumerate() {
            let it = &items[i];
            if frozen[k] {
                consumed += it.resolved_main + frame_outer(it);
            } else {
                consumed += it.hypothetical_main + frame_outer(it);
                if growing {
                    sum_factor += it.flex_grow;
                } else {
                    sum_scaled_shrink += it.flex_shrink * it.base_size;
                }
            }
        }
        let free = main_axis_size - consumed;

        // b) Distribute. If sum_factor (grow) is < 1 in grow mode,
        //    cap the distributed share so we don't over-allocate (per
        //    spec). For shrink, no such cap applies.
        let denom_grow = if growing && sum_factor < 1.0 {
            1.0
        } else {
            sum_factor.max(0.0)
        };

        for (k, &i) in line.iter().enumerate() {
            if frozen[k] {
                continue;
            }
            let it = &mut items[i];
            let new = if growing {
                if denom_grow <= 0.0 {
                    it.hypothetical_main
                } else {
                    let share = free * (it.flex_grow / denom_grow);
                    it.hypothetical_main + share.max(0.0)
                }
            } else if sum_scaled_shrink <= 0.0 {
                it.hypothetical_main
            } else {
                let ratio = (it.flex_shrink * it.base_size) / sum_scaled_shrink;
                // free is negative when shrinking
                let share = free * ratio;
                (it.hypothetical_main + share).max(0.0)
            };
            it.resolved_main = new;
        }

        // c) Clamp + freeze. Track the total violation across the
        //    line; if the violations all pull the same way (or all
        //    items hit their clamp), freeze them all at once,
        //    otherwise freeze only the matching set per spec.
        let mut total_violation = 0.0_f32;
        let mut any_violated = false;
        let mut violated = vec![0i8; line.len()];
        for (k, &i) in line.iter().enumerate() {
            if frozen[k] {
                continue;
            }
            let it = &mut items[i];
            let clamped = it.resolved_main.clamp(it.main_min, it.main_max).max(0.0);
            let diff = clamped - it.resolved_main;
            if diff.abs() > EPS {
                any_violated = true;
                total_violation += diff;
                violated[k] = if diff > 0.0 { 1 } else { -1 };
            }
            it.resolved_main = clamped;
        }

        if !any_violated {
            break;
        }
        // Freeze items whose violation matches the total direction.
        // (Spec: positive total → freeze min-violators; negative →
        // freeze max-violators; zero → freeze every violator.)
        let direction = if total_violation.abs() <= EPS {
            0
        } else if total_violation > 0.0 {
            1
        } else {
            -1
        };
        for k in 0..line.len() {
            if frozen[k] {
                continue;
            }
            if violated[k] != 0 && (direction == 0 || violated[k] == direction) {
                frozen[k] = true;
            }
        }
        if frozen.iter().all(|&f| f) {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// `justify-content` distribution
// ---------------------------------------------------------------------------

/// Returns `(start_offset, between_extra)`.
///
/// `between_extra` is a *per-pair* extra spacing added between
/// adjacent items on top of `gap_main`; the caller handles `gap` itself
/// while walking the items.
fn distribution(justify: &JustifyContent, free: f32, n: f32) -> (f32, f32) {
    use JustifyContent::*;
    if free <= 0.0 {
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
            if n > 0.0 {
                let s = free / n;
                (s * 0.5, s)
            } else {
                (0.0, 0.0)
            }
        }
        SpaceEvenly => {
            if n > 0.0 {
                let s = free / (n + 1.0);
                (s, s)
            } else {
                (0.0, 0.0)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `align-content` distribution (multi-line containers)
// ---------------------------------------------------------------------------

/// Returns `(start_offset, between_extra, per_line_stretch)`.
fn align_content_distribution(
    align: &AlignContent,
    free: f32,
    _gap_cross: f32,
    n_lines: f32,
    has_definite_cross: bool,
) -> (f32, f32, f32) {
    use AlignContent::*;
    if !has_definite_cross || n_lines <= 1.0 {
        // Single line or indefinite cross size: no redistribution.
        return (0.0, 0.0, 0.0);
    }
    if free <= 0.0 {
        // No spare cross space → stretch and start are no-ops.
        return (0.0, 0.0, 0.0);
    }
    match align {
        // CSS default for multi-line containers without `align-content`
        // is `stretch` per the spec.
        Normal | Stretch => (0.0, 0.0, free / n_lines),
        Start | FlexStart => (0.0, 0.0, 0.0),
        End | FlexEnd => (free, 0.0, 0.0),
        Center => (free * 0.5, 0.0, 0.0),
        SpaceBetween => {
            if n_lines > 1.0 {
                (0.0, free / (n_lines - 1.0), 0.0)
            } else {
                (0.0, 0.0, 0.0)
            }
        }
        SpaceAround => {
            let s = free / n_lines;
            (s * 0.5, s, 0.0)
        }
        SpaceEvenly => {
            let s = free / (n_lines + 1.0);
            (s, s, 0.0)
        }
    }
}

// ---------------------------------------------------------------------------
// Align-self resolution
// ---------------------------------------------------------------------------

/// One of the four cross-axis placements we actually emit. Anything
/// the spec calls out (e.g. `baseline`) but we don't yet support
/// degrades gracefully to `flex-start`.
enum ResolvedAlign {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
}

fn resolve_align_self(item: &AlignSelf, parent: &AlignItems) -> ResolvedAlign {
    let effective = match item {
        AlignSelf::Auto => parent_to_self(parent),
        other => other.clone(),
    };
    match effective {
        AlignSelf::FlexStart | AlignSelf::Start => ResolvedAlign::FlexStart,
        AlignSelf::FlexEnd | AlignSelf::End => ResolvedAlign::FlexEnd,
        AlignSelf::Center => ResolvedAlign::Center,
        AlignSelf::Stretch | AlignSelf::Normal | AlignSelf::Auto => ResolvedAlign::Stretch,
        // No baseline support yet — fall back to flex-start (browsers
        // use the item's first line baseline; we don't track per-block
        // baselines).
        AlignSelf::Baseline => ResolvedAlign::FlexStart,
    }
}

fn parent_to_self(parent: &AlignItems) -> AlignSelf {
    match parent {
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
