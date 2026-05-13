use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::engine::{layout_tree, LayoutEngine};
use lui_layout::incremental::layout_tree_incremental;
use lui_layout::LayoutBox;
use lui_core::Rect;

use crate::helpers::*;

fn rects_match(a: &[(& lui_parse::HtmlNode, Rect)], b: &[(&lui_parse::HtmlNode, Rect)]) -> bool {
    if a.len() != b.len() { return false; }
    for (ra, rb) in a.iter().zip(b.iter()) {
        if !std::ptr::eq(ra.0, rb.0) { return false; }
        if (ra.1.x - rb.1.x).abs() > 0.5
            || (ra.1.y - rb.1.y).abs() > 0.5
            || (ra.1.width - rb.1.width).abs() > 0.5
            || (ra.1.height - rb.1.height).abs() > 0.5
        {
            return false;
        }
    }
    true
}

fn boxes_match(a: &LayoutBox, b: &LayoutBox) -> bool {
    if (a.content.x - b.content.x).abs() > 0.5 { return false; }
    if (a.content.y - b.content.y).abs() > 0.5 { return false; }
    if (a.content.width - b.content.width).abs() > 0.5 { return false; }
    if (a.content.height - b.content.height).abs() > 0.5 { return false; }
    if a.children.len() != b.children.len() { return false; }
    for (ca, cb) in a.children.iter().zip(b.children.iter()) {
        if !boxes_match(ca, cb) { return false; }
    }
    true
}

// ============================================================================
// Correctness: incremental with 0 dirty matches full layout
// ============================================================================

#[test]
fn incremental_0_dirty_matches_full() {
    let html = r#"<div style="display:flex; width:400px">
        <div style="flex:1; height:50px">A</div>
        <div style="flex:1; height:50px">B</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let inc = layout_tree_incremental(&styled, &full, &[], 800.0, 600.0);
    assert!(boxes_match(&full.root, &inc.root),
        "incremental with 0 dirty should match full layout");
}

// ============================================================================
// Correctness: incremental with 1 dirty leaf matches full
// ============================================================================

#[test]
fn incremental_1_dirty_leaf_matches_full() {
    let html = r#"<div style="width:400px">
        <div style="height:40px">A</div>
        <div style="height:60px">B</div>
        <div style="height:30px">C</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 1]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    assert!(boxes_match(&full.root, &inc.root),
        "incremental with 1 dirty leaf should match full layout");
}

// ============================================================================
// Clean sibling gets correct position when earlier sibling is dirty
// ============================================================================

#[test]
fn clean_sibling_position_after_dirty() {
    let html = r#"<div style="width:400px">
        <div style="height:40px">A</div>
        <div style="height:60px">B</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 0]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    assert!(boxes_match(&full.root, &inc.root),
        "sibling after dirty should be correctly positioned");
}

// ============================================================================
// Viewport resize falls back to full layout
// ============================================================================

#[test]
fn viewport_resize_falls_back() {
    let html = r#"<div style="width:100%"><div style="height:50px">A</div></div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let prev = layout_tree(&styled, 800.0, 600.0);
    let inc = layout_tree_incremental(&styled, &prev, &[vec![0]], 1024.0, 768.0);
    let full = layout_tree(&styled, 1024.0, 768.0);
    assert!(boxes_match(&full.root, &inc.root),
        "viewport resize should produce correct layout");
}

// ============================================================================
// Clean flex subtree cloned correctly
// ============================================================================

#[test]
fn clean_flex_subtree_cloned() {
    let html = r#"<div style="width:800px">
        <div style="display:flex; gap:8px">
            <div style="flex:1; height:50px">A</div>
            <div style="flex:1; height:50px">B</div>
        </div>
        <div style="height:30px">C</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 1]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    assert!(boxes_match(&full.root, &inc.root),
        "clean flex subtree should be cloned correctly");
}

// ============================================================================
// Clean grid subtree cloned correctly
// ============================================================================

