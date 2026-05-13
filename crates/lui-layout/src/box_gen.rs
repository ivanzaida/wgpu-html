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

    let kind = resolve_box_kind_with_node(&styled.style, styled.node);
    let mut child_boxes = Vec::new();
    let mut pending_inlines: Vec<&StyledNode> = Vec::new();

    for child in &styled.children {
        collect_child(child, &mut pending_inlines, &mut child_boxes, styled);
    }
    flush_inlines(&mut pending_inlines, &mut child_boxes, styled);

    let mut b = LayoutBox::new(kind, styled.node, &styled.style);
    b.children = child_boxes;
    b
}

/// Wrap pending inline children in an anonymous block box.
fn collect_child<'a>(
    child: &'a StyledNode<'a>,
    pending_inlines: &mut Vec<&'a StyledNode<'a>>,
    child_boxes: &mut Vec<LayoutBox<'a>>,
    parent: &'a StyledNode<'a>,
) {
    if is_display_none(&child.style) { return; }
    if is_display_contents(&child.style) {
        for grandchild in &child.children {
            collect_child(grandchild, pending_inlines, child_boxes, parent);
        }
        return;
    }
    if child.node.element.is_text() {
        pending_inlines.push(child);
        return;
    }
    let child_kind = resolve_box_kind_with_node(&child.style, child.node);
    match child_kind {
        BoxKind::Block | BoxKind::FlexContainer | BoxKind::GridContainer
        | BoxKind::Table | BoxKind::TableRowGroup | BoxKind::TableCaption | BoxKind::ListItem => {
            flush_inlines(pending_inlines, child_boxes, parent);
            child_boxes.push(build_box(child));
        }
        BoxKind::Inline | BoxKind::InlineBlock | BoxKind::InlineFlex | BoxKind::InlineGrid => {
            pending_inlines.push(child);
        }
        _ => child_boxes.push(build_box(child)),
    }
}

fn is_display_contents(style: &ComputedStyle) -> bool {
    match style.display {
        Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => s.as_ref() == "contents",
        _ => false,
    }
}

fn flush_inlines<'a>(pending: &mut Vec<&'a StyledNode<'a>>, out: &mut Vec<LayoutBox<'a>>, parent: &'a StyledNode<'a>) {
    if pending.is_empty() { return; }
    let mut anon = LayoutBox::new(BoxKind::AnonymousBlock, parent.node, &parent.style);
    for s in pending.drain(..) {
        anon.children.push(build_box(s));
    }
    out.push(anon);
}

/// Map the `display` property to a `BoxKind`.
fn resolve_box_kind_with_node(style: &ComputedStyle, node: &lui_parse::HtmlNode) -> BoxKind {
    if let Some(kind) = resolve_display_property(style) {
        return kind;
    }
    match node.element.tag_name() {
        "table" => BoxKind::Table,
        "tr" => BoxKind::TableRow,
        "td" | "th" => BoxKind::TableCell,
        "li" => BoxKind::ListItem,
        "thead" | "tbody" | "tfoot" => BoxKind::TableRowGroup,
        "caption" => BoxKind::TableCaption,
        _ => BoxKind::Block,
    }
}

fn resolve_box_kind(style: &ComputedStyle) -> BoxKind {
    resolve_display_property(style).unwrap_or(BoxKind::Block)
}

fn resolve_display_property(style: &ComputedStyle) -> Option<BoxKind> {
    match style.display {
        Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => match s.as_ref() {
            "block" => Some(BoxKind::Block),
            "inline" => Some(BoxKind::Inline),
            "inline-block" => Some(BoxKind::InlineBlock),
            "flex" => Some(BoxKind::FlexContainer),
            "inline-flex" => Some(BoxKind::InlineFlex),
            "grid" => Some(BoxKind::GridContainer),
            "inline-grid" => Some(BoxKind::InlineGrid),
            "table" => Some(BoxKind::Table),
            "table-row" => Some(BoxKind::TableRow),
            "table-cell" => Some(BoxKind::TableCell),
            "table-row-group" | "table-header-group" | "table-footer-group" => Some(BoxKind::TableRowGroup),
            "table-caption" => Some(BoxKind::TableCaption),
            "list-item" => Some(BoxKind::ListItem),
            "none" => Some(BoxKind::Block),
            _ => None,
        },
        _ => None,
    }
}

fn is_display_none(style: &ComputedStyle) -> bool {
    match style.display {
        Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => s.as_ref() == "none",
        _ => false,
    }
}
