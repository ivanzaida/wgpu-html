use lui_cascade::{cascade::InteractionState, media::MediaContext};
use lui_layout::{BoxKind, engine::layout_tree};

use crate::helpers::*;

// ============================================================================
// 9. Flexbox tests
// ============================================================================

#[test]
fn flex_row_distributes_children_horizontally() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px"><div style="width:100px">A</div><div style="width:100px">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let ch = &flex.children;
  assert_eq!(ch.len(), 2);
  assert!(ch[1].content.x > ch[0].content.x);
}

#[test]
fn flex_column_stacks_children_vertically() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-direction:column; width:300px"><div style="height:50px">A</div><div style="height:50px">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(flex.children[1].content.y > flex.children[0].content.y);
}

#[test]
fn flex_grow_distributes_free_space() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px"><div style="flex-basis:0; flex-grow:1">A</div><div style="flex-basis:0; flex-grow:2">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let ratio = flex.children[1].content.width / flex.children[0].content.width;
  assert!((ratio - 2.0).abs() < 0.1, "ratio should be ~2.0, got {}", ratio);
}

#[test]
fn flex_justify_content_center() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; justify-content:center"><div style="width:100px">A</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let ch = &flex.children[0];
  let offset = (ch.content.x - ch.padding.left - ch.border.left - ch.margin.left) - flex.content.x;
  assert!((offset - 100.0).abs() < 1.0, "should be centered (offset={})", offset);
}

#[test]
fn flex_justify_content_space_between() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; justify-content:space-between"><div style="width:50px">A</div><div style="width:50px">B</div><div style="width:50px">C</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let first_x = flex.children[0].content.x;
  let last = &flex.children[2];
  let last_end = last.content.x + last.content.width;
  assert!((first_x - flex.content.x).abs() < 1.0, "first at start");
  assert!(
    ((flex.content.x + flex.content.width) - last_end).abs() < 1.0,
    "last at end"
  );
}

#[test]
fn flex_gap_adds_spacing() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; column-gap:20px"><div style="width:100px">A</div><div style="width:100px">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a_end = flex.children[0].content.x + flex.children[0].content.width;
  let b_start = flex.children[1].content.x;
  let gap = b_start - a_end;
  assert!((gap - 20.0).abs() < 1.0, "gap should be ~20px, got {}", gap);
}

#[test]
fn flex_gap_ignores_whitespace_text_nodes() {
  // Whitespace-only text nodes between flex items must not generate
  // flex items (CSS Flexbox §4: "not rendered").  With formatted HTML
  // the newlines/spaces between <div> tags create text nodes that
  // should be stripped, leaving only the real items for gap accounting.
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:500px; column-gap:20px">
        <div style="width:100px">A</div>
        <div style="width:100px">B</div>
        <div style="width:100px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();

  // Find the three real items (skip anonymous whitespace wrappers)
  let real: Vec<_> = flex.children.iter().filter(|c| c.content.width > 1.0).collect();
  assert_eq!(real.len(), 3, "should have 3 visible flex items, got {}", real.len());

  let gap_ab = real[1].content.x - (real[0].content.x + real[0].content.width);
  let gap_bc = real[2].content.x - (real[1].content.x + real[1].content.width);
  assert!((gap_ab - 20.0).abs() < 1.0, "gap A→B should be ~20px, got {}", gap_ab);
  assert!((gap_bc - 20.0).abs() < 1.0, "gap B→C should be ~20px, got {}", gap_bc);
}

