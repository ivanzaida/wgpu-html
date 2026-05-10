use crate::ArcStr;
use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, Preload};

#[derive(Debug, Clone, Default)]
pub struct Video {
  pub id: Option<ArcStr>,
  pub class: Option<ArcStr>,
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
  // html attr: aria-* (suffix → value)
  pub aria_attrs: HashMap<ArcStr, ArcStr>,
  // html attr: data-* (suffix → value)
  pub data_attrs: HashMap<ArcStr, ArcStr>,
  pub src: Option<ArcStr>,
  pub controls: Option<bool>,
  pub autoplay: Option<bool>,
  // html attr: loop
  pub r#loop: Option<bool>,
  pub muted: Option<bool>,
  pub poster: Option<ArcStr>,
  pub preload: Option<Preload>,
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub playsinline: Option<bool>,
  pub crossorigin: Option<CrossOrigin>,
}
