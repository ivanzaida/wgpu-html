use bumpalo::Bump;
use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_cascade::ComputedStyle;
use lui_parse::{HtmlElement, HtmlNode};
use lui_layout::{
    BoxKind, LayoutBox, LayoutContext, Point,
    block::layout_block,
    engine::layout_tree,
    text::TextContext,
};
use crate::helpers::*;

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
// 11. Min/max clamping tests
// ============================================================================

#[test]
fn min_width_prevents_shrinking() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px; min-width:200px; height:50px">clamped</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 200.0).abs() < 1.0, "min-width:200 should win over width:100, got {}", el.content.width);
}

#[test]
fn max_width_prevents_growing() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:500px; max-width:300px; height:50px">clamped</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 300.0).abs() < 1.0, "max-width:300 should cap width:500, got {}", el.content.width);
}

#[test]
fn min_height_prevents_collapsing() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px; min-height:80px"></div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.height - 80.0).abs() < 1.0, "min-height:80 should prevent collapse, got {}", el.content.height);
}

#[test]
fn max_height_caps_content() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px; height:300px; max-height:100px">tall</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.height - 100.0).abs() < 1.0, "max-height:100 should cap height:300, got {}", el.content.height);
}

#[test]
fn min_max_width_no_effect_when_within_range() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; min-width:100px; max-width:300px; height:50px">ok</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 200.0).abs() < 1.0, "within range, width should stay 200, got {}", el.content.width);
}

// ============================================================================
// 13. Margin collapsing tests
// ============================================================================

#[test]
fn adjacent_margins_collapse_to_larger() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:50px; margin-bottom:30px">A</div>
            <div style="height:50px; margin-top:20px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let a = &container.children[0];
    let b = &container.children[1];
    let a_bottom = a.content.y + a.content.height + a.padding.bottom + a.border.bottom;
    let b_top = b.content.y - b.border.top - b.padding.top;
    let visual_gap = b_top - a_bottom;
    // Should collapse 30+20 → 30 (the larger)
    assert!((visual_gap - 30.0).abs() < 1.0, "margins should collapse to 30, visual_gap={}", visual_gap);
}

#[test]
fn equal_margins_collapse_to_single() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:50px; margin-bottom:20px">A</div>
            <div style="height:50px; margin-top:20px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Total: 50 + 20(collapsed) + 50 = 120, not 50+20+20+50=140
    assert!((container.content.height - 120.0).abs() < 1.0,
        "collapsed height should be 120, got {}", container.content.height);
}

#[test]
fn zero_margin_does_not_collapse() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:50px; margin-bottom:0">A</div>
            <div style="height:50px; margin-top:20px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // No collapse: 50 + 0 + 20 + 50 = 120
    assert!((container.content.height - 120.0).abs() < 1.0,
        "no collapse when one margin is 0, got {}", container.content.height);
}

#[test]
fn negative_margin_collapses_with_positive() {
    // CSS2 §8.3.1: one positive + one negative → sum (max + min)
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:50px; margin-bottom:30px">A</div>
            <div style="height:50px; margin-top:-10px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let a = &container.children[0];
    let b = &container.children[1];
    let a_bottom = a.content.y + a.content.height + a.padding.bottom + a.border.bottom;
    let b_top = b.content.y - b.border.top - b.padding.top;
    let visual_gap = b_top - a_bottom;
    // 30 + (-10) = 20
    assert!((visual_gap - 20.0).abs() < 1.0,
        "positive + negative should collapse to sum (20), got {}", visual_gap);
}

#[test]
fn both_negative_margins_collapse_to_most_negative() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:50px; margin-bottom:-10px">A</div>
            <div style="height:50px; margin-top:-20px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let a = &container.children[0];
    let b = &container.children[1];
    let a_bottom = a.content.y + a.content.height + a.padding.bottom + a.border.bottom;
    let b_top = b.content.y - b.border.top - b.padding.top;
    let visual_gap = b_top - a_bottom;
    // Both negative: collapse to min(-10, -20) = -20
    assert!((visual_gap - (-20.0)).abs() < 1.0,
        "both negative should collapse to most negative (-20), got {}", visual_gap);
}

#[test]
fn three_siblings_collapse_pairwise() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:40px; margin-bottom:20px">A</div>
            <div style="height:40px; margin-top:30px; margin-bottom:10px">B</div>
            <div style="height:40px; margin-top:25px">C</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // A-B gap: collapse(20, 30) = 30
    // B-C gap: collapse(10, 25) = 25
    // Total: 40 + 30 + 40 + 25 + 40 = 175
    assert!((container.content.height - 175.0).abs() < 1.0,
        "three siblings pairwise collapse: expected 175, got {}", container.content.height);
}

#[test]
fn margin_collapsing_does_not_apply_in_flex() {
    // Flex formatting context prevents margin collapsing between flex items
    let (doc, ctx) = flex_lt(r#"
        <div style="display:flex; flex-direction:column; width:300px">
            <div style="height:50px; margin-bottom:30px">A</div>
            <div style="height:50px; margin-top:20px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let flex = body.children.first().unwrap();
    let a = &flex.children[0];
    let b = &flex.children[1];
    let a_bottom = a.content.y + a.content.height + a.padding.bottom + a.border.bottom;
    let b_top = b.content.y - b.border.top - b.padding.top;
    let visual_gap = b_top - a_bottom;
    // Flex: no collapsing → gap = 30 + 20 = 50
    assert!((visual_gap - 50.0).abs() < 1.0,
        "flex should NOT collapse margins, gap should be 50, got {}", visual_gap);
}

#[test]
fn negative_margin_pulls_sibling_up() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="height:50px">A</div>
            <div style="height:50px; margin-top:-15px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Total: 50 + (-15) + 50 = 85
    assert!((container.content.height - 85.0).abs() < 1.0,
        "negative margin should reduce height to 85, got {}", container.content.height);
}

