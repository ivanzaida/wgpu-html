pub use lui_models::common::css_enums::{BoxSizing, CssColor, CssLength, Cursor, Display, TextAlign};
#[allow(unused_imports)]
pub use lui_parser::{ComplexSelector, parse_stylesheet};

use crate::*;

pub(crate) fn elem_div() -> Element {
  Element::Div(lui_models::Div::default())
}

pub(crate) fn elem_div_with(id: Option<&str>, class: Option<&str>) -> Element {
  let mut d = lui_models::Div::default();
  d.id = id.map(|s| ArcStr::from(s));
  d.class = class.map(|s| ArcStr::from(s));
  Element::Div(d)
}

pub(crate) fn elem_p() -> Element {
  Element::P(lui_models::P::default())
}

pub(crate) fn first_div(tree: &Tree) -> CascadedNode {
  let cascaded = cascade(tree);
  let body = cascaded.root.expect("expected a root");
  body
    .children
    .into_iter()
    .find(|c| matches!(c.element, Element::Div(_)))
    .expect("expected a div under root")
}

pub(crate) fn find_style<F: Fn(&Element) -> bool>(node: &CascadedNode, pred: &F) -> Option<Style> {
  if pred(&node.element) {
    return Some(node.style.clone());
  }
  for child in &node.children {
    if let Some(s) = find_style(child, pred) {
      return Some(s);
    }
  }
  None
}
