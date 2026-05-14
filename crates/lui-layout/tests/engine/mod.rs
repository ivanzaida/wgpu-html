use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_layout::{BoxKind, engine::layout_tree};
use lui_parse::parse;

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
  let has_non_zero = lt
    .rects
    .iter()
    .any(|(_node, rect)| rect.width > 0.0 || rect.height > 0.0);
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
  assert_eq!(
    lt.root.content.width, 0.0,
    "content width should be 0 for zero viewport"
  );
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
    assert!(
      curr.content.y >= prev.content.y,
      "child {i} y={} should be >= previous child y={}",
      curr.content.y,
      prev.content.y
    );
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
