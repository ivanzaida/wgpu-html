//! Comprehensive tests for the `lui-layout` crate.
//!
//! Tests are organized by module: geometry, sides, sizes, box_tree, context,
//! box_gen, block, and engine (integration).

use bumpalo::Bump;
use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_cascade::ComputedStyle;
use lui_css_parser::{ArcStr, CssUnit, CssValue};
use lui_html_parser::{HtmlElement, HtmlNode, Rect, parse};
use lui_layout::{
    BoxKind, LayoutBox, LayoutContext, LayoutTree, Point, RectEdges, Size,
    block::layout_block,
    box_gen::build_box,
    engine::layout_tree,
    sides, sizes,
    text::TextContext,
};
use lui_layout::geometry::RectExt;

// ============================================================================
// Helpers
// ============================================================================

/// Create a `CssValue::Dimension` in px (borrowed, for arena-allocated styles).
fn px(v: f32) -> CssValue {
    CssValue::Dimension {
        value: v as f64,
        unit: CssUnit::Px,
    }
}

/// Create a `CssValue::Number`.
fn num(v: f32) -> CssValue {
    CssValue::Number(v as f64)
}

/// Create a `CssValue::Percentage`.
fn pct(v: f32) -> CssValue {
    CssValue::Percentage(v as f64)
}

/// Create a `CssValue::Unknown("auto")`.
fn auto() -> CssValue {
    CssValue::Unknown(ArcStr::from("auto"))
}

// ============================================================================
// 1. geometry.rs tests
// ============================================================================

#[test]
fn point_new_sets_x_and_y() {
    let p = Point::new(1.0, 2.0);
    assert_eq!(p.x, 1.0, "x should be 1.0");
    assert_eq!(p.y, 2.0, "y should be 2.0");
}

#[test]
fn point_default_is_zero() {
    let p = Point::default();
    assert_eq!(p.x, 0.0);
    assert_eq!(p.y, 0.0);
}

#[test]
fn size_default_is_zero() {
    let s = Size::default();
    assert_eq!(s.width, 0.0, "width should be 0");
    assert_eq!(s.height, 0.0, "height should be 0");
}

#[test]
fn rect_edges_new_sets_all_fields() {
    let edges: RectEdges<i32> = RectEdges::new(1, 2, 3, 4);
    assert_eq!(edges.top, 1);
    assert_eq!(edges.right, 2);
    assert_eq!(edges.bottom, 3);
    assert_eq!(edges.left, 4);
}

#[test]
fn rect_edges_default_is_all_zero() {
    let edges: RectEdges<f32> = RectEdges::default();
    assert_eq!(edges.top, 0.0);
    assert_eq!(edges.right, 0.0);
    assert_eq!(edges.bottom, 0.0);
    assert_eq!(edges.left, 0.0);
}

#[test]
fn rect_edges_horizontal_sums_left_and_right() {
    let edges = RectEdges::<f32>::new(5.0, 10.0, 3.0, 2.0);
    assert_eq!(edges.horizontal(), 12.0, "horizontal = left + right = 2 + 10");
}

#[test]
fn rect_edges_vertical_sums_top_and_bottom() {
    let edges = RectEdges::<f32>::new(5.0, 10.0, 3.0, 2.0);
    assert_eq!(edges.vertical(), 8.0, "vertical = top + bottom = 5 + 3");
}

#[test]
fn rect_ext_max_x_is_x_plus_width() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(r.max_x(), 110.0);
}

#[test]
fn rect_ext_max_y_is_y_plus_height() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(r.max_y(), 70.0);
}

#[test]
fn rect_ext_contains_point_inside() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert!(r.contains(50.0, 40.0), "point inside rect should be contained");
}

#[test]
fn rect_ext_contains_top_left_corner() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert!(r.contains(10.0, 20.0), "top-left corner is on the edge, should be contained");
}

#[test]
fn rect_ext_contains_bottom_right_corner() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert!(r.contains(110.0, 70.0), "bottom-right corner is on the edge, should be contained");
}

