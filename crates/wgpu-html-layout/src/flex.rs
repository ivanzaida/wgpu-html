//! Flex formatting context.
//!
//! Scope (v1):
//! - `flex-direction`: row / column / row-reverse / column-reverse
//! - `justify-content`: start / end / center / flex-start / flex-end /
//!   left / right / space-between / space-around / space-evenly
//! - `align-items`: start / end / center / flex-start / flex-end / stretch
//! - `gap` (single value applied between items on the main axis)
//! - **Not yet:** `flex-grow` / `flex-shrink` / `flex-basis`,
//!   `flex-wrap`, baseline alignment, multi-line flexboxes.
//!
//! Sizing model:
//! - An item's main-axis size is its explicit `width` (row) or `height`
//!   (column); auto / unset → 0 (no shrink-to-fit yet).
//! - An item's cross-axis size is its explicit cross dim; if missing
//!   and `align-items: stretch` and the parent's cross size is known,
//!   the child fills it; otherwise → 0.

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::{
    AlignItems, CssLength, FlexDirection, JustifyContent,
};
use wgpu_html_style::CascadedNode;

use crate::{Ctx, LayoutBox, length};

/// Lay flex children out and return them positioned at absolute pixel
/// coordinates, plus the total main-axis size used and the maximum
/// cross-axis size — both based on margin boxes.
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
    let justify = parent_style
        .justify_content
        .clone()
        .unwrap_or(JustifyContent::FlexStart);
    let align = parent_style
        .align_items
        .clone()
        .unwrap_or(AlignItems::Stretch);
    let gap = parent_style
        .gap
        .as_ref()
        .and_then(|g| length::resolve(Some(g), inner_width, ctx))
        .unwrap_or(0.0);

    let is_row = matches!(
        direction,
        FlexDirection::Row | FlexDirection::RowReverse
    );
    let is_reverse = matches!(
        direction,
        FlexDirection::RowReverse | FlexDirection::ColumnReverse
    );

    let main_axis_size = if is_row { inner_width } else { inner_height_explicit.unwrap_or(0.0) };
    let cross_axis_size = if is_row { inner_height_explicit } else { Some(inner_width) };

    // ----------------------------------------------------------------
    // Phase 1: lay each child out at a temporary (0, 0) origin to learn
    // its margin-box size on each axis. The size we hand to
    // `layout_block` as its container is the child's intended
    // width × height, which is sourced from its own `width`/`height`
    // style (or the parent's cross size for `align-items: stretch`).
    // ----------------------------------------------------------------

    let mut items: Vec<LayoutBox> = Vec::with_capacity(parent.children.len());
    for child in &parent.children {
        let (cw, ch) = intended_container_for_item(
            &child.style,
            is_row,
            inner_width,
            cross_axis_size,
            &align,
            ctx,
        );
        let laid = crate::layout_block_at(child, 0.0, 0.0, cw, ch, ctx);
        items.push(laid);
    }

    let n = items.len();
    if n == 0 {
        return (Vec::new(), 0.0, 0.0);
    }

    // ----------------------------------------------------------------
    // Phase 2: compute main-axis distribution.
    // ----------------------------------------------------------------
    let total_main: f32 = items
        .iter()
        .map(|b| if is_row { b.margin_rect.w } else { b.margin_rect.h })
        .sum();
    let nf = n as f32;
    let total_with_gaps = total_main + gap * (nf - 1.0).max(0.0);
    let free = (main_axis_size - total_with_gaps).max(0.0);

    let (start_main, between_gap) = distribution(&justify, free, gap, nf);

    // ----------------------------------------------------------------
    // Phase 3: max cross size (used for align-items when the parent
    // has no explicit cross size).
    // ----------------------------------------------------------------
    let max_cross: f32 = items
        .iter()
        .map(|b| if is_row { b.margin_rect.h } else { b.margin_rect.w })
        .fold(0.0_f32, f32::max);
    let cross_box = cross_axis_size.unwrap_or(max_cross);

    // ----------------------------------------------------------------
    // Phase 4: compute logical main-axis positions in source order
    // (cursor advancing forward), then mirror them for *-reverse.
    // ----------------------------------------------------------------
    let mut logical_main: Vec<f32> = Vec::with_capacity(n);
    {
        let mut cursor = start_main;
        for item in &items {
            logical_main.push(cursor);
            let item_main = if is_row { item.margin_rect.w } else { item.margin_rect.h };
            cursor += item_main + between_gap;
        }
    }
    if is_reverse {
        for (i, item) in items.iter().enumerate() {
            let item_main = if is_row { item.margin_rect.w } else { item.margin_rect.h };
            logical_main[i] = (main_axis_size - logical_main[i] - item_main).max(0.0);
        }
    }

    // ----------------------------------------------------------------
    // Phase 5: re-layout each child at its final absolute origin.
    // For `align-items: stretch` we synthesize the missing cross-axis
    // size on the child's style so the recursive block layout actually
    // sizes the box to the parent's cross extent.
    // ----------------------------------------------------------------
    let mut final_positions: Vec<LayoutBox> = Vec::with_capacity(n);
    for i in 0..n {
        let main_pos = logical_main[i];
        let item_cross = if is_row {
            items[i].margin_rect.h
        } else {
            items[i].margin_rect.w
        };

        let cross_offset = match align {
            AlignItems::FlexStart
            | AlignItems::Start
            | AlignItems::Stretch
            | AlignItems::Normal => 0.0,
            AlignItems::FlexEnd | AlignItems::End => (cross_box - item_cross).max(0.0),
            AlignItems::Center => ((cross_box - item_cross) * 0.5).max(0.0),
            AlignItems::Baseline => 0.0, // not implemented
        };

        let (origin_x, origin_y) = if is_row {
            (content_x + main_pos, content_y + cross_offset)
        } else {
            (content_x + cross_offset, content_y + main_pos)
        };

        let child = &parent.children[i];
        let (cw, ch) = intended_container_for_item(
            &child.style,
            is_row,
            inner_width,
            cross_axis_size,
            &align,
            ctx,
        );

        // Patch the child's style so that any cross-axis dimension we
        // synthesized for stretch / explicit sizing is honored when the
        // recursive block layout reads `style.height` / `style.width`.
        let mut patched = child.clone();
        if is_row {
            if patched.style.height.is_none() && ch > 0.0 {
                patched.style.height = Some(CssLength::Px(ch));
            }
            if patched.style.width.is_none() && cw > 0.0 {
                patched.style.width = Some(CssLength::Px(cw));
            }
        } else {
            if patched.style.width.is_none() && cw > 0.0 {
                patched.style.width = Some(CssLength::Px(cw));
            }
            if patched.style.height.is_none() && ch > 0.0 {
                patched.style.height = Some(CssLength::Px(ch));
            }
        }

        let laid = crate::layout_block_at(&patched, origin_x, origin_y, cw, ch, ctx);
        final_positions.push(laid);
    }

    // Natural content size = items + gaps (free space distributed by
    // justify-content does not contribute to it).
    let used_main = total_with_gaps;
    let (content_w_used, content_h_used) = if is_row {
        (used_main, max_cross)
    } else {
        (max_cross, used_main)
    };

    (final_positions, content_w_used, content_h_used)
}

