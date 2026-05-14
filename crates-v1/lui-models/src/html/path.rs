use crate::{
  ArcStr,
  common::html_enums::{AriaRole, HtmlDirection},
};

/// SVG `<path>` element.
#[derive(Debug, Clone, Default)]
pub struct SvgPath {
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
  /// aria-* attributes (suffix → value).
  /// data-* attributes (suffix → value).
  /// SVG `d` attribute — the path data string.
  pub d: Option<ArcStr>,
  pub fill: Option<ArcStr>,
  pub stroke: Option<ArcStr>,
  pub stroke_width: Option<ArcStr>,
  pub fill_rule: Option<ArcStr>,
  pub opacity: Option<ArcStr>,
  pub transform: Option<ArcStr>,
}