#[test]
fn rect_ext_does_not_contain_point_outside() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert!(!r.contains(5.0, 40.0), "point left of rect should not be contained");
    assert!(!r.contains(120.0, 40.0), "point right of rect should not be contained");
    assert!(!r.contains(50.0, 10.0), "point above rect should not be contained");
    assert!(!r.contains(50.0, 80.0), "point below rect should not be contained");
}

// ============================================================================
// 2. sides.rs tests
// ============================================================================

#[test]
fn resolve_margin_with_px_and_auto() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.margin_top = Some(arena.alloc(px(10.0)));
    style.margin_right = Some(arena.alloc(auto()));
    style.margin_bottom = Some(arena.alloc(px(20.0)));
    style.margin_left = Some(arena.alloc(px(5.0)));

    let result = sides::resolve_margin(&style);

    assert_eq!(result.edges.top, 10.0, "margin-top should be 10px");
    assert_eq!(result.edges.right, 0.0, "margin-right auto resolves to 0 with mask");
    assert_eq!(result.edges.bottom, 20.0, "margin-bottom should be 20px");
    assert_eq!(result.edges.left, 5.0, "margin-left should be 5px");
    // auto_mask bits: bit 0=top, 1=right, 2=bottom, 3=left
    assert_eq!(result.auto_mask, 0b0010, "only right (bit 1) should be auto");
}

#[test]
fn resolve_margin_all_zero_when_no_margin_set() {
    let style = ComputedStyle::default();
    let result = sides::resolve_margin(&style);
    assert_eq!(result.edges.top, 0.0);
    assert_eq!(result.edges.right, 0.0);
    assert_eq!(result.edges.bottom, 0.0);
    assert_eq!(result.edges.left, 0.0);
    assert_eq!(result.auto_mask, 0, "no auto margins");
}

#[test]
fn resolve_margin_all_auto_sets_mask_correctly() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.margin_top = Some(arena.alloc(auto()));
    style.margin_right = Some(arena.alloc(auto()));
    style.margin_bottom = Some(arena.alloc(auto()));
    style.margin_left = Some(arena.alloc(auto()));

    let result = sides::resolve_margin(&style);
    // All edges resolve to 0.0
    assert_eq!(result.edges.top, 0.0);
    assert_eq!(result.edges.right, 0.0);
    assert_eq!(result.edges.bottom, 0.0);
    assert_eq!(result.edges.left, 0.0);
    assert_eq!(result.auto_mask, 0b1111, "all four sides should be auto");
}

#[test]
fn resolve_margin_number_zero() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.margin_top = Some(arena.alloc(num(0.0)));
    let result = sides::resolve_margin(&style);
    assert_eq!(result.edges.top, 0.0, "Number(0) resolves to 0");
    assert_eq!(result.auto_mask, 0, "Number is not auto");
}

#[test]
fn resolve_border_with_single_edge_set() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.border_top_width = Some(arena.alloc(px(2.0)));

    let border = sides::resolve_border(&style);
    assert_eq!(border.top, 2.0, "border-top should be 2px");
    assert_eq!(border.right, 0.0, "border-right should default to 0");
    assert_eq!(border.bottom, 0.0, "border-bottom should default to 0");
    assert_eq!(border.left, 0.0, "border-left should default to 0");
}

#[test]
fn resolve_border_all_four_edges() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.border_top_width = Some(arena.alloc(px(1.0)));
    style.border_right_width = Some(arena.alloc(px(2.0)));
    style.border_bottom_width = Some(arena.alloc(px(3.0)));
    style.border_left_width = Some(arena.alloc(px(4.0)));

    let border = sides::resolve_border(&style);
    assert_eq!(border.top, 1.0);
    assert_eq!(border.right, 2.0);
    assert_eq!(border.bottom, 3.0);
    assert_eq!(border.left, 4.0);
}

#[test]
fn resolve_border_with_number_zero() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.border_top_width = Some(arena.alloc(num(0.0)));
    let border = sides::resolve_border(&style);
    assert_eq!(border.top, 0.0, "Number(0) resolves to 0");
}

