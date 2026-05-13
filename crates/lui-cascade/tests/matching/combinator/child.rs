use lui_cascade::matching::{AncestorEntry, matches_selector};
use lui_parse::{parse_selector_list, parse};
use crate::helpers::child_ctx;

#[test]
fn matches_direct_child() {
    let doc = parse(r#"<div class="parent"><span></span></div>"#);
    let parent = &doc.root.children[0];
    let span = &parent.children[0];

    let sel = parse_selector_list(".parent > span").unwrap();
    let ancestors = [AncestorEntry { node: parent, ctx: child_ctx(0, 1) }];
    assert!(matches_selector(&sel.0[0], span, &child_ctx(0, 1), &ancestors, Some(parent)));
}

#[test]
fn rejects_grandchild() {
    let doc = parse(r#"<div class="gp"><div><span></span></div></div>"#);
    let gp = &doc.root.children[0];
    let mid = &gp.children[0];
    let span = &mid.children[0];

    let sel = parse_selector_list(".gp > span").unwrap();
    let ancestors = [
        AncestorEntry { node: mid, ctx: child_ctx(0, 1) },
        AncestorEntry { node: gp, ctx: child_ctx(0, 1) },
    ];
    assert!(!matches_selector(&sel.0[0], span, &child_ctx(0, 1), &ancestors, Some(mid)));
}

#[test]
fn chained_child_combinators() {
    let doc = parse(r#"<div class="a"><div class="b"><span></span></div></div>"#);
    let a = &doc.root.children[0];
    let b = &a.children[0];
    let span = &b.children[0];

    let sel = parse_selector_list(".a > .b > span").unwrap();
    let ancestors = [
        AncestorEntry { node: b, ctx: child_ctx(0, 1) },
        AncestorEntry { node: a, ctx: child_ctx(0, 1) },
    ];
    assert!(matches_selector(&sel.0[0], span, &child_ctx(0, 1), &ancestors, Some(b)));
}

#[test]
fn mixed_descendant_and_child() {
    let doc = parse(r#"<div class="root"><section><div class="inner"><p></p></div></section></div>"#);
    let root = &doc.root.children[0];
    let section = &root.children[0];
    let inner = &section.children[0];
    let p = &inner.children[0];

    let sel = parse_selector_list(".root .inner > p").unwrap();
    let ancestors = [
        AncestorEntry { node: inner, ctx: child_ctx(0, 1) },
        AncestorEntry { node: section, ctx: child_ctx(0, 1) },
        AncestorEntry { node: root, ctx: child_ctx(0, 1) },
    ];
    assert!(matches_selector(&sel.0[0], p, &child_ctx(0, 1), &ancestors, Some(inner)));
}