// ============================================================================
// 16. Block layout gaps
// ============================================================================

#[test]
fn block_percentage_width() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="width:50%; height:50px">half</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child = &container.children[0];
    assert!((child.content.width - 200.0).abs() < 1.0, "50% of 400=200, got {}", child.content.width);
}

#[test]
fn block_with_padding_and_border_reduces_content() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px; padding:20px; border-width:5px; height:50px">padded</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // width:300 with 20px padding + 5px border on each side = 300 - 50 = 250 available
    // but width:300px is explicit, so content.width = 300
    assert_eq!(el.padding.left, 20.0);
    assert_eq!(el.border.left, 5.0);
}

// ── margin: auto centering ────────────────────────────────────────────

#[test]
fn block_margin_auto_centers_horizontally() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; margin-left:auto; margin-right:auto; height:50px">centered</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let el = &body.children[0];
    let center_x = el.content.x + el.content.width / 2.0;
    let body_center = body.content.x + body.content.width / 2.0;
    assert!((center_x - body_center).abs() < 1.0,
        "margin:auto should center (el_center={}, body_center={})", center_x, body_center);
}

#[test]
fn block_margin_auto_left_pushes_right() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; margin-left:auto; height:50px">right-aligned</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let el = &body.children[0];
    let el_right = el.content.x + el.content.width;
    let body_right = body.content.x + body.content.width;
    assert!((body_right - el_right).abs() < 1.0,
        "margin-left:auto should push to right edge");
}

// ── box-sizing: border-box ────────────────────────────────────────────

#[test]
fn box_sizing_border_box_includes_padding_in_width() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; padding:20px; box-sizing:border-box; height:100px">border-box</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // border-box: width:200px includes padding → content = 200 - 40 = 160
    assert!((el.content.width - 160.0).abs() < 1.0,
        "border-box: content should be 200-40=160, got {}", el.content.width);
    assert_eq!(el.padding.left, 20.0);
}

#[test]
fn box_sizing_border_box_includes_border_in_width() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; padding:10px; border-width:5px; box-sizing:border-box; height:100px">bb</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // border-box: width:200 includes padding(20) + border(10) → content = 200 - 30 = 170
    assert!((el.content.width - 170.0).abs() < 1.0,
        "border-box: content should be 200-30=170, got {}", el.content.width);
}

#[test]
fn box_sizing_border_box_height() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; height:100px; padding:10px; box-sizing:border-box">bb-h</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // height:100 border-box → content = 100 - 20 = 80
    assert!((el.content.height - 80.0).abs() < 1.0,
        "border-box height: content should be 100-20=80, got {}", el.content.height);
}

#[test]
fn box_sizing_content_box_is_default() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; padding:20px; height:50px">content-box</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Default content-box: width:200 is content width, padding is additional
    assert!((el.content.width - 200.0).abs() < 1.0,
        "content-box: content should be 200, got {}", el.content.width);
}

// ── parent-child margin collapsing ────────────────────────────────────

#[test]
fn parent_first_child_margin_collapse() {
    let (doc, ctx) = flex_lt(r#"
        <div style="margin-top:30px; width:300px">
            <div style="margin-top:20px; height:50px">child</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let parent = &body.children[0];
    // Parent margin-top:30, child margin-top:20 → collapsed to max(30,20)=30
    // Parent's effective margin-top should be 30
    assert!((parent.margin.top - 30.0).abs() < 1.0,
        "collapsed parent margin should be 30, got {}", parent.margin.top);
}

#[test]
fn parent_child_collapse_prevented_by_padding() {
    let (doc, ctx) = flex_lt(r#"
        <div style="margin-top:10px; padding-top:1px; width:300px">
            <div style="margin-top:20px; height:50px">child</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let parent = &body.children[0];
    // padding-top:1px prevents collapse → parent keeps margin-top:10
    assert!((parent.margin.top - 10.0).abs() < 1.0,
        "padding should prevent collapse, parent margin should stay 10, got {}", parent.margin.top);
}

#[test]
fn parent_child_collapse_prevented_by_border() {
    let (doc, ctx) = flex_lt(r#"
        <div style="margin-top:10px; border-top-width:1px; width:300px">
            <div style="margin-top:20px; height:50px">child</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let parent = &body.children[0];
    assert!((parent.margin.top - 10.0).abs() < 1.0,
        "border should prevent collapse, got {}", parent.margin.top);
}

#[test]
fn overflow_hidden_prevents_margin_collapse() {
    let (doc, ctx) = flex_lt(r#"
        <div style="margin-top:10px; overflow-x:hidden; width:300px">
            <div style="margin-top:20px; height:50px">child</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let body = find_by_tag(&lt.root, "body").unwrap();
    let parent = &body.children[0];
    // overflow:hidden creates BFC → prevents parent-child collapse
    assert!((parent.margin.top - 10.0).abs() < 1.0,
        "overflow:hidden should prevent collapse, got {}", parent.margin.top);
}
