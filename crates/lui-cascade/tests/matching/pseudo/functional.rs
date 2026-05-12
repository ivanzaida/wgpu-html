use lui_cascade::matching::any_selector_matches;
use lui_css_parser::parse_selector_list;
use lui_html_parser::parse;
use crate::helpers::{root_ctx, child_ctx};

// ── :nth-child ──

#[test]
fn nth_child_odd() {
    let doc = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-child(odd)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 4), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 4), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 4), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[3], &child_ctx(3, 4), &[], None).is_none());
}

#[test]
fn nth_child_even() {
    let doc = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-child(even)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 4), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 4), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 4), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[3], &child_ctx(3, 4), &[], None).is_some());
}

#[test]
fn nth_child_specific_number() {
    let doc = parse("<ul><li>1</li><li>2</li><li>3</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-child(2)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 3), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 3), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 3), &[], None).is_none());
}

#[test]
fn nth_child_2n_plus_1() {
    let doc = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-child(2n+1)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 5), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 5), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 5), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[3], &child_ctx(3, 5), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[4], &child_ctx(4, 5), &[], None).is_some());
}

#[test]
fn nth_child_3n() {
    let doc = parse("<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-child(3n)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 6), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 6), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 6), &[], None).is_some());
    assert!(any_selector_matches(&sel, &ul.children[5], &child_ctx(5, 6), &[], None).is_some());
}

// ── :nth-last-child ──

#[test]
fn nth_last_child_1_matches_last() {
    let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
    let ul = &doc.root.children[0];
    let sel = parse_selector_list("li:nth-last-child(1)").unwrap();

    assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 3), &[], None).is_none());
    assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 3), &[], None).is_some());
}

// ── :not() ──

#[test]
fn not_rejects_matching_element() {
    let doc = parse(r#"<div class="a"></div>"#);
    let div = &doc.root.children[0];

    let sel = parse_selector_list(":not(.a)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_none());
}

#[test]
fn not_accepts_non_matching_element() {
    let doc = parse(r#"<div class="a"></div>"#);
    let div = &doc.root.children[0];

    let sel = parse_selector_list(":not(.b)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_some());
}

#[test]
fn not_with_tag() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];

    let sel = parse_selector_list(":not(span)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_some());

    let sel = parse_selector_list(":not(div)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_none());
}

// ── :is() / :where() ──

#[test]
fn is_matches_any_in_list() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];

    let sel = parse_selector_list(":is(div, span, p)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_some());
}

#[test]
fn is_rejects_when_none_match() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];

    let sel = parse_selector_list(":is(span, p, section)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_none());
}

#[test]
fn where_matches_same_as_is() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];

    let sel = parse_selector_list(":where(div, span)").unwrap();
    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_some());
}
