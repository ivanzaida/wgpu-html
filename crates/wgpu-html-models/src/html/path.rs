use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, HtmlDirection};

/// SVG `<path>` element.
#[derive(Debug, Clone, Default)]
pub struct SvgPath {
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
  /// aria-* attributes (suffix → value).
  pub aria_attrs: HashMap<String, String>,
  /// data-* attributes (suffix → value).
  pub data_attrs: HashMap<String, String>,
  /// SVG `d` attribute — the path data string.
  pub d: Option<String>,
  pub fill: Option<String>,
  pub stroke: Option<String>,
  pub stroke_width: Option<String>,
  pub fill_rule: Option<String>,
  pub opacity: Option<String>,
  pub transform: Option<String>,
}
