use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, Preload};

#[derive(Debug, Clone, Default)]
pub struct Video {
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
  // html attr: aria-* (suffix → value)
  pub aria_attrs: HashMap<String, String>,
  // html attr: data-* (suffix → value)
  pub data_attrs: HashMap<String, String>,
  pub src: Option<String>,
  pub controls: Option<bool>,
  pub autoplay: Option<bool>,
  // html attr: loop
  pub r#loop: Option<bool>,
  pub muted: Option<bool>,
  pub poster: Option<String>,
  pub preload: Option<Preload>,
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub playsinline: Option<bool>,
  pub crossorigin: Option<CrossOrigin>,
}
