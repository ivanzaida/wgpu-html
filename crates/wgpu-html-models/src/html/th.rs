use crate::common::html_enums::{AriaRole, HtmlDirection, TableHeaderScope};

#[derive(Debug, Clone, Default)]
pub struct Th {
  pub id: Option<String>,
  pub class: Option<String>,
  pub style: Option<String>,
  pub title: Option<String>,
  pub lang: Option<String>,
  pub dir: Option<HtmlDirection>,
  pub hidden: Option<bool>,
  pub tabindex: Option<i32>,
  pub accesskey: Option<String>,
  pub contenteditable: Option<bool>,
  pub draggable: Option<bool>,
  pub spellcheck: Option<bool>,
  pub translate: Option<bool>,
  pub role: Option<AriaRole>,
  // html attr: aria-*
  pub aria_star: Option<String>,
  // html attr: data-*
  pub data_star: Option<String>,
  pub colspan: Option<u32>,
  pub rowspan: Option<u32>,
  pub headers: Option<String>,
  pub scope: Option<TableHeaderScope>,
  pub abbr: Option<String>,
}
