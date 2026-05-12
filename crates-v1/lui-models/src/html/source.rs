
use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection},
};

#[derive(Debug, Clone, Default)]
pub struct Source {
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
  pub src: Option<ArcStr>,
  pub srcset: Option<ArcStr>,
  pub sizes: Option<ArcStr>,
  pub media: Option<ArcStr>,
  // html attr: type
  pub r#type: Option<ArcStr>,
  pub width: Option<u32>,
  pub height: Option<u32>,
}