#[test]
fn flex_gap_in_centered_column_flex() {
  // Row flex (no explicit width) inside a column flex with
  // align-items:center.  The row should shrink to max-content width
  // (3×120 + 2×16 = 392) and whitespace text nodes must not inflate
  // that width with extra gaps.
  let (doc, ctx) = flex_lt(
    r#"
        <div style="display:flex; flex-direction:column; align-items:center">
            <div style="display:flex; gap:16px">
                <div style="width:120px; height:40px">A</div>
                <div style="width:120px; height:40px">B</div>
                <div style="width:120px; height:40px">C</div>
            </div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let outer = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let row_flex = &outer
    .children
    .iter()
    .find(|c| c.content.width > 10.0)
    .expect("should find the row flex container");

  // Max-content width should be 3×120 + 2×16 = 392, not inflated
  assert!(
    (row_flex.content.width - 392.0).abs() < 2.0,
    "row flex should shrink to 392px, got {}",
    row_flex.content.width
  );
}

#[test]
fn flex_gap_demo_exact_layout() {
  // Exact reproduction of the demo's flex row structure.
  // The row flex has gap:16px and three 120×120 children, nested inside
  // a column flex with align-items:center and padding:40px.
  let (doc, ctx) = flex_lt(
    r#"
        <div style="display:flex; flex-direction:column; align-items:center; padding:40px">
            <div style="display:flex; gap:16px; margin-bottom:32px">
                <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
                    <span style="font-size:14px">Block</span>
                </div>
                <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
                    <span style="font-size:14px">Flex</span>
                </div>
                <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
                    <span style="font-size:14px">Grid</span>
                </div>
            </div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let outer = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // The row flex is among the outer flex's children
  let row_flex = outer
    .children
    .iter()
    .find(|c| c.content.width > 100.0)
    .expect("should find the row flex container");

  let items: Vec<_> = row_flex.children.iter().filter(|c| c.content.width > 50.0).collect();
  assert_eq!(items.len(), 3, "should have 3 visible items");

  let gap_01 = items[1].content.x - (items[0].content.x + items[0].outer_width());
  let gap_12 = items[2].content.x - (items[1].content.x + items[1].outer_width());
  assert!(
    (gap_01 - 16.0).abs() < 2.0,
    "gap 0→1 should be ~16px, got {}. row_w={}, items: x0={} x1={} x2={}",
    gap_01,
    row_flex.content.width,
    items[0].content.x,
    items[1].content.x,
    items[2].content.x
  );
  assert!((gap_12 - 16.0).abs() < 2.0, "gap 1→2 should be ~16px, got {}", gap_12);
}

#[test]
fn flex_gap_demo_with_ua_stylesheet() {
  // Same test but using the WHATWG UA stylesheet (what the demo uses)
  // instead of the test-only reset stylesheet.
  let html = r#"<html>
    <body style="margin:0; font-family:sans-serif; background:#1a1a2e">
    <div style="display:flex; flex-direction:column; align-items:center; padding:40px">
        <div style="display:flex; gap:16px; margin-bottom:32px">
            <div style="width:120px; height:120px; background:#e94560; border-radius:12px; display:flex; align-items:center; justify-content:center">
                <span style="color:white; font-size:14px">Block</span>
            </div>
            <div style="width:120px; height:120px; background:#0f3460; border-radius:12px; display:flex; align-items:center; justify-content:center">
                <span style="color:white; font-size:14px">Flex</span>
            </div>
            <div style="width:120px; height:120px; background:#533483; border-radius:12px; display:flex; align-items:center; justify-content:center">
                <span style="color:white; font-size:14px">Grid</span>
            </div>
        </div>
    </div>
    </body></html>"#;

  let doc = lui_parse::parse(html);
  let mut ctx = lui_cascade::cascade::CascadeContext::new();
  let ua = lui_parse::parse_stylesheet(include_str!("../../../../crates/lui/ua/ua_whatwg.css")).unwrap();
  ctx.set_stylesheets(&[ua]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);

  let body = find_by_tag(&lt.root, "body").unwrap();
  let outer = body
    .children
    .iter()
    .find(|c| c.content.width > 100.0)
    .expect("should find column flex container");
  let row_flex = outer
    .children
    .iter()
    .find(|c| {
      let w = c.content.width;
      w > 100.0 && w < 600.0
    })
    .expect("should find the row flex container");

  let items: Vec<_> = row_flex.children.iter().filter(|c| c.content.width > 50.0).collect();
  assert_eq!(items.len(), 3, "should have 3 visible items, got {}", items.len());

  let gap_01 = items[1].content.x - (items[0].content.x + items[0].outer_width());
  let gap_12 = items[2].content.x - (items[1].content.x + items[1].outer_width());
  assert!(
    (gap_01 - 16.0).abs() < 2.0,
    "gap 0→1 should be ~16px, got {}. row_w={}, items: x0={} w0={} x1={} x2={}",
    gap_01,
    row_flex.content.width,
    items[0].content.x,
    items[0].outer_width(),
    items[1].content.x,
    items[2].content.x
  );
  assert!((gap_12 - 16.0).abs() < 2.0, "gap 1→2 should be ~16px, got {}", gap_12);
}

#[test]
fn ua_stylesheet_does_not_add_borders_to_plain_divs() {
  let html = r#"<html><body style="margin:0"><div style="width:120px; height:40px">A</div></body></html>"#;
  let doc = lui_parse::parse(html);
  let mut ctx = lui_cascade::cascade::CascadeContext::new();
  let ua = lui_parse::parse_stylesheet(include_str!("../../../../crates/lui/ua/ua_whatwg.css")).unwrap();
  ctx.set_stylesheets(&[ua]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);

  let div = find_by_tag(&lt.root, "div").unwrap();
  assert_eq!(
    [div.border.top, div.border.right, div.border.bottom, div.border.left],
    [0.0, 0.0, 0.0, 0.0],
    "UA stylesheet must not synthesize borders for a plain div"
  );
}

#[test]
fn flex_wrap_wraps_items() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-wrap:wrap; width:200px"><div style="width:120px; height:30px">A</div><div style="width:120px; height:30px">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(flex.children[1].content.y > flex.children[0].content.y, "should wrap");
}

#[test]
fn flex_order_reorders_visually() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px"><div style="width:50px; order:2">A</div><div style="width:50px; order:1">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    flex.children[0].content.x > flex.children[1].content.x,
    "order:2 should be after order:1"
  );
}

