use crate::common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, ReferrerPolicy};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Script {
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
  // html attr: type
  pub r#type: Option<String>,
  // html attr: async
  pub r#async: Option<bool>,
  pub defer: Option<bool>,
  pub crossorigin: Option<CrossOrigin>,
  pub integrity: Option<String>,
  pub nomodule: Option<bool>,
  pub nonce: Option<String>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
