//! Box generation: converts `StyledNode` tree into initial `LayoutBox` tree.
//! Handles anonymous box creation for inline-between-block content.

use bumpalo::Bump;
use lui_cascade::{ComputedStyle, StyledNode};
use lui_core::CssValue;

use crate::box_tree::{BoxKind, LayoutBox};

/// Build a `LayoutBox` tree from a `StyledNode` tree. Text nodes become
/// `AnonymousInline` boxes; elements are classified by `display`.
pub fn build_box<'a>(styled: &'a StyledNode<'a>, bump: &'a Bump) -> LayoutBox<'a> {
  if styled.node.element.is_text() {
    return LayoutBox::new(BoxKind::AnonymousInline, styled.node, &styled.style, bump);
  }

  let kind = resolve_box_kind_with_node(&styled.style, styled.node);
  let mut child_boxes = bumpalo::collections::Vec::new_in(bump);
  let mut pending_inlines: Vec<&StyledNode> = Vec::new();

  for child in &styled.children {
    collect_child(child, &mut pending_inlines, &mut child_boxes, styled, bump);
  }
  flush_inlines(&mut pending_inlines, &mut child_boxes, styled, bump);

  let mut b = LayoutBox::new(kind, styled.node, &styled.style, bump);
  b.children = child_boxes;
  b.scrollbar_pseudo = styled.scrollbar_pseudo.as_deref();
  fixup_anonymous_table_wrappers(&mut b, styled, bump);
  b
}

/// Wrap pending inline children in an anonymous block box.
fn collect_child<'a>(
  child: &'a StyledNode<'a>,
  pending_inlines: &mut Vec<&'a StyledNode<'a>>,
  child_boxes: &mut bumpalo::collections::Vec<'a, LayoutBox<'a>>,
  parent: &'a StyledNode<'a>,
  bump: &'a Bump,
) {
  if is_display_none(&child.style) {
    return;
  }
  if is_display_contents(&child.style) {
    for grandchild in &child.children {
      collect_child(grandchild, pending_inlines, child_boxes, parent, bump);
    }
    return;
  }
  if child.node.element.is_text() {
    pending_inlines.push(child);
    return;
  }
  let child_kind = resolve_box_kind_with_node(&child.style, child.node);
  match child_kind {
    BoxKind::Block
    | BoxKind::FlexContainer
    | BoxKind::GridContainer
    | BoxKind::Table
    | BoxKind::TableRowGroup
    | BoxKind::TableCaption
    | BoxKind::ListItem => {
      flush_inlines(pending_inlines, child_boxes, parent, bump);
      child_boxes.push(build_box(child, bump));
    }
    BoxKind::Inline | BoxKind::InlineBlock | BoxKind::InlineFlex | BoxKind::InlineGrid => {
      pending_inlines.push(child);
    }
    _ => child_boxes.push(build_box(child, bump)),
  }
}

fn is_display_contents(style: &ComputedStyle) -> bool {
  match style.display {
    Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => s.as_ref() == "contents",
    _ => false,
  }
}

fn flush_inlines<'a>(
  pending: &mut Vec<&'a StyledNode<'a>>,
  out: &mut bumpalo::collections::Vec<'a, LayoutBox<'a>>,
  parent: &'a StyledNode<'a>,
  bump: &'a Bump,
) {
  if pending.is_empty() {
    return;
  }
  let mut anon = LayoutBox::new(BoxKind::AnonymousBlock, parent.node, &parent.style, bump);
  for s in pending.drain(..) {
    anon.children.push(build_box(s, bump));
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
    "colgroup" => BoxKind::TableColumnGroup,
    "col" => BoxKind::TableColumn,
    _ => BoxKind::Block,
  }
}

fn _resolve_box_kind(style: &ComputedStyle) -> BoxKind {
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
      "table-column-group" => Some(BoxKind::TableColumnGroup),
      "table-column" => Some(BoxKind::TableColumn),
      "list-item" => Some(BoxKind::ListItem),
      "none" => Some(BoxKind::Block),
      _ => None,
    },
    _ => None,
  }
}

