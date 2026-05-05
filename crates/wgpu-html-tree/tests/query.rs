//! Failing-tests corpus for `querySelector` / `querySelectorAll`
//! features that `spec/query.md` lists as ❌ in §0 but that the
//! current `wgpu-html-tree::query` implementation does not yet
//! support.
//!
//! These tests are intentionally RED today. They encode the
//! browser-equivalent behaviour described in the spec so an
//! implementor can flip them green one phase at a time.
//!
//! Phases (mirroring `spec/query.md` §10):
//!   1. Logical pseudo-classes — `:is`, `:where`, `:not`, `:has`
//!   2. Structural pseudo-classes — `:first-child`, `:last-child`, `:only-child`, `:empty`, `:root`, `:scope`,
//!      `:nth-child`, `:nth-last-child`, `:first-of-type`, `:last-of-type`, `:nth-of-type`
//!   3. State pseudo-classes — `:disabled`, `:enabled`, `:checked`, `:required`, `:optional`, `:read-only`,
//!      `:read-write`, `:placeholder-shown`
//!   4. Interaction pseudo-classes — `:hover`, `:focus`, `:active`, `:focus-within`
//!   5. `:lang(...)`, `:dir(...)`
//!   6. Pseudo-elements — `::before`, `::after`, `::first-line` (parser must accept; selector matches nothing)
//!   7. Namespace prefixes — `svg|circle` (parser must accept)
//!   8. CSS escape sequences — `#has\.dot`, `.\31 23`
//!
//! Each test reads the spec line-for-line. Where a test can't be
//! expressed without modifying internal state (interaction
//! pseudo-classes need `tree.interaction.*`), the expected
//! integration point is captured in a comment so the implementor
//! knows where to wire it up.

use wgpu_html_models as m;
use wgpu_html_tree::{CompoundSelector, Node, SelectorList, Tree, query::AttrOp};

// ─── Tree-building helpers ────────────────────────────────────────────────

fn div(id: Option<&str>, class: Option<&str>) -> m::Div {
  m::Div {
    id: id.map(str::to_owned),
    class: class.map(str::to_owned),
    ..m::Div::default()
  }
}

fn span(id: Option<&str>, class: Option<&str>) -> m::Span {
  m::Span {
    id: id.map(str::to_owned),
    class: class.map(str::to_owned),
    ..m::Span::default()
  }
}

fn p(id: Option<&str>, class: Option<&str>) -> m::P {
  m::P {
    id: id.map(str::to_owned),
    class: class.map(str::to_owned),
    ..m::P::default()
  }
}

/// Tree shaped like:
///
/// ```html
/// <body>
///   <div id="outer" class="box hero">
///     <span class="label">hi</span>
///     <p class="lead">first p</p>
///     <p class="lead">second p</p>
///     <div id="inner" class="box">
///       <span class="label">two</span>
///     </div>
///   </div>
///   <span id="solo" class="label primary"></span>
/// </body>
/// ```
fn sample() -> Tree {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(div(Some("outer"), Some("box hero"))).with_children(vec![
      Node::new(span(None, Some("label"))).with_children(vec![Node::new("hi")]),
      Node::new(p(None, Some("lead"))).with_children(vec![Node::new("first p")]),
      Node::new(p(None, Some("lead"))).with_children(vec![Node::new("second p")]),
      Node::new(div(Some("inner"), Some("box"))).with_children(vec![
        Node::new(span(None, Some("label"))).with_children(vec![Node::new("two")]),
      ]),
    ]),
    Node::new(span(Some("solo"), Some("label primary"))),
  ]);
  Tree::new(body)
}

// ── Phase 1: logical pseudo-classes ───────────────────────────────────────

#[test]
fn negation_pseudo_class_excludes_matches() {
  // `:not(.label)` should match every element except those carrying
  // the `label` class. Today the parser rejects any `:` token, so
  // the lenient path collapses this to "matches nothing" — the
  // assertion will fail.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("span:not(.label)");
  // The only span without the `label` class would be… none in
  // this tree; assert the *strict* parse succeeds first.
  SelectorList::parse("span:not(.label)").expect("`:not()` must parse");
  assert_eq!(hits.len(), 0);

  let hits = tree.query_selector_all_paths("div:not(#outer)");
  assert_eq!(hits.len(), 1, "only the inner div is left");
}

#[test]
fn is_pseudo_class_unions_compounds() {
  let mut tree = sample();
  SelectorList::parse(":is(span, p)").expect("`:is()` must parse");
  let hits = tree.query_selector_all_paths(":is(span, p)");
  // 3 spans + 2 ps = 5.
  assert_eq!(hits.len(), 5);
}

#[test]
fn where_pseudo_class_is_zero_specificity_is_alias() {
  // For querySelector purposes `:where()` behaves identically to
  // `:is()` — specificity isn't observable here.
  let mut tree = sample();
  SelectorList::parse(":where(span, p)").expect("`:where()` must parse");
  let hits = tree.query_selector_all_paths(":where(span, p)");
  assert_eq!(hits.len(), 5);
}

#[test]
fn has_pseudo_class_subject_contains_match() {
  // `div:has(> span.label)` — divs that have a span.label as a
  // direct child. Both divs in the sample qualify.
  let mut tree = sample();
  SelectorList::parse("div:has(> span.label)").expect("`:has()` must parse");
  let hits = tree.query_selector_all_paths("div:has(> span.label)");
  assert_eq!(hits.len(), 2);

  // `div:has(p.lead)` — divs anywhere above a p.lead. Only the
  // outer div has p.lead descendants.
  let hits = tree.query_selector_all_paths("div:has(p.lead)");
  assert_eq!(hits.len(), 1);
}

// ── Phase 2: structural pseudo-classes ────────────────────────────────────

#[test]
fn first_child_matches_first_element_child() {
  let mut tree = sample();
  SelectorList::parse(":first-child").expect("`:first-child` must parse");
  // First-child set: body (it's the root, no parent), outer div
  // (first child of body), label span (first child of outer),
  // inner span (first child of inner div).
  // Spec semantics: `:first-child` doesn't match a root with no
  // parent — exclude body.
  let hits = tree.query_selector_all_paths("*:first-child");
  assert_eq!(hits.len(), 3);
}

