use crate::ArcStr;
use std::collections::HashMap;

use crate::common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, ImageDecoding, Loading, ReferrerPolicy};

#[derive(Debug, Clone, Default)]
pub struct Img {
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
  pub src: Option<ArcStr>,
  pub alt: Option<ArcStr>,
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub srcset: Option<ArcStr>,
  pub sizes: Option<ArcStr>,
  pub loading: Option<Loading>,
  pub decoding: Option<ImageDecoding>,
  pub crossorigin: Option<CrossOrigin>,
  pub usemap: Option<ArcStr>,
  pub ismap: Option<bool>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
