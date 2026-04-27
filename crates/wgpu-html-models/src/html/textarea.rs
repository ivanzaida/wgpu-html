use crate::common::html_enums::{AriaRole, HtmlDirection, TextareaWrap};

#[derive(Debug, Clone, Default)]
pub struct Textarea {
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
  pub name: Option<String>,
  pub placeholder: Option<String>,
  pub required: Option<bool>,
  pub disabled: Option<bool>,
  pub readonly: Option<bool>,
  pub rows: Option<u32>,
  pub cols: Option<u32>,
  pub minlength: Option<u32>,
  pub maxlength: Option<u32>,
  pub wrap: Option<TextareaWrap>,
  pub autocomplete: Option<String>,
  pub autofocus: Option<bool>,
  pub form: Option<String>,
}
