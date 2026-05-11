use std::collections::HashMap;

use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection, Loading, ReferrerPolicy},
};

#[derive(Debug, Clone, Default)]
pub struct Iframe {
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
  pub srcdoc: Option<ArcStr>,
  pub name: Option<ArcStr>,
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub allow: Option<ArcStr>,
  pub allowfullscreen: Option<bool>,
  pub loading: Option<Loading>,
  pub referrerpolicy: Option<ReferrerPolicy>,
  pub sandbox: Option<ArcStr>,
}
