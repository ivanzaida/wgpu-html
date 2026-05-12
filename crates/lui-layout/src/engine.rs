//! Layout engine: entry point and recursive dispatcher.

use lui_cascade::StyledNode;
use lui_html_parser::{HtmlNode, Rect};

use crate::box_gen::build_box;
use crate::box_tree::{BoxKind, LayoutBox, LayoutTree};
use crate::context::LayoutContext;
use crate::flow;
use crate::geometry::Point;
use crate::text::TextContext;

/// Compute layout for the entire styled tree.
pub fn layout_tree<'a>(styled: &'a StyledNode<'a>, viewport_width: f32, viewport_height: f32) -> LayoutTree<'a> {
    let ctx = LayoutContext::new(viewport_width, viewport_height);
    let mut text_ctx = TextContext::new();
    let mut rects = Vec::new();
    let root = build_box(styled);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects);
    LayoutTree { root, rects }
}

pub fn layout_node<'a>(
    mut b: LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
) -> LayoutBox<'a> {
    match b.kind {
        BoxKind::Block | BoxKind::FlexContainer | BoxKind::GridContainer | BoxKind::Root => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects);
        }
        BoxKind::Inline | BoxKind::AnonymousInline => {
            flow::layout_inline(&mut b, ctx, pos, text_ctx);
        }
        BoxKind::AnonymousBlock => {
            crate::block::layout_anonymous_block(&mut b, ctx, pos, text_ctx, rects);
        }
        _ => {}
    }

    rects.push((b.node, b.content));
    b
}
