use lui_cascade::matching::{AncestorEntry, any_selector_matches};
use lui_parse::{parse, parse_selector_list};

use crate::helpers::{child_ctx, root_ctx};

#[test]
fn not_with_descendant_selector() {
  let doc = parse(r#"<div class="outer"><span class="inner"></span></div>"#);
  let outer = &doc.root.children[0];
  let inner = &outer.children[0];

  let sel = parse_selector_list(":not(.outer span)").unwrap();
  let ancestors = [AncestorEntry {
    node: outer,
    ctx: child_ctx(0, 1),
  }];

  // inner IS .outer span → :not(.outer span) should reject
  assert!(any_selector_matches(&sel, inner, &child_ctx(0, 1), &ancestors, Some(outer)).is_none());

  // outer is NOT .outer span → :not(.outer span) should match
  assert!(any_selector_matches(&sel, outer, &root_ctx(), &[], Some(&doc.root)).is_some());
}

#[test]
fn is_with_multiple_complex_selectors() {
  let doc = parse(r#"<div class="a"><p class="b"></p></div>"#);
  let div = &doc.root.children[0];
  let p = &div.children[0];

  let sel = parse_selector_list(":is(.a > .b)").unwrap();
  let ancestors = [AncestorEntry {
    node: div,
    ctx: child_ctx(0, 1),
  }];

  assert!(any_selector_matches(&sel, p, &child_ctx(0, 1), &ancestors, Some(div)).is_some());
}

#[test]
fn is_with_comma_separated_complex() {
  let doc = parse(r#"<div class="x"><span></span></div>"#);
  let div = &doc.root.children[0];
  let span = &div.children[0];

  // span matches the second alternative (.x > span)
  let sel = parse_selector_list(":is(.nope > p, .x > span)").unwrap();
  let ancestors = [AncestorEntry {
    node: div,
    ctx: child_ctx(0, 1),
  }];

  assert!(any_selector_matches(&sel, span, &child_ctx(0, 1), &ancestors, Some(div)).is_some());
}

#[test]
fn not_with_child_combinator() {
  let doc = parse(r#"<div class="parent"><p></p></div>"#);
  let parent = &doc.root.children[0];
  let p = &parent.children[0];

  let sel = parse_selector_list(":not(.parent > p)").unwrap();
  let ancestors = [AncestorEntry {
    node: parent,
    ctx: child_ctx(0, 1),
  }];

  // p IS .parent > p → :not should reject
  assert!(any_selector_matches(&sel, p, &child_ctx(0, 1), &ancestors, Some(parent)).is_none());
}
