use bumpalo::Bump;
use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_parse::parse;
use lui_layout::{BoxKind, box_gen::build_box, engine::layout_tree};
use crate::helpers::*;

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
    let bump = Bump::new();
    let lb = build_box(text_child, &bump);
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
    let bump = Bump::new();
    let lb = build_box(div, &bump);
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
    let bump = Bump::new();
    let lb = build_box(div, &bump);
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
    let bump = Bump::new();
    let lb = build_box(div, &bump);

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
// 18. Box generation tests
// ============================================================================

#[test]
fn box_gen_display_flex_produces_flex_container() {
    let (doc, ctx) = flex_lt(r#"<div style="display:flex"><div>child</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(flex.kind, BoxKind::FlexContainer);
}

#[test]
fn box_gen_display_grid_produces_grid_container() {
    let (doc, ctx) = flex_lt(r#"<div style="display:grid"><div>child</div></div>"#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let grid = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(grid.kind, BoxKind::GridContainer);
}

#[test]
fn display_none_element_still_in_tree() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <div style="display:none; height:100px">hidden</div>
            <div style="height:50px">visible</div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // display:none should not contribute to height
    assert!(container.content.height < 60.0,
        "display:none should not add height, got {}", container.content.height);
}