/// CSS 2.1 §17.2.1 — wrap orphaned table-internal elements in anonymous boxes.
///
/// - A TableCell not inside a TableRow gets wrapped in an anonymous TableRow.
/// - A TableRow not inside a Table/TableRowGroup gets wrapped in an anonymous Table.
/// - A TableRowGroup not inside a Table gets wrapped in an anonymous Table.
fn fixup_anonymous_table_wrappers<'a>(b: &mut LayoutBox<'a>, styled: &'a StyledNode<'a>, bump: &'a Bump) {
  let parent_kind = b.kind;

  // Cells outside a row → wrap consecutive cells in an anonymous row
  if !matches!(parent_kind, BoxKind::TableRow) {
    let mut new_children: bumpalo::collections::Vec<'a, LayoutBox<'a>> = bumpalo::collections::Vec::new_in(bump);
    let mut pending_cells: Vec<LayoutBox<'a>> = Vec::new();

    for child in b.children.drain(..) {
      if child.kind == BoxKind::TableCell {
        pending_cells.push(child);
      } else {
        if !pending_cells.is_empty() {
          let mut anon_row = LayoutBox::new(BoxKind::TableRow, styled.node, &styled.style, bump);
          anon_row.children = bumpalo::collections::Vec::from_iter_in(pending_cells.drain(..), bump);
          // If parent is also not a table, wrap the row in a table too
          if !matches!(parent_kind, BoxKind::Table | BoxKind::TableRowGroup) {
            let mut anon_table = LayoutBox::new(BoxKind::Table, styled.node, &styled.style, bump);
            anon_table.children.push(anon_row);
            new_children.push(anon_table);
          } else {
            new_children.push(anon_row);
          }
        }
        new_children.push(child);
      }
    }
    if !pending_cells.is_empty() {
      let mut anon_row = LayoutBox::new(BoxKind::TableRow, styled.node, &styled.style, bump);
      anon_row.children = bumpalo::collections::Vec::from_iter_in(pending_cells.into_iter(), bump);
      if !matches!(parent_kind, BoxKind::Table | BoxKind::TableRowGroup) {
        let mut anon_table = LayoutBox::new(BoxKind::Table, styled.node, &styled.style, bump);
        anon_table.children.push(anon_row);
        new_children.push(anon_table);
      } else {
        new_children.push(anon_row);
      }
    }
    b.children = new_children;
  }

  // Rows outside a table → wrap consecutive rows in an anonymous table
  if !matches!(parent_kind, BoxKind::Table | BoxKind::TableRowGroup) {
    let mut new_children: bumpalo::collections::Vec<'a, LayoutBox<'a>> = bumpalo::collections::Vec::new_in(bump);
    let mut pending_rows: Vec<LayoutBox<'a>> = Vec::new();

    for child in b.children.drain(..) {
      if matches!(child.kind, BoxKind::TableRow | BoxKind::TableRowGroup) {
        pending_rows.push(child);
      } else {
        if !pending_rows.is_empty() {
          let mut anon_table = LayoutBox::new(BoxKind::Table, styled.node, &styled.style, bump);
          anon_table.children = bumpalo::collections::Vec::from_iter_in(pending_rows.drain(..), bump);
          new_children.push(anon_table);
        }
        new_children.push(child);
      }
    }
    if !pending_rows.is_empty() {
      let mut anon_table = LayoutBox::new(BoxKind::Table, styled.node, &styled.style, bump);
      anon_table.children = bumpalo::collections::Vec::from_iter_in(pending_rows.into_iter(), bump);
      new_children.push(anon_table);
    }
    b.children = new_children;
  }
}

/// Like `build_box` but skips recursion into clean subtrees.
/// Clean nodes get a LayoutBox with correct node/style but empty children —
/// the incremental cache restores them from snapshots.
pub fn build_box_incremental<'a>(
  styled: &'a StyledNode<'a>,
  dirty: &rustc_hash::FxHashSet<*const lui_parse::HtmlNode>,
  bump: &'a Bump,
) -> LayoutBox<'a> {
  if styled.node.element.is_text() {
    return LayoutBox::new(BoxKind::AnonymousInline, styled.node, &styled.style, bump);
  }

  let kind = resolve_box_kind_with_node(&styled.style, styled.node);
  let ptr = styled.node as *const lui_parse::HtmlNode;

  if !dirty.contains(&ptr) {
    return LayoutBox::new(kind, styled.node, &styled.style, bump);
  }

  let mut child_boxes = bumpalo::collections::Vec::new_in(bump);
  let mut pending_inlines: Vec<&StyledNode> = Vec::new();

  for child in &styled.children {
    collect_child_incremental(child, &mut pending_inlines, &mut child_boxes, styled, dirty, bump);
  }
  flush_inlines(&mut pending_inlines, &mut child_boxes, styled, bump);

  let mut b = LayoutBox::new(kind, styled.node, &styled.style, bump);
  b.children = child_boxes;
  b.scrollbar_pseudo = styled.scrollbar_pseudo.as_deref();
  fixup_anonymous_table_wrappers(&mut b, styled, bump);
  b
}

fn collect_child_incremental<'a>(
  child: &'a StyledNode<'a>,
  pending_inlines: &mut Vec<&'a StyledNode<'a>>,
  child_boxes: &mut bumpalo::collections::Vec<'a, LayoutBox<'a>>,
  parent: &'a StyledNode<'a>,
  dirty: &rustc_hash::FxHashSet<*const lui_parse::HtmlNode>,
  bump: &'a Bump,
) {
  if is_display_none(&child.style) {
    return;
  }
  if is_display_contents(&child.style) {
    for grandchild in &child.children {
      collect_child_incremental(grandchild, pending_inlines, child_boxes, parent, dirty, bump);
    }
    return;
  }
  if child.node.element.is_text() {
    pending_inlines.push(child);
    return;
  }
  let child_kind = resolve_box_kind_with_node(&child.style, child.node);
  match child_kind {
    BoxKind::Block
    | BoxKind::FlexContainer
    | BoxKind::GridContainer
    | BoxKind::Table
    | BoxKind::TableRowGroup
    | BoxKind::TableCaption
    | BoxKind::ListItem => {
      flush_inlines(pending_inlines, child_boxes, parent, bump);
      child_boxes.push(build_box_incremental(child, dirty, bump));
    }
    BoxKind::Inline | BoxKind::InlineBlock | BoxKind::InlineFlex | BoxKind::InlineGrid => {
      pending_inlines.push(child);
    }
    _ => child_boxes.push(build_box_incremental(child, dirty, bump)),
  }
}

fn is_display_none(style: &ComputedStyle) -> bool {
  match style.display {
    Some(&CssValue::Unknown(ref s)) | Some(&CssValue::String(ref s)) => s.as_ref() == "none",
    _ => false,
  }
}
