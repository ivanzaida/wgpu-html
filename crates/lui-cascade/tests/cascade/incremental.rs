use lui_cascade::cascade::{CascadeContext, InteractionState};
use lui_cascade::media::MediaContext;
use lui_css_parser::{parse_stylesheet, parse_value};
use lui_html_parser::parse;

fn val(css: &str) -> lui_css_parser::CssValue { parse_value(css).unwrap() }

#[test]
fn clean_subtree_preserves_styles() {
    let doc = parse(r#"<div class="a"><span class="b"></span></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(".a { color: red; } .b { color: blue; }").unwrap()]);

    let prev = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert_eq!(*prev.children[0].style.color.unwrap(), val("red"));
    assert_eq!(*prev.children[0].children[0].style.color.unwrap(), val("blue"));

    // No dirty paths → everything copied from prev
    let next = ctx.cascade_dirty(
        &doc.root, &prev, &[],
        &MediaContext::default(), &InteractionState::default(), None,
    );
    assert_eq!(*next.children[0].style.color.unwrap(), val("red"));
    assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("blue"));
}

#[test]
fn dirty_subtree_recascaded() {
    let doc = parse(r#"<div><p class="x"></p><span></span></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(".x { color: red; } span { color: green; }").unwrap()]);

    let prev = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert_eq!(*prev.children[0].children[0].style.color.unwrap(), val("red"));
    assert_eq!(*prev.children[0].children[1].style.color.unwrap(), val("green"));

    // Mark only the first child [0,0] as dirty
    let next = ctx.cascade_dirty(
        &doc.root, &prev, &[vec![0, 0]],
        &MediaContext::default(), &InteractionState::default(), None,
    );

    // Dirty child recascaded
    assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("red"));
    // Clean sibling preserved
    assert_eq!(*next.children[0].children[1].style.color.unwrap(), val("green"));
}

#[test]
fn dirty_parent_recascades_children() {
    let doc = parse(r#"<div class="parent"><span></span></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet(".parent { color: red; }").unwrap()]);

    let prev = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());

    // Mark parent [0] as dirty — its children should be recascaded too
    let next = ctx.cascade_dirty(
        &doc.root, &prev, &[vec![0]],
        &MediaContext::default(), &InteractionState::default(), None,
    );

    // span inherits color from recascaded parent
    assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("red"));
}

#[test]
fn multiple_dirty_paths() {
    let doc = parse(r#"<div><p></p><span></span><em></em></div>"#);
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("p { color: red; } span { color: blue; } em { color: green; }").unwrap()]);

    let prev = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());

    // Mark first and third children as dirty
    let next = ctx.cascade_dirty(
        &doc.root, &prev, &[vec![0, 0], vec![0, 2]],
        &MediaContext::default(), &InteractionState::default(), None,
    );

    assert_eq!(*next.children[0].children[0].style.color.unwrap(), val("red"));
    assert_eq!(*next.children[0].children[1].style.color.unwrap(), val("blue"));
    assert_eq!(*next.children[0].children[2].style.color.unwrap(), val("green"));
}

#[test]
fn full_cascade_after_incremental() {
    let doc = parse("<div></div>");
    let mut ctx = CascadeContext::new();
    ctx.set_stylesheets(&[parse_stylesheet("div { color: red; }").unwrap()]);

    let prev = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    let _inc = ctx.cascade_dirty(
        &doc.root, &prev, &[],
        &MediaContext::default(), &InteractionState::default(), None,
    );

    // Full cascade still works after incremental
    let full = ctx.cascade(&doc.root, &MediaContext::default(), &InteractionState::default());
    assert_eq!(*full.children[0].style.color.unwrap(), val("red"));
}
