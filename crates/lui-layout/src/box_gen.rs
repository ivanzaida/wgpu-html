//! Box generation: converts `StyledNode` tree into initial `LayoutBox` tree.
//! Handles anonymous box creation for inline-between-block content.

use lui_cascade::{ComputedStyle, StyledNode};
use lui_core::CssValue;

use crate::box_tree::{BoxKind, LayoutBox};

/// Build a `LayoutBox` tree from a `StyledNode` tree. Text nodes become
/// `AnonymousInline` boxes; elements are classified by `display`.
pub fn build_box<'a>(styled: &'a StyledNode<'a>) -> LayoutBox<'a> {
    if styled.node.element.is_text() {
        return LayoutBox::new(BoxKind::AnonymousInline, styled.node, &styled.style);
    }

    let kind = resolve_box_kind(&styled.style);
    let mut child_boxes = Vec::new();
    let mut pending_inlines: Vec<&StyledNode> = Vec::new();

    for child in &styled.children {
        if child.node.element.is_text() {
            pending_inlines.push(child);
            continue;
        }
        let child_kind = resolve_box_kind(&child.style);
        match child_kind {
            BoxKind::Block | BoxKind::FlexContainer | BoxKind::GridContainer => {
                flush_inlines(&mut pending_inlines, &mut child_boxes);
                child_boxes.push(build_box(child));
            }
            BoxKind::Inline | BoxKind::InlineBlock => {
                pending_inlines.push(child);
            }
            _ => child_boxes.push(build_box(child)),
        }
    }
    flush_inlines(&mut pending_inlines, &mut child_boxes);

    let mut b = LayoutBox::new(kind, styled.node, &styled.style);
    b.children = child_boxes;
    b
}

/// Wrap pending inline children in an anonymous block box.
fn flush_inlines<'a>(pending: &mut Vec<&'a StyledNode<'a>>, out: &mut Vec<LayoutBox<'a>>) {
    if pending.is_empty() { return; }
    let mut anon = LayoutBox::new(BoxKind::AnonymousBlock, pending[0].node, &pending[0].style);
    for s in pending.drain(..) {
        anon.children.push(build_box(s));
    }
    out.push(anon);
}

/// Map the `display` property to a `BoxKind`.
fn resolve_box_kind(style: &ComputedStyle) -> BoxKind {
    match style.display {
        Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => match s.as_ref() {
            "block" => BoxKind::Block,
            "inline" => BoxKind::Inline,
            "inline-block" => BoxKind::InlineBlock,
            "flex" => BoxKind::FlexContainer,
            "grid" => BoxKind::GridContainer,
            "none" => BoxKind::Block,
            _ => BoxKind::Block,
        },
        _ => BoxKind::Block,
    }
}
