use lui_cascade::{cascade::InteractionState, media::MediaContext};
use lui_layout::engine::layout_tree;

use crate::helpers::*;

// ============================================================================
// 10. Positioned layout tests
// ============================================================================

#[test]
fn position_absolute_removed_from_flow() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="position:relative; width:300px">
            <div style="height:50px">in-flow</div>
            <div style="position:absolute; top:0; left:0; width:100px; height:100px">abs</div>
            <div style="height:50px">also in-flow</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Container height should only include in-flow children (50+50=100), not the absolute child
  assert!(
    (container.content.height - 100.0).abs() < 1.0,
    "container height should be ~100 (two 50px in-flow), got {}",
    container.content.height
  );
}

#[test]
fn position_absolute_with_top_left() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="position:relative; width:300px; height:200px">
            <div style="position:absolute; top:20px; left:30px; width:50px; height:50px">abs</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let abs = &container.children[0];
  let offset_x = abs.content.x - container.content.x;
  let offset_y = abs.content.y - container.content.y;
  assert!((offset_x - 30.0).abs() < 1.0, "left:30px offset, got {}", offset_x);
  assert!((offset_y - 20.0).abs() < 1.0, "top:20px offset, got {}", offset_y);
}

#[test]
fn position_absolute_with_right_bottom() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="position:relative; width:300px; height:200px">
            <div style="position:absolute; right:10px; bottom:10px; width:50px; height:50px">abs</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let abs = &container.children[0];
  let right_edge = abs.content.x + abs.content.width;
  let bottom_edge = abs.content.y + abs.content.height;
  let container_right = container.content.x + container.content.width;
  let container_bottom = container.content.y + container.content.height;
  assert!((container_right - right_edge - 10.0).abs() < 1.0, "right:10px");
  assert!((container_bottom - bottom_edge - 10.0).abs() < 1.0, "bottom:10px");
}

#[test]
fn position_relative_offsets_from_normal_position() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:300px">
            <div style="height:50px">before</div>
            <div style="position:relative; top:10px; left:20px; height:50px">shifted</div>
            <div style="height:50px">after</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let before = &container.children[0];
  let shifted = &container.children[1];
  let after = &container.children[2];
  // Relative positioning doesn't affect flow — "after" should start at y=100 (50+50)
  let after_offset = after.content.y - before.content.y;
  assert!(
    (after_offset - 100.0).abs() < 1.0,
    "after should be at normal flow position (100px from before), got {}",
    after_offset
  );
  // But "shifted" should be offset by +10 top, +20 left from its normal position
  let shifted_x_offset = shifted.content.x - before.content.x;
  assert!(
    (shifted_x_offset - 20.0).abs() < 1.0,
    "left:20px relative offset, got {}",
    shifted_x_offset
  );
}

#[test]
fn position_absolute_width_from_left_right() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="position:relative; width:400px; height:200px">
            <div style="position:absolute; left:50px; right:50px; height:30px">stretch</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let abs = &container.children[0];
  assert!(
    (abs.content.width - 300.0).abs() < 1.0,
    "width should be 400-50-50=300, got {}",
    abs.content.width
  );
}

#[test]
fn position_absolute_no_insets_uses_static_position() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="position:relative; width:300px">
            <div style="height:40px">before</div>
            <div style="position:absolute; width:50px; height:50px">abs-no-inset</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let before = &container.children[0];
  let abs = &container.children[1];
  // Without insets, absolute element should use its static position (after "before")
  assert!(
    abs.content.y >= before.content.y + before.content.height - 1.0,
    "should be at static position below 'before'"
  );
}

#[test]
fn position_static_stays_in_flow() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:300px">
            <div style="position:static; height:50px">normal</div>
            <div style="height:50px">also normal</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    (container.content.height - 100.0).abs() < 1.0,
    "both in flow = 100px height"
  );
}

// ============================================================================
// 15. Expanded positioned tests
// ============================================================================

#[test]
fn position_fixed_uses_viewport() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:300px; height:200px">
            <div style="position:fixed; top:10px; left:10px; width:50px; height:50px">fix</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  // Fixed element should be at viewport (10,10) regardless of parent
  let body = find_by_tag(&lt.root, "body").unwrap();
  let container = body.children.first().unwrap();
  let fixed = &container.children[0];
  assert!(
    (fixed.content.x - 10.0).abs() < 1.0,
    "fixed left:10 from viewport, got {}",
    fixed.content.x
  );
  assert!(
    (fixed.content.y - 10.0).abs() < 1.0,
    "fixed top:10 from viewport, got {}",
    fixed.content.y
  );
}