#[test]
fn flex_align_items_center() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; height:200px; align-items:center"><div style="width:50px; height:50px">A</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child_cy = flex.children[0].content.y + flex.children[0].content.height / 2.0;
  let flex_cy = flex.content.y + flex.content.height / 2.0;
  assert!((child_cy - flex_cy).abs() < 1.0, "should be centered");
}

#[test]
fn flex_shrink_reduces_overflowing_items() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:200px"><div style="width:150px; flex-shrink:1">A</div><div style="width:150px; flex-shrink:1">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
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
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(flex.kind, BoxKind::FlexContainer);
}

#[test]
fn flex_container_detected_as_flex_kind() {
  let (doc, ctx) = flex_lt(r#"<div style="display:flex"><div>child</div></div>"#, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(flex.kind, BoxKind::FlexContainer);
}

#[test]
fn flex_row_reverse() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-direction:row-reverse; width:300px"><div style="width:50px">A</div><div style="width:50px">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    flex.children[0].content.x > flex.children[1].content.x,
    "row-reverse: first child should be rightmost"
  );
}

// ============================================================================
// 14. Expanded flex tests
// ============================================================================

#[test]
fn flex_column_reverse() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-direction:column-reverse; width:300px; height:200px">
        <div style="height:50px">A</div><div style="height:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    flex.children[0].content.y > flex.children[1].content.y,
    "column-reverse: first child should be below second"
  );
}

#[test]
fn flex_justify_content_flex_end() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; justify-content:flex-end">
        <div style="width:50px">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child_end = flex.children[0].content.x + flex.children[0].content.width;
  let flex_end = flex.content.x + flex.content.width;
  assert!((flex_end - child_end).abs() < 1.0, "flex-end: child at right edge");
}

#[test]
fn flex_justify_content_space_around() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; justify-content:space-around">
        <div style="width:50px">A</div><div style="width:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Free space = 200, 2 items → each gets 100px, half on each side = 50px
  let a_start = flex.children[0].content.x - flex.content.x;
  assert!(
    a_start > 40.0 && a_start < 60.0,
    "space-around: first item should have ~50px before it, got {}",
    a_start
  );
}

#[test]
fn flex_justify_content_space_evenly() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; justify-content:space-evenly">
        <div style="width:50px">A</div><div style="width:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Free space = 200, 3 slots → ~66.7px each
  let a_start = flex.children[0].content.x - flex.content.x;
  assert!(
    a_start > 60.0 && a_start < 73.0,
    "space-evenly: ~66.7px before first, got {}",
    a_start
  );
}

#[test]
fn flex_align_items_flex_start() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; height:200px; align-items:flex-start">
        <div style="width:50px; height:50px">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let offset = flex.children[0].content.y - flex.content.y;
  assert!(offset.abs() < 1.0, "flex-start: child at top, offset={}", offset);
}

#[test]
fn flex_align_items_flex_end() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; height:200px; align-items:flex-end">
        <div style="width:50px; height:50px">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child_bottom = flex.children[0].content.y + flex.children[0].content.height;
  let flex_bottom = flex.content.y + flex.content.height;
  assert!((flex_bottom - child_bottom).abs() < 1.0, "flex-end: child at bottom");
}

#[test]
fn flex_align_items_stretch() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; height:200px; align-items:stretch">
        <div style="width:50px">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    (flex.children[0].content.height - 200.0).abs() < 1.0,
    "stretch: child should fill cross axis, got {}",
    flex.children[0].content.height
  );
}

#[test]
fn flex_align_self_overrides_align_items() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; height:200px; align-items:flex-start">
        <div style="width:50px; height:50px; align-self:flex-end">A</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child_bottom = flex.children[0].content.y + flex.children[0].content.height;
  let flex_bottom = flex.content.y + flex.content.height;
  assert!(
    (flex_bottom - child_bottom).abs() < 1.0,
    "align-self:flex-end should override align-items:flex-start"
  );
}

#[test]
fn flex_basis_sets_initial_size() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px">
        <div style="flex-basis:100px">A</div><div style="flex-basis:200px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    (flex.children[0].content.width - 100.0).abs() < 1.0,
    "flex-basis:100px, got {}",
    flex.children[0].content.width
  );
  assert!(
    (flex.children[1].content.width - 200.0).abs() < 1.0,
    "flex-basis:200px, got {}",
    flex.children[1].content.width
  );
}

#[test]
fn flex_wrap_reverse() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-wrap:wrap-reverse; width:200px; height:200px">
        <div style="width:120px; height:50px">A</div><div style="width:120px; height:50px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // wrap-reverse: second line should be ABOVE the first line
  assert!(
    flex.children[0].content.y > flex.children[1].content.y,
    "wrap-reverse: first item should be below second (y0={}, y1={})",
    flex.children[0].content.y,
    flex.children[1].content.y
  );
}

// ============================================================================
// 19. Flex edge case tests (audit fixes)
// ============================================================================

