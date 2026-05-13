use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::{BoxKind, engine::layout_tree};
use crate::helpers::*;

// ============================================================================
// 9. Flexbox tests
// ============================================================================

#[test]
fn flex_row_distributes_children_horizontally() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px"><div style="width:100px">A</div><div style="width:100px">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let ch = &flex.children;
    assert_eq!(ch.len(), 2);
    assert!(ch[1].content.x > ch[0].content.x);
}

#[test]
fn flex_column_stacks_children_vertically() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-direction:column; width:300px"><div style="height:50px">A</div><div style="height:50px">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[1].content.y > flex.children[0].content.y);
}

#[test]
fn flex_grow_distributes_free_space() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px"><div style="flex-grow:1">A</div><div style="flex-grow:2">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let ratio = flex.children[1].content.width / flex.children[0].content.width;
    assert!((ratio - 2.0).abs() < 0.1, "ratio should be ~2.0, got {}", ratio);
}

#[test]
fn flex_justify_content_center() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; justify-content:center"><div style="width:100px">A</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let ch = &flex.children[0];
    let offset = (ch.content.x - ch.padding.left - ch.border.left - ch.margin.left) - flex.content.x;
    assert!((offset - 100.0).abs() < 1.0, "should be centered (offset={})", offset);
}

#[test]
fn flex_justify_content_space_between() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; justify-content:space-between"><div style="width:50px">A</div><div style="width:50px">B</div><div style="width:50px">C</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let first_x = flex.children[0].content.x;
    let last = &flex.children[2];
    let last_end = last.content.x + last.content.width;
    assert!((first_x - flex.content.x).abs() < 1.0, "first at start");
    assert!(((flex.content.x + flex.content.width) - last_end).abs() < 1.0, "last at end");
}

#[test]
fn flex_gap_adds_spacing() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; column-gap:20px"><div style="width:100px">A</div><div style="width:100px">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let a_end = flex.children[0].content.x + flex.children[0].content.width;
    let b_start = flex.children[1].content.x;
    let gap = b_start - a_end;
    assert!((gap - 20.0).abs() < 1.0, "gap should be ~20px, got {}", gap);
}

#[test]
fn flex_wrap_wraps_items() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-wrap:wrap; width:200px"><div style="width:120px; height:30px">A</div><div style="width:120px; height:30px">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[1].content.y > flex.children[0].content.y, "should wrap");
}

#[test]
fn flex_order_reorders_visually() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px"><div style="width:50px; order:2">A</div><div style="width:50px; order:1">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[0].content.x > flex.children[1].content.x, "order:2 should be after order:1");
}

#[test]
fn flex_align_items_center() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; height:200px; align-items:center"><div style="width:50px; height:50px">A</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child_cy = flex.children[0].content.y + flex.children[0].content.height / 2.0;
    let flex_cy = flex.content.y + flex.content.height / 2.0;
    assert!((child_cy - flex_cy).abs() < 1.0, "should be centered");
}

#[test]
fn flex_shrink_reduces_overflowing_items() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:200px"><div style="width:150px; flex-shrink:1">A</div><div style="width:150px; flex-shrink:1">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[0].content.width < 150.0, "should shrink");
    let total = flex.children[0].content.width + flex.children[1].content.width;
    assert!((total - 200.0).abs() < 1.0, "total should be ~200px, got {}", total);
}

#[test]
fn flex_no_children_does_not_panic() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px"></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(flex.kind, BoxKind::FlexContainer);
}

#[test]
fn flex_container_detected_as_flex_kind() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex"><div>child</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(flex.kind, BoxKind::FlexContainer);
}

#[test]
fn flex_row_reverse() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-direction:row-reverse; width:300px"><div style="width:50px">A</div><div style="width:50px">B</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[0].content.x > flex.children[1].content.x, "row-reverse: first child should be rightmost");
}

// ============================================================================
// 14. Expanded flex tests
// ============================================================================

#[test]
fn flex_column_reverse() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-direction:column-reverse; width:300px; height:200px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[0].content.y > flex.children[1].content.y,
        "column-reverse: first child should be below second");
}

#[test]
fn flex_justify_content_flex_end() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; justify-content:flex-end">
        <div style="width:50px">A</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child_end = flex.children[0].content.x + flex.children[0].content.width;
    let flex_end = flex.content.x + flex.content.width;
    assert!((flex_end - child_end).abs() < 1.0, "flex-end: child at right edge");
}

