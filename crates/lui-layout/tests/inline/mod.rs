use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::{BoxKind, LayoutBox, engine::layout_tree};
use crate::helpers::*;

// ============================================================================
// 12. Inline line breaking tests
// ============================================================================

#[test]
fn inline_text_wraps_at_container_width() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:80px">The quick brown fox jumps over the lazy dog</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Text should wrap into multiple lines, making height > single line
    assert!(container.content.height > 20.0, "text should wrap, height={}", container.content.height);
}

#[test]
fn inline_short_text_fits_single_line() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:800px">Hi</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(container.content.height > 0.0, "should have nonzero height");
    assert!(container.content.height < 40.0, "short text should be one line, height={}", container.content.height);
}

#[test]
fn inline_container_wraps_children() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px">
            <span>aaa </span><span>bbb </span><span>ccc </span><span>ddd </span><span>eee </span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(container.content.height > 0.0, "should have content");
}

// ============================================================================
// 21. Inline-block tests
// ============================================================================

#[test]
fn inline_block_respects_width_and_height() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <span style="display:inline-block; width:100px; height:50px">box</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let _body = find_by_tag(&lt.root, "body").unwrap();
    // Find the inline-block by looking for InlineBlock kind
    fn find_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind) -> Option<&'a LayoutBox<'a>> {
        if b.kind == kind { return Some(b); }
        for c in &b.children { if let Some(f) = find_kind(c, kind) { return Some(f); } }
        None
    }
    let ib = find_kind(&lt.root, BoxKind::InlineBlock);
    assert!(ib.is_some(), "should find InlineBlock box");
    let ib = ib.unwrap();
    assert!((ib.content.width - 100.0).abs() < 1.0, "width:100px, got {}", ib.content.width);
    assert!((ib.content.height - 50.0).abs() < 1.0, "height:50px, got {}", ib.content.height);
}

#[test]
fn inline_block_flows_horizontally() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <span style="display:inline-block; width:100px; height:30px">A</span>
            <span style="display:inline-block; width:100px; height:30px">B</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_all_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind, out: &mut Vec<&'a LayoutBox<'a>>) {
        if b.kind == kind { out.push(b); }
        for c in &b.children { find_all_kind(c, kind, out); }
    }
    let mut ibs = Vec::new();
    find_all_kind(&lt.root, BoxKind::InlineBlock, &mut ibs);
    assert_eq!(ibs.len(), 2, "should have 2 inline-blocks");
    assert!(ibs[1].content.x > ibs[0].content.x, "second inline-block should be to the right");
}

#[test]
fn inline_block_wraps_when_exceeds_container() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:150px">
            <span style="display:inline-block; width:100px; height:30px">A</span>
            <span style="display:inline-block; width:100px; height:30px">B</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_all_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind, out: &mut Vec<&'a LayoutBox<'a>>) {
        if b.kind == kind { out.push(b); }
        for c in &b.children { find_all_kind(c, kind, out); }
    }
    let mut ibs = Vec::new();
    find_all_kind(&lt.root, BoxKind::InlineBlock, &mut ibs);
    assert_eq!(ibs.len(), 2);
    assert!(ibs[1].content.y > ibs[0].content.y,
        "second inline-block should wrap to next line (y0={}, y1={})",
        ibs[0].content.y, ibs[1].content.y);
}

#[test]
fn inline_block_with_padding_and_border() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <span style="display:inline-block; width:80px; height:40px; padding:10px; border-width:5px">padded</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind) -> Option<&'a LayoutBox<'a>> {
        if b.kind == kind { return Some(b); }
        for c in &b.children { if let Some(f) = find_kind(c, kind) { return Some(f); } }
        None
    }
    let ib = find_kind(&lt.root, BoxKind::InlineBlock).unwrap();
    assert_eq!(ib.padding.left, 10.0);
    assert_eq!(ib.border.left, 5.0);
    assert!((ib.content.width - 80.0).abs() < 1.0, "content width should be 80, got {}", ib.content.width);
}
