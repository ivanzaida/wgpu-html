//! Inline layout: horizontal flow with line breaking.

use crate::box_tree::LayoutBox;
use crate::engine::LayoutContext;
use crate::geometry::Point;

/// Layout inline content — simple single-line for now.
pub fn layout_inline(b: &mut LayoutBox, _ctx: &LayoutContext, pos: Point) {
    let cursor_x = pos.x;
    let line_height: f32 = 16.0; // TODO: from font metrics

    // Text content
    if let Some(text) = get_text(b.node) {
        let char_count = text.chars().count() as f32;
        b.content.width = char_count * 8.0; // rough: 8px per char
        b.content.height = line_height;
        b.content.x = cursor_x;
        b.content.y = pos.y;
    } else {
        b.content.width = 0.0;
        b.content.height = line_height;
    }
}

fn get_text(node: &lui_html_parser::HtmlNode) -> Option<String> {
    if let lui_html_parser::HtmlElement::Text(t) = &node.element {
        Some(t.to_string())
    } else {
        None
    }
}
