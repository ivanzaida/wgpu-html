use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet, parse_value};

fn val(css: &str) -> lui_core::CssValue {
  parse_value(css).unwrap()
}

#[test]
fn inline_beats_class_selector() {
  let doc = parse(r#"<div class="c" style="color: blue"></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(".c { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn inline_beats_id_selector() {
  let doc = parse(r#"<div id="x" style="color: blue"></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("#x { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn important_in_stylesheet_beats_normal_inline() {
  let doc = parse(r#"<div class="c" style="color: blue"></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(".c { color: red !important; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*styled.children[0].style.color.unwrap(), val("red"));
}

#[test]
fn important_inline_beats_important_stylesheet() {
  let doc = parse(r#"<div class="c" style="color: blue !important"></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(".c { color: red !important; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}