#[test]
fn flex_gap_shorthand_applies_to_both_axes() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; gap:20px">
        <div style="width:100px">A</div><div style="width:100px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
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
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px">
        <div style="flex-basis:0; flex-grow:1">A</div>
        <div style="flex-basis:0; flex-grow:1">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Both start at 0 and grow equally → 150px each
  assert!(
    (flex.children[0].content.width - 150.0).abs() < 1.0,
    "flex-basis:0 + grow:1 → 150, got {}",
    flex.children[0].content.width
  );
}

#[test]
fn flex_shrink_weighted_by_base_size() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:200px">
        <div style="width:200px; flex-shrink:1">big</div>
        <div style="width:100px; flex-shrink:1">small</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Overflow = 100px. shrink factors weighted: big=200*1=200, small=100*1=100
  // big shrinks by 100*(200/300)=66.7 → 133.3, small shrinks by 100*(100/300)=33.3 → 66.7
  let big = flex.children[0].content.width;
  let small = flex.children[1].content.width;
  assert!(
    big > small,
    "bigger item should still be wider after proportional shrink (big={}, small={})",
    big,
    small
  );
  assert!(
    (big + small - 200.0).abs() < 1.0,
    "total should be 200, got {}",
    big + small
  );
}

#[test]
fn flex_align_self_center_with_parent_stretch() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; height:200px; align-items:stretch">
        <div style="width:50px; height:50px; align-self:center">centered</div>
        <div style="width:50px">stretched</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
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
  assert!(
    (stretched.content.height - 200.0).abs() < 1.0,
    "stretched child should be 200px, got {}",
    stretched.content.height
  );
}

#[test]
fn flex_auto_margins_absorb_free_space() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px">
        <div style="width:100px; margin-left:auto">pushed right</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child = &flex.children[0];
  let child_right = child.content.x + child.content.width;
  let flex_right = flex.content.x + flex.content.width;
  assert!(
    (flex_right - child_right).abs() < 1.0,
    "margin-left:auto should push child to right edge"
  );
}

#[test]
fn flex_min_width_on_item_prevents_shrinking() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:200px">
        <div style="width:150px; flex-shrink:1; min-width:120px">clamped</div>
        <div style="width:150px; flex-shrink:1">unclamped</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    flex.children[0].content.width >= 119.0,
    "min-width:120 should prevent shrinking below 120, got {}",
    flex.children[0].content.width
  );
}

#[test]
fn flex_max_width_on_item_prevents_growing() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:400px">
        <div style="flex-grow:1; max-width:100px">capped</div>
        <div style="flex-grow:1">uncapped</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    flex.children[0].content.width <= 101.0,
    "max-width:100 should cap growth, got {}",
    flex.children[0].content.width
  );
}

#[test]
fn flex_nested_flex_in_flex() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="display:flex; width:400px">
            <div style="display:flex; flex-grow:1">
                <div style="width:50px">inner-A</div>
                <div style="width:50px">inner-B</div>
            </div>
            <div style="width:100px">outer-B</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(flex.children.len(), 2);
  // Inner flex should grow to fill remaining space (300px)
  let inner = &flex.children[0];
  assert!(
    inner.content.width > 200.0,
    "inner flex should grow, got {}",
    inner.content.width
  );
}

// ============================================================================
// 20. align-content multi-line tests
// ============================================================================

#[test]
fn flex_align_content_center_multiline() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:center">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Lines should be centered in the 300px cross axis
  let first_y = flex.children[0].content.y - flex.content.y;
  assert!(
    first_y > 50.0,
    "align-content:center should push lines down from top, first_y={}",
    first_y
  );
}

#[test]
fn flex_align_content_flex_end_multiline() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:flex-end">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let last = &flex.children[1];
  let last_bottom = last.content.y + last.content.height;
  let flex_bottom = flex.content.y + flex.content.height;
  assert!(
    (flex_bottom - last_bottom).abs() < 1.0,
    "align-content:flex-end should push lines to bottom"
  );
}

#[test]
fn flex_align_content_space_between_multiline() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:space-between">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
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
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-wrap:wrap; width:200px; height:300px; align-content:stretch">
        <div style="width:120px; height:40px">A</div>
        <div style="width:120px; height:40px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Two lines, each gets 150px of cross space (300/2). Lines should be spaced 150px apart.
  let a_y = flex.children[0].content.y;
  let b_y = flex.children[1].content.y;
  let line_gap = b_y - a_y;
  assert!(
    line_gap > 140.0 && line_gap < 160.0,
    "stretch should distribute cross space equally, line_gap={}",
    line_gap
  );
}

// ============================================================================
// Intrinsic sizing — flex-basis:auto content measurement
// ============================================================================

#[test]
fn flex_basis_auto_sizes_to_text_content() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:500px"><div>Hello</div><div>World</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a_w = flex.children[0].content.width;
  let b_w = flex.children[1].content.width;
  assert!(
    a_w > 5.0,
    "flex item with text 'Hello' should have non-zero width, got {}",
    a_w
  );
  assert!(
    b_w > 5.0,
    "flex item with text 'World' should have non-zero width, got {}",
    b_w
  );
}

