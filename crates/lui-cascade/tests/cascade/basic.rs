use lui_cascade::{
  cascade::{CascadeContext, InteractionState},
  media::MediaContext,
};
use lui_parse::{parse, parse_stylesheet};

#[test]
fn single_rule_applies_to_matching_element() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet("div { display: block; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.display.is_some());
}

#[test]
fn non_matching_rule_does_not_apply() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet("span { color: red; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.color.is_none());
}

#[test]
fn non_matching_hr_rule_does_not_apply_border_width() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet("hr { border-width: 1px; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.border_top_width.is_none());
}

#[test]
fn ua_stylesheet_does_not_apply_border_width_to_plain_div() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet(include_str!("../../../lui/ua/ua_whatwg.css")).unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(
    styled.children[0].style.border_top_width.is_none(),
    "plain div picked up border-top-width={:?}",
    styled.children[0].style.border_top_width
  );
}

#[test]
fn pseudo_element_rule_does_not_apply_to_normal_elements() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet("::picker(select) { border: 1px solid; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.border_top_width.is_none());
}

#[test]
fn class_selector_applies() {
  let doc = parse(r#"<div class="card"></div>"#);
  let sheet = parse_stylesheet(".card { color: red; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.color.is_some());
}

#[test]
fn id_selector_applies() {
  let doc = parse(r#"<div id="main"></div>"#);
  let sheet = parse_stylesheet("#main { width: 100px; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.width.is_some());
}

#[test]
fn descendant_selector_applies() {
  let doc = parse(r#"<div class="outer"><p></p></div>"#);
  let sheet = parse_stylesheet(".outer p { color: blue; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].children[0].style.color.is_some());
}

#[test]
fn empty_stylesheet_produces_default_styles() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet("").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.display.is_none());
}

#[test]
fn multiple_stylesheets_applied_in_order() {
  let doc = parse("<div></div>");
  let s1 = parse_stylesheet("div { color: red; }").unwrap();
  let s2 = parse_stylesheet("div { color: blue; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[s1, s2]);
  let media = MediaContext::default();
  let interaction = InteractionState::default();
  let styled = ctx.cascade(&doc.root, &media, &interaction);
  assert!(styled.children[0].style.color.is_some());
}

#[test]
fn cascade_can_be_called_again_after_drop() {
  let doc = parse("<div></div>");
  let sheet = parse_stylesheet("div { color: red; }").unwrap();
  let mut ctx = CascadeContext::new();
  ctx.set_stylesheets(&[sheet]);

  let color1 = {
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    styled.children[0].style.color.is_some()
  };

  let color2 = {
    let media = MediaContext::default();
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    styled.children[0].style.color.is_some()
  };

  assert!(color1);
  assert!(color2);
}