#[test]
fn flex_justify_content_space_around() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; justify-content:space-around">
        <div style="width:50px">A</div><div style="width:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Free space = 200, 2 items → each gets 100px, half on each side = 50px
    let a_start = flex.children[0].content.x - flex.content.x;
    assert!(a_start > 40.0 && a_start < 60.0, "space-around: first item should have ~50px before it, got {}", a_start);
}

#[test]
fn flex_justify_content_space_evenly() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; justify-content:space-evenly">
        <div style="width:50px">A</div><div style="width:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Free space = 200, 3 slots → ~66.7px each
    let a_start = flex.children[0].content.x - flex.content.x;
    assert!(a_start > 60.0 && a_start < 73.0, "space-evenly: ~66.7px before first, got {}", a_start);
}

#[test]
fn flex_align_items_flex_start() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; height:200px; align-items:flex-start">
        <div style="width:50px; height:50px">A</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let offset = flex.children[0].content.y - flex.content.y;
    assert!(offset.abs() < 1.0, "flex-start: child at top, offset={}", offset);
}

#[test]
fn flex_align_items_flex_end() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; height:200px; align-items:flex-end">
        <div style="width:50px; height:50px">A</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child_bottom = flex.children[0].content.y + flex.children[0].content.height;
    let flex_bottom = flex.content.y + flex.content.height;
    assert!((flex_bottom - child_bottom).abs() < 1.0, "flex-end: child at bottom");
}

#[test]
fn flex_align_items_stretch() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; height:200px; align-items:stretch">
        <div style="width:50px">A</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((flex.children[0].content.height - 200.0).abs() < 1.0,
        "stretch: child should fill cross axis, got {}", flex.children[0].content.height);
}

#[test]
fn flex_align_self_overrides_align_items() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; height:200px; align-items:flex-start">
        <div style="width:50px; height:50px; align-self:flex-end">A</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child_bottom = flex.children[0].content.y + flex.children[0].content.height;
    let flex_bottom = flex.content.y + flex.content.height;
    assert!((flex_bottom - child_bottom).abs() < 1.0,
        "align-self:flex-end should override align-items:flex-start");
}

#[test]
fn flex_basis_sets_initial_size() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px">
        <div style="flex-basis:100px">A</div><div style="flex-basis:200px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((flex.children[0].content.width - 100.0).abs() < 1.0,
        "flex-basis:100px, got {}", flex.children[0].content.width);
    assert!((flex.children[1].content.width - 200.0).abs() < 1.0,
        "flex-basis:200px, got {}", flex.children[1].content.width);
}

#[test]
fn flex_wrap_reverse() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-wrap:wrap-reverse; width:200px; height:200px">
        <div style="width:120px; height:50px">A</div><div style="width:120px; height:50px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // wrap-reverse: second line should be ABOVE the first line
    assert!(flex.children[0].content.y > flex.children[1].content.y,
        "wrap-reverse: first item should be below second (y0={}, y1={})",
        flex.children[0].content.y, flex.children[1].content.y);
}

// ============================================================================
// 19. Flex edge case tests (audit fixes)
// ============================================================================

#[test]
fn flex_gap_shorthand_applies_to_both_axes() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; gap:20px">
        <div style="width:100px">A</div><div style="width:100px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let a_end = flex.children[0].content.x + flex.children[0].content.width;
    let b_start = flex.children[1].content.x;
    let gap = b_start - a_end;
    assert!((gap - 20.0).abs() < 1.0, "gap shorthand should apply, got {}", gap);
}

#[test]
fn flex_basis_zero_shrinks_to_content() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px">
        <div style="flex-basis:0; flex-grow:1">A</div>
        <div style="flex-basis:0; flex-grow:1">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Both start at 0 and grow equally → 150px each
    assert!((flex.children[0].content.width - 150.0).abs() < 1.0,
        "flex-basis:0 + grow:1 → 150, got {}", flex.children[0].content.width);
}