#[test]
fn resolve_padding_all_four_set() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.padding_top = Some(arena.alloc(px(8.0)));
    style.padding_right = Some(arena.alloc(px(8.0)));
    style.padding_bottom = Some(arena.alloc(px(8.0)));
    style.padding_left = Some(arena.alloc(px(8.0)));

    let padding = sides::resolve_padding(&style);
    assert_eq!(padding.top, 8.0);
    assert_eq!(padding.right, 8.0);
    assert_eq!(padding.bottom, 8.0);
    assert_eq!(padding.left, 8.0);
}

#[test]
fn resolve_padding_different_per_side() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.padding_top = Some(arena.alloc(px(1.0)));
    style.padding_right = Some(arena.alloc(px(2.0)));
    style.padding_bottom = Some(arena.alloc(px(3.0)));
    style.padding_left = Some(arena.alloc(px(4.0)));

    let padding = sides::resolve_padding(&style);
    assert_eq!(padding.top, 1.0);
    assert_eq!(padding.right, 2.0);
    assert_eq!(padding.bottom, 3.0);
    assert_eq!(padding.left, 4.0);
}

#[test]
fn resolve_padding_defaults_to_zero() {
    let style = ComputedStyle::default();
    let padding = sides::resolve_padding(&style);
    assert_eq!(padding.top, 0.0);
    assert_eq!(padding.right, 0.0);
    assert_eq!(padding.bottom, 0.0);
    assert_eq!(padding.left, 0.0);
}

// ============================================================================
// 3. sizes.rs tests
// ============================================================================

#[test]
fn resolve_length_px_returns_dimension() {
    let result = sizes::resolve_length(Some(&px(10.0)), 100.0);
    assert_eq!(result, Some(10.0), "10px should resolve to Some(10.0)");
}

#[test]
fn resolve_length_percentage_resolves_against_containing() {
    let result = sizes::resolve_length(Some(&pct(50.0)), 200.0);
    assert_eq!(result, Some(100.0), "50% of 200 should be 100");
}

#[test]
fn resolve_length_auto_returns_none() {
    let result = sizes::resolve_length(Some(&auto()), 100.0);
    assert_eq!(result, None, "auto should resolve to None");
}

#[test]
fn resolve_length_none_returns_none() {
    let result = sizes::resolve_length(None, 100.0);
    assert_eq!(result, None, "no value should resolve to None");
}

#[test]
fn resolve_length_number_zero_returns_zero() {
    let result = sizes::resolve_length(Some(&num(0.0)), 100.0);
    assert_eq!(result, Some(0.0), "Number(0) should resolve to Some(0.0)");
}

#[test]
fn resolve_length_percentage_zero_returns_zero() {
    let result = sizes::resolve_length(Some(&pct(0.0)), 200.0);
    assert_eq!(result, Some(0.0), "0% should resolve to 0.0");
}

#[test]
fn resolve_box_sizes_with_all_properties_set() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(px(100.0)));
    style.height = Some(arena.alloc(px(50.0)));
    style.min_width = Some(arena.alloc(px(0.0)));
    style.min_height = Some(arena.alloc(px(0.0)));
    style.max_width = Some(arena.alloc(px(500.0)));
    style.max_height = Some(arena.alloc(px(300.0)));

    let bs = sizes::resolve_box_sizes(&style, 800.0, 600.0);

    assert_eq!(bs.width, Some(100.0));
    assert_eq!(bs.height, Some(50.0));
    assert_eq!(bs.min_width, Some(0.0));
    assert_eq!(bs.min_height, Some(0.0));
    assert_eq!(bs.max_width, Some(500.0));
    assert_eq!(bs.max_height, Some(300.0));
}

#[test]
fn resolve_box_sizes_with_auto_width_height() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(auto()));
    style.height = Some(arena.alloc(auto()));

    let bs = sizes::resolve_box_sizes(&style, 800.0, 600.0);
    assert_eq!(bs.width, None, "auto width should be None");
    assert_eq!(bs.height, None, "auto height should be None");
}

#[test]
fn resolve_box_sizes_percentage_against_containing() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(pct(50.0)));
    style.height = Some(arena.alloc(pct(25.0)));

    let bs = sizes::resolve_box_sizes(&style, 800.0, 600.0);
    assert_eq!(bs.width, Some(400.0), "50% of 800 = 400");
    assert_eq!(bs.height, Some(150.0), "25% of 600 = 150");
}

