
use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection},
};

#[derive(Debug, Clone, Default)]
pub struct Meter {
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
  pub value: Option<f64>,
  pub min: Option<f64>,
  pub max: Option<f64>,
  pub low: Option<f64>,
  pub high: Option<f64>,
  pub optimum: Option<f64>,
}
