use crate::common::html_enums::{AriaRole, CaptureMode, HtmlDirection, InputType};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Input {
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
    pub r#type: Option<InputType>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub placeholder: Option<String>,
    pub required: Option<bool>,
    pub disabled: Option<bool>,
    pub readonly: Option<bool>,
    pub checked: Option<bool>,
    pub min: Option<String>,
    pub max: Option<String>,
    pub step: Option<String>,
    pub minlength: Option<u32>,
    pub maxlength: Option<u32>,
    pub pattern: Option<String>,
    pub autocomplete: Option<String>,
    pub autofocus: Option<bool>,
    pub multiple: Option<bool>,
    pub accept: Option<String>,
    pub capture: Option<CaptureMode>,
    pub size: Option<u32>,
    pub list: Option<String>,
    pub form: Option<String>,
}
