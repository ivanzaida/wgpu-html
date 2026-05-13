//! Resolve margin, border, and padding from `ComputedStyle` into `RectEdges<f32>`.
//!
//! Each function reads the four longhand properties (e.g. `margin-top`,
//! `margin-right`, …), extracts pixel values from `CssValue::Dimension(Px)`,
//! and handles `auto` margins (returning 0.0 with an `auto_mask` for later
//! block-centering).
//!
//! Per CSS spec, percentage margins and padding resolve against the
//! containing block's **width** (even for vertical sides).

use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};

use crate::geometry::RectEdges;

/// Resolved margin edges. If a side is `auto`, its value is 0.0 and the
/// corresponding bit in `auto_mask` is set.
pub struct MarginResult {
  pub edges: RectEdges<f32>,
  /// Bitmask: bit 0 = top, 1 = right, 2 = bottom, 3 = left.
  pub auto_mask: u8,
}

/// Resolve margin from computed style.
pub fn resolve_margin(style: &ComputedStyle) -> MarginResult {
  resolve_margin_against(style, 0.0)
}

/// Resolve margin with a containing width for percentage resolution.
pub fn resolve_margin_against(style: &ComputedStyle, containing_width: f32) -> MarginResult {
  let mut auto_mask: u8 = 0;
  let mut m = |i: usize, val: Option<&CssValue>| -> f32 {
    match val {
      Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
      Some(&CssValue::Number(n)) => n as f32,
      Some(&CssValue::Percentage(p)) => (p as f32 / 100.0) * containing_width,
      Some(CssValue::Unknown(s)) | Some(CssValue::String(s)) if s.as_ref() == "auto" => {
        auto_mask |= 1 << i;
        0.0
      }
      _ => 0.0,
    }
  };
  MarginResult {
    edges: RectEdges::new(
      m(0, style.margin_top),
      m(1, style.margin_right),
      m(2, style.margin_bottom),
      m(3, style.margin_left),
    ),
    auto_mask,
  }
}

/// Resolve border widths from computed style.
pub fn resolve_border(style: &ComputedStyle) -> RectEdges<f32> {
  RectEdges::new(
    resolve_size(style.border_top_width),
    resolve_size(style.border_right_width),
    resolve_size(style.border_bottom_width),
    resolve_size(style.border_left_width),
  )
}

/// Resolve padding from computed style.
pub fn resolve_padding(style: &ComputedStyle) -> RectEdges<f32> {
  resolve_padding_against(style, 0.0)
}

/// Resolve padding with a containing width for percentage resolution.
pub fn resolve_padding_against(style: &ComputedStyle, containing_width: f32) -> RectEdges<f32> {
  RectEdges::new(
    resolve_size_pct(style.padding_top, containing_width),
    resolve_size_pct(style.padding_right, containing_width),
    resolve_size_pct(style.padding_bottom, containing_width),
    resolve_size_pct(style.padding_left, containing_width),
  )
}

fn resolve_size(val: Option<&CssValue>) -> f32 {
  match val {
    Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
    Some(&CssValue::Number(n)) => n as f32,
    _ => 0.0,
  }
}

fn resolve_size_pct(val: Option<&CssValue>, containing_width: f32) -> f32 {
  match val {
    Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
    Some(&CssValue::Number(n)) => n as f32,
    Some(&CssValue::Percentage(p)) => (p as f32 / 100.0) * containing_width,
    _ => 0.0,
  }
}