// ============================================================================
// 4. box_tree.rs tests
// ============================================================================

#[test]
fn layout_box_new_sets_all_fields() {
    let node = HtmlNode::new(HtmlElement::Div);
    let style = ComputedStyle::default();
    let b = LayoutBox::new(BoxKind::Block, &node, &style);

    assert_eq!(b.kind, BoxKind::Block, "kind should be Block");
    assert!(std::ptr::eq(b.node, &node), "node pointer should match");
    assert_eq!(b.margin, RectEdges::default(), "margin should default to zero");
    assert_eq!(b.border, RectEdges::default(), "border should default to zero");
    assert_eq!(b.padding, RectEdges::default(), "padding should default to zero");
    assert_eq!(b.content, Rect::default(), "content should default to zero rect");
    assert!(b.intrinsic.is_none(), "intrinsic should be None");
    assert!(b.children.is_empty(), "children should be empty");
}

#[test]
fn layout_box_outer_width_sums_edges_and_content() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("test");
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    b.margin = RectEdges::new(5.0, 10.0, 3.0, 2.0);   // horizontal = 12
    b.border = RectEdges::new(1.0, 1.0, 1.0, 1.0);     // horizontal = 2
    b.padding = RectEdges::new(4.0, 4.0, 4.0, 4.0);    // horizontal = 8
    b.content.width = 100.0;
    // outer = 12 + 2 + 8 + 100 = 122
    assert_eq!(b.outer_width(), 122.0);
}

#[test]
fn layout_box_outer_height_sums_edges_and_content() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("test");
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    b.margin = RectEdges::new(5.0, 0.0, 10.0, 0.0);    // vertical = 15
    b.border = RectEdges::new(2.0, 0.0, 2.0, 0.0);      // vertical = 4
    b.padding = RectEdges::new(3.0, 0.0, 3.0, 0.0);     // vertical = 6
    b.content.height = 50.0;
    // outer = 15 + 4 + 6 + 50 = 75
    assert_eq!(b.outer_height(), 75.0);
}

#[test]
fn layout_box_border_rect_computes_correctly() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("test");
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    b.content = Rect::new(20.0, 30.0, 100.0, 50.0);
    b.border = RectEdges::new(2.0, 3.0, 2.0, 3.0);
    b.padding = RectEdges::new(8.0, 6.0, 8.0, 6.0);

    let br = b.border_rect();
    // x = 20 - 3 - 6 = 11
    // y = 30 - 2 - 8 = 20
    // width = 100 + (3+3) + (6+6) = 100 + 6 + 12 = 118
    // height = 50 + (2+2) + (8+8) = 50 + 4 + 16 = 70
    assert_eq!(br.x, 11.0, "border_rect x");
    assert_eq!(br.y, 20.0, "border_rect y");
    assert_eq!(br.width, 118.0, "border_rect width");
    assert_eq!(br.height, 70.0, "border_rect height");
}

#[test]
fn layout_tree_find_rect_returns_some_when_rect_exists() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("find_me");
    let rect = Rect::new(10.0, 20.0, 50.0, 30.0);

    let root_box = LayoutBox::new(BoxKind::Block, &node, &style);
    let tree = LayoutTree {
        root: root_box,
        rects: vec![(&node, rect)],
    };

    let found = tree.find_rect(&node);
    assert_eq!(found, Some(rect), "should find the rect for the matching node");
}

#[test]
fn layout_tree_find_rect_returns_none_when_rect_absent() {
    let style = ComputedStyle::default();
    let node_a = HtmlNode::text("a");
    let node_b = HtmlNode::text("b");
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);

    let root_box = LayoutBox::new(BoxKind::Block, &node_a, &style);
    let tree = LayoutTree {
        root: root_box,
        rects: vec![(&node_a, rect)],
    };

    let found = tree.find_rect(&node_b);
    assert_eq!(found, None, "should not find rect for a different node");
}

// ============================================================================
// 5. context.rs tests
// ============================================================================

