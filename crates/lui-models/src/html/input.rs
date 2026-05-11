use std::collections::HashMap;

use crate::{
  ArcStr,
  common::html_enums::{AriaRole, CaptureMode, HtmlDirection, InputType},
};

#[derive(Debug, Clone)]
pub struct FileInfo {
  pub name: ArcStr,
  pub size: u64,
  pub mime_type: ArcStr,
  pub last_modified: u64,
  pub path: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct Input {
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
  // html attr: type
  pub r#type: Option<InputType>,
  pub name: Option<ArcStr>,
  pub value: Option<ArcStr>,
  pub placeholder: Option<ArcStr>,
  pub required: Option<bool>,
  pub disabled: Option<bool>,
  pub readonly: Option<bool>,
  pub checked: Option<bool>,
  pub min: Option<ArcStr>,
  pub max: Option<ArcStr>,
  pub step: Option<ArcStr>,
  pub minlength: Option<u32>,
  pub maxlength: Option<u32>,
  pub pattern: Option<ArcStr>,
  pub autocomplete: Option<ArcStr>,
  pub autofocus: Option<bool>,
  pub multiple: Option<bool>,
  pub accept: Option<ArcStr>,
  pub capture: Option<CaptureMode>,
  pub size: Option<u32>,
  pub list: Option<ArcStr>,
  pub form: Option<ArcStr>,
  pub files: Vec<FileInfo>,
}
