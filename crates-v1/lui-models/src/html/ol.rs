
use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection, OlType},
};

#[derive(Debug, Clone, Default)]
pub struct Ol {
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
  pub reversed: Option<bool>,
  pub start: Option<i32>,
  // html attr: type
  pub r#type: Option<OlType>,
}
