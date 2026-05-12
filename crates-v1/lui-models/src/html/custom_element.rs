use std::collections::HashMap;

use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection},
};

#[derive(Debug, Clone)]
pub struct CustomElement {
  pub tag_name: ArcStr,
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
  pub custom_attrs: HashMap<ArcStr, ArcStr>,
}

impl CustomElement {
  pub fn new(tag_name: impl Into<ArcStr>) -> Self {
    Self {
      tag_name: tag_name.into(),
      id: None,
      style: None,
      title: None,
      lang: None,
      dir: None,
      hidden: None,
      tabindex: None,
      accesskey: None,
      contenteditable: None,
      draggable: None,
      spellcheck: None,
      translate: None,
      role: None,
      custom_attrs: HashMap::new(),
    }
  }
}
