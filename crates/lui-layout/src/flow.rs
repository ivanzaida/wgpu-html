//! Inline layout: horizontal flow with line breaking.

use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_tree::{BoxKind, LayoutBox};
use crate::context::LayoutContext;
use crate::geometry::Point;
use crate::sides;
use crate::sizes;
use crate::text::TextContext;

/// Layout inline content with line wrapping.
pub fn layout_inline<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) {
    if let Some(text) = get_text(b.node) {
        layout_text_node(b, ctx, pos, &text, text_ctx);
    } else {
        layout_inline_container(b, ctx, pos, text_ctx, rects);
    }
}

fn layout_text_node(
    b: &mut LayoutBox,
    ctx: &LayoutContext,
    pos: Point,
    text: &str,
    text_ctx: &mut TextContext,
) {
    let style = lui_glyph::text_style_from_cascade(b.style);
    let max_width = ctx.containing_width;

    if max_width > 0.0 && text.len() > 1 {
        let lines = text_ctx.font_ctx.break_into_lines(text, &style, max_width);
        if lines.is_empty() {
            b.content.x = pos.x;
            b.content.y = pos.y;
            b.content.width = 0.0;
            b.content.height = 0.0;
            return;
        }
        let mut total_height = 0.0_f32;
        let mut max_line_width = 0.0_f32;
        for line in &lines {
            max_line_width = max_line_width.max(line.width);
            total_height += line.height;
        }
        b.content.x = pos.x;
        b.content.y = pos.y;
        b.content.width = max_line_width;
        b.content.height = total_height;
    } else {
        let run = text_ctx.font_ctx.shape(text, &style);
        b.content.x = pos.x;
        b.content.y = pos.y;
        b.content.width = run.width;
        b.content.height = run.height;
    }
}

fn layout_inline_container<'a>(
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

    let frame_left = margin.edges.left + border.left + padding.left;

    b.content.x = pos.x + frame_left;
    b.content.y = pos.y + margin.edges.top + border.top + padding.top;

    let max_width = ctx.containing_width;
    let mut cursor_x = 0.0_f32;
    let mut cursor_y = 0.0_f32;
    let mut line_height = 0.0_f32;
    let mut max_line_width = 0.0_f32;

    for child in b.children.iter_mut() {
        let is_inline_block = child.kind == BoxKind::InlineBlock;

        if is_inline_block {
            let placeholder = LayoutBox::new(BoxKind::InlineBlock, child.node, child.style);
            let old = std::mem::replace(child, placeholder);
            let result = crate::engine::layout_node(
                old, ctx,
                Point::new(b.content.x + cursor_x, b.content.y + cursor_y),
                text_ctx, rects,
            );
            *child = result;
        } else {
            layout_inline(child, ctx, Point::new(b.content.x + cursor_x, b.content.y + cursor_y), text_ctx, rects);
        }

        let child_w = child.outer_width();
        let child_h = child.outer_height();

        if cursor_x > 0.0 && max_width > 0.0 && cursor_x + child_w > max_width {
            max_line_width = max_line_width.max(cursor_x);
            cursor_y += line_height;
            cursor_x = 0.0;
            line_height = 0.0;
            if is_inline_block {
                let placeholder = LayoutBox::new(BoxKind::InlineBlock, child.node, child.style);
                let old = std::mem::replace(child, placeholder);
                let result = crate::engine::layout_node(
                    old, ctx,
                    Point::new(b.content.x, b.content.y + cursor_y),
                    text_ctx, rects,
                );
                *child = result;
            } else {
                layout_inline(child, ctx, Point::new(b.content.x, b.content.y + cursor_y), text_ctx, rects);
            }
            let child_w_new = child.outer_width();
            let child_h_new = child.outer_height();
            cursor_x = child_w_new;
            line_height = line_height.max(child_h_new);
        } else {
            cursor_x += child_w;
            line_height = line_height.max(child_h);
        }
    }
    max_line_width = max_line_width.max(cursor_x);

    let explicit_w = sizes::resolve_length(b.style.width, ctx.containing_width);
    let explicit_h = sizes::resolve_length(b.style.height, ctx.containing_height);
    b.content.width = explicit_w.unwrap_or(max_line_width);
    b.content.height = explicit_h.unwrap_or(cursor_y + line_height);
}

fn get_text(node: &lui_core::HtmlNode) -> Option<String> {
    if let lui_core::HtmlElement::Text(t) = &node.element {
        Some(t.to_string())
    } else {
        None
    }
}