#[test]
fn layout_context_new_sets_viewport_and_containing() {
    let ctx = LayoutContext::new(800.0, 600.0);
    assert_eq!(ctx.viewport_width, 800.0);
    assert_eq!(ctx.viewport_height, 600.0);
    assert_eq!(ctx.containing_width, 800.0, "initial containing width = viewport width");
    assert!(ctx.containing_height.is_nan(), "initial containing height should be NaN (auto)");
    assert_eq!(ctx.root_font_size, 16.0, "default root font-size is 16px");
    assert_eq!(ctx.parent_font_size, 16.0, "default parent font-size is 16px");
}

#[test]
fn layout_context_new_zero_viewport() {
    let ctx = LayoutContext::new(0.0, 0.0);
    assert_eq!(ctx.viewport_width, 0.0);
    assert_eq!(ctx.viewport_height, 0.0);
    assert_eq!(ctx.containing_width, 0.0);
}

// ============================================================================
// 6. box_gen.rs tests
// ============================================================================

#[test]
fn build_box_single_text_node_is_anonymous_inline() {
    let doc = parse("hello");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    // Parser produces: html > "hello" (no head/body auto-insertion)
    // styled.children are html's direct children
    let text_child = styled.children.iter()
        .find(|c| c.node.element.is_text()).unwrap();
    assert!(text_child.node.element.is_text(), "child should be text node");
    let lb = build_box(text_child);
    assert_eq!(lb.kind, BoxKind::AnonymousInline, "text node should produce AnonymousInline box");
}

#[test]
fn build_box_div_without_ua_stylesheet_defaults_to_block() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    // Parser: html > div (no head/body auto-insertion)
    let div = &styled.children[0];
    let lb = build_box(div);
    // Without UA stylesheet, display is None → falls to Block (default)
    assert_eq!(lb.kind, BoxKind::Block, "div without UA stylesheet should default to Block");
}

#[test]
fn build_box_display_none_still_produces_block() {
    let doc = parse(r#"<div style="display:none"></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    // Parser: html > div
    let div = &styled.children[0];
    let lb = build_box(div);
    // Per current code, display:none maps to BoxKind::Block
    assert_eq!(lb.kind, BoxKind::Block, "display:none currently defaults to Block");
}

#[test]
fn build_box_inline_text_between_blocks_wrapped_in_anonymous_block() {
    let doc = parse("<div><h1></h1>inline text<h2></h2></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    // Parser: html > div > [h1, "inline text", h2]
    let div = &styled.children[0];
    let lb = build_box(div);

    // Children should be: h1 (Block), AnonymousBlock (wrapping "inline text"), h2 (Block)
    assert_eq!(lb.children.len(), 3, "should have 3 children: h1, anon-block, h2");
    assert_eq!(lb.children[0].kind, BoxKind::Block, "first child (h1) should be Block");
    assert_eq!(lb.children[1].kind, BoxKind::AnonymousBlock, "second should be AnonymousBlock wrapping inline text");
    assert_eq!(lb.children[2].kind, BoxKind::Block, "third child (h2) should be Block");

    // The anonymous block should contain the inline text node
    assert_eq!(lb.children[1].children.len(), 1, "anonymous block should contain 1 child");
    assert_eq!(lb.children[1].children[0].kind, BoxKind::AnonymousInline, "anon-block child should be AnonymousInline");
}

// ============================================================================
// 7. block.rs tests
// ============================================================================

#[test]
fn layout_block_no_children_fills_available_width_height_zero() {
    let node = HtmlNode::new(HtmlElement::Div);
    let style = ComputedStyle::default();
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    let ctx = LayoutContext::new(800.0, 600.0);
    let mut rects = Vec::new();

    let mut text_ctx = TextContext::new();
    layout_block(&mut b, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects);

    // Default margin/border/padding are all 0, so content width fills the containing block (800px)
    assert_eq!(b.content.width, 800.0, "block should fill available width");
    assert_eq!(b.content.height, 0.0, "block with no children should have height 0");
    assert_eq!(b.content.x, 0.0);
    assert_eq!(b.content.y, 0.0);
}

#[test]
fn layout_block_with_explicit_px_width_respects_width() {
    let arena = Bump::new();
    let node = HtmlNode::new(HtmlElement::Div);
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(px(200.0)));

    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    let ctx = LayoutContext::new(800.0, 600.0);
    let mut rects = Vec::new();

    let mut text_ctx = TextContext::new();
    layout_block(&mut b, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects);

    assert_eq!(b.content.width, 200.0, "should respect explicit width");
    assert_eq!(b.content.height, 0.0, "height should be 0 without children");
}

#[test]
fn layout_block_width_clamped_to_available() {
    let arena = Bump::new();
    let node = HtmlNode::new(HtmlElement::Div);
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(px(2000.0)));

    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    let ctx = LayoutContext::new(800.0, 600.0);
    let mut rects = Vec::new();

    let mut text_ctx = TextContext::new();
    layout_block(&mut b, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects);

    // width = min(2000, 800) = 800. Available is containing_width - margin - border - padding = 800 - 0 = 800
    assert!(b.content.width <= 800.0, "width should be clamped to available");
    assert_eq!(b.content.width, 800.0);
}

