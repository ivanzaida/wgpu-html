use crate::common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, Preload};

#[derive(Debug, Clone, Default)]
pub struct Audio {
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
  pub src: Option<String>,
  pub controls: Option<bool>,
  pub autoplay: Option<bool>,
  // html attr: loop
  pub r#loop: Option<bool>,
  pub muted: Option<bool>,
  pub preload: Option<Preload>,
  pub crossorigin: Option<CrossOrigin>,
}
