use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_css_parser::{parse_stylesheet, parse_value};
use lui_html_parser::parse;

fn val(css: &str) -> lui_css_parser::CssValue { parse_value(css).unwrap() }

#[test]
fn important_beats_higher_specificity() {
    let doc = parse(r#"<div id="x" class="c"></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("#x { color: red; } .c { color: blue !important; }").unwrap()]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn important_beats_later_normal() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { color: red !important; } div { color: blue; }").unwrap()]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert_eq!(*styled.children[0].style.color.unwrap(), val("red"));
}

#[test]
fn later_important_beats_earlier_important() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { color: red !important; } div { color: blue !important; }").unwrap()]);
    let styled = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}
