use crate::ArcStr;
use crate::common::html_enums::{AriaRole, HtmlDirection};
use std::collections::HashMap;

/// Generic SVG child element (circle, rect, g, defs, text, etc.).
/// Stores the tag name and all attributes as raw key-value pairs
/// so the serializer can emit valid XML for resvg to rasterize.
/// Global HTML attrs are present for macro compatibility.
#[derive(Debug, Clone, Default)]
pub struct SvgElement {
  pub tag: ArcStr,
  pub attrs: Vec<(ArcStr, ArcStr)>,
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
  pub aria_attrs: HashMap<ArcStr, ArcStr>,
  pub data_attrs: HashMap<ArcStr, ArcStr>,
}
