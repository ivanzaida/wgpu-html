use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_layout::{BoxKind, LayoutBox, Overflow, engine::layout_tree};

use crate::helpers::*;

#[path = "scrollbar_gutter/stable_both_edges_reserves_mirrored_width_for_vertical_scrollbar.rs"]
mod stable_both_edges_reserves_mirrored_width_for_vertical_scrollbar;

#[path = "scrollbar_gutter/stable_reserves_height_for_horizontal_auto_scrollbar.rs"]
mod stable_reserves_height_for_horizontal_auto_scrollbar;

#[path = "scrollbar_gutter/stable_reserves_width_for_vertical_auto_scrollbar.rs"]
mod stable_reserves_width_for_vertical_auto_scrollbar;

// ── overflow: hidden ──────────────────────────────────────────────────

#[test]
fn overflow_hidden_sets_clip_rect() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-x:hidden; overflow-y:hidden">
            <div style="height:300px">tall content</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.overflow_x, Overflow::Hidden);
  assert_eq!(container.overflow_y, Overflow::Hidden);
  assert!(container.clip.is_some(), "overflow:hidden should set clip rect");
  let clip = container.clip.unwrap();
  assert!((clip.width - 200.0).abs() < 1.0, "clip width = content width");
  assert!((clip.height - 100.0).abs() < 1.0, "clip height = content height");
}

#[test]
fn overflow_hidden_does_not_change_layout_width() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-x:hidden">
            <div style="height:50px">child</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // overflow:hidden shouldn't add scrollbar space
  assert!(
    (container.content.width - 200.0).abs() < 1.0,
    "hidden: no scrollbar reduction, got {}",
    container.content.width
  );
}

#[test]
fn overflow_hidden_has_scroll_info() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:hidden">
            <div style="height:300px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert!(container.scroll.is_some());
  let scroll = container.scroll.unwrap();
  assert!(
    scroll.scroll_height >= 299.0,
    "scroll_height should reflect content extent, got {}",
    scroll.scroll_height
  );
}

// ── overflow: scroll ──────────────────────────────────────────────────

#[test]
fn overflow_scroll_reserves_scrollbar_width() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:50px">child</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.overflow_y, Overflow::Scroll);
  // overflow:scroll always reserves scrollbar width (default 15px)
  assert!(
    container.content.width < 200.0,
    "scroll should reduce content width, got {}",
    container.content.width
  );
  assert!(
    (container.content.width - 185.0).abs() < 1.0,
    "200 - 15px scrollbar = 185, got {}",
    container.content.width
  );
}

#[test]
fn overflow_scroll_x_reserves_scrollbar_height() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-x:scroll">
            <div style="height:50px">child</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.overflow_x, Overflow::Scroll);
  // overflow-x:scroll reduces height by scrollbar
  assert!(
    (container.content.height - 85.0).abs() < 1.0,
    "100 - 15px scrollbar = 85, got {}",
    container.content.height
  );
}

#[test]
fn overflow_scroll_has_scrollbar_info() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:50px">child</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let scroll = container.scroll.unwrap();
  assert!((scroll.scrollbar_width - 15.0).abs() < 1.0);
  assert_eq!(scroll.scroll_x, 0.0);
  assert_eq!(scroll.scroll_y, 0.0);
}

// ── overflow: auto ────────────────────────────────────────────────────

#[test]
fn overflow_auto_no_scrollbar_when_fits() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:auto">
            <div style="height:50px">fits</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.overflow_y, Overflow::Auto);
  // Content fits — no scrollbar needed
  assert!(
    (container.content.width - 200.0).abs() < 1.0,
    "auto: no scrollbar when fits, got {}",
    container.content.width
  );
}

#[test]
fn overflow_auto_scrollbar_when_overflows() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:auto">
            <div style="height:300px">overflows</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // Content overflows — auto shows scrollbar
  assert!(
    container.content.width < 200.0,
    "auto: should reduce width when overflows, got {}",
    container.content.width
  );
}

// ── overflow: visible ─────────────────────────────────────────────────

#[test]
fn overflow_visible_no_clip() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px">
            <div style="height:300px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.overflow_x, Overflow::Visible);
  assert_eq!(container.overflow_y, Overflow::Visible);
  assert!(container.clip.is_none(), "visible: no clipping");
  assert!(container.scroll.is_none(), "visible: no scroll info");
}

// ── scrollbar-width ───────────────────────────────────────────────────

#[test]
fn scrollbar_width_thin() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll; scrollbar-width:thin">
            <div style="height:50px">child</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // thin scrollbar = 8px
  assert!(
    (container.content.width - 192.0).abs() < 1.0,
    "thin scrollbar: 200-8=192, got {}",
    container.content.width
  );
  assert!((container.scroll.unwrap().scrollbar_width - 8.0).abs() < 1.0);
}

