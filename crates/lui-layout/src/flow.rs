//! Inline layout: horizontal flow with line breaking.
//! Uses `TextContext` for shaping and line measurement.

use crate::box_tree::LayoutBox;
use crate::context::LayoutContext;
use crate::geometry::Point;
use crate::text::TextContext;

/// Layout inline content. Text nodes are shaped; block-in-inline children
/// are laid out recursively (for inline-block).
pub fn layout_inline(b: &mut LayoutBox, ctx: &LayoutContext, pos: Point, text_ctx: &mut TextContext) {
    if get_text(b.node).is_some() {
        let text = get_text(b.node).unwrap_or_default();
        let run = text_ctx.shape_run(&text, b.style);
        b.content.width = run.width;
        b.content.height = run.height;
        b.content.x = pos.x;
        b.content.y = pos.y;
    } else {
        b.content.width = 0.0;
        b.content.height = 16.0; // fallback line height
        b.content.x = pos.x;
        b.content.y = pos.y;

        // Recurse into inline children
        let mut cursor_x = pos.x;
        for child in b.children.iter_mut() {
            layout_inline(child, ctx, Point::new(cursor_x, pos.y), text_ctx);
            cursor_x += child.content.width;
        }
    }
}

fn get_text(node: &lui_parse::HtmlNode) -> Option<String> {
    if let lui_parse::HtmlElement::Text(t) = &node.element {
        Some(t.to_string())
    } else {
        None
    }
}
