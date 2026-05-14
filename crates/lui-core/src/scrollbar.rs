use crate::CssValue;

pub const DEFAULT_SCROLLBAR_WIDTH: f32 = 15.0;
pub const THIN_SCROLLBAR_WIDTH: f32 = 8.0;

pub fn resolve_scrollbar_width(v: Option<&CssValue>) -> f32 {
  match css_str(v) {
    "none" => 0.0,
    "thin" => THIN_SCROLLBAR_WIDTH,
    _ => DEFAULT_SCROLLBAR_WIDTH,
  }
}

fn css_str(v: Option<&CssValue>) -> &str {
  match v {
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}