#[test]
fn last_child_matches_last_element_child() {
  let mut tree = sample();
  SelectorList::parse(":last-child").expect("`:last-child` must parse");
  // solo span (last child of body), inner div (last child of
  // outer), inner label span (last and only child of inner).
  let hits = tree.query_selector_all_paths("*:last-child");
  assert_eq!(hits.len(), 3);
}

#[test]
fn only_child_matches_sole_element_child() {
  let mut tree = sample();
  SelectorList::parse(":only-child").expect("`:only-child` must parse");
  // The inner span is the only child of the inner div.
  let hits = tree.query_selector_all_paths("*:only-child");
  assert_eq!(hits.len(), 1);
}

#[test]
fn empty_pseudo_class_matches_no_children() {
  let mut tree = sample();
  SelectorList::parse(":empty").expect("`:empty` must parse");
  // `:empty` matches elements with no element AND no text
  // children. The `solo` span has neither.
  let hits = tree.query_selector_all_paths("*:empty");
  assert_eq!(hits.len(), 1);
}

#[test]
fn root_pseudo_class_matches_html_root() {
  // Build a document with an explicit <html> wrapper so `:root`
  // has a unique target.
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(div(Some("a"), None))]);
  let html = Node::new(m::Html::default()).with_children(vec![body]);
  let mut tree = Tree::new(html);
  SelectorList::parse(":root").expect("`:root` must parse");
  let hits = tree.query_selector_all_paths(":root");
  assert_eq!(hits, vec![Vec::<usize>::new()]);
}

#[test]
fn scope_pseudo_class_matches_node_receiver() {
  let mut tree = sample();
  SelectorList::parse(":scope").expect("`:scope` must parse");
  // On a Node receiver, `:scope` is the receiver itself. We test
  // via `Node::query_selector_all_paths` to scope the call.
  let outer = tree.root.as_mut().unwrap().at_path_mut(&[0]).expect("outer div");
  let hits = outer.query_selector_all_paths(":scope");
  assert_eq!(hits, vec![Vec::<usize>::new()]);
}

#[test]
fn nth_child_keyword_odd_even() {
  let mut tree = sample();
  SelectorList::parse(":nth-child(odd)").expect("`:nth-child(odd)` must parse");

  // Within outer div the children are at indices 0..=3:
  //   0: span.label  (1st → odd)
  //   1: p.lead      (2nd → even)
  //   2: p.lead      (3rd → odd)
  //   3: div#inner   (4th → even)
  let hits = tree.query_selector_all_paths("#outer > :nth-child(odd)");
  assert_eq!(hits.len(), 2);

  let hits = tree.query_selector_all_paths("#outer > :nth-child(even)");
  assert_eq!(hits.len(), 2);
}

#[test]
fn nth_child_an_plus_b_formula() {
  let mut tree = sample();
  // 2n+1 == odd.
  let odd = tree.query_selector_all_paths("#outer > :nth-child(2n+1)");
  let kw = tree.query_selector_all_paths("#outer > :nth-child(odd)");
  assert_eq!(odd, kw);

  // `:nth-child(2)` — the 2nd child of its parent.
  let hits = tree.query_selector_all_paths("#outer > :nth-child(2)");
  assert_eq!(hits.len(), 1);
}

#[test]
fn nth_last_child_counts_from_end() {
  let mut tree = sample();
  SelectorList::parse(":nth-last-child(1)").expect("`:nth-last-child` must parse");
  // Equivalent to `:last-child` for `(1)`.
  let last = tree.query_selector_all_paths("*:nth-last-child(1)");
  let by_last_child = tree.query_selector_all_paths("*:last-child");
  assert_eq!(last, by_last_child);
}

#[test]
fn first_of_type_only_among_same_tag() {
  let mut tree = sample();
  SelectorList::parse("p:first-of-type").expect("`:first-of-type` must parse");
  // The first <p> inside outer is at index 1 of its parent.
  let hits = tree.query_selector_all_paths("p:first-of-type");
  assert_eq!(hits.len(), 1);
}

#[test]
fn last_of_type_only_among_same_tag() {
  let mut tree = sample();
  SelectorList::parse("p:last-of-type").expect("`:last-of-type` must parse");
  let hits = tree.query_selector_all_paths("p:last-of-type");
  assert_eq!(hits.len(), 1);
}

#[test]
fn nth_of_type_indexes_within_same_tag() {
  let mut tree = sample();
  SelectorList::parse("p:nth-of-type(2)").expect("`:nth-of-type` must parse");
  let hits = tree.query_selector_all_paths("p:nth-of-type(2)");
  assert_eq!(hits.len(), 1);
}

// ── Phase 3: state pseudo-classes ─────────────────────────────────────────

#[test]
fn disabled_and_enabled_pseudo_classes() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("a".into()),
      disabled: Some(true),
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("b".into()),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  SelectorList::parse(":disabled").expect("`:disabled` must parse");
  SelectorList::parse(":enabled").expect("`:enabled` must parse");

  let dis = tree.query_selector(":disabled").unwrap();
  assert_eq!(dis.element.id(), Some("a"));
  let en = tree.query_selector(":enabled").unwrap();
  assert_eq!(en.element.id(), Some("b"));
}

#[test]
fn checked_pseudo_class() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("a".into()),
      checked: Some(true),
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("b".into()),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  SelectorList::parse(":checked").expect("`:checked` must parse");
  let hit = tree.query_selector(":checked").unwrap();
  assert_eq!(hit.element.id(), Some("a"));
}

#[test]
fn required_and_optional_pseudo_classes() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("a".into()),
      required: Some(true),
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("b".into()),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  let req = tree.query_selector(":required").unwrap();
  assert_eq!(req.element.id(), Some("a"));
  let opt = tree.query_selector(":optional").unwrap();
  assert_eq!(opt.element.id(), Some("b"));
}

