use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, HtmlDirection};

#[derive(Debug, Clone, Default)]
pub struct Meter {
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
  pub value: Option<f64>,
  pub min: Option<f64>,
  pub max: Option<f64>,
  pub low: Option<f64>,
  pub high: Option<f64>,
  pub optimum: Option<f64>,
}
