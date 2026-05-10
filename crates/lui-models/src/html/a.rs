use crate::ArcStr;
use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, HtmlDirection, LinkTarget, ReferrerPolicy};

#[derive(Debug, Clone, Default)]
pub struct A {
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
  pub href: Option<ArcStr>,
  pub target: Option<LinkTarget>,
  pub download: Option<ArcStr>,
  pub rel: Option<ArcStr>,
  pub hreflang: Option<ArcStr>,
  // html attr: type
  pub r#type: Option<ArcStr>,
  pub ping: Option<ArcStr>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
