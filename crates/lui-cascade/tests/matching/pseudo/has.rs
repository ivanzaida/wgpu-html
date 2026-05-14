use lui_cascade::matching::any_selector_matches;
use lui_parse::{parse, parse_selector_list};

use crate::helpers::root_ctx;

#[test]
fn has_descendant_matches() {
  let doc = parse(r#"<div class="parent"><p><span class="target"></span></p></div>"#);
  let div = &doc.root.children[0];
  let sel = parse_selector_list("div:has(.target)").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_some());
}

#[test]
fn has_descendant_rejects_when_absent() {
  let doc = parse(r#"<div class="parent"><p>text</p></div>"#);
  let div = &doc.root.children[0];
  let sel = parse_selector_list("div:has(.target)").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_none());
}

#[test]
fn has_direct_child() {
  let doc = parse(r#"<div><span class="child"></span></div>"#);
  let div = &doc.root.children[0];
  let sel = parse_selector_list("div:has(> .child)").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_some());
}

#[test]
fn has_direct_child_rejects_grandchild() {
  let doc = parse(r#"<div><p><span class="deep"></span></p></div>"#);
  let div = &doc.root.children[0];
  let sel = parse_selector_list("div:has(> .deep)").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_none());
}

#[test]
fn has_with_tag_selector() {
  let doc = parse("<div><img></div>");
  let div = &doc.root.children[0];
  let sel = parse_selector_list("div:has(img)").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_some());
}

#[test]
fn has_nested_descendant() {
  let doc = parse("<section><div><ul><li></li></ul></div></section>");
  let section = &doc.root.children[0];
  let sel = parse_selector_list("section:has(li)").unwrap();

  assert!(any_selector_matches(&sel, section, &root_ctx(), &[], Some(&doc.root)).is_some());
}
