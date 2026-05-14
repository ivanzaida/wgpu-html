use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection, TableHeaderScope},
};

#[derive(Debug, Clone, Default)]
pub struct Th {
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
  pub colspan: Option<u32>,
  pub rowspan: Option<u32>,
  pub headers: Option<ArcStr>,
  pub scope: Option<TableHeaderScope>,
  pub abbr: Option<ArcStr>,
}