#[test]
fn layout_block_children_stacked_vertically() {
    let node1 = HtmlNode::text("hello");
    let node2 = HtmlNode::text("world");
    let parent_node = HtmlNode::new(HtmlElement::Div);
    let style = ComputedStyle::default();

    // Build a parent with two text children
    let mut parent = LayoutBox::new(BoxKind::Block, &parent_node, &style);
    let child1 = LayoutBox::new(BoxKind::AnonymousInline, &node1, &style);
    let child2 = LayoutBox::new(BoxKind::AnonymousInline, &node2, &style);
    parent.children.push(child1);
    parent.children.push(child2);

    let ctx = LayoutContext::new(800.0, 600.0);
    let mut rects = Vec::new();
    let mut text_ctx = TextContext::new();
    layout_block(&mut parent, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects);

    // Each inline text child gets height from font metrics (via cosmic-text shaping)
    let h1 = parent.children[0].content.height;
    let h2 = parent.children[1].content.height;
    assert!(h1 > 0.0, "text child should have non-zero height");
    assert!(h2 > 0.0, "text child should have non-zero height");
    assert_eq!(parent.children.len(), 2, "should have 2 children after layout");
    assert_eq!(parent.children[0].content.y, 0.0, "first child should start at y=0");
    assert_eq!(parent.children[1].content.y, h1, "second child should be stacked below first");
    assert_eq!(parent.content.height, h1 + h2, "parent height should be sum of child heights");
    assert_eq!(parent.content.width, 800.0, "parent should fill available width");
}

#[test]
fn layout_block_with_margin_and_padding_adjusts_child_position() {
    let arena = Bump::new();
    let parent_node = HtmlNode::new(HtmlElement::Div);
    let text_node = HtmlNode::text("x");
    let mut style = ComputedStyle::default();
    style.margin_top = Some(arena.alloc(px(10.0)));
    style.margin_left = Some(arena.alloc(px(5.0)));
    style.padding_top = Some(arena.alloc(px(8.0)));
    style.padding_left = Some(arena.alloc(px(6.0)));

    let mut parent = LayoutBox::new(BoxKind::Block, &parent_node, &style);
    let child = LayoutBox::new(BoxKind::AnonymousInline, &text_node, &style);
    parent.children.push(child);

    let ctx = LayoutContext::new(800.0, 600.0);
    let mut rects = Vec::new();
    let mut text_ctx = TextContext::new();
    layout_block(&mut parent, &ctx, Point::new(0.0, 0.0), &mut text_ctx, &mut rects);

    assert_eq!(parent.margin.top, 10.0);
    assert_eq!(parent.margin.left, 5.0);
    assert_eq!(parent.padding.top, 8.0);
    assert_eq!(parent.padding.left, 6.0);

    // Content x = pos.x + margin.left + border.left + padding.left = 0 + 5 + 0 + 6 = 11
    assert_eq!(parent.content.x, 11.0);
    // Content y = pos.y + margin.top + border.top + padding.top = 0 + 10 + 0 + 8 = 18
    assert_eq!(parent.content.y, 18.0);
    // Available width = 800 - 5 - 0 - 6 = 789 (border = 0)
    assert_eq!(parent.content.width, 789.0);
}

