//! Block layout: stack children vertically in a block formatting context.

use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_tree::{BoxKind, LayoutBox};
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

/// Layout a block-level box and its children.
pub fn layout_block<'a>(
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

    let font_size = sizes::resolve_font_size(b.style, ctx.parent_font_size);
    let self_ctx = LayoutContext { parent_font_size: font_size, ..*ctx };

    let frame_h = border.horizontal() + padding.horizontal();
    let available = ctx.containing_width - margin.edges.horizontal() - frame_h;

    // Width resolution with box-sizing
    let raw_w = sizes::resolve_length_ctx(b.style.width, ctx.containing_width, &self_ctx);
    let content_w = match raw_w {
        Some(w) => {
            if is_border_box(b.style) {
                (w - frame_h).max(0.0)
            } else {
                w
            }
        }
        None => available.max(0.0),
    };
    let min_w = sizes::resolve_length_ctx(b.style.min_width, ctx.containing_width, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_h).max(0.0) } else { v });
    let max_w = sizes::resolve_length_ctx(b.style.max_width, ctx.containing_width, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_h).max(0.0) } else { v });
    b.content.width = sizes::clamp_with_minmax(content_w.min(available.max(0.0)), min_w, max_w);

    // Auto margin centering: if width is explicit and both left+right margins are auto,
    // distribute remaining space equally.
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

    let frame_v = border.vertical() + padding.vertical();
    let explicit_h = sizes::resolve_length_ctx(b.style.height, ctx.containing_height, &self_ctx)
        .map(|h| if is_border_box(b.style) { (h - frame_v).max(0.0) } else { h });

    let child_ctx = LayoutContext {
        containing_width: b.content.width,
        containing_height: explicit_h.unwrap_or(0.0),
        parent_font_size: font_size,
        ..*ctx
    };
    let containing_block = Rect::new(b.content.x, b.content.y, b.content.width, explicit_h.unwrap_or(0.0));

    // Parent-child margin collapsing: if parent has no top border/padding,
    // first in-flow child's top margin collapses with parent's top margin.
    let parent_has_top_separator = border.top > 0.0 || padding.top > 0.0;
    let parent_has_bottom_separator = border.bottom > 0.0 || padding.bottom > 0.0;
    let creates_bfc = is_bfc_root(b.style);

    let mut cursor_y = b.content.y;
    let mut prev_margin_bottom = 0.0_f32;
    let mut is_first_in_flow = true;

    for child in b.children.iter_mut() {
        if positioned::is_out_of_flow(child.style) {
            let static_pos = Point::new(b.content.x, cursor_y);
            let placeholder = LayoutBox::new(child.kind, child.node, child.style);
            let old = std::mem::replace(child, placeholder);
            let mut result = old;
            positioned::layout_out_of_flow(
                &mut result, &child_ctx, static_pos, containing_block, text_ctx, rects,
            );
            rects.push((result.node, result.content));
            *child = result;
        } else {
            let placeholder = LayoutBox::new(BoxKind::Block, child.node, child.style);
            let old = std::mem::replace(child, placeholder);
            let mut result = crate::engine::layout_node(old, &child_ctx, Point::new(b.content.x, cursor_y), text_ctx, rects);

            let mt = result.margin.top;

            // Parent-first-child margin collapsing
            if is_first_in_flow && !parent_has_top_separator && !creates_bfc {
                let parent_mt = b.margin.top;
                if parent_mt != 0.0 || mt != 0.0 {
                    let collapsed = collapse_margins(parent_mt, mt);
                    b.margin.top = collapsed;
                    // Shift child up by the child's top margin (it's absorbed into parent)
                    let dy = -mt;
                    if dy.abs() > 0.001 {
                        result.content.y += dy;
                        for gc in &mut result.children {
                            crate::positioned::translate_recursive_pub(gc, 0.0, dy);
                        }
                        cursor_y += dy;
                    }
                    // Reposition parent based on new margin
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
            }

            is_first_in_flow = false;
            prev_margin_bottom = result.margin.bottom;
            *child = result;
            cursor_y += child.outer_height();

            positioned::apply_relative_offset(child, b.content.width, b.content.height);
        }
    }

    // Parent-last-child margin collapsing
    if !parent_has_bottom_separator && !creates_bfc && !is_first_in_flow {
        let collapsed = collapse_margins(b.margin.bottom, prev_margin_bottom);
        b.margin.bottom = collapsed;
    }

    let content_h = (cursor_y - b.content.y).max(0.0);
    let raw_h = explicit_h.unwrap_or(content_h);
    let min_h = sizes::resolve_length_ctx(b.style.min_height, ctx.containing_height, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_v).max(0.0) } else { v });
    let max_h = sizes::resolve_length_ctx(b.style.max_height, ctx.containing_height, &self_ctx)
        .map(|v| if is_border_box(b.style) { (v - frame_v).max(0.0) } else { v });
    b.content.height = sizes::clamp_with_minmax(raw_h, min_h, max_h);
}

fn collapse_margins(a: f32, b: f32) -> f32 {
    if a >= 0.0 && b >= 0.0 {
        a.max(b)
    } else if a < 0.0 && b < 0.0 {
        a.min(b)
    } else {
        a + b
    }
}

fn is_bfc_root(style: &lui_cascade::ComputedStyle) -> bool {
    let overflow_x = css_str(style.overflow_x);
    let overflow_y = css_str(style.overflow_y);
    let display = css_str(style.display);
    matches!(overflow_x, "hidden" | "auto" | "scroll" | "clip")
        || matches!(overflow_y, "hidden" | "auto" | "scroll" | "clip")
        || matches!(display, "flex" | "grid" | "inline-block" | "flow-root")
        || positioned::is_out_of_flow(style)
}

/// Layout an anonymous block wrapper — contains inline-level children
/// that should flow horizontally, not stack vertically.
pub fn layout_anonymous_block<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) {
    crate::flow::layout_inline(b, ctx, pos, text_ctx, rects);
}
