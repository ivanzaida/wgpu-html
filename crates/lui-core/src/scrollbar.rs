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

pub fn resolve_pseudo_scrollbar_width(v: Option<&CssValue>) -> Option<f32> {
  match v {
    Some(CssValue::Dimension { value, unit }) if *unit == crate::CssUnit::Px => Some((*value as f32).max(0.0)),
    Some(CssValue::Number(n)) => Some((*n as f32).max(0.0)),
    _ => None,
  }
}

fn css_str(v: Option<&CssValue>) -> &str {
  match v {
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollbarMode {
  Auto,
  Classic,
  Overlay,
  None,
}

impl ScrollbarMode {
  pub fn from_css(v: Option<&CssValue>) -> Self {
    match css_str(v) {
      "classic" => ScrollbarMode::Classic,
      "overlay" => ScrollbarMode::Overlay,
      "none" => ScrollbarMode::None,
      _ => ScrollbarMode::Auto,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollbarPartStyle {
  pub background_color: [f32; 4],
  pub border_radius: [f32; 4],
  pub opacity: f32,
}

impl Default for ScrollbarPartStyle {
  fn default() -> Self {
    Self {
      background_color: [0.0; 4],
      border_radius: [0.0; 4],
      opacity: 1.0,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ScrollbarStyles {
  pub mode: ScrollbarMode,
  pub width: f32,
  pub inset: [f32; 4],
  pub min_thumb_size: f32,
  pub thumb: ScrollbarPartStyle,
  pub track: ScrollbarPartStyle,
  pub corner: ScrollbarPartStyle,
}

impl Default for ScrollbarStyles {
  fn default() -> Self {
    Self {
      mode: ScrollbarMode::Auto,
      width: DEFAULT_SCROLLBAR_WIDTH,
      inset: [0.0; 4],
      min_thumb_size: 20.0,
      thumb: ScrollbarPartStyle {
        background_color: [0.7, 0.7, 0.7, 0.6],
        border_radius: [0.0; 4],
        opacity: 1.0,
      },
      track: ScrollbarPartStyle {
        background_color: [0.95, 0.95, 0.95, 1.0],
        border_radius: [0.0; 4],
        opacity: 1.0,
      },
      corner: ScrollbarPartStyle {
        background_color: [0.95, 0.95, 0.95, 1.0],
        border_radius: [0.0; 4],
        opacity: 1.0,
      },
    }
  }
}

pub fn resolve_scrollbar_inset(v: Option<&CssValue>) -> [f32; 4] {
  match v {
    Some(CssValue::Dimension { value, .. }) => {
      let px = *value as f32;
      [px, px, px, px]
    }
    Some(CssValue::Number(n)) => {
      let px = *n as f32;
      [px, px, px, px]
    }
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => parse_inset_string(s.as_ref()),
    _ => [0.0; 4],
  }
}

fn parse_inset_string(s: &str) -> [f32; 4] {
  let parts: Vec<f32> = s
    .split_ascii_whitespace()
    .filter_map(|tok| tok.strip_suffix("px").unwrap_or(tok).parse::<f32>().ok())
    .collect();
  match parts.len() {
    1 => [parts[0]; 4],
    2 => [parts[0], parts[1], parts[0], parts[1]],
    3 => [parts[0], parts[1], parts[2], parts[1]],
    4 => [parts[0], parts[1], parts[2], parts[3]],
    _ => [0.0; 4],
  }
}

pub fn resolve_scrollbar_min_thumb_size(v: Option<&CssValue>) -> f32 {
  match v {
    Some(CssValue::Dimension { value, .. }) => *value as f32,
    Some(CssValue::Number(n)) => *n as f32,
    Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s
      .as_ref()
      .strip_suffix("px")
      .unwrap_or(s.as_ref())
      .parse::<f32>()
      .unwrap_or(20.0),
    _ => 20.0,
  }
}
