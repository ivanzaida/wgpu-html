use crate::{
  ArcStr,
  common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, ReferrerPolicy},
};

#[derive(Debug, Clone, Default)]
pub struct Script {
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
  // html attr: type
  pub r#type: Option<ArcStr>,
  // html attr: async
  pub r#async: Option<bool>,
  pub defer: Option<bool>,
  pub crossorigin: Option<CrossOrigin>,
  pub integrity: Option<ArcStr>,
  pub nomodule: Option<bool>,
  pub nonce: Option<ArcStr>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
