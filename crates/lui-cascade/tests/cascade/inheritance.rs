use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet, parse_value};

fn val(css: &str) -> lui_core::CssValue {
  parse_value(css).unwrap()
}

#[test]
fn color_inherits_to_child() {
  let doc = parse("<div><span></span></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*styled.children[0].children[0].style.color.unwrap(), val("red"));
}

#[test]
fn display_does_not_inherit() {
  let doc = parse("<div><span></span></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { display: flex; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].children[0].style.display.is_none());
}

#[test]
fn child_value_overrides_inherited() {
  let doc = parse("<div><span></span></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { color: red; } span { color: blue; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*styled.children[0].children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn deep_inheritance() {
  let doc = parse("<div><section><article><p></p></article></section></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { font-family: Arial; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(
    styled.children[0].children[0].children[0].children[0]
      .style
      .font_family
      .is_some()
  );
}

#[test]
fn text_node_inherits_from_parent() {
  let doc = parse("<p>hello</p>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("p { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  let text = &styled.children[0].children[0];
  assert!(text.node.element().is_text());
  assert_eq!(*text.style.color.unwrap(), val("red"));
}
