use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection},
};

#[derive(Debug, Clone, Default)]
pub struct OptionElement {
  pub id: Option<ArcStr>,
  pub style: Option<ArcStr>,
  pub title: Option<ArcStr>,
  pub lang: Option<ArcStr>,
  pub dir: Option<HtmlDirection>,
  pub hidden: Option<bool>,
  pub tabindex: Option<i32>,
  pub accesskey: Option<ArcStr>,
  pub contenteditable: Option<bool>,
  pub draggable: Option<bool>,
  pub spellcheck: Option<bool>,
  pub translate: Option<bool>,
  pub role: Option<AriaRole>,
  pub value: Option<ArcStr>,
  pub label: Option<ArcStr>,
  pub selected: Option<bool>,
  pub disabled: Option<bool>,
}
