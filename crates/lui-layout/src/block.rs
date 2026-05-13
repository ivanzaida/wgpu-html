//! Block layout: stack children vertically in a block formatting context.

use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_tree::{BoxKind, LayoutBox, Overflow, ScrollInfo};
use crate::context::LayoutContext;
use crate::geometry::Point;
use crate::positioned;
use crate::sides;
use crate::sizes;
use crate::text::TextContext;

fn css_str(v: Option<&lui_core::CssValue>) -> &str {
    match v {
        Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
        _ => "",
    }
}

fn is_border_box(style: &lui_cascade::ComputedStyle) -> bool {
    css_str(style.box_sizing) == "border-box"
}

fn parse_overflow(v: Option<&lui_core::CssValue>) -> Overflow {
    match css_str(v) {
        "hidden" => Overflow::Hidden,
        "scroll" => Overflow::Scroll,
        "auto" => Overflow::Auto,
        "clip" => Overflow::Clip,
        _ => Overflow::Visible,
    }
}

const DEFAULT_SCROLLBAR_WIDTH: f32 = 15.0;
const THIN_SCROLLBAR_WIDTH: f32 = 8.0;

fn resolve_scrollbar_width(style: &lui_cascade::ComputedStyle) -> f32 {
    match css_str(style.scrollbar_width) {
        "none" => 0.0,
        "thin" => THIN_SCROLLBAR_WIDTH,
        _ => DEFAULT_SCROLLBAR_WIDTH,
    }
}

// ── Float tracking ────────────────────────────────────────────────────

struct FloatRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct FloatList {
    lefts: Vec<FloatRect>,
    rights: Vec<FloatRect>,
}

impl FloatList {
    fn new() -> Self { Self { lefts: Vec::new(), rights: Vec::new() } }

    fn clear_left_y(&self) -> f32 {
        self.lefts.iter().map(|f| f.y + f.height).fold(0.0_f32, f32::max)
    }

    fn clear_right_y(&self) -> f32 {
        self.rights.iter().map(|f| f.y + f.height).fold(0.0_f32, f32::max)
    }

    fn clear_both_y(&self) -> f32 {
        self.clear_left_y().max(self.clear_right_y())
    }

    fn left_intrusion_at(&self, y: f32, h: f32) -> f32 {
        let mut max_right = 0.0_f32;
        for f in &self.lefts {
            if f.y < y + h && f.y + f.height > y {
                max_right = max_right.max(f.x + f.width);
            }
        }
        max_right
    }

    fn right_intrusion_at(&self, y: f32, h: f32, container_right: f32) -> f32 {
        let mut min_left = container_right;
        for f in &self.rights {
            if f.y < y + h && f.y + f.height > y {
                min_left = min_left.min(f.x);
            }
        }
        container_right - min_left
    }

    fn available_at(&self, y: f32, h: f32, container_x: f32, container_w: f32) -> (f32, f32) {
        let left_edge = self.left_intrusion_at(y, h);
        let right_intr = self.right_intrusion_at(y, h, container_x + container_w);
        let start = left_edge.max(container_x);
        let end = (container_x + container_w - right_intr).max(start);
        (start, end - start)
    }

    fn next_left_x(&self, y: f32, h: f32) -> f32 {
        self.left_intrusion_at(y, h)
    }

    fn next_right_x(&self, y: f32, h: f32, container_right: f32) -> f32 {
        let mut min_left = container_right;
        for f in &self.rights {
            if f.y < y + h && f.y + f.height > y {
                min_left = min_left.min(f.x);
            }
        }
        min_left
    }
}

// ── Block layout ──────────────────────────────────────────────────────

