use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet};

#[test]
fn margin_shorthand_expands_to_longhands() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { margin: 10px; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let div = &styled.children[0];
  assert!(div.style.margin_top.is_some());
  assert!(div.style.margin_right.is_some());
  assert!(div.style.margin_bottom.is_some());
  assert!(div.style.margin_left.is_some());
}

#[test]
fn padding_shorthand_expands() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { padding: 5px; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let div = &styled.children[0];
  assert!(div.style.padding_top.is_some());
  assert!(div.style.padding_right.is_some());
  assert!(div.style.padding_bottom.is_some());
  assert!(div.style.padding_left.is_some());
}

#[test]
fn longhand_after_shorthand_overrides() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { margin: 10px; margin-left: 20px; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let div = &styled.children[0];
  assert!(div.style.margin_left.is_some());
  assert!(div.style.margin_top.is_some());
}
