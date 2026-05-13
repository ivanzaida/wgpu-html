//! Layout engine: entry point and recursive dispatcher.

use lui_cascade::StyledNode;
use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::box_gen::build_box;
use crate::box_tree::{BoxKind, LayoutBox, LayoutTree};
use crate::context::LayoutContext;
use crate::flow;
use crate::geometry::Point;
use crate::incremental::LayoutCache;
use crate::text::TextContext;

/// Compute layout for the entire styled tree.
pub fn layout_tree<'a>(styled: &'a StyledNode<'a>, viewport_width: f32, viewport_height: f32) -> LayoutTree<'a> {
    let ctx = LayoutContext::new(viewport_width, viewport_height);
    let mut text_ctx = TextContext::new();
    let mut rects = Vec::new();
    let cache = LayoutCache::empty();
    let root = build_box(styled);
    let root = layout_node(root, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects, &cache);
    LayoutTree { root, rects }
}

pub fn layout_node<'a>(
    mut b: LayoutBox<'a>,
    ctx: &LayoutContext,
    pos: Point,
    text_ctx: &mut TextContext,
    rects: &mut Vec<(&'a HtmlNode, Rect)>,
    cache: &LayoutCache,
) -> LayoutBox<'a> {
    if crate::incremental::try_clone_from_cache(&mut b, cache, ctx, pos, rects) {
        return b;
    }
    match b.kind {
        BoxKind::FlexContainer | BoxKind::InlineFlex => {
            crate::flex::layout_flex(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::GridContainer | BoxKind::InlineGrid => {
            crate::grid::layout_grid(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::Table => {
            crate::table::layout_table(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::Block | BoxKind::Root | BoxKind::ListItem => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::InlineBlock => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::Inline | BoxKind::AnonymousInline => {
            flow::layout_inline(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::AnonymousBlock => {
            crate::block::layout_anonymous_block(&mut b, ctx, pos, text_ctx, rects, cache);
        }
        BoxKind::TableRow | BoxKind::TableCell | BoxKind::TableRowGroup | BoxKind::TableCaption => {
            crate::block::layout_block(&mut b, ctx, pos, text_ctx, rects, cache);
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
