use lui_cascade::matching::{MatchContext, any_selector_matches};
use lui_parse::{parse, parse_selector_list};

use crate::helpers::{child_ctx, root_ctx};

#[test]
fn first_child_matches_first() {
  let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
  let ul = &doc.root.children[0];
  let sel = parse_selector_list("li:first-child").unwrap();

  assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 3), &[], None).is_some());
}

#[test]
fn first_child_rejects_non_first() {
  let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
  let ul = &doc.root.children[0];
  let sel = parse_selector_list("li:first-child").unwrap();

  assert!(any_selector_matches(&sel, &ul.children[1], &child_ctx(1, 3), &[], None).is_none());
  assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 3), &[], None).is_none());
}

#[test]
fn last_child_matches_last() {
  let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
  let ul = &doc.root.children[0];
  let sel = parse_selector_list("li:last-child").unwrap();

  assert!(any_selector_matches(&sel, &ul.children[2], &child_ctx(2, 3), &[], None).is_some());
}

#[test]
fn last_child_rejects_non_last() {
  let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
  let ul = &doc.root.children[0];
  let sel = parse_selector_list("li:last-child").unwrap();

  assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 3), &[], None).is_none());
}

#[test]
fn only_child_matches_single_child() {
  let doc = parse("<ul><li>a</li></ul>");
  let ul = &doc.root.children[0];
  let sel = parse_selector_list("li:only-child").unwrap();

  assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 1), &[], None).is_some());
}

#[test]
fn only_child_rejects_when_siblings_exist() {
  let doc = parse("<ul><li>a</li><li>b</li></ul>");
  let ul = &doc.root.children[0];
  let sel = parse_selector_list("li:only-child").unwrap();

  assert!(any_selector_matches(&sel, &ul.children[0], &child_ctx(0, 2), &[], None).is_none());
}

#[test]
fn root_matches_root_element() {
  let doc = parse("<div></div>");
  let div = &doc.root.children[0];
  let sel = parse_selector_list(":root").unwrap();

  let ctx = MatchContext {
    is_root: true,
    ..Default::default()
  };
  assert!(any_selector_matches(&sel, div, &ctx, &[], None).is_some());
}

#[test]
fn root_rejects_non_root() {
  let doc = parse("<div></div>");
  let div = &doc.root.children[0];
  let sel = parse_selector_list(":root").unwrap();

  assert!(any_selector_matches(&sel, div, &child_ctx(0, 1), &[], None).is_none());
}

#[test]
fn empty_matches_childless_element() {
  let doc = parse("<div></div>");
  let div = &doc.root.children[0];
  let sel = parse_selector_list(":empty").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_some());
}

#[test]
fn empty_rejects_element_with_children() {
  let doc = parse("<div><span></span></div>");
  let div = &doc.root.children[0];
  let sel = parse_selector_list(":empty").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_none());
}
