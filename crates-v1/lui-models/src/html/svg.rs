
use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection, SvgLength},
};

#[derive(Debug, Clone, Default)]
pub struct Svg {
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
  pub width: Option<SvgLength>,
  pub height: Option<SvgLength>,
  // html attr: viewBox
  pub view_box: Option<ArcStr>,
  pub xmlns: Option<ArcStr>,
  pub fill: Option<ArcStr>,
  pub stroke: Option<ArcStr>,
}
