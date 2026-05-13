//! Layout engine: entry point and recursive dispatcher.

use bumpalo::Bump;
use lui_cascade::StyledNode;
use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_gen::build_box;
use crate::box_tree::{BoxKind, LayoutBox, LayoutTree};
use crate::context::LayoutContext;
use crate::flow;
use crate::geometry::Point;
use crate::incremental::{CacheView, LayoutCache};
use crate::text::TextContext;

/// Stateful layout engine. Owns the font context and caches previous
/// frame results so incremental re-layout can skip clean subtrees.
///
/// Mirrors `CascadeContext`'s OOP pattern: create once, call `layout()`
/// or `layout_dirty()` each frame.
pub struct LayoutEngine {
    text_ctx: TextContext,
    cache: LayoutCache,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            text_ctx: TextContext::new(),
            cache: LayoutCache::empty(),
        }
    }

    /// Full layout — recompute everything. Stores results for the next
    /// `layout_dirty()` call.
    pub fn layout<'a>(
        &mut self,
        styled: &'a StyledNode<'a>,
        viewport_width: f32,
        viewport_height: f32,
    ) -> LayoutTree<'a> {
        let tree = layout_tree_with(styled, viewport_width, viewport_height, &mut self.text_ctx);
        self.cache = LayoutCache::snapshot(&tree);
        tree
    }

    /// Incremental layout — only recompute subtrees on dirty paths.
    /// Falls back to full layout on viewport resize or empty dirty set.
    pub fn layout_dirty<'a>(
        &mut self,
        styled: &'a StyledNode<'a>,
        dirty_paths: &[Vec<usize>],
        viewport_width: f32,
        viewport_height: f32,
    ) -> LayoutTree<'a> {
        let tree = crate::incremental::layout_incremental_with(
            styled, &self.cache, dirty_paths,
            viewport_width, viewport_height, &mut self.text_ctx,
        );
        self.cache = LayoutCache::snapshot(&tree);
        tree
    }

    pub fn text_ctx(&mut self) -> &mut TextContext {
        &mut self.text_ctx
    }
}

/// Compute layout for the entire styled tree (convenience free function).
pub fn layout_tree<'a>(styled: &'a StyledNode<'a>, viewport_width: f32, viewport_height: f32) -> LayoutTree<'a> {
    let mut text_ctx = TextContext::new();
    layout_tree_with(styled, viewport_width, viewport_height, &mut text_ctx)
}

/// Compute layout reusing an existing `TextContext` (avoids re-scanning system fonts).
pub fn layout_tree_with<'a>(styled: &'a StyledNode<'a>, viewport_width: f32, viewport_height: f32, text_ctx: &mut TextContext) -> LayoutTree<'a> {
    let arena_ptr = Box::into_raw(Box::new(Bump::new()));
    // SAFETY: arena_ptr is valid, and the &'a Bump reference lives as long as the LayoutTree
    // (which owns the arena and drops root before freeing it).
    let bump: &'a Bump = unsafe { &*arena_ptr };

    let ctx = LayoutContext::new(viewport_width, viewport_height);
    let mut rects = Vec::new();
    let view = CacheView::Full;
    let root = build_box(styled, bump);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), text_ctx, &mut rects, &view, bump);
    LayoutTree::new(root, rects, arena_ptr)
}

pub fn layout_node<'a>(
    mut b: LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
    cache: &CacheView,
    bump: &'a Bump,
) -> LayoutBox<'a> {
    if cache.try_clone(&mut b, ctx, pos, rects, bump) {
        return b;
    }
    match b.kind {
        BoxKind::FlexContainer | BoxKind::InlineFlex => {
            crate::flex::layout_flex(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::GridContainer | BoxKind::InlineGrid => {
            crate::grid::layout_grid(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::Table => {
            crate::table::layout_table(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::Block | BoxKind::Root | BoxKind::ListItem => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::InlineBlock => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::Inline | BoxKind::AnonymousInline => {
            flow::layout_inline(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::AnonymousBlock => {
            crate::block::layout_anonymous_block(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::TableRow | BoxKind::TableCell | BoxKind::TableRowGroup | BoxKind::TableCaption => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects, cache, bump);
        }
        BoxKind::TableColumnGroup | BoxKind::TableColumn => {
            // Column groups/columns don't produce visible boxes;
            // their width hints are read by the table layout algorithm.
        }
        _ => {}
    }

    crate::positioned::apply_z_index(&mut b);
    apply_text_overflow_ellipsis(&mut b);
    apply_text_decoration(&mut b);
    if b.kind == BoxKind::ListItem {
        fn css_str_li(v: Option<&lui_core::CssValue>) -> &str {
            match v { Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(), _ => "" }
        }
        let marker = match css_str_li(b.style.list_style_type) {
            "none" => None,
            "disc" | "" => Some("\u{2022} ".to_owned()),
            "circle" => Some("\u{25CB} ".to_owned()),
            "square" => Some("\u{25A0} ".to_owned()),
            "decimal" => Some("1. ".to_owned()),
            other => Some(format!("{} ", other)),
        };
        b.list_marker = marker;
    }
    {
        fn css_str2(v: Option<&lui_core::CssValue>) -> &str {
            match v {
                Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
                _ => "",
            }
        }
        let wm = css_str2(b.style.writing_mode);
        if !wm.is_empty() && wm != "horizontal-tb" {
            b.writing_mode = Some(wm.to_owned());
        }
    }
    rects.push((b.node, b.content));
    b
}

fn apply_text_decoration(b: &mut LayoutBox) {
    fn css_str(v: Option<&lui_core::CssValue>) -> &str {
        match v {
            Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
            _ => "",
        }
    }
    let line = css_str(b.style.text_decoration_line);
    if !line.is_empty() && line != "none" {
        b.text_decoration = Some(line.to_owned());
    }
}

fn apply_text_overflow_ellipsis(b: &mut LayoutBox) {
    fn css_str(v: Option<&lui_core::CssValue>) -> &str {
        match v {
            Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
            _ => "",
        }
    }
    if css_str(b.style.text_overflow) != "ellipsis" { return; }
    let overflow = css_str(b.style.overflow_x);
    if !matches!(overflow, "hidden" | "clip" | "scroll") { return; }
    for child in &mut b.children {
        if child.content.width > b.content.width {
            child.text_overflow_ellipsis = true;
        }
    }
}
