use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};

use super::color;

pub fn css_str(v: Option<&CssValue>) -> &str {
  match v {
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}

pub fn css_f32(v: Option<&CssValue>) -> f32 {
  match v {
    Some(CssValue::Dimension {
      value,
      unit: CssUnit::Px,
    }) => *value as f32,
    Some(CssValue::Number(n)) => *n as f32,
    _ => 0.0,
  }
}

/// CSS initial value of `font-size` is `medium` = 16px.
pub fn css_font_size(v: Option<&CssValue>) -> f32 {
  match v {
    Some(CssValue::Dimension {
      value,
      unit: CssUnit::Px,
    }) => (*value as f32).max(1.0),
    Some(CssValue::Number(n)) => (*n as f32).max(1.0),
    _ => 16.0,
  }
}

pub fn css_color(v: Option<&CssValue>) -> Option<[f32; 4]> {
  color::resolve_color(v)
}

pub fn css_opacity(style: &ComputedStyle) -> f32 {
  match style.opacity {
    Some(CssValue::Number(n)) => (*n as f32).clamp(0.0, 1.0),
    _ => 1.0,
  }
}

pub fn is_visible(style: &ComputedStyle) -> bool {
  css_str(style.visibility) != "hidden"
}

pub fn border_radii(style: &ComputedStyle, bw: f32, bh: f32) -> ([f32; 4], [f32; 4]) {
  let resolve = |v: Option<&CssValue>, reference: f32| -> f32 {
    match v {
      Some(CssValue::Dimension {
        value,
        unit: CssUnit::Px,
      }) => *value as f32,
      Some(CssValue::Percentage(p)) => (*p as f32 / 100.0) * reference,
      Some(CssValue::Number(n)) => *n as f32,
      _ => 0.0,
    }
  };
  let h = [
    resolve(style.border_top_left_radius, bw),
    resolve(style.border_top_right_radius, bw),
    resolve(style.border_bottom_right_radius, bw),
    resolve(style.border_bottom_left_radius, bw),
  ];
  let v = [
    resolve(style.border_top_left_radius, bh),
    resolve(style.border_top_right_radius, bh),
    resolve(style.border_bottom_right_radius, bh),
    resolve(style.border_bottom_left_radius, bh),
  ];
  (h, v)
}

pub fn padding_box_radii(
  outer_h: [f32; 4],
  outer_v: [f32; 4],
  border: &lui_layout::RectEdges<f32>,
) -> ([f32; 4], [f32; 4]) {
  let h = [
    (outer_h[0] - border.left).max(0.0),
    (outer_h[1] - border.right).max(0.0),
    (outer_h[2] - border.right).max(0.0),
    (outer_h[3] - border.left).max(0.0),
  ];
  let v = [
    (outer_v[0] - border.top).max(0.0),
    (outer_v[1] - border.top).max(0.0),
    (outer_v[2] - border.bottom).max(0.0),
    (outer_v[3] - border.bottom).max(0.0),
  ];
  (h, v)
}