#[test]
fn scrollbar_width_none() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll; scrollbar-width:none">
            <div style="height:50px">child</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // scrollbar-width:none — no space reserved
  assert!(
    (container.content.width - 200.0).abs() < 1.0,
    "no scrollbar space with scrollbar-width:none, got {}",
    container.content.width
  );
}

// ── overflow: clip ────────────────────────────────────────────────────

#[test]
fn overflow_clip_sets_clip_rect() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-x:clip; overflow-y:clip">
            <div style="height:300px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  assert_eq!(container.overflow_x, Overflow::Clip);
  assert!(container.clip.is_some(), "clip: should have clip rect");
  // clip doesn't reserve scrollbar space
  assert!((container.content.width - 200.0).abs() < 1.0);
}

// ── Scroll position API ───────────────────────────────────────────────

#[test]
fn set_scroll_clamps_to_max() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:500px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let mut lt = layout_tree(&styled, 800.0, 600.0);
  let node_ptr = lt.root.children[0].children[0].node as *const _;

  lt.root.children[0].children[0].set_scroll(0.0, 99999.0);

  let container = &lt.root.children[0].children[0];
  let info = container.scroll.unwrap();
  let max_y = info.max_scroll_y(container.content.height);
  assert!(
    (info.scroll_y - max_y).abs() < 1.0,
    "scroll_y should be clamped to max ({}), got {}",
    max_y,
    info.scroll_y
  );
  assert!(info.scroll_y > 0.0, "should have scrolled some amount");
  let _ = node_ptr;
}

#[test]
fn set_scroll_clamps_to_zero() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:500px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let mut lt = layout_tree(&styled, 800.0, 600.0);

  lt.root.children[0].children[0].set_scroll(0.0, -100.0);

  let container = &lt.root.children[0].children[0];
  assert_eq!(container.scroll.unwrap().scroll_y, 0.0, "negative scroll clamped to 0");
}

#[test]
fn scroll_by_increments_position() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:500px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let mut lt = layout_tree(&styled, 800.0, 600.0);

  lt.root.children[0].children[0].scroll_by(0.0, 50.0);
  let s1 = lt.root.children[0].children[0].scroll.unwrap().scroll_y;
  assert!((s1 - 50.0).abs() < 1.0, "first scroll_by +50, got {}", s1);

  lt.root.children[0].children[0].scroll_by(0.0, 30.0);
  let s2 = lt.root.children[0].children[0].scroll.unwrap().scroll_y;
  assert!((s2 - 80.0).abs() < 1.0, "second scroll_by +30 = 80, got {}", s2);
}

#[test]
fn set_scroll_returns_false_on_non_scroll_container() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px">
            <div style="height:50px">no scroll</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let mut lt = layout_tree(&styled, 800.0, 600.0);

  let changed = lt.root.children[0].children[0].set_scroll(0.0, 50.0);
  assert!(!changed, "non-scroll container should return false");
}

#[test]
fn max_scroll_y_correct() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:400px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  let info = container.scroll.unwrap();
  let max_y = info.max_scroll_y(container.content.height);
  // scroll_height ~400, content_height ~85 (100-15 scrollbar), max = ~315
  assert!(max_y > 200.0, "max_scroll_y should be large, got {}", max_y);
}

#[test]
fn child_visible_rect_offsets_by_scroll() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:400px">tall</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let mut lt = layout_tree(&styled, 800.0, 600.0);

  lt.root.children[0].children[0].set_scroll(0.0, 50.0);

  let container = &lt.root.children[0].children[0];
  let child_rect = container.children[0].content;
  let visible = container.child_visible_rect(child_rect);
  assert!(
    (visible.y - (child_rect.y - 50.0)).abs() < 1.0,
    "visible rect should be offset by scroll"
  );
}

// ── Hit testing ───────────────────────────────────────────────────────

#[test]
fn hit_test_finds_element() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px">
            <div style="width:100px; height:50px">target</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let hit = lt.hit_test(50.0, 25.0);
  assert!(hit.is_some(), "should hit something at (50, 25)");
}

#[test]
fn hit_test_misses_outside() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:100px; height:50px">small</div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let hit = lt.hit_test(9999.0, 9999.0);
  // Should miss — outside all boxes
  assert!(hit.is_none(), "should miss at far coordinates");
}

#[test]
fn scroll_to_reveal_computes_offset() {
  let (doc, ctx) = flex_lt(
    r#"
        <div style="width:200px; height:100px; overflow-y:scroll">
            <div style="height:50px">A</div>
            <div style="height:50px">B</div>
            <div style="height:50px">C</div>
            <div style="height:50px">D</div>
        </div>
    "#,
    800.0,
  );
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let lt = layout_tree(&styled, 800.0, 600.0);
  let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
  // D is at y ≈ 150, container height ≈ 85 → need to scroll
  if container.children.len() >= 4 {
    let d_rect = container.children[3].content;
    let result = container.scroll_to_reveal(d_rect);
    assert!(result.is_some());
    let (_, sy) = result.unwrap();
    assert!(sy > 0.0, "should need to scroll to reveal D, sy={}", sy);
  }
}
