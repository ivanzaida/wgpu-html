//! Layout engine: converts a `StyledNode` tree into a `LayoutBox` tree
//! with computed positions and sizes.

use lui_cascade::StyledNode;
use lui_css_parser::CssValue;
use lui_html_parser::{HtmlNode, Rect};

use crate::box_tree::{BoxKind, LayoutBox, LayoutTree};
use crate::geometry::{Point, RectEdges};

use crate::flow;

/// Context passed through the layout tree walk.
pub struct LayoutContext {
    /// Viewport width in px.
    pub viewport_width: f32,
    /// Viewport height in px.
    pub viewport_height: f32,
    /// Available width for content (content-box width of containing block).
    pub available_width: f32,
}

/// Compute layout for the entire styled tree.
pub fn layout_tree<'a>(styled: &StyledNode<'a>, viewport_width: f32, viewport_height: f32) -> LayoutTree<'a> {
    let ctx = LayoutContext {
        viewport_width,
        viewport_height,
        available_width: viewport_width,
    };
    let mut rects = Vec::new();
    let root = build_box(styled);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), &mut rects);
    LayoutTree { root, rects }
}

// ---------------------------------------------------------------------------
// Box generation
// ---------------------------------------------------------------------------

fn build_box<'a>(styled: &StyledNode<'a>) -> LayoutBox<'a> {
    if styled.node.element.is_text() {
        let mut b = LayoutBox::new(BoxKind::AnonymousInline, styled.node);
        return b;
    }

    let kind = resolve_box_kind(styled);

    let mut b = LayoutBox::new(kind, styled.node);

    // Build children — handle anonymous block box generation for inline
    // children between block siblings.
    let mut child_boxes = Vec::new();
    let mut pending_inlines: Vec<&StyledNode> = Vec::new();

    for child in &styled.children {
        if child.node.element.is_text() {
            pending_inlines.push(child);
            continue;
        }

        let child_kind = resolve_box_kind(child);
        let is_block = matches!(child_kind, BoxKind::Block | BoxKind::FlexContainer | BoxKind::GridContainer);
        let is_inline = matches!(child_kind, BoxKind::Inline | BoxKind::InlineBlock);

        if is_block {
            flush_inlines(&mut pending_inlines, &mut child_boxes);
            child_boxes.push(build_box(child));
        } else if is_inline {
            pending_inlines.push(child);
        } else {
            child_boxes.push(build_box(child));
        }
    }
    flush_inlines(&mut pending_inlines, &mut child_boxes);

    b.children = child_boxes;
    b
}

fn flush_inlines<'a>(pending: &mut Vec<&StyledNode<'a>>, out: &mut Vec<LayoutBox<'a>>) {
    if pending.is_empty() { return; }
    // Wrap inline children in an anonymous block box
    let mut anon = LayoutBox::new(BoxKind::AnonymousBlock, pending[0].node);
    for s in pending.drain(..) {
        anon.children.push(build_box(s));
    }
    out.push(anon);
}

fn resolve_box_kind(styled: &StyledNode) -> BoxKind {
    match styled.style.display {
        Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => {
            match s.as_ref() {
                "block" => BoxKind::Block,
                "inline" => BoxKind::Inline,
                "inline-block" => BoxKind::InlineBlock,
                "flex" => BoxKind::FlexContainer,
                "grid" => BoxKind::GridContainer,
                "none" => BoxKind::Block, // not rendered, but keep for now
                _ => BoxKind::Block,
            }
        }
        _ => BoxKind::Block, // default
    }
}

// ---------------------------------------------------------------------------
// Recursive layout
// ---------------------------------------------------------------------------

fn layout_node<'a>(
    mut b: LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    rects: &mut Vec<(*const HtmlNode, Rect)>,
) -> LayoutBox<'a> {
    // Resolve margins, borders, padding from style (TODO: from ComputedStyle)
    b.margin = resolve_margin(&b);
    b.border = resolve_border(&b);
    b.padding = resolve_padding(&b);

    let inner_width = (ctx.available_width - b.margin.horizontal() - b.border.horizontal() - b.padding.horizontal()).max(0.0);

    match b.kind {
        BoxKind::Block | BoxKind::FlexContainer | BoxKind::GridContainer | BoxKind::Root => {
            b.content.width = inner_width;
            b.content.x = pos.x + b.margin.left + b.border.left + b.padding.left;
            b.content.y = pos.y + b.margin.top + b.border.top + b.padding.top;

            let child_ctx = LayoutContext {
                available_width: inner_width,
                ..*ctx
            };

            let mut cursor_y = b.content.y;
            for child in b.children.iter_mut() {
                let child_result = layout_node(
                    std::mem::replace(child, LayoutBox::new(BoxKind::Block, b.node)),
                    &child_ctx,
                    Point::new(b.content.x, cursor_y),
                    rects,
                );
                *child = child_result;
                cursor_y += child.outer_height();
            }
            b.content.height = cursor_y - b.content.y;
        }
        BoxKind::Inline | BoxKind::AnonymousInline => {
            flow::layout_inline(&mut b, ctx, pos);
        }
        BoxKind::AnonymousBlock => {
            b.content.width = inner_width;
            b.content.x = pos.x + b.margin.left + b.border.left + b.padding.left;
            b.content.y = pos.y + b.margin.top + b.border.top + b.padding.top;

            let child_ctx = LayoutContext {
                available_width: inner_width,
                ..*ctx
            };

            let mut cursor_y = b.content.y;
            for child in b.children.iter_mut() {
                let child_result = layout_node(
                    std::mem::replace(child, LayoutBox::new(BoxKind::Block, b.node)),
                    &child_ctx,
                    Point::new(b.content.x, cursor_y),
                    rects,
                );
                *child = child_result;
                cursor_y += child.outer_height();
            }
            b.content.height = cursor_y - b.content.y;
        }
        _ => {}
    }

    rects.push((b.node as *const HtmlNode, b.content));
    b
}

// ---------------------------------------------------------------------------
// Style → geometry helpers (TODO: real ComputedStyle property access)
// ---------------------------------------------------------------------------

fn resolve_margin(_b: &LayoutBox) -> RectEdges<f32> {
    RectEdges::new(0.0, 0.0, 0.0, 0.0)
}

fn resolve_border(_b: &LayoutBox) -> RectEdges<f32> {
    RectEdges::new(0.0, 0.0, 0.0, 0.0)
}

fn resolve_padding(_b: &LayoutBox) -> RectEdges<f32> {
    RectEdges::new(0.0, 0.0, 0.0, 0.0)
}