#[test]
fn flex_basis_auto_with_padding_includes_content() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:500px"><div style="padding:10px">Hello</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child = &flex.children[0];
  assert!(
    child.content.width > 5.0,
    "flex item content width should reflect text, got {}",
    child.content.width
  );
}

#[test]
fn flex_min_width_zero_allows_full_shrink() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:100px"><div style="min-width:0">Superlongword</div><div style="min-width:0">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let total: f32 = flex.children.iter().map(|c| c.outer_width()).sum();
  assert!(
    total <= 101.0,
    "min-width:0 should allow shrinking to fit container, got {}",
    total
  );
}

#[test]
fn flex_auto_min_prevents_shrinking_below_content() {
  // Use min-width:0 version as baseline, then check auto min is wider
  let (doc_zero, ctx_zero) = flex_lt(
    r#"<div style="display:flex; width:5px"><div style="min-width:0">Longword</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled_zero = ctx_zero.cascade(&doc_zero.root, &media, &interaction);
  let lt_zero = layout_tree(&styled_zero, 800.0, 600.0);
  let w_zero = find_by_tag(&lt_zero.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  let (doc_auto, ctx_auto) = flex_lt(
    r#"<div style="display:flex; width:5px"><div>Longword</div></div>"#,
    800.0,
  );
  let styled_auto = ctx_auto.cascade(&doc_auto.root, &media, &interaction);
  let lt_auto = layout_tree(&styled_auto, 800.0, 600.0);
  let w_auto = find_by_tag(&lt_auto.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  assert!(
    w_auto > w_zero,
    "auto min-width ({}) should be wider than min-width:0 ({})",
    w_auto,
    w_zero
  );
}

#[test]
fn flex_auto_min_with_multiword_uses_widest_word() {
  // Multi-word text: min-content = widest word, max-content = full text
  // Item should shrink to widest word width but no further
  let (doc_auto, ctx_auto) = flex_lt(
    r#"<div style="display:flex; width:5px"><div>A Longword here</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx_auto.cascade(&doc_auto.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let w_auto = find_by_tag(&lt.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  // With min-width:0, can shrink freely
  let (doc_zero, ctx_zero) = flex_lt(
    r#"<div style="display:flex; width:5px"><div style="min-width:0">A Longword here</div></div>"#,
    800.0,
  );
  let styled_zero = ctx_zero.cascade(&doc_zero.root, &media, &interaction);
  let lt_zero = layout_tree(&styled_zero, 800.0, 600.0);
  let w_zero = find_by_tag(&lt_zero.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  assert!(
    w_auto > w_zero,
    "auto min ({}) should be wider than min-width:0 ({})",
    w_auto,
    w_zero
  );
}

// ============================================================================
// visibility: collapse on flex items
// ============================================================================

#[test]
fn flex_visibility_collapse_zero_main_size() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px">
        <div style="width:100px; height:50px">A</div>
        <div style="width:100px; height:50px; visibility:collapse">B</div>
        <div style="width:100px; height:50px">C</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let collapsed = &flex.children[1];
  assert!(
    collapsed.content.width < 1.0,
    "collapsed item main size should be ~0, got {}",
    collapsed.content.width
  );
}

#[test]
fn flex_visibility_collapse_preserves_cross_size() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; align-items:stretch">
        <div style="width:100px">short</div>
        <div style="width:100px; height:80px; visibility:collapse">tall</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let short = &flex.children[0];
  assert!(
    short.content.height >= 79.0,
    "non-collapsed item should stretch to collapsed item's cross size (80), got {}",
    short.content.height
  );
}

// ============================================================================
// Column-direction intrinsic sizing
// ============================================================================

#[test]
fn flex_column_basis_auto_sizes_to_text_content() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; flex-direction:column; width:200px">
        <div>Hello</div><div>World</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a_h = flex.children[0].content.height;
  let b_h = flex.children[1].content.height;
  assert!(
    a_h > 5.0,
    "column flex item with text should have non-zero height, got {}",
    a_h
  );
  assert!(
    b_h > 5.0,
    "column flex item with text should have non-zero height, got {}",
    b_h
  );
}

// ============================================================================
// Baseline alignment
// ============================================================================

#[test]
fn flex_align_items_baseline_aligns_text() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:400px; align-items:baseline">
        <div style="padding-top:20px">A</div>
        <div>B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a = &flex.children[0];
  let b = &flex.children[1];
  // A has 20px padding-top, B has none. For baselines to align,
  // B should be pushed down by ~20px relative to flex start.
  let b_offset = b.content.y - b.padding.top - b.border.top - b.margin.top - flex.content.y;
  assert!(
    b_offset > 15.0,
    "baseline alignment should push B down to match A's baseline, B offset={}",
    b_offset
  );
}

#[test]
fn flex_align_self_baseline_with_others_stretch() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:400px; height:100px; align-items:stretch">
        <div style="align-self:baseline; padding-top:30px">base</div>
        <div>stretched</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let stretched = &flex.children[1];
  assert!(
    stretched.content.height >= 99.0,
    "stretched item should still fill cross axis, got {}",
    stretched.content.height
  );
}