fn intended_container_for_item(
    child_style: &Style,
    is_row: bool,
    parent_inner_w: f32,
    parent_inner_cross: Option<f32>,
    align: &AlignItems,
    ctx: &mut Ctx,
) -> (f32, f32) {
    // Treat percent on width against parent's main width, and percent on
    // height against parent's cross height (when known).
    let parent_h_for_pct = parent_inner_cross.unwrap_or(0.0);

    let w_explicit = length::resolve(child_style.width.as_ref(), parent_inner_w, ctx);
    let h_explicit = length::resolve(child_style.height.as_ref(), parent_h_for_pct, ctx);

    let (main_explicit, cross_explicit) = if is_row {
        (w_explicit, h_explicit)
    } else {
        (h_explicit, w_explicit)
    };

    let main_size = main_explicit.unwrap_or(0.0);
    let cross_size = match cross_explicit {
        Some(v) => v,
        None => match align {
            AlignItems::Stretch => parent_inner_cross.unwrap_or(0.0),
            _ => 0.0,
        },
    };

    if is_row {
        (main_size, cross_size)
    } else {
        (cross_size, main_size)
    }
}

fn distribution(
    justify: &JustifyContent,
    free: f32,
    gap: f32,
    n: f32,
) -> (f32, f32) {
    use JustifyContent::*;
    match justify {
        Start | FlexStart | Left => (0.0, gap),
        End | FlexEnd | Right => (free, gap),
        Center => (free * 0.5, gap),
        SpaceBetween => {
            if n > 1.0 {
                (0.0, gap + free / (n - 1.0))
            } else {
                (0.0, gap)
            }
        }
        SpaceAround => {
            if n > 0.0 {
                let s = free / n;
                (s * 0.5, gap + s)
            } else {
                (0.0, gap)
            }
        }
        SpaceEvenly => {
            if n > 0.0 {
                let s = free / (n + 1.0);
                (s, gap + s)
            } else {
                (0.0, gap)
            }
        }
    }
}
