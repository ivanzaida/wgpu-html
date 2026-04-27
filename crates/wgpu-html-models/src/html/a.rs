use crate::common::html_enums::{AriaRole, HtmlDirection, LinkTarget, ReferrerPolicy};

#[derive(Debug, Clone, Default)]
pub struct A {
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
  pub href: Option<String>,
  pub target: Option<LinkTarget>,
  pub download: Option<String>,
  pub rel: Option<String>,
  pub hreflang: Option<String>,
  // html attr: type
  pub r#type: Option<String>,
  pub ping: Option<String>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