// ============================================================================
// flex shorthand keywords
// ============================================================================

#[test]
fn flex_shorthand_single_number() {
  // flex:1 → flex-grow:1, flex-shrink:1, flex-basis:0
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px"><div style="flex:1">A</div><div style="flex:2">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let ratio = flex.children[1].content.width / flex.children[0].content.width;
  assert!(
    (ratio - 2.0).abs() < 0.1,
    "flex:1 vs flex:2 should give 2:1 ratio, got {}",
    ratio
  );
}

#[test]
fn flex_shorthand_none() {
  // flex:none → flex-grow:0, flex-shrink:0, flex-basis:auto
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px"><div style="flex:none; width:100px">A</div><div style="flex:1">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    (flex.children[0].content.width - 100.0).abs() < 1.0,
    "flex:none should keep width:100px, got {}",
    flex.children[0].content.width
  );
}

// ============================================================================
// Absolutely-positioned flex children
// ============================================================================

#[test]
fn flex_absolute_child_not_a_flex_item() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:300px; position:relative">
        <div style="width:100px">A</div>
        <div style="position:absolute; top:0; left:0; width:50px; height:50px">abs</div>
        <div style="width:100px">B</div>
    </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // A and B should be adjacent (abs child removed from flow)
  let a = &flex.children[0];
  let b = &flex.children[1];
  let gap = b.content.x - (a.content.x + a.content.width);
  assert!(
    gap < 1.0,
    "A and B should be adjacent (abs child not in flow), gap={}",
    gap
  );
}

// ============================================================================
// Percentage margins/padding
// ============================================================================

#[test]
fn block_percentage_padding_resolves_against_width() {
  let (doc, ctx) = flex_lt(r#"<div style="width:200px; padding:10%">content</div>"#, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // 10% of containing width (800) = 80px on each side
  assert!(
    (div.padding.left - 80.0).abs() < 1.0,
    "10% padding should be 80px (10% of 800), got {}",
    div.padding.left
  );
}

#[test]
fn flex_item_percentage_margin() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:400px"><div style="width:100px; margin-left:10%">A</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let item = &flex.children[0];
  // 10% of flex container inner width (400) = 40px
  assert!(
    (item.margin.left - 40.0).abs() < 1.0,
    "10% margin-left should be 40px (10% of 400), got {}",
    item.margin.left
  );
}

// ============================================================================
// flex-basis keywords: content, max-content, min-content
// ============================================================================

#[test]
fn flex_basis_max_content_uses_content_width() {
  // Two items with flex-basis:max-content should size to their text
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:500px">
            <div style="flex-basis:max-content">Hello</div>
            <div style="flex-basis:max-content">World</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(
    flex.children[0].content.width > 1.0,
    "flex-basis:max-content should size to text, got {}",
    flex.children[0].content.width
  );
}