#[test]
fn read_only_and_read_write_pseudo_classes() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("ro".into()),
      readonly: Some(true),
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("rw".into()),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  let ro = tree.query_selector(":read-only").unwrap();
  assert_eq!(ro.element.id(), Some("ro"));
  let rw = tree.query_selector(":read-write").unwrap();
  assert_eq!(rw.element.id(), Some("rw"));
}

#[test]
fn placeholder_shown_pseudo_class() {
  // `:placeholder-shown` matches an input whose value is empty
  // and that has a placeholder set.
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("ph".into()),
      placeholder: Some("Enter name".into()),
      value: None,
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("filled".into()),
      placeholder: Some("Enter name".into()),
      value: Some("alice".into()),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  SelectorList::parse(":placeholder-shown").expect("`:placeholder-shown` must parse");
  let hits = tree.query_selector_all_paths(":placeholder-shown");
  assert_eq!(hits.len(), 1);
}

// ── Phase 4: interaction pseudo-classes ───────────────────────────────────
//
// Per `spec/query.md` §10.4 these read from `tree.interaction`. The
// matcher will need access to `InteractionState`; either we plumb a
// reference through `SelectorList::matches` or expose a
// `query_selector_in_state(&InteractionState, …)` variant.
//
// The cases below exercise the wire-up. Today every `:hover` /
// `:focus` selector is rejected as a parse error, so the tests fail.

#[test]
fn hover_pseudo_class_reads_interaction_state() {
  let mut tree = sample();
  // Path of the inner span `outer > inner_div > inner_span`.
  let inner_span_path = vec![0, 3, 0];
  tree.interaction.hover_path = Some(inner_span_path.clone());

  SelectorList::parse(":hover").expect("`:hover` must parse");
  let hit = tree
    .query_selector_path(":hover")
    .expect("inner span should match :hover");
  assert_eq!(hit, inner_span_path);
}

#[test]
fn focus_pseudo_class_reads_interaction_state() {
  let mut tree = sample();
  let solo_path = vec![1];
  tree.interaction.focus_path = Some(solo_path.clone());
  SelectorList::parse(":focus").expect("`:focus` must parse");
  let hit = tree.query_selector_path(":focus").expect("focus match");
  assert_eq!(hit, solo_path);
}

#[test]
fn active_pseudo_class_reads_interaction_state() {
  let mut tree = sample();
  let outer_path = vec![0];
  tree.interaction.active_path = Some(outer_path.clone());
  SelectorList::parse(":active").expect("`:active` must parse");
  let hit = tree.query_selector_path(":active").expect("active match");
  assert_eq!(hit, outer_path);
}

#[test]
fn focus_within_includes_ancestors_of_focus() {
  let mut tree = sample();
  // Focus the inner span; outer div, inner div, body, and the span
  // itself should all match `:focus-within`.
  let inner_span_path = vec![0, 3, 0];
  tree.interaction.focus_path = Some(inner_span_path);

  SelectorList::parse(":focus-within").expect("`:focus-within` must parse");
  let hits = tree.query_selector_all_paths(":focus-within");
  // body + outer div + inner div + inner span = 4
  assert_eq!(hits.len(), 4);
}

// ── Phase 5: lang / dir ───────────────────────────────────────────────────

#[test]
fn lang_pseudo_class_dash_matches() {
  let mut e = m::Div::default();
  e.lang = Some("en-US".into());
  let mut e2 = m::Div::default();
  e2.lang = Some("fr".into());
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(e), Node::new(e2)]);
  let mut tree = Tree::new(body);
  SelectorList::parse(":lang(en)").expect("`:lang()` must parse");
  let hits = tree.query_selector_all_paths(":lang(en)");
  assert_eq!(hits.len(), 1);
}

#[test]
fn dir_pseudo_class_matches_explicit_direction() {
  use m::common::html_enums::HtmlDirection;
  let mut e = m::Div::default();
  e.dir = Some(HtmlDirection::Rtl);
  let mut e2 = m::Div::default();
  e2.dir = Some(HtmlDirection::Ltr);
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(e), Node::new(e2)]);
  let mut tree = Tree::new(body);
  SelectorList::parse(":dir(rtl)").expect("`:dir()` must parse");
  let hits = tree.query_selector_all_paths(":dir(rtl)");
  assert_eq!(hits.len(), 1);
}

// ── Phase 6: pseudo-elements ──────────────────────────────────────────────
//
// Per CSS spec, `querySelector` can never match a pseudo-element.
// But the syntax must *parse* — selectors like `p::before` should
// be a valid no-op rather than a parse error that collapses the
// whole list.

#[test]
fn pseudo_elements_parse_but_match_nothing() {
  SelectorList::parse("p::before").expect("`::before` must parse");
  SelectorList::parse("p::after").expect("`::after` must parse");
  SelectorList::parse("p::first-line").expect("`::first-line` must parse");

  let mut tree = sample();
  assert!(tree.query_selector("p::before").is_none());
  assert!(tree.query_selector_all("p::before").is_empty());
}

// ── Phase 7: namespace prefixes ───────────────────────────────────────────

#[test]
fn namespace_prefix_parses() {
  // We don't carry namespaces in the model, so the matcher will
  // never return a hit — but the parser must accept the syntax.
  SelectorList::parse("svg|circle").expect("namespace prefix must parse");
  SelectorList::parse("*|circle").expect("`*|tag` must parse");
  SelectorList::parse("|circle").expect("default-namespace prefix must parse");
}

// ── Phase 8: CSS escape sequences ─────────────────────────────────────────

#[test]
fn css_escape_in_id_selector() {
  // An id literally containing a dot: `<div id="has.dot">`.
  // CSS escapes the dot as `\.`. Today the parser stops at the
  // backslash and rejects it.
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(div(Some("has.dot"), None))]);
  let mut tree = Tree::new(body);
  SelectorList::parse(r"#has\.dot").expect("escaped `.` must parse");
  let hit = tree.query_selector(r"#has\.dot").unwrap();
  assert_eq!(hit.element.id(), Some("has.dot"));
}

