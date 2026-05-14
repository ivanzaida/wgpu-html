use crate::{
  ArcStr,
  common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, LinkAs, ReferrerPolicy},
};

#[derive(Debug, Clone, Default)]
pub struct Link {
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
  pub href: Option<ArcStr>,
  pub rel: Option<ArcStr>,
  // html attr: type
  pub r#type: Option<ArcStr>,
  pub media: Option<ArcStr>,
  pub sizes: Option<ArcStr>,
  pub hreflang: Option<ArcStr>,
  // html attr: as
  pub r#as: Option<LinkAs>,
  pub crossorigin: Option<CrossOrigin>,
  pub integrity: Option<ArcStr>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