#[test]
fn flex_basis_min_content_differs_from_zero() {
  let (doc_min, ctx_min) = flex_lt(
    r#"<div style="display:flex; width:500px">
            <div style="flex-basis:min-content">Hello</div>
        </div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled_min = ctx_min.cascade(&doc_min.root, &media, &interaction);
  let lt_min = layout_tree(&styled_min, 800.0, 600.0);
  let w_min = find_by_tag(&lt_min.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  let (doc_zero, ctx_zero) = flex_lt(
    r#"<div style="display:flex; width:500px">
            <div style="flex-basis:0">Hello</div>
        </div>"#,
    800.0,
  );
  let styled_zero = ctx_zero.cascade(&doc_zero.root, &media, &interaction);
  let lt_zero = layout_tree(&styled_zero, 800.0, 600.0);
  let w_zero = find_by_tag(&lt_zero.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  assert!(
    (w_min - w_zero).abs() > 0.1,
    "flex-basis:min-content ({}) should differ from flex-basis:0 ({})",
    w_min,
    w_zero
  );
}

#[test]
fn flex_basis_content_same_as_max_content() {
  let (doc_content, ctx_content) = flex_lt(
    r#"<div style="display:flex; width:500px"><div style="flex-basis:content">Hello</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx_content.cascade(&doc_content.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let w_content = find_by_tag(&lt.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  let (doc_max, ctx_max) = flex_lt(
    r#"<div style="display:flex; width:500px"><div style="flex-basis:max-content">Hello</div></div>"#,
    800.0,
  );
  let styled_max = ctx_max.cascade(&doc_max.root, &media, &interaction);
  let lt_max = layout_tree(&styled_max, 800.0, 600.0);
  let w_max = find_by_tag(&lt_max.root, "body")
    .unwrap()
    .children
    .first()
    .unwrap()
    .children[0]
    .content
    .width;

  assert!(
    (w_content - w_max).abs() < 0.1,
    "flex-basis:content ({}) should equal flex-basis:max-content ({})",
    w_content,
    w_max
  );
}

#[test]
fn flex_row_with_explicit_sized_children() {
  let html = r#"<div style="display:flex; gap:16px">
        <div style="width:120px; height:120px">A</div>
        <div style="width:120px; height:120px">B</div>
        <div style="width:120px; height:120px">C</div>
    </div>"#;
  let (doc, ctx) = flex_lt(html, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let ch = &flex.children;
  assert_eq!(ch.len(), 3, "flex should have 3 children");
  // All children should be on the same row (same y)
  assert!(
    (ch[0].content.y - ch[1].content.y).abs() < 1.0,
    "children should share y: {} vs {}",
    ch[0].content.y,
    ch[1].content.y
  );
  // Children should be distributed horizontally with gaps
  assert!(
    ch[1].content.x > ch[0].content.x + 100.0,
    "child B x={} should be > child A x={} + 100",
    ch[1].content.x,
    ch[0].content.x
  );
  assert!(
    ch[2].content.x > ch[1].content.x + 100.0,
    "child C x={} should be > child B x={} + 100",
    ch[2].content.x,
    ch[1].content.x
  );
  // Container height should be at least 120px
  assert!(
    flex.content.height >= 119.0,
    "flex height should be >= 120px, got {}",
    flex.content.height
  );
}

#[test]
fn nested_column_flex_with_row_flex_children() {
  let html = r#"<div style="display:flex; flex-direction:column; padding:40px">
        <div style="display:flex; gap:16px">
            <div style="width:120px; height:120px">A</div>
            <div style="width:120px; height:120px">B</div>
            <div style="width:120px; height:120px">C</div>
        </div>
    </div>"#;
  let (doc, ctx) = flex_lt(html, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let outer = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let inner = &outer.children[0];
  eprintln!(
    "outer: kind={:?} w={} h={}",
    outer.kind, outer.content.width, outer.content.height
  );
  eprintln!(
    "inner: kind={:?} w={} h={}",
    inner.kind, inner.content.width, inner.content.height
  );
  for (i, c) in inner.children.iter().enumerate() {
    eprintln!(
      "  child[{}]: x={} y={} w={} h={}",
      i, c.content.x, c.content.y, c.content.width, c.content.height
    );
  }
  // Inner flex row children should be horizontal
  assert!(
    (inner.children[0].content.y - inner.children[1].content.y).abs() < 1.0,
    "row children should share y"
  );
  assert!(
    inner.children[1].content.x > inner.children[0].content.x + 100.0,
    "child B should be right of child A"
  );
}

#[test]
fn debug_shrink_issue() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:200px"><div style="width:150px; flex-shrink:1">A</div><div style="width:150px; flex-shrink:1">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  eprintln!(
    "flex: kind={:?} w={} h={} children={}",
    flex.kind,
    flex.content.width,
    flex.content.height,
    flex.children.len()
  );
  for (i, c) in flex.children.iter().enumerate() {
    eprintln!(
      "  child[{}]: kind={:?} w={} h={} x={}",
      i, c.kind, c.content.width, c.content.height, c.content.x
    );
  }
}

#[test]
fn flex_align_items_center_same_y_for_same_height_children() {
  let html = r#"<div style="display:flex; gap:16px">
        <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
            <span style="font-size:14px">Block</span>
        </div>
        <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
            <span style="font-size:14px">Flex</span>
        </div>
        <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
            <span style="font-size:14px">Grid</span>
        </div>
    </div>"#;
  let (doc, ctx) = flex_lt(html, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let outer = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();

  let spans: Vec<_> = outer
    .children
    .iter()
    .map(|card| {
      let span = &card.children[0];
      eprintln!(
        "span tag={} y={:.1} h={:.1} content_y={:.1}",
        span.node.element().tag_name(),
        span.content.y,
        span.content.height,
        span.content.y
      );
      span
    })
    .collect();

  let y0 = spans[0].content.y;
  let y1 = spans[1].content.y;
  let y2 = spans[2].content.y;
  eprintln!("y positions: {:.1} {:.1} {:.1}", y0, y1, y2);

  assert!(
    (y0 - y1).abs() < 2.0,
    "Block and Flex should be at same y: {} vs {}",
    y0,
    y1
  );
  assert!(
    (y1 - y2).abs() < 2.0,
    "Flex and Grid should be at same y: {} vs {}",
    y1,
    y2
  );
}

#[test]
fn flex_column_align_items_center_shrinks_cross_axis() {
  let html = r#"<div style="display:flex; flex-direction:column; align-items:center; width:800px">
        <h1 style="font-size:32px">lui v2</h1>
    </div>"#;
  let (doc, ctx) = flex_lt(html, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let h1 = &flex.children[0];
  let h1_outer_left = h1.content.x - h1.padding.left - h1.border.left - h1.margin.left;
  let h1_outer_right = h1.content.x + h1.content.width + h1.padding.right + h1.border.right + h1.margin.right;
  let h1_outer_width = h1_outer_right - h1_outer_left;
  let flex_center = flex.content.x + flex.content.width / 2.0;
  let h1_center = (h1_outer_left + h1_outer_right) / 2.0;
  eprintln!(
    "flex: x={} w={} center={}",
    flex.content.x, flex.content.width, flex_center
  );
  eprintln!(
    "h1: outer_left={} outer_right={} outer_w={} center={}",
    h1_outer_left, h1_outer_right, h1_outer_width, h1_center
  );
  // The h1 should be narrower than the container (shrink-wrapped to text)
  assert!(
    h1_outer_width < flex.content.width * 0.8,
    "h1 should shrink-wrap to text content, not fill container. h1_outer_width={}, container_width={}",
    h1_outer_width,
    flex.content.width
  );
  // The h1 should be centered
  assert!(
    (h1_center - flex_center).abs() < 2.0,
    "h1 should be centered in container. h1_center={}, flex_center={}",
    h1_center,
    flex_center
  );
}

#[test]
fn flex_column_align_items_center_with_padding() {
  let html = r#"<div style="display:flex; flex-direction:column; align-items:center; width:800px">
        <div style="padding:20px; font-size:32px">Hello</div>
    </div>"#;
  let (doc, ctx) = flex_lt(html, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let child = &flex.children[0];
  let outer_left = child.content.x - child.padding.left - child.border.left - child.margin.left;
  let outer_right =
    child.content.x + child.content.width + child.padding.right + child.border.right + child.margin.right;
  let outer_width = outer_right - outer_left;
  let child_center = (outer_left + outer_right) / 2.0;
  let flex_center = flex.content.x + flex.content.width / 2.0;
  eprintln!(
    "child: content.w={} padding=({},{}) outer_w={}",
    child.content.width, child.padding.left, child.padding.right, outer_width
  );
  // Should shrink-wrap to content + padding, not fill container
  assert!(
    outer_width < flex.content.width * 0.8,
    "child should shrink-wrap, not fill container. outer_w={}, container_w={}",
    outer_width,
    flex.content.width
  );
  // Should be centered
  assert!(
    (child_center - flex_center).abs() < 2.0,
    "child should be centered. child_center={}, flex_center={}",
    child_center,
    flex_center
  );
  // Padding should be present
  assert!(
    (child.padding.left - 20.0).abs() < 1.0,
    "padding.left should be 20, got {}",
    child.padding.left
  );
  assert!(
    (child.padding.right - 20.0).abs() < 1.0,
    "padding.right should be 20, got {}",
    child.padding.right
  );
}

#[test]
fn flex_column_center_preserves_explicit_child_sizes() {
  let html = r#"<div style="display:flex; flex-direction:column; align-items:center; width:800px; padding:40px">
        <div style="display:flex; gap:16px">
            <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
                <span style="font-size:14px">Block</span>
            </div>
            <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
                <span style="font-size:14px">Flex</span>
            </div>
            <div style="width:120px; height:120px; display:flex; align-items:center; justify-content:center">
                <span style="font-size:14px">Grid</span>
            </div>
        </div>
    </div>"#;
  let (doc, ctx) = flex_lt(html, 800.0);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let outer = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let row = &outer.children[0];
  // The row flex container should be centered
  let row_outer_left = row.content.x - row.padding.left - row.border.left - row.margin.left;
  let row_outer_right = row.content.x + row.content.width + row.padding.right + row.border.right + row.margin.right;
  let row_center = (row_outer_left + row_outer_right) / 2.0;
  // Cards should be 120x120
  for (i, card) in row.children.iter().enumerate() {
    assert!(
      (card.content.width - 120.0).abs() < 1.0,
      "card[{}] width should be 120, got {}",
      i,
      card.content.width
    );
    assert!(
      (card.content.height - 120.0).abs() < 1.0,
      "card[{}] height should be 120, got {}",
      i,
      card.content.height
    );
  }
  // Row should be centered within the outer container's layout area
  // (layout_flex uses inner_width which accounts for padding)
  let layout_center = outer.content.x + (outer.content.width - outer.padding.horizontal()) / 2.0;
  assert!(
    (row_center - layout_center).abs() < 2.0,
    "row should be centered. row_center={}, layout_center={}",
    row_center,
    layout_center
  );
}

#[test]
fn flex_items_no_grow_padding_not_doubled() {
  let (doc, ctx) = flex_lt(
    r#"<div style="display:flex; width:400px"><div style="padding:0 20px">A</div><div style="padding:0 20px">B</div></div>"#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let a = &flex.children[0];
  let b = &flex.children[1];
  let a_outer = a.content.width + a.padding.left + a.padding.right + a.border.left + a.border.right;
  let b_outer = b.content.width + b.padding.left + b.padding.right + b.border.left + b.border.right;
  let total = a_outer + b_outer;
  assert!(
    total < 200.0,
    "items with no flex-grow should be content-sized, total outer={}",
    total
  );
}
