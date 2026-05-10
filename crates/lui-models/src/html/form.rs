use crate::ArcStr;
use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, AutoComplete, FormEncoding, FormMethod, HtmlDirection, LinkTarget};

#[derive(Debug, Clone, Default)]
pub struct Form {
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
  pub action: Option<ArcStr>,
  pub method: Option<FormMethod>,
  pub enctype: Option<FormEncoding>,
  pub target: Option<LinkTarget>,
  pub autocomplete: Option<AutoComplete>,
  pub novalidate: Option<bool>,
  pub name: Option<ArcStr>,
  pub rel: Option<ArcStr>,
}
