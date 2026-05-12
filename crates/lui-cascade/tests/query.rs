use lui_cascade::query::{query_selector_all, query_selector, matches, closest};
use lui_cascade::matching::AncestorEntry;
use lui_html_parser::{HtmlElement, parse};

// ── query_selector_all ──

#[test]
fn select_all_by_tag() {
    let doc = parse("<div><p>a</p><p>b</p><span>c</span><p>d</p></div>");
    let div = &doc.root.children[0];
    let results = query_selector_all(div, "p");
    assert_eq!(results.len(), 3);
    for r in &results {
        assert_eq!(r.element, HtmlElement::P);
    }
}

#[test]
fn select_all_by_class() {
    let doc = parse(r#"<ul><li class="a">1</li><li>2</li><li class="a">3</li></ul>"#);
    let ul = &doc.root.children[0];
    let results = query_selector_all(ul, ".a");
    assert_eq!(results.len(), 2);
}

#[test]
fn select_all_nested() {
    let doc = parse("<div><section><p>deep</p></section><p>shallow</p></div>");
    let div = &doc.root.children[0];
    let results = query_selector_all(div, "p");
    assert_eq!(results.len(), 2);
}

#[test]
fn select_all_with_descendant_combinator() {
    let doc = parse(r#"<div class="a"><p>yes</p></div><div><p>no</p></div>"#);
    let results = query_selector_all(&doc.root, ".a p");
    assert_eq!(results.len(), 1);
}

#[test]
fn select_all_returns_empty_on_no_match() {
    let doc = parse("<div><span>x</span></div>");
    let div = &doc.root.children[0];
    let results = query_selector_all(div, "p");
    assert!(results.is_empty());
}

#[test]
fn select_all_returns_empty_on_invalid_selector() {
    let doc = parse("<div></div>");
    let results = query_selector_all(&doc.root, "[[[invalid");
    assert!(results.is_empty());
}

#[test]
fn select_all_does_not_include_root() {
    let doc = parse(r#"<div class="x"><span></span></div>"#);
    let div = &doc.root.children[0];
    let results = query_selector_all(div, ".x");
    assert!(results.is_empty());
}

// ── query_selector ──

#[test]
fn select_first_match() {
    let doc = parse("<div><span>1</span><span>2</span></div>");
    let div = &doc.root.children[0];
    let result = query_selector(div, "span");
    assert!(result.is_some());
    assert_eq!(result.unwrap().children.len(), 1);
}

#[test]
fn select_returns_none_on_no_match() {
    let doc = parse("<div><span>x</span></div>");
    let div = &doc.root.children[0];
    assert!(query_selector(div, "p").is_none());
}

#[test]
fn select_returns_none_on_invalid_selector() {
    let doc = parse("<div></div>");
    assert!(query_selector(&doc.root, "!!!").is_none());
}

#[test]
fn select_finds_deeply_nested() {
    let doc = parse("<div><section><article><p>deep</p></article></section></div>");
    let div = &doc.root.children[0];
    let result = query_selector(div, "p");
    assert!(result.is_some());
    assert_eq!(result.unwrap().element, HtmlElement::P);
}

// ── matches ──

#[test]
fn matches_tag() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    assert!(matches(div, "div"));
    assert!(!matches(div, "span"));
}

#[test]
fn matches_class() {
    let doc = parse(r#"<div class="foo bar"></div>"#);
    let div = &doc.root.children[0];
    assert!(matches(div, ".foo"));
    assert!(matches(div, ".bar"));
    assert!(!matches(div, ".baz"));
}

#[test]
fn matches_compound() {
    let doc = parse(r#"<div id="x" class="c"></div>"#);
    let div = &doc.root.children[0];
    assert!(matches(div, "div.c#x"));
    assert!(!matches(div, "span.c#x"));
}

#[test]
fn matches_returns_false_on_invalid_selector() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    assert!(!matches(div, "!!!"));
}

// ── closest ──

#[test]
fn closest_matches_self() {
    let doc = parse(r#"<div class="target"></div>"#);
    let div = &doc.root.children[0];
    let result = closest(div, ".target", &[], None);
    assert!(result.is_some());
    assert!(std::ptr::eq(result.unwrap(), div));
}

#[test]
fn closest_finds_ancestor() {
    let doc = parse(r#"<div class="outer"><p><span></span></p></div>"#);
    let outer = &doc.root.children[0];
    let p = &outer.children[0];
    let span = &p.children[0];

    let ancestors = [
        AncestorEntry { node: p, ctx: Default::default() },
        AncestorEntry { node: outer, ctx: Default::default() },
    ];
    let result = closest(span, ".outer", &ancestors, Some(p));
    assert!(result.is_some());
    assert!(std::ptr::eq(result.unwrap(), outer));
}

#[test]
fn closest_returns_none_when_no_match() {
    let doc = parse("<div><span></span></div>");
    let div = &doc.root.children[0];
    let span = &div.children[0];

    let ancestors = [
        AncestorEntry { node: div, ctx: Default::default() },
    ];
    let result = closest(span, ".missing", &ancestors, Some(div));
    assert!(result.is_none());
}

#[test]
fn closest_prefers_nearest_ancestor() {
    let doc = parse(r#"<div class="x"><div class="x"><span></span></div></div>"#);
    let outer = &doc.root.children[0];
    let inner = &outer.children[0];
    let span = &inner.children[0];

    let ancestors = [
        AncestorEntry { node: inner, ctx: Default::default() },
        AncestorEntry { node: outer, ctx: Default::default() },
    ];
    let result = closest(span, ".x", &ancestors, Some(inner));
    assert!(result.is_some());
    assert!(std::ptr::eq(result.unwrap(), inner));
}
