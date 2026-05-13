use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_parse::{parse_stylesheet, parse_value, parse};

fn val(css: &str) -> lui_core::CssValue { parse_value(css).unwrap() }

#[test]
fn custom_property_resolved_in_same_element() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { --color: red; color: var(--color); }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].style.color.unwrap(), val("red"));
}

#[test]
fn custom_property_inherited_and_resolved() {
    let doc = parse("<div><span></span></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { --theme: blue; } span { color: var(--theme); }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn var_fallback_used_when_undefined() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { color: var(--missing, green); }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert_eq!(*styled.children[0].style.color.unwrap(), val("green"));
}
