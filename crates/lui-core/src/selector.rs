use crate::{CssCombinator, CssPseudo};

#[derive(Debug, Clone, PartialEq)]
pub struct SelectorList(pub Vec<ComplexSelector>);

#[derive(Debug, Clone, PartialEq)]
pub struct ComplexSelector {
  pub compounds: Vec<CompoundSelector>,
  pub combinators: Vec<CssCombinator>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CompoundSelector {
  pub tag: Option<String>,
  pub classes: Vec<String>,
  pub id: Option<String>,
  pub attrs: Vec<AttributeSelector>,
  pub pseudos: Vec<PseudoSelector>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
  pub name: String,
  pub op: Option<AttrOp>,
  pub value: Option<String>,
  pub modifier: Option<char>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrOp {
  Eq,
  Contains,
  StartsWith,
  EndsWith,
  Includes,
  Hyphen,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PseudoSelector {
  pub pseudo: CssPseudo,
  pub arg: Option<String>,
}
