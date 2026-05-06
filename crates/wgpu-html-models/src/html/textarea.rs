use crate::ArcStr;
use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, HtmlDirection, TextareaWrap};

#[derive(Debug, Clone, Default)]
pub struct Textarea {
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
  pub name: Option<ArcStr>,
  /// The current text content set by editing. When `None`, layout
  /// falls back to RAWTEXT children (the HTML-parsed content).
  /// Mirrors `Input::value`.
  pub value: Option<ArcStr>,
  pub placeholder: Option<ArcStr>,
  pub required: Option<bool>,
  pub disabled: Option<bool>,
  pub readonly: Option<bool>,
  pub rows: Option<u32>,
  pub cols: Option<u32>,
  pub minlength: Option<u32>,
  pub maxlength: Option<u32>,
  pub wrap: Option<TextareaWrap>,
  pub autocomplete: Option<ArcStr>,
  pub autofocus: Option<bool>,
  pub form: Option<ArcStr>,
}