/// Layout a block-level box and its children.
pub fn layout_block<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
    cache: &crate::incremental::CacheView,
) {
    let margin = sides::resolve_margin_against(b.style, ctx.containing_width);
    let border = sides::resolve_border(b.style);
    let padding = sides::resolve_padding_against(b.style, ctx.containing_width);

    b.margin = margin.edges;
    b.border = border;
    b.padding = padding;

    let font_size = sizes::resolve_font_size(b.style, ctx.parent_font_size);
    let self_ctx = LayoutContext { parent_font_size: font_size, ..*ctx };

    let frame_h = border.horizontal() + padding.horizontal();
    let available = ctx.containing_width - margin.edges.horizontal() - frame_h;

    let raw_w = sizes::resolve_length_ctx(b.style.width, ctx.containing_width, &self_ctx);
    let content_w = match raw_w {
        Some(w) => if is_border_box(b.style) { (w - frame_h).max(0.0) } else { w },
        None => available.max(0.0),
    };
    let min_w = sizes::resolve_length_ctx(b.style.min_width, ctx.containing_width, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_h).max(0.0) } else { v });
    let max_w = sizes::resolve_length_ctx(b.style.max_width, ctx.containing_width, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_h).max(0.0) } else { v });
    b.content.width = sizes::clamp_with_minmax(content_w.min(available.max(0.0)), min_w, max_w);

    let has_auto_left = margin.auto_mask & (1 << 3) != 0;
    let has_auto_right = margin.auto_mask & (1 << 1) != 0;
    if raw_w.is_some() && has_auto_left && has_auto_right {
        let free = (ctx.containing_width - b.content.width - frame_h).max(0.0);
        b.margin.left = free / 2.0;
        b.margin.right = free / 2.0;
    } else if raw_w.is_some() && has_auto_left {
        let free = (ctx.containing_width - b.content.width - frame_h - margin.edges.right).max(0.0);
        b.margin.left = free;
    } else if raw_w.is_some() && has_auto_right {
        let free = (ctx.containing_width - b.content.width - frame_h - margin.edges.left).max(0.0);
        b.margin.right = free;
    }

    b.content.x = pos.x + b.margin.left + border.left + padding.left;
    b.content.y = pos.y + b.margin.top + border.top + padding.top;

    // Overflow handling
    let ov_x = parse_overflow(b.style.overflow_x);
    let ov_y = parse_overflow(b.style.overflow_y);
    b.overflow_x = ov_x;
    b.overflow_y = ov_y;

    let scrollbar_w = resolve_scrollbar_width(b.style);
    let has_y_scrollbar = ov_y == Overflow::Scroll;
    let has_x_scrollbar = ov_x == Overflow::Scroll;
    let y_scrollbar_reduction = if has_y_scrollbar { scrollbar_w } else { 0.0 };
    let x_scrollbar_reduction = if has_x_scrollbar { scrollbar_w } else { 0.0 };

    // Reduce content area for always-visible scrollbars
    b.content.width = (b.content.width - y_scrollbar_reduction).max(0.0);

    let frame_v = border.vertical() + padding.vertical();
    let explicit_h = sizes::resolve_length_ctx(b.style.height, ctx.containing_height, &self_ctx)
        .map(|h| if is_border_box(b.style) { (h - frame_v).max(0.0) } else { h })
        .map(|h| (h - x_scrollbar_reduction).max(0.0));

    let child_ctx = LayoutContext {
        containing_width: b.content.width,
        containing_height: explicit_h.unwrap_or(0.0),
        parent_font_size: font_size,
        ..*ctx
    };
    let containing_block = Rect::new(b.content.x, b.content.y, b.content.width, explicit_h.unwrap_or(0.0));

    let parent_has_top_separator = border.top > 0.0 || padding.top > 0.0;
    let parent_has_bottom_separator = border.bottom > 0.0 || padding.bottom > 0.0;
    let creates_bfc = is_bfc_root(b.style);
    let text_align = css_str(b.style.text_align);

    let mut cursor_y = b.content.y;
    let mut prev_margin_bottom = 0.0_f32;
    let mut is_first_in_flow = true;
    let mut floats = FloatList::new();

    for child in b.children.iter_mut() {
        if positioned::is_out_of_flow(child.style) {
            let static_pos = Point::new(b.content.x, cursor_y);
            let placeholder = LayoutBox::new(child.kind, child.node, child.style);
            let old = std::mem::replace(child, placeholder);
            let mut result = old;
            positioned::layout_out_of_flow(
                &mut result, &child_ctx, static_pos, containing_block, text_ctx, rects, cache,
            );
            rects.push((result.node, result.content));
            *child = result;
            continue;
        }

        let float_val = css_str(child.style.float);
        let clear_val = css_str(child.style.clear);

        // Apply clear before positioning
        if matches!(clear_val, "left" | "both") {
            let clear_y = floats.clear_left_y();
            if clear_y > cursor_y { cursor_y = clear_y; }
        }
        if matches!(clear_val, "right" | "both") {
            let clear_y = floats.clear_right_y();
            if clear_y > cursor_y { cursor_y = clear_y; }
        }

        if float_val == "left" || float_val == "right" {
            // Float child: layout then position at edge
            let placeholder = LayoutBox::new(BoxKind::Block, child.node, child.style);
            let old = std::mem::replace(child, placeholder);
            let mut result = crate::engine::layout_node(old, &child_ctx, Point::new(b.content.x, cursor_y), text_ctx, rects, cache);

            let float_w = result.outer_width();
            let float_h = result.outer_height();

            if float_val == "left" {
                let fx = floats.next_left_x(cursor_y, float_h).max(b.content.x);
                let dx = fx - (result.content.x - result.padding.left - result.border.left - result.margin.left);
                if dx.abs() > 0.001 {
                    crate::positioned::translate_recursive_pub(&mut result, dx, 0.0);
                }
                floats.lefts.push(FloatRect { x: fx, y: cursor_y, width: float_w, height: float_h });
            } else {
                let container_right = b.content.x + b.content.width;
                let fx = floats.next_right_x(cursor_y, float_h, container_right) - float_w;
                let fx = fx.max(b.content.x);
                let dx = fx - (result.content.x - result.padding.left - result.border.left - result.margin.left);
                if dx.abs() > 0.001 {
                    crate::positioned::translate_recursive_pub(&mut result, dx, 0.0);
                }
                floats.rights.push(FloatRect { x: fx, y: cursor_y, width: float_w, height: float_h });
            }

            is_first_in_flow = false;
            *child = result;
        } else {
            // Normal in-flow child — adjust for floats
            let (avail_x, avail_w) = floats.available_at(cursor_y, 1.0, b.content.x, b.content.width);
            let float_adjusted_ctx = if avail_w < b.content.width - 0.5 {
                LayoutContext { containing_width: avail_w, ..child_ctx }
            } else {
                child_ctx
            };

            let placeholder = LayoutBox::new(BoxKind::Block, child.node, child.style);
            let old = std::mem::replace(child, placeholder);
            let mut result = crate::engine::layout_node(old, &float_adjusted_ctx, Point::new(avail_x, cursor_y), text_ctx, rects, cache);

            let mt = result.margin.top;

            // Check if this is an empty self-collapsing block BEFORE
            // running normal sibling collapse, since it needs special handling.
            let is_empty = result.content.height == 0.0
                && result.border.vertical() == 0.0
                && result.padding.vertical() == 0.0
                && result.children.is_empty();

            if is_empty && !is_first_in_flow {
                // CSS2 §8.3.1: empty block self-collapses, then collapses
                // with the previous sibling's bottom margin. We advance
                // cursor by the net new margin contribution.
                let self_collapsed = collapse_margins(result.margin.top, result.margin.bottom);
                let merged = collapse_margins(prev_margin_bottom, self_collapsed);
                let advance = (merged - prev_margin_bottom).max(0.0);
                cursor_y += advance;
                prev_margin_bottom = merged;
                is_first_in_flow = false;
                *child = result;
            } else if is_first_in_flow && !parent_has_top_separator && !creates_bfc {
                let parent_mt = b.margin.top;
                if parent_mt != 0.0 || mt != 0.0 {
                    let collapsed = collapse_margins(parent_mt, mt);
                    b.margin.top = collapsed;
                    let dy = -mt;
                    if dy.abs() > 0.001 {
                        result.content.y += dy;
                        for gc in &mut result.children {
                            crate::positioned::translate_recursive_pub(gc, 0.0, dy);
                        }
                        cursor_y += dy;
                    }
                    let new_content_y = pos.y + b.margin.top + border.top + padding.top;
                    if (new_content_y - b.content.y).abs() > 0.001 {
                        let parent_dy = new_content_y - b.content.y;
                        b.content.y = new_content_y;
                        cursor_y += parent_dy;
                        result.content.y += parent_dy;
                        for gc in &mut result.children {
                            crate::positioned::translate_recursive_pub(gc, 0.0, parent_dy);
                        }
                    }
                }
                is_first_in_flow = false;
                if is_empty {
                    let self_collapsed = collapse_margins(result.margin.top, result.margin.bottom);
                    prev_margin_bottom = self_collapsed;
                    *child = result;
                } else {
                    prev_margin_bottom = result.margin.bottom;
                    *child = result;
                    cursor_y += child.outer_height();
                }
            } else {
                // Sibling margin collapsing
                let mb_prev = prev_margin_bottom;
                if mb_prev != 0.0 || mt != 0.0 {
                    let collapsed = collapse_margins(mb_prev, mt);
                    let sum = mb_prev + mt;
                    let overlap = sum - collapsed;
                    if overlap.abs() > 0.001 {
                        let dy = -overlap;
                        result.content.y += dy;
                        for gc in &mut result.children {
                            crate::positioned::translate_recursive_pub(gc, 0.0, dy);
                        }
                        cursor_y -= overlap;
                    }
                }
                is_first_in_flow = false;
                prev_margin_bottom = result.margin.bottom;
                *child = result;
                cursor_y += child.outer_height();
            }

            apply_text_align(child, text_align, b.content.width);
            positioned::apply_relative_offset(child, b.content.width, b.content.height);
        }
    }

    // BFC root contains floats: extend height to cover them
    if creates_bfc {
        let float_bottom = floats.clear_both_y();
        if float_bottom > cursor_y { cursor_y = float_bottom; }
    }

    if !parent_has_bottom_separator && !creates_bfc && !is_first_in_flow {
        let collapsed = collapse_margins(b.margin.bottom, prev_margin_bottom);
        b.margin.bottom = collapsed;
    }

    // Shrink-to-fit for inline-block with auto width
    if matches!(b.kind, BoxKind::InlineBlock | BoxKind::InlineFlex | BoxKind::InlineGrid) && raw_w.is_none() {
        let mut max_right = 0.0_f32;
        for child in b.children.iter() {
            let cr = child.content.x + child.content.width
                + child.padding.right + child.border.right + child.margin.right
                - b.content.x;
            max_right = max_right.max(cr);
        }
        if max_right > 0.0 && max_right < b.content.width {
            b.content.width = max_right;
        }
    }

    let content_h = (cursor_y - b.content.y).max(0.0);
    let raw_h = explicit_h.unwrap_or(content_h);
    let min_h = sizes::resolve_length_ctx(b.style.min_height, ctx.containing_height, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_v).max(0.0) } else { v });
    let max_h = sizes::resolve_length_ctx(b.style.max_height, ctx.containing_height, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_v).max(0.0) } else { v });
    b.content.height = sizes::clamp_with_minmax(raw_h, min_h, max_h);

    // Overflow finalization: compute scroll extent, handle auto scrollbars,
    // set clip rect.
    let is_scroll_container = !matches!(ov_x, Overflow::Visible)
        || !matches!(ov_y, Overflow::Visible);

    if is_scroll_container {
        // Compute maximum child extent (scroll area)
        let mut max_child_right = 0.0_f32;
        let mut max_child_bottom = 0.0_f32;
        for child in &b.children {
            let cr = child.content.x + child.content.width
                + child.padding.right + child.border.right + child.margin.right;
            let cb = child.content.y + child.content.height
                + child.padding.bottom + child.border.bottom + child.margin.bottom;
            max_child_right = max_child_right.max(cr);
            max_child_bottom = max_child_bottom.max(cb);
        }

        let scroll_width = (max_child_right - b.content.x).max(b.content.width);
        let scroll_height = (max_child_bottom - b.content.y).max(b.content.height);

        let overflows_x = scroll_width > b.content.width + 0.5;
        let overflows_y = scroll_height > b.content.height + 0.5;

        // Auto scrollbar: only show when content overflows
        let show_y_scrollbar = has_y_scrollbar
            || (ov_y == Overflow::Auto && overflows_y);
        let show_x_scrollbar = has_x_scrollbar
            || (ov_x == Overflow::Auto && overflows_x);

        // If auto scrollbar now appears, reduce content area retroactively.
        // This is a simplification — browsers do a second layout pass.
        if show_y_scrollbar && !has_y_scrollbar {
            b.content.width = (b.content.width - scrollbar_w).max(0.0);
        }
        if show_x_scrollbar && !has_x_scrollbar {
            b.content.height = (b.content.height - scrollbar_w).max(0.0);
        }

        let sb_w = if show_y_scrollbar || show_x_scrollbar { scrollbar_w } else { 0.0 };

        b.scroll = Some(ScrollInfo {
            scroll_width,
            scroll_height,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scrollbar_width: sb_w,
        });

        // Clip rect = padding box (content + padding area, excluding border)
        b.clip = Some(b.padding_rect());
    }
}

