use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet, parse_value};

fn val(css: &str) -> lui_core::CssValue {
  parse_value(css).unwrap()
}

#[test]
fn clean_subtree_preserves_styles() {
  let doc = parse(r#"<div class="a"><span class="b"></span></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(".a { color: red; } .b { color: blue; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  let prev = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*prev.children[0].style.color.unwrap(), val("red"));
  assert_eq!(*prev.children[0].children[0].style.color.unwrap(), val("blue"));

  let next = ctx.cascade_dirty(&doc.root, &prev, &[], &media, &interaction);
  assert_eq!(*next.children[0].style.color.unwrap(), val("red"));
  assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn dirty_subtree_recascaded() {
  let doc = parse(r#"<div><p class="x"></p><span></span></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(".x { color: red; } span { color: green; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  let prev = ctx.cascade(&doc.root, &media, &interaction);
  let next = ctx.cascade_dirty(&doc.root, &prev, &[vec![0, 0]], &media, &interaction);

  assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("red"));
  assert_eq!(*next.children[0].children[1].style.color.unwrap(), val("green"));
}

#[test]
fn dirty_parent_recascades_children() {
  let doc = parse(r#"<div class="parent"><span></span></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet(".parent { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  let prev = ctx.cascade(&doc.root, &media, &interaction);
  let next = ctx.cascade_dirty(&doc.root, &prev, &[vec![0]], &media, &interaction);

  assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("red"));
}

#[test]
fn multiple_dirty_paths() {
  let doc = parse(r#"<div><p></p><span></span><em></em></div>"#);
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("p { color: red; } span { color: blue; } em { color: green; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  let prev = ctx.cascade(&doc.root, &media, &interaction);
  let next = ctx.cascade_dirty(&doc.root, &prev, &[vec![0, 0], vec![0, 2]], &media, &interaction);

  assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("red"));
  assert_eq!(*next.children[0].children[1].style.color.unwrap(), val("blue"));
  assert_eq!(*next.children[0].children[2].style.color.unwrap(), val("green"));
}

#[test]
fn full_cascade_after_incremental() {
  let doc = parse("<div></div>");
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[parse_stylesheet("div { color: red; }").unwrap()]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();

  let prev = ctx.cascade(&doc.root, &media, &interaction);
  let _inc = ctx.cascade_dirty(&doc.root, &prev, &[], &media, &interaction);

  let full = ctx.cascade(&doc.root, &media, &interaction);
  assert_eq!(*full.children[0].style.color.unwrap(), val("red"));
}