#[test]
fn css_numeric_escape_in_class_selector() {
  // `.\31 23` — escapes to the class `123`.
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(div(None, Some("123")))]);
  let mut tree = Tree::new(body);
  SelectorList::parse(r".\31 23").expect("numeric escape must parse");
  let hit = tree.query_selector(r".\31 23").unwrap();
  assert_eq!(hit.element.class(), Some("123"));
}

// ── Cross-phase combinators with new pseudo-classes ───────────────────────

#[test]
fn pseudo_class_inside_compound_with_combinators() {
  // `div > :first-child` — combine new pseudo-classes with the
  // existing combinator machinery. Should match each div's first
  // element child.
  let mut tree = sample();
  SelectorList::parse("div > :first-child").expect("must parse");
  let hits = tree.query_selector_all_paths("div > :first-child");
  // outer div's first child = label span, inner div's first child
  // = its label span. Both qualify.
  assert_eq!(hits.len(), 2);
}

#[test]
fn not_with_attribute_selector_inside() {
  let mut tree = sample();
  SelectorList::parse(":not([class~=label])").expect("must parse");
  // Every element that does NOT carry the `label` class token.
  // body, outer div, both p's, inner div = 5 elements.
  let hits = tree.query_selector_all_paths(":not([class~=label])");
  assert_eq!(hits.len(), 5);
}

// ══════════════════════════════════════════════════════════════════════════
// Edge cases & error cases for each phase
// ══════════════════════════════════════════════════════════════════════════

// ── Phase 1 edges: logical pseudo-classes ─────────────────────────────────

#[test]
fn not_accepts_selector_list_inside_parens() {
  // CSS-L4: `:not(a, b, c)` is shorthand for "not a, not b, not c".
  let mut tree = sample();
  SelectorList::parse(":not(span, p)").expect("must parse");
  let hits = tree.query_selector_all_paths(":not(span, p)");
  // body + outer div + inner div = 3
  assert_eq!(hits.len(), 3);
}

#[test]
fn not_with_empty_parens_is_a_parse_error() {
  assert!(SelectorList::parse(":not()").is_err());
}

#[test]
fn is_with_empty_parens_is_a_parse_error() {
  assert!(SelectorList::parse(":is()").is_err());
}

#[test]
fn is_with_whitespace_only_parens_is_a_parse_error() {
  assert!(SelectorList::parse(":is(   )").is_err());
}

#[test]
fn nested_logical_pseudo_classes() {
  // `:is(:not(.label), .lead)` — every element that isn't `.label`
  // OR is `.lead`. The 2 ps + 3 non-label elements (body, outer
  // div, inner div) = 5 unique elements (the ps aren't `.label`
  // either, so dedup keeps them once).
  let mut tree = sample();
  SelectorList::parse(":is(:not(.label), .lead)").expect("must parse");
  let hits = tree.query_selector_all_paths(":is(:not(.label), .lead)");
  assert_eq!(hits.len(), 5);
}

#[test]
fn has_with_next_sibling_combinator() {
  // `p:has(+ p)` — every `p` immediately followed by another `p`.
  // First `.lead` is followed by the second `.lead` → 1 match.
  let mut tree = sample();
  SelectorList::parse("p:has(+ p)").expect("must parse");
  let hits = tree.query_selector_all_paths("p:has(+ p)");
  assert_eq!(hits.len(), 1);
}

#[test]
fn has_with_subsequent_sibling_combinator() {
  let mut tree = sample();
  SelectorList::parse("span:has(~ p)").expect("must parse");
  // The label span at outer/0 has p siblings later in outer.
  let hits = tree.query_selector_all_paths("span:has(~ p)");
  assert_eq!(hits.len(), 1);
}

#[test]
fn has_nesting_is_supported() {
  // `:has(:has(...))` — nested `:has` is allowed by the modern
  // spec (older drafts forbade it). One match minimum is fine;
  // the test just enforces parser & matcher accept the syntax.
  let mut tree = sample();
  SelectorList::parse("body:has(div:has(p))").expect("must parse");
  let hits = tree.query_selector_all_paths("body:has(div:has(p))");
  assert_eq!(hits.len(), 1);
}

#[test]
fn is_does_not_split_at_descendant_combinator_in_outer_selector() {
  // `div :is(.label, .lead)` — descendants of `div` matching
  // either class. 2 inner spans (`.label`) + 2 ps (`.lead`) = 4.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("div :is(.label, .lead)");
  assert_eq!(hits.len(), 4);
}

#[test]
fn where_specificity_is_zero_but_querySelector_doesnt_care() {
  // `:where(.label)` and `.label` should produce identical match
  // sets in `query_selector_all` since specificity isn't observed.
  let mut tree = sample();
  let a = tree.query_selector_all_paths(":where(.label)");
  let b = tree.query_selector_all_paths(".label");
  assert_eq!(a, b);
}

// ── Phase 2 edges: structural pseudo-classes ──────────────────────────────

#[test]
fn nth_child_negative_coefficient() {
  // `:nth-child(-n+3)` — first three children.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("#outer > :nth-child(-n+3)");
  assert_eq!(hits.len(), 3);
}

#[test]
fn nth_child_zero_coefficient_equals_constant() {
  let mut tree = sample();
  let a = tree.query_selector_all_paths("#outer > :nth-child(0n+2)");
  let b = tree.query_selector_all_paths("#outer > :nth-child(2)");
  assert_eq!(a, b);
}

#[test]
fn nth_child_plain_n_matches_every_child() {
  let mut tree = sample();
  // `n` == every child.
  let n = tree.query_selector_all_paths("#outer > :nth-child(n)");
  assert_eq!(n.len(), 4);
}

#[test]
fn nth_child_zero_matches_nothing() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("#outer > :nth-child(0)");
  assert_eq!(hits.len(), 0);
}