#[test]
fn clean_grid_subtree_cloned() {
    let html = r#"<div style="width:800px">
        <div style="display:grid; grid-template-columns:1fr 1fr; gap:4px">
            <div style="height:40px">A</div>
            <div style="height:40px">B</div>
        </div>
        <div style="height:30px">C</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 1]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    assert!(boxes_match(&full.root, &inc.root),
        "clean grid subtree should be cloned correctly");
}

// ============================================================================
// Clean table subtree cloned correctly
// ============================================================================

#[test]
fn clean_table_subtree_cloned() {
    let html = r#"<div style="width:800px">
        <table style="width:400px">
            <tr><td style="height:30px">A</td><td style="height:30px">B</td></tr>
        </table>
        <div style="height:20px">C</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 1]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    assert!(boxes_match(&full.root, &inc.root),
        "clean table subtree should be cloned correctly");
}

// ============================================================================
// Scroll state preserved on clean subtree
// ============================================================================

#[test]
fn scroll_state_preserved() {
    let html = r#"<div style="width:800px">
        <div style="overflow:scroll; width:200px; height:100px">
            <div style="height:300px">tall content</div>
        </div>
        <div style="height:20px">other</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let scroll_box = find_by_tag(&full.root, "body").unwrap()
        .children[0].children[0].scroll;
    assert!(scroll_box.is_some(), "should have scroll container");

    let dirty = vec![vec![0, 0, 1]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    let inc_scroll = find_by_tag(&inc.root, "body").unwrap()
        .children[0].children[0].scroll;
    assert!(inc_scroll.is_some(), "scroll state should be preserved");
}

// ============================================================================
// Rects vector populated for cloned subtrees
// ============================================================================

#[test]
fn rects_populated_for_cloned_subtrees() {
    let html = r#"<div style="width:400px">
        <div style="height:40px">A</div>
        <div style="height:60px">B</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let full = layout_tree(&styled, 800.0, 600.0);
    let dirty = vec![vec![0, 0, 0]];
    let inc = layout_tree_incremental(&styled, &full, &dirty, 800.0, 600.0);
    assert!(!inc.rects.is_empty(), "rects should be populated");
    for (node, rect) in &full.rects {
        let found = inc.rects.iter().any(|(n, r)| {
            std::ptr::eq(*n, *node)
                && (r.x - rect.x).abs() < 0.5
                && (r.y - rect.y).abs() < 0.5
                && (r.width - rect.width).abs() < 0.5
                && (r.height - rect.height).abs() < 0.5
        });
        assert!(found, "full layout rect for {} not found in incremental rects",
            node.element.tag_name());
    }
}

// ============================================================================
// LayoutEngine OOP API
// ============================================================================

#[test]
fn engine_layout_matches_free_function() {
    let html = r#"<div style="display:flex; width:400px">
        <div style="flex:1; height:50px">A</div>
        <div style="flex:1; height:50px">B</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let free = layout_tree(&styled, 800.0, 600.0);
    let mut engine = LayoutEngine::new();
    let mut text_ctx = lui_layout::text::TextContext::new();
    let eng = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    assert!(boxes_match(&free.root, &eng.root),
        "LayoutEngine::layout should match free function");
}

#[test]
fn engine_layout_dirty_matches_full() {
    let html = r#"<div style="width:400px">
        <div style="height:40px">A</div>
        <div style="height:60px">B</div>
        <div style="height:30px">C</div>
    </div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let mut engine = LayoutEngine::new();
    let mut text_ctx = lui_layout::text::TextContext::new();
    let _first = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    let dirty = engine.layout_dirty(&styled, &[vec![0, 0, 1]], 800.0, 600.0, &mut text_ctx);
    let full = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    assert!(boxes_match(&full.root, &dirty.root),
        "LayoutEngine::layout_dirty should match full layout");
}

#[test]
fn engine_takes_external_text_context() {
    let html = r#"<div style="height:30px">hello</div>"#;
    let (doc, ctx) = flex_lt(html, 800.0);
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);

    let mut engine = LayoutEngine::new();
    let mut text_ctx = lui_layout::text::TextContext::new();
    let t1 = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    let t2 = engine.layout(&styled, 800.0, 600.0, &mut text_ctx);
    assert!(boxes_match(&t1.root, &t2.root),
        "repeated layout calls should produce identical results");
}