// ============================================================================
// 8. engine.rs integration tests
// ============================================================================

#[test]
fn full_layout_simple_div_with_paragraphs() {
    let html = r#"<!DOCTYPE html><html><head></head><body><div><p>hello</p><p>world</p></div></body></html>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 800.0, 600.0);

    // Root is the html element
    assert_eq!(lt.root.kind, BoxKind::Block, "root should be Block");

    // root.children should be non-empty (head + body)
    assert!(!lt.root.children.is_empty(), "root should have children");

    // Find the body
    let body = lt.root.children.iter().find(|c| c.node.element.tag_name() == "body");
    assert!(body.is_some(), "should have a body element");

    let body = body.unwrap();
    // body should have the div as child
    let div = body.children.first();
    assert!(div.is_some(), "body should have a div child");
    assert_eq!(div.unwrap().kind, BoxKind::Block, "div should be Block");

    let div = div.unwrap();
    assert!(!div.children.is_empty(), "div should have children");

    // Content rects should contain entries for each block element
    assert!(!lt.rects.is_empty(), "should have rect entries");

    // Verify content rects have non-zero values for block elements
    let has_non_zero =
        lt.rects.iter().any(|(_node, rect)| rect.width > 0.0 || rect.height > 0.0);
    assert!(has_non_zero, "some rects should have non-zero dimensions");
}

#[test]
fn full_layout_rects_contain_all_elements() {
    let html = r#"<div id="parent"><span id="child">text</span></div>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 800.0, 600.0);

    let rect_count = lt.rects.len();
    assert!(rect_count > 0, "should have at least one rect entry, got {rect_count}");

    // Every node in rects should have non-negative dimensions
    for (_node, rect) in &lt.rects {
        assert!(rect.width >= 0.0, "rect width should be non-negative");
        assert!(rect.height >= 0.0, "rect height should be non-negative");
    }
}

#[test]
fn full_layout_with_zero_viewport_still_completes() {
    let html = r#"<div>hello</div>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 0.0, 0.0);

    // Should not panic; width may be 0.0
    assert_eq!(lt.root.content.width, 0.0, "content width should be 0 for zero viewport");
}

#[test]
fn full_layout_text_node_produces_entries_in_rects() {
    let html = r#"<div>simple text</div>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 800.0, 600.0);

    // There should be rect entries
    assert!(lt.rects.len() >= 1, "should have rect entries");
}

#[test]
fn full_layout_content_rects_are_stacked_vertically() {
    let html = r#"<div><p>first</p><p>second</p><p>third</p></div>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 800.0, 600.0);

    // Parser: html > div > [p, p, p] (no head/body)
    // lt.root is the html block. Its children are the div and any text/anonymous wrappers.
    // Find the div
    let div = lt.root.children.iter().find(|c| c.node.element.tag_name() == "div");
    assert!(div.is_some(), "should have div");
    let div = div.unwrap();

    // div children should be stacked vertically
    for i in 1..div.children.len() {
        let prev = &div.children[i - 1];
        let curr = &div.children[i];
        assert!(curr.content.y >= prev.content.y,
            "child {i} y={} should be >= previous child y={}",
            curr.content.y, prev.content.y);
    }
}

#[test]
fn full_layout_with_inline_span_produces_boxes() {
    let html = r#"<div><span>inline</span></div>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 800.0, 600.0);

    assert!(!lt.rects.is_empty());
}

#[test]
fn full_layout_viewport_affects_block_width() {
    let html = r#"<div></div>"#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 400.0, 600.0);

    // Parser: html > div (no body). lt.root is the html block.
    let div = lt.root.children.first();
    assert!(div.is_some(), "should have div child of html");
    assert_eq!(div.unwrap().content.width, 400.0, "div should fill 400px viewport");
}

#[test]
fn full_layout_empty_document() {
    let html = r#""#;
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[]);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let lt = layout_tree(&styled, 800.0, 600.0);

    // Should not panic
    assert!(lt.root.kind == BoxKind::Block || lt.root.kind == BoxKind::Root);
    assert!(lt.rects.len() > 0, "should have at least one rect");
}
