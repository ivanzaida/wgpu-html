
use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection, TrackKind},
};

#[derive(Debug, Clone, Default)]
pub struct Track {
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
  pub src: Option<ArcStr>,
  pub kind: Option<TrackKind>,
  pub srclang: Option<ArcStr>,
  pub label: Option<ArcStr>,
  pub default: Option<bool>,
}