#[test]
fn nth_child_with_of_selector_form() {
  // CSS-L4: `:nth-child(1 of p)` — first child that ALSO matches
  // `p`. Within outer that's the first `.lead` (index 1).
  let mut tree = sample();
  SelectorList::parse(":nth-child(1 of p)").expect("must parse");
  let hits = tree.query_selector_all_paths("#outer > :nth-child(1 of p)");
  assert_eq!(hits.len(), 1);
}

#[test]
fn nth_child_empty_parens_is_a_parse_error() {
  assert!(SelectorList::parse(":nth-child()").is_err());
}

#[test]
fn nth_child_malformed_formula_is_a_parse_error() {
  assert!(SelectorList::parse(":nth-child(2n+)").is_err());
  assert!(SelectorList::parse(":nth-child(+)").is_err());
  assert!(SelectorList::parse(":nth-child(abc)").is_err());
}

#[test]
fn nth_child_keyword_case_insensitive() {
  // `Odd` / `EVEN` should parse the same as lowercase forms.
  SelectorList::parse(":nth-child(ODD)").expect("must parse");
  SelectorList::parse(":nth-child(Even)").expect("must parse");
}

#[test]
fn first_of_type_with_zero_elements_of_type_is_safe() {
  let mut tree = sample();
  // `<table>` does not exist anywhere in the sample tree. Should
  // not panic; should match nothing.
  let hits = tree.query_selector_all_paths("table:first-of-type");
  assert_eq!(hits.len(), 0);
}

#[test]
fn empty_pseudo_class_treats_text_children_as_content() {
  // A span containing only a text node is NOT `:empty`. The
  // sample's label span has a Text child "hi". Use a selector
  // that targets only the label spans *inside* divs (which all
  // have text children) to exclude the empty solo span.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("div span.label:empty");
  assert_eq!(hits.len(), 0);
}

#[test]
fn empty_pseudo_class_with_whitespace_text_still_not_empty() {
  // CSS spec: `:empty` matches elements with no children OF ANY KIND
  // (including text). Whitespace-only text counts as content.
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(span(Some("ws"), None)).with_children(vec![Node::new("   ")]),
    Node::new(span(Some("real_empty"), None)),
  ]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("span:empty");
  assert_eq!(hits, vec![vec![1]]);
}

#[test]
fn root_pseudo_class_does_not_match_inner_html_like_element() {
  // If a tree has an Html element nested inside body (synthetic),
  // `:root` only matches the top-level html.
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::Html::default())]);
  let html = Node::new(m::Html::default()).with_children(vec![body]);
  let mut tree = Tree::new(html);
  let hits = tree.query_selector_all_paths(":root");
  assert_eq!(hits.len(), 1);
  assert_eq!(hits[0], Vec::<usize>::new());
}

#[test]
fn scope_in_tree_query_matches_root() {
  // On a Tree receiver, `:scope` is the document root.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths(":scope");
  assert_eq!(hits, vec![Vec::<usize>::new()]);
}

#[test]
fn scope_with_descendant_combinator() {
  // `:scope > .label` from a Node receiver scopes to that node.
  let mut tree = sample();
  let outer = tree.root.as_mut().unwrap().at_path_mut(&[0]).expect("outer div");
  let hits = outer.query_selector_all_paths(":scope > .label");
  assert_eq!(hits, vec![vec![0]]);
}

#[test]
fn pseudo_class_name_is_ascii_case_insensitive() {
  // `:HOVER`, `:Focus`, `:NTH-child(2)` should all parse and behave
  // identically to their lowercase forms.
  SelectorList::parse(":HOVER").expect("must parse");
  SelectorList::parse(":Focus").expect("must parse");
  SelectorList::parse(":NTH-child(2)").expect("must parse");
  SelectorList::parse(":First-Child").expect("must parse");
}

// ── Phase 3 edges: state pseudo-classes ───────────────────────────────────

#[test]
fn checked_with_some_false_does_not_match() {
  // `Some(false)` means the attribute was authored as `checked=false`
  // (or programmatically cleared). Per spec it is NOT `:checked`.
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::Input {
    id: Some("a".into()),
    checked: Some(false),
    ..m::Input::default()
  })]);
  let mut tree = Tree::new(body);
  SelectorList::parse(":checked").expect("must parse");
  assert!(tree.query_selector(":checked").is_none());
}

#[test]
fn checked_matches_option_with_selected() {
  // `:checked` also matches `<option selected>` per spec.
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::Select::default()).with_children(vec![
    Node::new(m::OptionElement {
      id: Some("a".into()),
      selected: Some(true),
      ..m::OptionElement::default()
    }),
    Node::new(m::OptionElement {
      id: Some("b".into()),
      ..m::OptionElement::default()
    }),
  ])]);
  let mut tree = Tree::new(body);
  let hit = tree.query_selector(":checked").unwrap();
  assert_eq!(hit.element.id(), Some("a"));
}

#[test]
fn disabled_matches_option_too() {
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::OptionElement {
    id: Some("d".into()),
    disabled: Some(true),
    ..m::OptionElement::default()
  })]);
  let mut tree = Tree::new(body);
  let hit = tree.query_selector(":disabled").unwrap();
  assert_eq!(hit.element.id(), Some("d"));
}

#[test]
fn placeholder_shown_on_textarea_with_no_text_children() {
  // Textarea's "value" is its child text. No children + placeholder
  // set → `:placeholder-shown`.
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Textarea {
      id: Some("ph".into()),
      placeholder: Some("type here".into()),
      ..m::Textarea::default()
    }),
    Node::new(m::Textarea {
      id: Some("filled".into()),
      placeholder: Some("type here".into()),
      ..m::Textarea::default()
    })
    .with_children(vec![Node::new("alice")]),
  ]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths(":placeholder-shown");
  assert_eq!(hits, vec![vec![0]]);
}

#[test]
fn enabled_does_not_match_non_form_elements() {
  // `:enabled` only matches elements that CAN be disabled
  // (form controls, fieldset, optgroup, …). A bare `<div>`
  // doesn't.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths(":enabled");
  assert_eq!(hits.len(), 0);
}

// ── Phase 4 edges: interaction pseudo-classes ─────────────────────────────

