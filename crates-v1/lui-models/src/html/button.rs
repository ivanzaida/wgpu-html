use crate::{
  ArcStr,
  common::html_enums::{AriaRole, ButtonType, FormEncoding, FormMethod, HtmlDirection, LinkTarget},
};

#[derive(Debug, Clone, Default)]
pub struct Button {
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
  // html attr: type
  pub r#type: Option<ButtonType>,
  pub name: Option<ArcStr>,
  pub value: Option<ArcStr>,
  pub disabled: Option<bool>,
  pub autofocus: Option<bool>,
  pub form: Option<ArcStr>,
  pub formaction: Option<ArcStr>,
  pub formenctype: Option<FormEncoding>,
  pub formmethod: Option<FormMethod>,
  pub formnovalidate: Option<bool>,
  pub formtarget: Option<LinkTarget>,
}
