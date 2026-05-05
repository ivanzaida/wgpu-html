use wgpu_html_parser::{Combinator, ComplexSelector, CompoundSelector};

use super::helpers::*;
use crate::*;

// --------------------------------------------------------------------------
// Selector matching
// --------------------------------------------------------------------------

fn simple_compound(f: impl FnOnce(&mut CompoundSelector)) -> ComplexSelector {
  let mut c = CompoundSelector::default();
  f(&mut c);
  ComplexSelector {
    compounds: vec![c],
    combinators: vec![],
  }
}

fn tag_sel(tag: &str) -> ComplexSelector {
  simple_compound(|c| c.tag = Some(tag.into()))
}

fn id_sel(id: &str) -> ComplexSelector {
  simple_compound(|c| c.id = Some(id.into()))
}

fn class_sel(class: &str) -> ComplexSelector {
  simple_compound(|c| c.classes = vec![class.into()])
}

fn universal_sel() -> ComplexSelector {
  ComplexSelector::default()
}

#[test]
fn matches_tag_only() {
  let sel = tag_sel("div");
  assert!(matches_selector(&sel, &elem_div()));
  assert!(!matches_selector(&sel, &elem_p()));
}

#[test]
fn matches_id() {
  let sel = id_sel("hero");
  assert!(matches_selector(&sel, &elem_div_with(Some("hero"), None)));
  assert!(!matches_selector(&sel, &elem_div_with(Some("other"), None)));
  assert!(!matches_selector(&sel, &elem_div_with(None, None)));
}

#[test]
fn matches_class_one_of_many() {
  let sel = class_sel("card");
  assert!(matches_selector(&sel, &elem_div_with(None, Some("big card primary"))));
  assert!(!matches_selector(&sel, &elem_div_with(None, Some("big primary"))));
}

#[test]
fn matches_compound_all_required() {
  let sel = simple_compound(|c| {
    c.tag = Some("div".into());
    c.id = Some("hero".into());
    c.classes = vec!["card".into(), "big".into()];
  });
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
  let sel = universal_sel();
  assert!(matches_selector(&sel, &elem_div()));
  assert!(matches_selector(&sel, &elem_p()));
}

#[test]
fn matches_selector_rejects_descendant_without_ancestors() {
  // ".row .item" → compounds=[.row, .item], combinators=[Descendant]
  let sel = ComplexSelector {
    compounds: vec![
      CompoundSelector {
        classes: vec!["row".into()],
        ..Default::default()
      },
      CompoundSelector {
        classes: vec!["item".into()],
        ..Default::default()
      },
    ],
    combinators: vec![Combinator::Descendant],
  };
  let item = elem_div_with(None, Some("item"));
  assert!(!matches_selector(&sel, &item));
}

#[test]
fn matches_selector_in_tree_walks_ancestors() {
  let sel = ComplexSelector {
    compounds: vec![
      CompoundSelector {
        classes: vec!["row".into()],
        ..Default::default()
      },
      CompoundSelector {
        classes: vec!["item".into()],
        ..Default::default()
      },
    ],
    combinators: vec![Combinator::Descendant],
  };
  let row = elem_div_with(None, Some("row"));
  let item = elem_div_with(None, Some("item"));
  assert!(matches_selector_in_tree(&sel, &item, &[&row]));
  let neutral = elem_div_with(None, Some("box"));
  assert!(!matches_selector_in_tree(&sel, &item, &[&neutral]));
  assert!(matches_selector_in_tree(&sel, &item, &[&neutral, &row]));
}
