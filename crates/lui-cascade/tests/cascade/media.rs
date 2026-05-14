use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet};

#[test]
fn media_query_applies_when_matching() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("@media (min-width: 768px) { div { color: red; } }").unwrap()]);
  let interaction = InteractionState::default();
  let media = MediaContext {
    viewport_width: 1024.0,
    ..Default::default()
  };
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.color.is_some());
}

#[test]
fn media_query_skipped_when_not_matching() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("@media (min-width: 768px) { div { color: red; } }").unwrap()]);
  let interaction = InteractionState::default();
  let media = MediaContext {
    viewport_width: 500.0,
    ..Default::default()
  };
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.color.is_none());
}

#[test]
fn normal_and_media_rules_combine() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[
    parse_stylesheet("div { display: block; } @media (min-width: 768px) { div { color: red; } }").unwrap(),
  ]);
  let interaction = InteractionState::default();
  let media = MediaContext {
    viewport_width: 1024.0,
    ..Default::default()
  };
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.display.is_some());
  assert!(styled.children[0].style.color.is_some());
}
