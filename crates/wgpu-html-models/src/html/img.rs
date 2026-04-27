use crate::common::html_enums::{AriaRole, CrossOrigin, HtmlDirection, ImageDecoding, Loading, ReferrerPolicy};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Img {
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
  pub src: Option<String>,
  pub alt: Option<String>,
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub srcset: Option<String>,
  pub sizes: Option<String>,
  pub loading: Option<Loading>,
  pub decoding: Option<ImageDecoding>,
  pub crossorigin: Option<CrossOrigin>,
  pub usemap: Option<String>,
  pub ismap: Option<bool>,
  pub referrerpolicy: Option<ReferrerPolicy>,
}