#[test]
fn position_relative_with_right() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:300px">
            <div style="position:relative; right:20px; height:50px">shifted</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let shifted = &container.children[0];
  // right:20px with no left → offset left by -20px
  let offset = shifted.content.x - container.content.x;
  assert!(
    (offset - (-20.0)).abs() < 1.0,
    "right:20 → shift left by 20, offset={}",
    offset
  );
}

#[test]
fn position_relative_with_bottom() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:300px">
            <div style="position:relative; bottom:15px; height:50px">shifted</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let shifted = &container.children[0];
  let offset = shifted.content.y - container.content.y;
  assert!(
    (offset - (-15.0)).abs() < 1.0,
    "bottom:15 → shift up by 15, offset={}",
    offset
  );
}

#[test]
fn position_absolute_percentage_insets() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="position:relative; width:400px; height:200px">
            <div style="position:absolute; left:25%; top:50%; width:50px; height:50px">pct</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let abs = &container.children[0];
  let offset_x = abs.content.x - container.content.x;
  let offset_y = abs.content.y - container.content.y;
  assert!((offset_x - 100.0).abs() < 1.0, "left:25% of 400=100, got {}", offset_x);
  assert!((offset_y - 100.0).abs() < 1.0, "top:50% of 200=100, got {}", offset_y);
}

// ============================================================================
// position: sticky
// ============================================================================

#[test]
fn position_sticky_stays_in_flow() {
  let (doc, ctx) = flex_lt(
    r#"<div style="width:300px">
        <div style="height:50px">before</div>
        <div style="position:sticky; top:10px; height:30px">sticky</div>
        <div style="height:50px">after</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let before = &container.children[0];
  let sticky = &container.children[1];
  let after = &container.children[2];
  // Sticky element is in-flow: after "before" (50px) and before "after"
  assert!(sticky.content.y > before.content.y, "sticky after before");
  assert!(after.content.y > sticky.content.y, "after after sticky");
  // Total height = 50 + 30 + 50 = 130
  assert!(
    (container.content.height - 130.0).abs() < 1.0,
    "sticky stays in flow, total height should be 130, got {}",
    container.content.height
  );
}

#[test]
fn position_sticky_has_insets() {
  let (doc, ctx) = flex_lt(
    r#"<div style="width:200px">
        <div style="position:sticky; top:10px; height:30px">sticky</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let sticky = &container.children[0];
  assert!(sticky.sticky.is_some(), "sticky element should have StickyInsets");
  let insets = sticky.sticky.unwrap();
  assert!(
    (insets.top.unwrap() - 10.0).abs() < 0.1,
    "top:10px, got {:?}",
    insets.top
  );
}

// ============================================================================
// z-index
// ============================================================================

#[test]
fn z_index_stored_on_layout_box() {
  let (doc, ctx) = flex_lt(
    r#"<div style="width:200px">
        <div style="position:relative; z-index:5; height:30px">front</div>
        <div style="position:relative; z-index:1; height:30px">back</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let front = &container.children[0];
  let back = &container.children[1];
  assert_eq!(front.z_index, Some(5), "z-index:5");
  assert_eq!(back.z_index, Some(1), "z-index:1");
}

#[test]
fn z_index_none_when_not_set() {
  let (doc, ctx) = flex_lt(
    r#"<div style="width:200px">
        <div style="height:30px">no z-index</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.children[0].z_index, None);
}

// ============================================================================
// Containing block from transforms
// ============================================================================

#[test]
fn transform_establishes_containing_block() {
  let (doc, ctx) = flex_lt(
    r#"<div style="width:400px; height:200px; transform:translateX(0)">
        <div style="position:absolute; top:10px; left:10px; width:50px; height:50px">abs</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // The abs child should be positioned relative to the transform container
  let abs = container.children.last().unwrap();
  let offset_x = abs.content.x - container.content.x;
  let offset_y = abs.content.y - container.content.y;
  assert!(
    (offset_x - 10.0).abs() < 1.0,
    "abs left:10px relative to transform container, got {}",
    offset_x
  );
  assert!(
    (offset_y - 10.0).abs() < 1.0,
    "abs top:10px relative to transform container, got {}",
    offset_y
  );
}
