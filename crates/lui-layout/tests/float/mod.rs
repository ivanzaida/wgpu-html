use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_layout::{BoxKind, LayoutBox, engine::layout_tree};
use crate::helpers::*;

// ── float: left ───────────────────────────────────────────────────────

#[test]
fn float_left_moves_to_left_edge() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:80px">float</div>
            <div style="height:50px">in-flow</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let float_child = &container.children[0];
    // Float should be at the left edge of the container
    let offset = float_child.content.x - float_child.padding.left - float_child.border.left
        - float_child.margin.left - container.content.x;
    assert!(offset.abs() < 1.0, "float:left should be at left edge, offset={}", offset);
}

#[test]
fn float_left_does_not_affect_container_height() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:200px">tall float</div>
            <div style="height:50px">short in-flow</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Without BFC, container height should be based on in-flow content only
    assert!(container.content.height < 100.0,
        "container should not extend for float, got {}", container.content.height);
}

// ── float: right ──────────────────────────────────────────────────────

#[test]
fn float_right_moves_to_right_edge() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:right; width:100px; height:80px">float</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let float_child = &container.children[0];
    let float_right = float_child.content.x + float_child.content.width
        + float_child.padding.right + float_child.border.right + float_child.margin.right;
    let container_right = container.content.x + container.content.width;
    assert!((container_right - float_right).abs() < 1.0,
        "float:right should be at right edge");
}

// ── clear ─────────────────────────────────────────────────────────────

#[test]
fn clear_left_moves_below_left_float() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:80px">float</div>
            <div style="clear:left; height:50px">cleared</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let float_child = &container.children[0];
    let cleared = &container.children[1];
    let float_bottom = float_child.content.y + float_child.content.height
        + float_child.padding.bottom + float_child.border.bottom + float_child.margin.bottom;
    assert!(cleared.content.y >= float_bottom - 1.0,
        "clear:left should be below float (cleared_y={}, float_bottom={})",
        cleared.content.y, float_bottom);
}

#[test]
fn clear_both_moves_below_all_floats() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:60px">left</div>
            <div style="float:right; width:100px; height:100px">right</div>
            <div style="clear:both; height:50px">cleared</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let right_float = &container.children[1];
    let cleared = &container.children[2];
    let right_bottom = right_float.content.y + right_float.content.height
        + right_float.padding.bottom + right_float.border.bottom + right_float.margin.bottom;
    assert!(cleared.content.y >= right_bottom - 1.0,
        "clear:both should be below tallest float");
}

// ── BFC containment ───────────────────────────────────────────────────

#[test]
fn overflow_hidden_contains_floats() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px; overflow-x:hidden">
            <div style="float:left; width:100px; height:200px">tall float</div>
            <div style="height:50px">short</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // overflow:hidden creates BFC → container extends to contain float
    assert!(container.content.height >= 199.0,
        "BFC should contain float, height={}", container.content.height);
}

// ── Multiple floats ───────────────────────────────────────────────────

#[test]
fn multiple_left_floats_stack_horizontally() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:50px">A</div>
            <div style="float:left; width:100px; height:50px">B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let a = &container.children[0];
    let b = &container.children[1];
    // Second float should be to the right of the first
    let a_right = a.content.x + a.content.width + a.padding.right + a.border.right + a.margin.right;
    let b_left = b.content.x - b.padding.left - b.border.left - b.margin.left;
    assert!(b_left >= a_right - 1.0,
        "second left float should be right of first (a_right={}, b_left={})", a_right, b_left);
}

#[test]
fn float_left_and_right_on_same_line() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:50px">left</div>
            <div style="float:right; width:100px; height:50px">right</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let left = &container.children[0];
    let right = &container.children[1];
    // Left float at left edge, right float at right edge
    assert!(right.content.x > left.content.x + left.content.width,
        "right float should be to the right of left float");
}

// ── In-flow content alongside floats ──────────────────────────────────

#[test]
fn in_flow_block_narrows_beside_float() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:150px; height:100px">float</div>
            <div style="height:50px">narrowed</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let in_flow = &container.children[1];
    // In-flow content should start after the float
    let float_right = container.children[0].content.x + container.children[0].content.width
        + container.children[0].padding.right + container.children[0].border.right
        + container.children[0].margin.right;
    assert!(in_flow.content.x >= float_right - 1.0,
        "in-flow should start after float (in_flow_x={}, float_right={})",
        in_flow.content.x, float_right);
}

#[test]
fn float_does_not_panic_with_no_other_content() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <div style="float:left; width:100px; height:50px">alone</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(container.children.len() >= 1);
}
