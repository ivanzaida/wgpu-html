use crate::common::html_enums::{
    AriaRole, ButtonType, FormEncoding, FormMethod, HtmlDirection, LinkTarget,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Button {
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
    // html attr: type
    pub r#type: Option<ButtonType>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub disabled: Option<bool>,
    pub autofocus: Option<bool>,
    pub form: Option<String>,
    pub formaction: Option<String>,
    pub formenctype: Option<FormEncoding>,
    pub formmethod: Option<FormMethod>,
    pub formnovalidate: Option<bool>,
    pub formtarget: Option<LinkTarget>,
}
