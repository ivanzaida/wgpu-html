use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_parse::{parse_stylesheet, parse};

fn cascade_check(html: &str, css: &str) -> bool {
    let doc = parse(html);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(css).unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    styled.children[0].style.color.is_some()
}

#[test]
fn supports_known_property_applies() {
    assert!(cascade_check("<div></div>", "@supports (display: grid) { div { color: red; } }"));
}

#[test]
fn supports_unknown_property_skipped() {
    assert!(!cascade_check("<div></div>", "@supports (foo-bar: baz) { div { color: red; } }"));
}

#[test]
fn supports_not_unknown_applies() {
    assert!(cascade_check("<div></div>", "@supports not (foo-bar: baz) { div { color: red; } }"));
}

#[test]
fn supports_not_known_skipped() {
    assert!(!cascade_check("<div></div>", "@supports not (display: grid) { div { color: red; } }"));
}

#[test]
fn supports_and_all_known_applies() {
    assert!(cascade_check("<div></div>", "@supports (display: grid) and (color: red) { div { color: red; } }"));
}

#[test]
fn supports_and_one_unknown_skipped() {
    assert!(!cascade_check("<div></div>", "@supports (display: grid) and (fake-prop: x) { div { color: red; } }"));
}

#[test]
fn supports_or_mixed_applies() {
    assert!(cascade_check("<div></div>", "@supports (fake-prop: x) or (display: block) { div { color: red; } }"));
}

#[test]
fn supports_or_all_unknown_skipped() {
    assert!(!cascade_check("<div></div>", "@supports (fake-prop: x) or (bogus: y) { div { color: red; } }"));
}

#[test]
fn supports_selector_applies() {
    let doc = parse(r#"<div class="foo"><span class="bar"></span></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("@supports selector(.foo > .bar) { .bar { color: red; } }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert!(styled.children[0].children[0].style.color.is_some());
}

#[test]
fn supports_not_selector_skipped() {
    let doc = parse(r#"<div class="foo"></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("@supports not selector(.foo) { .foo { color: red; } }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert!(styled.children[0].style.color.is_none());
}

#[test]
fn normal_and_supports_rules_combine() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { display: block; } @supports (display: grid) { div { color: red; } }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert!(styled.children[0].style.display.is_some());
    assert!(styled.children[0].style.color.is_some());
}

#[test]
fn normal_outside_and_failed_supports_inside() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { display: block; } @supports (fake: x) { div { color: red; } }").unwrap()]);
    let media = MediaContext::default(); let interaction = InteractionState::default(); let styled = ctx.cascade(&doc.root, &media, &interaction);
    assert!(styled.children[0].style.display.is_some());
    assert!(styled.children[0].style.color.is_none());
}
