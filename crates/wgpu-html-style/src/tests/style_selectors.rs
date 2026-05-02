use crate::*;
use super::helpers::*;
// --------------------------------------------------------------------------
// Selector matching
// --------------------------------------------------------------------------

#[test]
fn matches_tag_only() {
  let sel = Selector {
    tag: Some("div".into()),
    ..Default::default()
  };
  assert!(matches_selector(&sel, &elem_div()));
  assert!(!matches_selector(&sel, &elem_p()));
}

#[test]
fn matches_id() {
  let sel = Selector {
    id: Some("hero".into()),
    ..Default::default()
  };
  assert!(matches_selector(&sel, &elem_div_with(Some("hero"), None)));
  assert!(!matches_selector(&sel, &elem_div_with(Some("other"), None)));
  assert!(!matches_selector(&sel, &elem_div_with(None, None)));
}

#[test]
fn matches_class_one_of_many() {
  let sel = Selector {
    classes: vec!["card".into()],
    ..Default::default()
  };
  assert!(matches_selector(&sel, &elem_div_with(None, Some("big card primary"))));
  assert!(!matches_selector(&sel, &elem_div_with(None, Some("big primary"))));
}

#[test]
fn matches_compound_all_required() {
  let sel = Selector {
    tag: Some("div".into()),
    id: Some("hero".into()),
    classes: vec!["card".into(), "big".into()],
    ..Default::default()
  };
  assert!(matches_selector(
    &sel,
    &elem_div_with(Some("hero"), Some("card big primary"))
  ));
  // missing one class → fails
  assert!(!matches_selector(
    &sel,
    &elem_div_with(Some("hero"), Some("card primary"))
  ));
}

#[test]
fn universal_matches_any() {
  let sel = Selector {
    universal: true,
    ..Default::default()
  };
  assert!(matches_selector(&sel, &elem_div()));
  assert!(matches_selector(&sel, &elem_p()));
}

#[test]
fn matches_selector_rejects_descendant_without_ancestors() {
  // `.row .item` against an `.item` element with no ancestor
  // context must NOT match — the simple wrapper has no chain.
  let sel = Selector {
    classes: vec!["item".into()],
    ancestors: vec![Selector {
      classes: vec!["row".into()],
      ..Default::default()
    }],
    ..Default::default()
  };
  let item = elem_div_with(None, Some("item"));
  assert!(!matches_selector(&sel, &item));
}

#[test]
fn matches_selector_in_tree_walks_ancestors() {
  let sel = Selector {
    classes: vec!["item".into()],
    ancestors: vec![Selector {
      classes: vec!["row".into()],
      ..Default::default()
    }],
    ..Default::default()
  };
  let row = elem_div_with(None, Some("row"));
  let item = elem_div_with(None, Some("item"));
  // Direct parent matches → fires.
  assert!(matches_selector_in_tree(&sel, &item, &[&row]));
  // No ancestor `.row` → fails.
  let neutral = elem_div_with(None, Some("box"));
  assert!(!matches_selector_in_tree(&sel, &item, &[&neutral]));
  // Deeper ancestor `.row` (with an unrelated parent in between) →
  // descendant combinator is non-adjacent, still fires.
  assert!(matches_selector_in_tree(&sel, &item, &[&neutral, &row]));
}
