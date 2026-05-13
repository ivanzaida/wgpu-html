use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_parse::{parse_stylesheet, parse_value, parse};

fn val(css: &str) -> lui_core::CssValue { parse_value(css).unwrap() }

#[test]
fn id_beats_class() {
    let doc = parse(r#"<div id="x" class="c"></div>"#);
    let sheet = parse_stylesheet(".c { color: red; } #x { color: blue; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn class_beats_tag() {
    let doc = parse(r#"<div class="c"></div>"#);
    let sheet = parse_stylesheet("div { color: red; } .c { color: blue; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn source_order_breaks_ties() {
    let doc = parse(r#"<div class="a b"></div>"#);
    let sheet = parse_stylesheet(".a { color: red; } .b { color: blue; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn higher_specificity_wins_regardless_of_order() {
    let doc = parse(r#"<div id="x" class="c"></div>"#);
    let sheet = parse_stylesheet("#x { color: red; } .c { color: blue; }").unwrap();
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[sheet]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].style.color.unwrap(), val("red"));
}
