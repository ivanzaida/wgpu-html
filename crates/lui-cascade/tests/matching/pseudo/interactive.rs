use lui_cascade::matching::{MatchContext, any_selector_matches};
use lui_parse::{parse, parse_selector_list};

use crate::helpers::root_ctx;

#[test]
fn hover_matches_when_hovered() {
  let doc = parse("<div></div>");
  let div = &doc.root.children()[0];
  let sel = parse_selector_list("div:hover").unwrap();

  let ctx = MatchContext {
    is_hover: true,
    ..root_ctx()
  };
  assert!(any_selector_matches(&sel, div, &ctx, &[], None).is_some());
}

#[test]
fn hover_rejects_when_not_hovered() {
  let doc = parse("<div></div>");
  let div = &doc.root.children()[0];
  let sel = parse_selector_list("div:hover").unwrap();

  assert!(any_selector_matches(&sel, div, &root_ctx(), &[], None).is_none());
}

#[test]
fn active_matches_when_active() {
  let doc = parse("<button></button>");
  let btn = &doc.root.children()[0];
  let sel = parse_selector_list("button:active").unwrap();

  let ctx = MatchContext {
    is_active: true,
    ..root_ctx()
  };
  assert!(any_selector_matches(&sel, btn, &ctx, &[], None).is_some());
}

#[test]
fn focus_matches_when_focused() {
  let doc = parse("<input>");
  let inp = &doc.root.children()[0];
  let sel = parse_selector_list("input:focus").unwrap();

  let ctx = MatchContext {
    is_focus: true,
    ..root_ctx()
  };
  assert!(any_selector_matches(&sel, inp, &ctx, &[], None).is_some());
}

#[test]
fn focus_rejects_when_not_focused() {
  let doc = parse("<input>");
  let inp = &doc.root.children()[0];
  let sel = parse_selector_list("input:focus").unwrap();

  assert!(any_selector_matches(&sel, inp, &root_ctx(), &[], None).is_none());
}

#[test]
fn focus_visible_matches() {
  let doc = parse("<input>");
  let inp = &doc.root.children()[0];
  let sel = parse_selector_list("input:focus-visible").unwrap();

  let ctx = MatchContext {
    is_focus_visible: true,
    ..root_ctx()
  };
  assert!(any_selector_matches(&sel, inp, &ctx, &[], None).is_some());
}

#[test]
fn disabled_matches_disabled_input() {
  let doc = parse(r#"<input disabled>"#);
  let inp = &doc.root.children()[0];
  let sel = parse_selector_list(":disabled").unwrap();

  assert!(any_selector_matches(&sel, inp, &root_ctx(), &[], None).is_some());
}

#[test]
fn enabled_rejects_disabled_input() {
  let doc = parse(r#"<input disabled>"#);
  let inp = &doc.root.children()[0];
  let sel = parse_selector_list(":enabled").unwrap();

  assert!(any_selector_matches(&sel, inp, &root_ctx(), &[], None).is_none());
}

#[test]
fn checked_matches_checked_input() {
  let doc = parse(r#"<input checked>"#);
  let inp = &doc.root.children()[0];
  let sel = parse_selector_list(":checked").unwrap();

  assert!(any_selector_matches(&sel, inp, &root_ctx(), &[], None).is_some());
}

#[test]
fn link_matches_anchor_with_href() {
  let doc = parse(r#"<a href="/page">link</a>"#);
  let a = &doc.root.children()[0];
  let sel = parse_selector_list(":link").unwrap();

  assert!(any_selector_matches(&sel, a, &root_ctx(), &[], None).is_some());
}

#[test]
fn link_rejects_anchor_without_href() {
  let doc = parse(r#"<a>no href</a>"#);
  let a = &doc.root.children()[0];
  let sel = parse_selector_list(":link").unwrap();

  assert!(any_selector_matches(&sel, a, &root_ctx(), &[], None).is_none());
}
