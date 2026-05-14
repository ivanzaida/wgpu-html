use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet};

#[test]
fn before_pseudo_element_collected() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(r#"div::before { content: "hello"; color: red; }"#).unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  let div = &styled.children[0];
  assert!(div.before.is_some(), "::before should be collected");
  let before = div.before.as_ref().unwrap();
  assert_eq!(before.content_text.as_ref(), "hello");
  assert!(before.style.color.is_some());
}

#[test]
fn after_pseudo_element_collected() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(r#"div::after { content: "world"; }"#).unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  let div = &styled.children[0];
  assert!(div.after.is_some(), "::after should be collected");
  assert_eq!(div.after.as_ref().unwrap().content_text.as_ref(), "world");
}

#[test]
fn no_pseudo_without_content() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div::before { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  let div = &styled.children[0];
  assert!(div.before.is_none(), "::before without content should not be collected");
}

#[test]
fn content_none_produces_no_pseudo() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(r#"div::before { content: none; }"#).unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  assert!(styled.children[0].before.is_none());
}

#[test]
fn pseudo_inherits_from_parent_style() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(
    r#"
        div { color: red; }
        div::before { content: "x"; }
    "#,
  )
  .unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  let before = styled.children[0].before.as_ref().unwrap();
  assert!(before.style.color.is_some(), "::before should inherit color from div");
}

#[test]
fn pseudo_on_non_matching_element_not_collected() {
  let doc = parse("<span></span>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(r#"div::before { content: "x"; }"#).unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);

  assert!(styled.children[0].before.is_none());
}