fn collapse_margins(a: f32, b: f32) -> f32 {
    if a >= 0.0 && b >= 0.0 { a.max(b) }
    else if a < 0.0 && b < 0.0 { a.min(b) }
    else { a + b }
}

fn is_bfc_root(style: &lui_cascade::ComputedStyle) -> bool {
    let overflow_x = css_str(style.overflow_x);
    let overflow_y = css_str(style.overflow_y);
    let display = css_str(style.display);
    let float = css_str(style.float);
    matches!(overflow_x, "hidden" | "auto" | "scroll" | "clip")
        || matches!(overflow_y, "hidden" | "auto" | "scroll" | "clip")
        || matches!(display, "flex" | "inline-flex" | "grid" | "inline-grid" | "inline-block" | "flow-root")
        || matches!(float, "left" | "right")
        || positioned::is_out_of_flow(style)
}

fn apply_text_align(child: &mut LayoutBox, text_align: &str, container_width: f32) {
    if matches!(text_align, "" | "left" | "start") { return; }
    if !matches!(child.kind, BoxKind::AnonymousBlock | BoxKind::AnonymousInline) { return; }

    let content_width = child.content.width;
    let free = (container_width - content_width).max(0.0);
    match text_align {
        "center" => {
            if free > 0.001 { positioned::translate_recursive_pub(child, free / 2.0, 0.0); }
        }
        "right" | "end" => {
            if free > 0.001 { positioned::translate_recursive_pub(child, free, 0.0); }
        }
        "justify" => {
            child.content.width = container_width;
        }
        _ => {}
    }
}

/// Layout an anonymous block wrapper — contains inline-level children
/// that should flow horizontally, not stack vertically.
pub fn layout_anonymous_block<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
    cache: &crate::incremental::CacheView,
) {
    crate::flow::layout_inline(b, ctx, pos, text_ctx, rects, cache);
}