#[test]
fn focus_with_no_focus_path_matches_nothing() {
  let mut tree = sample();
  tree.interaction.focus_path = None;
  SelectorList::parse(":focus").expect("must parse");
  assert!(tree.query_selector(":focus").is_none());
  assert!(tree.query_selector_all(":focus").is_empty());
}

#[test]
fn focus_within_includes_the_focused_element_itself() {
  let mut tree = sample();
  let p = vec![0, 1]; // first p in outer
  tree.interaction.focus_path = Some(p.clone());
  let hits = tree.query_selector_all_paths(":focus-within");
  // body + outer + p itself = 3
  assert_eq!(hits.len(), 3);
  assert!(hits.iter().any(|h| *h == p));
}

#[test]
fn hover_matches_only_leaf_path_not_ancestors() {
  // The current `tree.interaction.hover_path` carries one path.
  // `:hover` should match only that node — ancestors get
  // `:hover` only if the engine chooses to bubble. This test
  // pins down "leaf only" so the implementor doesn't accidentally
  // match the whole ancestor chain.
  let mut tree = sample();
  let p = vec![0, 1];
  tree.interaction.hover_path = Some(p.clone());
  let hits = tree.query_selector_all_paths(":hover");
  assert_eq!(hits, vec![p]);
}

#[test]
fn interaction_pseudo_classes_compose_with_combinators() {
  // `body :focus` — focus inside body. Just checks combinator+
  // pseudo-class composition wires up correctly.
  let mut tree = sample();
  tree.interaction.focus_path = Some(vec![1]);
  let hits = tree.query_selector_all_paths("body :focus");
  assert_eq!(hits, vec![vec![1]]);
}

// ── Phase 5 edges: lang / dir ─────────────────────────────────────────────

#[test]
fn lang_inherits_from_ancestor() {
  // CSS spec: `:lang(en)` matches an element whose computed
  // language is en, including via inheritance. The implementor
  // can either walk ancestors here OR consult cascade. Either way,
  // the test must pass.
  let mut outer = m::Div::default();
  outer.lang = Some("en-GB".into());
  let inner_div = m::Div {
    id: Some("inner".into()),
    ..m::Div::default()
  };
  let body =
    Node::new(m::Body::default()).with_children(vec![Node::new(outer).with_children(vec![Node::new(inner_div)])]);
  let mut tree = Tree::new(body);
  let hit = tree.query_selector("#inner:lang(en)").unwrap();
  assert_eq!(hit.element.id(), Some("inner"));
}

#[test]
fn lang_does_not_match_unrelated_language() {
  let mut e = m::Div::default();
  e.lang = Some("fr".into());
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(e)]);
  let mut tree = Tree::new(body);
  assert!(tree.query_selector(":lang(en)").is_none());
}

#[test]
fn dir_default_when_unset() {
  // Without an explicit `dir` attribute, `:dir(ltr)` should match
  // (per spec, default is ltr). This test pins down "fall back
  // to ltr".
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(div(Some("a"), None))]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("#a:dir(ltr)");
  assert_eq!(hits.len(), 1);
}

// ── Phase 6 edges: pseudo-elements ────────────────────────────────────────

#[test]
fn legacy_single_colon_pseudo_elements_parse() {
  // CSS2.1 pseudo-elements use a single colon: `:before`,
  // `:after`, `:first-line`, `:first-letter`. Modern selectors
  // require `::` for pseudo-elements but parsers must accept the
  // legacy form for the four pre-CSS3 names.
  SelectorList::parse("p:before").expect("legacy `:before` must parse");
  SelectorList::parse("p:after").expect("legacy `:after` must parse");
  SelectorList::parse("p:first-line").expect("legacy `:first-line` must parse");
  SelectorList::parse("p:first-letter").expect("legacy `:first-letter` must parse");
}

#[test]
fn pseudo_element_with_combinator_parses() {
  // `div > p::before` — combinator + pseudo-element should parse
  // cleanly, even if it never matches.
  SelectorList::parse("div > p::before").expect("must parse");
}

#[test]
fn unknown_pseudo_element_is_a_parse_error_not_a_panic() {
  // Defensive: `::totally-fake-element` should be a clean parse
  // error from `SelectorList::parse`, and lenient `From<&str>`
  // should produce a no-match list.
  assert!(SelectorList::parse("p::totally-fake-element").is_err());
  let mut tree = sample();
  assert!(tree.query_selector("p::totally-fake-element").is_none());
}

// ── Phase 7 edges: namespace prefixes ─────────────────────────────────────

#[test]
fn namespace_prefix_matches_nothing_in_namespaceless_model() {
  // We don't carry namespaces — every element is in the implicit
  // HTML namespace. `svg|circle` should parse but not match.
  let mut tree = sample();
  assert!(tree.query_selector("svg|circle").is_none());
}

#[test]
fn star_prefix_matches_any_namespace() {
  // `*|*` is "any element in any namespace" — should match the
  // universal set.
  SelectorList::parse("*|*").expect("must parse");
  let mut tree = sample();
  let starstar = tree.query_selector_all_paths("*|*");
  let star = tree.query_selector_all_paths("*");
  assert_eq!(starstar, star);
}

// ── Phase 8 edges: CSS escape sequences ───────────────────────────────────

#[test]
fn css_escape_with_six_hex_digits() {
  // `\000041` → 'A'. Per spec, hex escape can be up to 6 digits.
  SelectorList::parse(r"#\000041").expect("must parse");
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(div(Some("A"), None))]);
  let mut tree = Tree::new(body);
  let hit = tree.query_selector(r"#\000041").unwrap();
  assert_eq!(hit.element.id(), Some("A"));
}

#[test]
fn css_escape_terminator_space_is_consumed() {
  // `\31 ` — the space terminates the hex escape and is consumed.
  // So `\31 23` is `1` + `23` = `123`.
  SelectorList::parse(r"#\31 23").expect("must parse");
}

#[test]
fn css_escape_literal_backslash() {
  // `\\` → literal `\`. (Awkward in real CSS but spec-required.)
  SelectorList::parse(r"#\\").expect("must parse");
}

