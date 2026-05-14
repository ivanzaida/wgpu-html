use lui_cascade::matching::{AncestorEntry, matches_selector};
use lui_parse::{parse, parse_selector_list};

use crate::helpers::child_ctx;

#[test]
fn matches_direct_child_as_descendant() {
  let doc = parse(r#"<div class="outer"><span></span></div>"#);
  let outer = &doc.root.children[0];
  let span = &outer.children[0];

  let sel = parse_selector_list(".outer span").unwrap();
  let ancestors = [AncestorEntry {
    node: outer,
    ctx: child_ctx(0, 1),
  }];
  assert!(matches_selector(
    &sel.0[0],
    span,
    &child_ctx(0, 1),
    &ancestors,
    Some(outer)
  ));
}

#[test]
fn matches_grandchild_as_descendant() {
  let doc = parse(r#"<div class="outer"><p><span></span></p></div>"#);
  let outer = &doc.root.children[0];
  let p = &outer.children[0];
  let span = &p.children[0];

  let sel = parse_selector_list(".outer span").unwrap();
  let ancestors = [
    AncestorEntry {
      node: p,
      ctx: child_ctx(0, 1),
    },
    AncestorEntry {
      node: outer,
      ctx: child_ctx(0, 1),
    },
  ];
  assert!(matches_selector(&sel.0[0], span, &child_ctx(0, 1), &ancestors, Some(p)));
}

#[test]
fn rejects_when_ancestor_absent() {
  let doc = parse(r#"<div><span></span></div>"#);
  let div = &doc.root.children[0];
  let span = &div.children[0];

  let sel = parse_selector_list(".missing span").unwrap();
  let ancestors = [AncestorEntry {
    node: div,
    ctx: child_ctx(0, 1),
  }];
  assert!(!matches_selector(
    &sel.0[0],
    span,
    &child_ctx(0, 1),
    &ancestors,
    Some(div)
  ));
}

#[test]
fn multi_level_descendant() {
  let doc = parse(r#"<section class="s"><div class="d"><p><span></span></p></div></section>"#);
  let section = &doc.root.children[0];
  let div = &section.children[0];
  let p = &div.children[0];
  let span = &p.children[0];

  let sel = parse_selector_list(".s .d span").unwrap();
  let ancestors = [
    AncestorEntry {
      node: p,
      ctx: child_ctx(0, 1),
    },
    AncestorEntry {
      node: div,
      ctx: child_ctx(0, 1),
    },
    AncestorEntry {
      node: section,
      ctx: child_ctx(0, 1),
    },
  ];
  assert!(matches_selector(&sel.0[0], span, &child_ctx(0, 1), &ancestors, Some(p)));
}