#[test]
fn flex_shrink_weighted_by_base_size() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:200px">
        <div style="width:200px; flex-shrink:1">big</div>
        <div style="width:100px; flex-shrink:1">small</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Overflow = 100px. shrink factors weighted: big=200*1=200, small=100*1=100
    // big shrinks by 100*(200/300)=66.7 → 133.3, small shrinks by 100*(100/300)=33.3 → 66.7
    let big = flex.children[0].content.width;
    let small = flex.children[1].content.width;
    assert!(big > small, "bigger item should still be wider after proportional shrink (big={}, small={})", big, small);
    assert!((big + small - 200.0).abs() < 1.0, "total should be 200, got {}", big + small);
}

#[test]
fn flex_align_self_center_with_parent_stretch() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px; height:200px; align-items:stretch">
        <div style="width:50px; height:50px; align-self:center">centered</div>
        <div style="width:50px">stretched</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let centered = &flex.children[0];
    let stretched = &flex.children[1];
    // Centered child should be in the middle
    let center_y = centered.content.y + centered.content.height / 2.0;
    let flex_center = flex.content.y + flex.content.height / 2.0;
    assert!((center_y - flex_center).abs() < 1.0, "align-self:center should center");
    // Stretched child should fill the cross axis
    assert!((stretched.content.height - 200.0).abs() < 1.0,
        "stretched child should be 200px, got {}", stretched.content.height);
}

#[test]
fn flex_auto_margins_absorb_free_space() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:300px">
        <div style="width:100px; margin-left:auto">pushed right</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let child = &flex.children[0];
    let child_right = child.content.x + child.content.width;
    let flex_right = flex.content.x + flex.content.width;
    assert!((flex_right - child_right).abs() < 1.0,
        "margin-left:auto should push child to right edge");
}

#[test]
fn flex_min_width_on_item_prevents_shrinking() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:200px">
        <div style="width:150px; flex-shrink:1; min-width:120px">clamped</div>
        <div style="width:150px; flex-shrink:1">unclamped</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[0].content.width >= 119.0,
        "min-width:120 should prevent shrinking below 120, got {}", flex.children[0].content.width);
}

#[test]
fn flex_max_width_on_item_prevents_growing() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; width:400px">
        <div style="flex-grow:1; max-width:100px">capped</div>
        <div style="flex-grow:1">uncapped</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(flex.children[0].content.width <= 101.0,
        "max-width:100 should cap growth, got {}", flex.children[0].content.width);
}

#[test]
fn flex_nested_flex_in_flex() {
    let (doc, ctx) = flex_lt(r#"
        <div style="display:flex; width:400px">
            <div style="display:flex; flex-grow:1">
                <div style="width:50px">inner-A</div>
                <div style="width:50px">inner-B</div>
            </div>
            <div style="width:100px">outer-B</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(flex.children.len(), 2);
    // Inner flex should grow to fill remaining space (300px)
    let inner = &flex.children[0];
    assert!(inner.content.width > 200.0, "inner flex should grow, got {}", inner.content.width);
}

// ============================================================================
// 20. align-content multi-line tests
// ============================================================================

#[test]
fn flex_align_content_center_multiline() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:center">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Lines should be centered in the 300px cross axis
    let first_y = flex.children[0].content.y - flex.content.y;
    assert!(first_y > 50.0,
        "align-content:center should push lines down from top, first_y={}", first_y);
}

#[test]
fn flex_align_content_flex_end_multiline() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:flex-end">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let last = &flex.children[1];
    let last_bottom = last.content.y + last.content.height;
    let flex_bottom = flex.content.y + flex.content.height;
    assert!((flex_bottom - last_bottom).abs() < 1.0,
        "align-content:flex-end should push lines to bottom");
}

#[test]
fn flex_align_content_space_between_multiline() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:space-between">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // First line at top, second line at bottom
    let first_y = flex.children[0].content.y - flex.content.y;
    let last = &flex.children[1];
    let last_bottom = last.content.y + last.content.height;
    let flex_bottom = flex.content.y + flex.content.height;
    assert!(first_y.abs() < 1.0, "first line at top, got offset {}", first_y);
    assert!((flex_bottom - last_bottom).abs() < 1.0, "last line at bottom");
}

#[test]
fn flex_align_content_stretch_multiline() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:stretch">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Two lines, each gets 150px of cross space (300/2). Lines should be spaced 150px apart.
    let a_y = flex.children[0].content.y;
    let b_y = flex.children[1].content.y;
    let line_gap = b_y - a_y;
    assert!(line_gap > 140.0 && line_gap < 160.0,
        "stretch should distribute cross space equally, line_gap={}", line_gap);
}
