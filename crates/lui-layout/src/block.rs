//! Block layout: stack children vertically in a block formatting context.

use lui_html_parser::{HtmlNode, Rect};

use crate::box_tree::{BoxKind, LayoutBox};
use crate::context::LayoutContext;
use crate::geometry::Point;
use crate::sides;
use crate::sizes;
use crate::text::TextContext;

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

    let available = ctx.containing_width - margin.edges.horizontal() - border.horizontal() - padding.horizontal();
    let w = sizes::resolve_length(b.style.width, ctx.containing_width).unwrap_or(available.max(0.0));
    b.content.width = w.min(available.max(0.0));
    b.content.x = pos.x + margin.edges.left + border.left + padding.left;
    b.content.y = pos.y + margin.edges.top + border.top + padding.top;

    let child_ctx = LayoutContext { containing_width: b.content.width, ..*ctx };
    let mut cursor_y = b.content.y;
    for child in b.children.iter_mut() {
        let placeholder = LayoutBox::new(BoxKind::Block, child.node, child.style);
        let old = std::mem::replace(child, placeholder);
        let result = crate::engine::layout_node(old, &child_ctx, Point::new(b.content.x, cursor_y), text_ctx, rects);
        *child = result;
        cursor_y += child.outer_height();
    }
    b.content.height = (cursor_y - b.content.y).max(0.0);
}

/// Layout an anonymous block wrapper.
pub fn layout_anonymous_block<'a>(
    b: &mut LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) {
    layout_block(b, ctx, pos, text_ctx, rects)
}
