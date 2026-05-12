
use crate::{
  ArcStr,
  common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, Preload},
};

#[derive(Debug, Clone, Default)]
pub struct Audio {
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
  pub controls: Option<bool>,
  pub autoplay: Option<bool>,
  // html attr: loop
  pub r#loop: Option<bool>,
  pub muted: Option<bool>,
  pub preload: Option<Preload>,
  pub crossorigin: Option<CrossOrigin>,
}