#[test]
fn css_escape_unterminated_at_eof_is_an_error() {
  // Trailing bare backslash with no escape body.
  assert!(SelectorList::parse(r"#a\").is_err());
}

#[test]
fn identifier_with_double_dash_prefix() {
  // CSS-L4 allows `--name` as a valid identifier (custom-property
  // style). Class selector `.--my-class` should parse.
  SelectorList::parse(".--my-class").expect("must parse");
}

// ── Lenient-path & robustness ─────────────────────────────────────────────

#[test]
fn unknown_pseudo_class_strict_errors_lenient_no_match() {
  // Strict path: error.
  assert!(SelectorList::parse(":totally-fake").is_err());
  // Lenient path: empty match list (NOT a panic).
  let mut tree = sample();
  assert!(tree.query_selector(":totally-fake").is_none());
  assert!(tree.query_selector_all(":totally-fake").is_empty());
}

#[test]
fn selector_list_partial_failure_is_strict() {
  // `SelectorList::parse("div, :totally-fake")` should fail
  // even though `div` alone is valid — the list is all-or-nothing
  // under strict parsing.
  assert!(SelectorList::parse("div, :totally-fake").is_err());
}

#[test]
fn selector_list_partial_failure_is_lenient_in_from() {
  // The `From<&str>` lenient path collapses any failure to "matches
  // nothing", even if half the list is valid. This is the
  // documented behaviour of `query_selector*`.
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("div, :totally-fake");
  assert_eq!(hits.len(), 0);
}

// ── API / contract tests across phases ────────────────────────────────────

#[test]
fn matches_emit_in_document_order_across_selector_list() {
  // `#solo, #outer` — `#outer` appears earlier in document order,
  // so it must come first regardless of the order in the list.
  let mut tree = sample();
  let paths = tree.query_selector_all_paths("#solo, #outer");
  assert_eq!(paths, vec![vec![0], vec![1]]);
}

#[test]
fn dedup_when_two_arms_match_same_element() {
  // The `outer` div matches both `#outer` and `.box`. It must
  // appear once in the merged result.
  let mut tree = sample();
  let paths = tree.query_selector_all_paths("#outer, .box");
  // outer (matches both) + inner div (matches `.box` only) = 2
  assert_eq!(paths.len(), 2);
}

#[test]
fn path_round_trips_through_at_path_mut() {
  let mut tree = sample();
  let path = tree.query_selector_path("#inner").expect("inner exists");
  let n = tree.root.as_mut().unwrap().at_path_mut(&path).expect("path resolves");
  assert_eq!(n.element.id(), Some("inner"));
}

#[test]
fn pre_parsed_selector_reused_after_tree_mutation() {
  // Parse once, query, mutate, query again — same selector handle
  // must keep working.
  let sel = SelectorList::parse(".label").unwrap();
  let mut tree = sample();
  assert_eq!(tree.query_selector_all(&sel).len(), 3);

  // Drop the inner div → its label span goes away too.
  let outer = tree.root.as_mut().unwrap().at_path_mut(&[0]).expect("outer");
  outer.children.pop();
  assert_eq!(tree.query_selector_all(&sel).len(), 2);
}

#[test]
fn node_query_selector_is_scoped_to_subtree() {
  // `Node::query_selector_all` MUST NOT see siblings of `self`.
  let mut tree = sample();
  let outer = tree.root.as_mut().unwrap().at_path_mut(&[0]).expect("outer div");
  // From outer's perspective, `#solo` is not in the subtree.
  assert!(outer.query_selector("#solo").is_none());
  // But `.label` matches the two label spans inside outer.
  let hits = outer.query_selector_all_paths(".label");
  assert_eq!(hits.len(), 2);
}

#[test]
fn path_returned_matches_path_query_selector_dereferences() {
  // `query_selector_path` returns the path that
  // `query_selector` would walk. Pin them together.
  let mut tree = sample();
  let path = tree.query_selector_path("#inner").unwrap();
  let id_via_borrow: Option<String> = tree.query_selector("#inner").unwrap().element.id().map(str::to_owned);
  let id_via_path: Option<String> = tree
    .root
    .as_mut()
    .unwrap()
    .at_path_mut(&path)
    .and_then(|nn| nn.element.id().map(str::to_owned));
  assert_eq!(id_via_path, id_via_borrow);
}

#[test]
fn empty_selector_string_matches_nothing() {
  let mut tree = sample();
  assert!(tree.query_selector("").is_none());
  assert!(tree.query_selector_all("").is_empty());
}

#[test]
fn whitespace_only_selector_string_matches_nothing() {
  let mut tree = sample();
  assert!(tree.query_selector("   ").is_none());
  assert!(tree.query_selector_all("   ").is_empty());
}

#[test]
fn trailing_combinator_is_a_strict_parse_error() {
  assert!(SelectorList::parse("div >").is_err());
  assert!(SelectorList::parse("div +").is_err());
  assert!(SelectorList::parse("div ~").is_err());
}

#[test]
fn leading_combinator_is_only_valid_inside_has() {
  // `> p` at top level is not a valid complete selector.
  assert!(SelectorList::parse("> p").is_err());
  // But `:has(> p)` is fine (relative selector).
  SelectorList::parse(":has(> p)").expect("must parse");
}

// ══════════════════════════════════════════════════════════════════════════
// Basic DOM query tests (from query_tests.rs)
// ══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_compound_keeps_old_grammar() {
  let s = CompoundSelector::parse("div.box#outer.hero").unwrap();
  assert_eq!(s.tag.as_deref(), Some("div"));
  assert_eq!(s.id.as_deref(), Some("outer"));
  assert_eq!(s.classes, vec!["box".to_string(), "hero".to_string()]);

  let s = CompoundSelector::parse("*.label").unwrap();
  assert!(s.tag.is_none());
  assert_eq!(s.classes, vec!["label".to_string()]);

  assert!(CompoundSelector::parse("div span").is_err());
  assert!(CompoundSelector::parse("a, b").is_err());
  assert!(CompoundSelector::parse("a > b").is_err());
}

#[test]
fn parse_attribute_operators() {
  let list = SelectorList::parse("[a][b=v][c~=v][d|=en][e^=p][f$=q][g*=r]").unwrap();
  let cs = &list.selectors[0].compounds[0];
  let ops: Vec<_> = cs.attrs.iter().map(|f| f.op).collect();
  assert_eq!(
    ops,
    vec![
      AttrOp::Exists,
      AttrOp::Equals,
      AttrOp::Includes,
      AttrOp::DashMatch,
      AttrOp::Prefix,
      AttrOp::Suffix,
      AttrOp::Substring,
    ]
  );
}

#[test]
fn parse_attribute_case_flags() {
  let list = SelectorList::parse("[type=PASSWORD i]").unwrap();
  let f = &list.selectors[0].compounds[0].attrs[0];
  assert_eq!(f.value, "PASSWORD");
  assert!(f.case_insensitive);

  let list = SelectorList::parse("[type=password s]").unwrap();
  let f = &list.selectors[0].compounds[0].attrs[0];
  assert!(!f.case_insensitive);
}

#[test]
fn attribute_op_includes_matches_class_token() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(div(None, Some("foo bar baz"))),
    Node::new(div(None, Some("foobar"))),
  ]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("[class~=\"bar\"]");
  assert_eq!(hits, vec![vec![0]]);
}

