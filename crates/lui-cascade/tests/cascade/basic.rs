use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_css_parser::parse_stylesheet;
use lui_html_parser::parse;

#[test]
fn single_rule_applies_to_matching_element() {
    let doc = parse("<div></div>");
    let sheet = parse_stylesheet("div { display: block; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].style.display.is_some());
}

#[test]
fn non_matching_rule_does_not_apply() {
    let doc = parse("<div></div>");
    let sheet = parse_stylesheet("span { color: red; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].style.color.is_none());
}

#[test]
fn class_selector_applies() {
    let doc = parse(r#"<div class="card"></div>"#);
    let sheet = parse_stylesheet(".card { color: red; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].style.color.is_some());
}

#[test]
fn id_selector_applies() {
    let doc = parse(r#"<div id="main"></div>"#);
    let sheet = parse_stylesheet("#main { width: 100px; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].style.width.is_some());
}

#[test]
fn descendant_selector_applies() {
    let doc = parse(r#"<div class="outer"><p></p></div>"#);
    let sheet = parse_stylesheet(".outer p { color: blue; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].children[0].style.color.is_some());
}

#[test]
fn empty_stylesheet_produces_default_styles() {
    let doc = parse("<div></div>");
    let sheet = parse_stylesheet("").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].style.display.is_none());
}

#[test]
fn multiple_stylesheets_applied_in_order() {
    let doc = parse("<div></div>");
    let s1 = parse_stylesheet("div { color: red; }").unwrap();
    let s2 = parse_stylesheet("div { color: blue; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[s1, s2]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert!(styled.children[0].style.color.is_some());
}

#[test]
fn cascade_can_be_called_again_after_drop() {
    let doc = parse("<div></div>");
    let sheet = parse_stylesheet("div { color: red; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);

    let color1 = {
        let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
        styled.children[0].style.color.is_some()
    };

    let color2 = {
        let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
        styled.children[0].style.color.is_some()
    };

    assert!(color1);
    assert!(color2);
}