#[test]
fn attribute_op_dashmatch_for_lang() {
  let mut e = m::Div::default();
  e.lang = Some("en-US".to_owned());
  let mut e2 = m::Div::default();
  e2.lang = Some("english".to_owned());
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(e), Node::new(e2)]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("[lang|=en]");
  assert_eq!(hits, vec![vec![0]]);
}

#[test]
fn attribute_op_prefix_suffix_substring() {
  let mut a1 = m::A::default();
  a1.href = Some("https://example.com/path".to_owned());
  let mut a2 = m::A::default();
  a2.href = Some("/local".to_owned());
  let mut a3 = m::A::default();
  a3.href = Some("https://example.com/file.pdf".to_owned());
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(a1), Node::new(a2), Node::new(a3)]);
  let mut tree = Tree::new(body);

  assert_eq!(
    tree.query_selector_all_paths("a[href^=\"https://\"]"),
    vec![vec![0], vec![2]]
  );
  assert_eq!(tree.query_selector_all_paths("a[href$=\".pdf\"]"), vec![vec![2]]);
  assert_eq!(
    tree.query_selector_all_paths("a[href*=\"example\"]"),
    vec![vec![0], vec![2]]
  );
}

#[test]
fn attribute_case_insensitive_flag() {
  use m::common::html_enums::InputType;
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::Input {
    id: Some("pw".to_owned()),
    r#type: Some(InputType::Password),
    ..m::Input::default()
  })]);
  let mut tree = Tree::new(body);
  assert!(tree.query_selector("input[type=PASSWORD]").is_none());
  assert!(tree.query_selector("input[type=PASSWORD i]").is_some());
}

#[test]
fn descendant_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("body div span");
  assert_eq!(hits.len(), 2);
}

#[test]
fn child_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("div > span");
  assert_eq!(hits.len(), 2);
}

#[test]
fn next_sibling_combinator() {
  // Build a tree where span is immediately followed by a div
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(div(None, None)).with_children(vec![Node::new(span(None, None)), Node::new(div(None, None))]),
  ]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("span + div");
  assert_eq!(hits.len(), 1);
}

#[test]
fn subsequent_sibling_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("span ~ div");
  assert_eq!(hits.len(), 1);
}

#[test]
fn selector_list_unions_matches() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("#outer, #solo");
  assert_eq!(hits.len(), 2);

  let hits = tree.query_selector_all_paths("div, [class~=primary]");
  assert_eq!(hits.len(), 3);
}

#[test]
fn query_selector_by_id() {
  let mut tree = sample();
  let n = tree.query_selector("#outer").unwrap();
  assert_eq!(n.element.tag_name(), "div");
  assert_eq!(n.element.id(), Some("outer"));
  assert!(tree.query_selector("#missing").is_none());
}

#[test]
fn query_selector_by_tag() {
  let mut tree = sample();
  let first = tree.query_selector("div").unwrap();
  assert_eq!(first.element.id(), Some("outer"));
}

#[test]
fn query_selector_by_class_compound() {
  let mut tree = sample();
  let n = tree.query_selector("span.primary").unwrap();
  assert_eq!(n.element.id(), Some("solo"));
  assert!(tree.query_selector("span.box").is_none());
}

#[test]
fn universal_selector_includes_root_self() {
  let mut tree = sample();
  let all = tree.query_selector_all("*");
  // sample tree has 8 element nodes (body, outer, span.label, p.lead, p.lead, inner, inner span, solo)
  assert_eq!(all.len(), 8);
}

#[test]
fn empty_tree_is_safe() {
  let mut tree = Tree::default();
  assert!(tree.query_selector("div").is_none());
  assert!(tree.query_selector_all("div").is_empty());
  assert!(tree.query_selector_all("a, b, c").is_empty());
}

#[test]
fn pre_parsed_selector_reuses_across_calls() {
  let sel = SelectorList::parse(".label, #solo").unwrap();
  let mut tree = sample();
  assert_eq!(tree.query_selector_all(&sel).len(), 3);
  assert_eq!(tree.query_selector_all(sel).len(), 3);
}

#[test]
fn compound_selector_into_list() {
  let cs = CompoundSelector::parse("span.label").unwrap();
  let mut tree = sample();
  assert_eq!(tree.query_selector_all(cs).len(), 3);
}

#[test]
fn input_type_password_user_case() {
  use m::common::html_enums::InputType;
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("user".to_owned()),
      r#type: Some(InputType::Text),
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("pass".to_owned()),
      r#type: Some(InputType::Password),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  let hit = tree.query_selector("input[type=\"password\"]").unwrap();
  assert_eq!(hit.element.id(), Some("pass"));
}
