//! CSS gradient parsing and rasterization.
//!
//! Parses `linear-gradient()`, `radial-gradient()`, `conic-gradient()` and
//! their `repeating-*` variants from the raw CSS function string stored in
//! `CssImage::Function`. Rasterizes to an RGBA8 sRGB pixel buffer that the
//! existing image pipeline renders unchanged.

use std::collections::hash_map::DefaultHasher;
use std::f32::consts::PI;
use std::hash::{Hash, Hasher};

use crate::color::{color_to_srgb_u8, parse_color_str, Color};

// ─── Types ───────────────────────────────────────────────────────────────

pub(crate) struct Gradient {
  kind: GradientKind,
  raw_stops: Vec<RawColorStop>,
  repeating: bool,
}

enum GradientKind {
  Linear(LinearDirection),
  Radial { shape: RadialShape, size: RadialSize, cx: f32, cy: f32 },
  Conic { from_angle: f32, cx: f32, cy: f32 },
}

enum LinearDirection {
  Angle(f32),
  ToTop,
  ToBottom,
  ToLeft,
  ToRight,
  ToTopRight,
  ToTopLeft,
  ToBottomRight,
  ToBottomLeft,
}

enum RadialShape {
  Circle,
  Ellipse,
}

enum RadialSize {
  ClosestSide,
  FarthestSide,
  ClosestCorner,
  FarthestCorner,
  Lengths(f32, Option<f32>),
}

#[derive(Clone, Copy)]
enum StopPosition {
  Fraction(f32),
  Pixels(f32),
}

struct RawColorStop {
  color: Color,
  position: Option<StopPosition>,
}

struct ColorStop {
  color: Color,
  position: f32,
}

// ─── Public API ──────────────────────────────────────────────────────────

pub(crate) fn parse_gradient(func: &str) -> Option<Gradient> {
  let func = func.trim();
  let paren = func.find('(')?;
  if !func.ends_with(')') {
    return None;
  }
  let name = func[..paren].trim();
  let args = &func[paren + 1..func.len() - 1];
  let lower = name.to_ascii_lowercase();

  match lower.as_str() {
    "linear-gradient" => parse_linear(args, false),
    "repeating-linear-gradient" => parse_linear(args, true),
    "radial-gradient" => parse_radial(args, false),
    "repeating-radial-gradient" => parse_radial(args, true),
    "conic-gradient" => parse_conic(args, false),
    "repeating-conic-gradient" => parse_conic(args, true),
    _ => None,
  }
}

pub(crate) fn rasterize(gradient: &Gradient, w: u32, h: u32) -> Vec<u8> {
  let cap = (w as usize) * (h as usize) * 4;
  let mut pixels = Vec::with_capacity(cap);

  match &gradient.kind {
    GradientKind::Linear(dir) => {
      rasterize_linear(dir, &gradient.raw_stops, gradient.repeating, w, h, &mut pixels)
    }
    GradientKind::Radial { shape, size, cx, cy } => {
      rasterize_radial(shape, size, *cx, *cy, &gradient.raw_stops, gradient.repeating, w, h, &mut pixels)
    }
    GradientKind::Conic { from_angle, cx, cy } => {
      rasterize_conic(*from_angle, *cx, *cy, &gradient.raw_stops, gradient.repeating, w, h, &mut pixels)
    }
  }

  pixels
}

pub(crate) fn gradient_image_id(func: &str, w: u32, h: u32) -> u64 {
  let mut hasher = DefaultHasher::new();
  func.hash(&mut hasher);
  w.hash(&mut hasher);
  h.hash(&mut hasher);
  hasher.finish()
}

// ─── Parsing helpers ─────────────────────────────────────────────────────

fn split_top_level(s: &str, sep: char) -> Vec<&str> {
  let mut result = Vec::new();
  let mut depth: u32 = 0;
  let mut start = 0;
  for (i, c) in s.char_indices() {
    match c {
      '(' => depth += 1,
      ')' => depth = depth.saturating_sub(1),
      c if c == sep && depth == 0 => {
        result.push(s[start..i].trim());
        start = i + c.len_utf8();
      }
      _ => {}
    }
  }
  let last = s[start..].trim();
  if !last.is_empty() {
    result.push(last);
  }
  result
}

fn parse_css_angle(s: &str) -> Option<f32> {
  let s = s.trim();
  if let Some(v) = s.strip_suffix("deg") {
    return v.trim().parse::<f32>().ok().map(|d| d.to_radians());
  }
  if let Some(v) = s.strip_suffix("rad") {
    return v.trim().parse().ok();
  }
  if let Some(v) = s.strip_suffix("turn") {
    return v.trim().parse::<f32>().ok().map(|t| t * 2.0 * PI);
  }
  if let Some(v) = s.strip_suffix("grad") {
    return v.trim().parse::<f32>().ok().map(|g| g * PI / 200.0);
  }
  None
}

// ─── Linear gradient parsing ────────────────────────────────────────────

fn parse_linear(args: &str, repeating: bool) -> Option<Gradient> {
  let parts = split_top_level(args, ',');
  if parts.is_empty() {
    return None;
  }

  let (direction, stop_start) = if let Some(dir) = parse_linear_direction(parts[0]) {
    (dir, 1)
  } else {
    (LinearDirection::ToBottom, 0)
  };

  let raw_stops = parse_color_stops(&parts[stop_start..])?;
  Some(Gradient { kind: GradientKind::Linear(direction), raw_stops, repeating })
}

fn parse_linear_direction(s: &str) -> Option<LinearDirection> {
  let s = s.trim();
  let lower = s.to_ascii_lowercase();

  if let Some(rest) = lower.strip_prefix("to ") {
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    return match tokens.as_slice() {
      ["top"] => Some(LinearDirection::ToTop),
      ["bottom"] => Some(LinearDirection::ToBottom),
      ["left"] => Some(LinearDirection::ToLeft),
      ["right"] => Some(LinearDirection::ToRight),
      ["top", "right"] | ["right", "top"] => Some(LinearDirection::ToTopRight),
      ["top", "left"] | ["left", "top"] => Some(LinearDirection::ToTopLeft),
      ["bottom", "right"] | ["right", "bottom"] => Some(LinearDirection::ToBottomRight),
      ["bottom", "left"] | ["left", "bottom"] => Some(LinearDirection::ToBottomLeft),
      _ => None,
    };
  }

  parse_css_angle(&lower).map(LinearDirection::Angle)
}

// ─── Radial gradient parsing ────────────────────────────────────────────

fn parse_radial(args: &str, repeating: bool) -> Option<Gradient> {
  let parts = split_top_level(args, ',');
  if parts.is_empty() {
    return None;
  }

  let (shape, size, cx, cy, stop_start) = if let Some((s, sz, x, y)) = parse_radial_config(parts[0]) {
    (s, sz, x, y, 1)
  } else {
    (RadialShape::Ellipse, RadialSize::FarthestCorner, 0.5, 0.5, 0)
  };

  let raw_stops = parse_color_stops(&parts[stop_start..])?;
  Some(Gradient { kind: GradientKind::Radial { shape, size, cx, cy }, raw_stops, repeating })
}

fn parse_radial_config(s: &str) -> Option<(RadialShape, RadialSize, f32, f32)> {
  let s = s.trim();
  let lower = s.to_ascii_lowercase();

  let (shape_part, pos_part) = if let Some(at_idx) = lower.find(" at ") {
    (lower[..at_idx].to_string(), Some(lower[at_idx + 4..].to_string()))
  } else if let Some(rest) = lower.strip_prefix("at ") {
    (String::new(), Some(rest.to_string()))
  } else {
    (lower.clone(), None)
  };

  let (cx, cy) = pos_part.as_deref().map(parse_position_keywords).unwrap_or((0.5, 0.5));

  let shape_part = shape_part.trim();
  if shape_part.is_empty() && pos_part.is_some() {
    return Some((RadialShape::Ellipse, RadialSize::FarthestCorner, cx, cy));
  }

  let tokens: Vec<&str> = shape_part.split_whitespace().collect();
  let mut shape = RadialShape::Ellipse;
  let mut size = RadialSize::FarthestCorner;
  let mut found = false;
  let mut explicit_lengths: Vec<f32> = Vec::new();

  for token in &tokens {
    match *token {
      "circle" => {
        shape = RadialShape::Circle;
        found = true;
      }
      "ellipse" => {
        shape = RadialShape::Ellipse;
        found = true;
      }
      "closest-side" => {
        size = RadialSize::ClosestSide;
        found = true;
      }
      "farthest-side" => {
        size = RadialSize::FarthestSide;
        found = true;
      }
      "closest-corner" => {
        size = RadialSize::ClosestCorner;
        found = true;
      }
      "farthest-corner" => {
        size = RadialSize::FarthestCorner;
        found = true;
      }
      _ => {
        if let Some(len) = parse_length_value(token) {
          explicit_lengths.push(len);
          found = true;
        }
      }
    }
  }

  if !explicit_lengths.is_empty() {
    let second = if explicit_lengths.len() > 1 { Some(explicit_lengths[1]) } else { None };
    size = RadialSize::Lengths(explicit_lengths[0], second);
  }

  if found { Some((shape, size, cx, cy)) } else { None }
}

fn parse_length_value(s: &str) -> Option<f32> {
  if let Some(v) = s.strip_suffix("px") {
    return v.trim().parse().ok();
  }
  if let Some(v) = s.strip_suffix('%') {
    return v.trim().parse().ok();
  }
  s.parse().ok()
}

fn parse_position_keywords(s: &str) -> (f32, f32) {
  let tokens: Vec<&str> = s.split_whitespace().collect();
  match tokens.as_slice() {
    ["center"] => (0.5, 0.5),
    ["left"] => (0.0, 0.5),
    ["right"] => (1.0, 0.5),
    ["top"] => (0.5, 0.0),
    ["bottom"] => (0.5, 1.0),
    [x, y] => (parse_position_token(x), parse_position_token(y)),
    _ => (0.5, 0.5),
  }
}

fn parse_position_token(s: &str) -> f32 {
  match s.trim() {
    "left" | "top" => 0.0,
    "center" => 0.5,
    "right" | "bottom" => 1.0,
    s => {
      if let Some(v) = s.strip_suffix('%') {
        v.parse::<f32>().unwrap_or(50.0) / 100.0
      } else if let Some(v) = s.strip_suffix("px") {
        v.parse::<f32>().unwrap_or(0.0)
      } else {
        0.5
      }
    }
  }
}

// ─── Conic gradient parsing ─────────────────────────────────────────────

fn parse_conic(args: &str, repeating: bool) -> Option<Gradient> {
  let parts = split_top_level(args, ',');
  if parts.is_empty() {
    return None;
  }

  let (from_angle, cx, cy, stop_start) = if let Some((a, x, y)) = parse_conic_config(parts[0]) {
    (a, x, y, 1)
  } else {
    (0.0, 0.5, 0.5, 0)
  };

  let raw_stops = parse_color_stops(&parts[stop_start..])?;
  Some(Gradient { kind: GradientKind::Conic { from_angle, cx, cy }, raw_stops, repeating })
}

fn parse_conic_config(s: &str) -> Option<(f32, f32, f32)> {
  let lower = s.trim().to_ascii_lowercase();

  let (before_at, after_at) = if let Some(at_idx) = lower.find(" at ") {
    (&lower[..at_idx], Some(&lower[at_idx + 4..]))
  } else if let Some(rest) = lower.strip_prefix("at ") {
    ("", Some(rest))
  } else {
    (lower.as_str(), None)
  };

  let mut from_angle = 0.0f32;
  let mut found = false;

  if let Some(rest) = before_at.strip_prefix("from ") {
    if let Some(angle) = parse_css_angle(rest.trim()) {
      from_angle = angle;
      found = true;
    }
  }

  let (cx, cy) = if let Some(pos) = after_at {
    found = true;
    parse_position_keywords(pos.trim())
  } else {
    (0.5, 0.5)
  };

  if found { Some((from_angle, cx, cy)) } else { None }
}

// ─── Color stop parsing ─────────────────────────────────────────────────

fn parse_color_stops(parts: &[&str]) -> Option<Vec<RawColorStop>> {
  if parts.len() < 2 {
    return None;
  }
  let mut stops = Vec::with_capacity(parts.len());
  for part in parts {
    stops.push(parse_one_stop(part)?);
  }
  Some(stops)
}

fn parse_one_stop(s: &str) -> Option<RawColorStop> {
  let s = s.trim();

  let (color_str, position) = if let Some(paren_end) = s.rfind(')') {
    let after = s[paren_end + 1..].trim();
    if after.is_empty() {
      (s, None)
    } else {
      (&s[..paren_end + 1], parse_stop_position(after))
    }
  } else if let Some(last_space) = s.rfind(char::is_whitespace) {
    let maybe_pos = s[last_space + 1..].trim();
    if looks_like_position(maybe_pos) {
      (&s[..last_space], parse_stop_position(maybe_pos))
    } else {
      (s, None)
    }
  } else {
    (s, None)
  };

  let color = parse_color_str(color_str.trim())?;
  Some(RawColorStop { color, position })
}

fn looks_like_position(s: &str) -> bool {
  s.ends_with('%') || s.ends_with("px") || {
    let s = s.trim_end_matches("px").trim_end_matches('%');
    s.parse::<f32>().is_ok()
  }
}

fn parse_stop_position(s: &str) -> Option<StopPosition> {
  let s = s.trim();
  if let Some(v) = s.strip_suffix('%') {
    return v.trim().parse::<f32>().ok().map(|p| StopPosition::Fraction(p / 100.0));
  }
  if let Some(v) = s.strip_suffix("px") {
    return v.trim().parse::<f32>().ok().map(StopPosition::Pixels);
  }
  s.parse::<f32>().ok().map(StopPosition::Pixels)
}

fn resolve_positions(raw: &[RawColorStop], gradient_length: f32) -> Vec<ColorStop> {
  let n = raw.len();
  if n == 0 {
    return Vec::new();
  }

  let resolve = |pos: &StopPosition| -> f32 {
    match pos {
      StopPosition::Fraction(f) => *f,
      StopPosition::Pixels(px) => {
        if gradient_length > 0.0 {
          px / gradient_length
        } else {
          0.0
        }
      }
    }
  };

  let mut positions: Vec<Option<f32>> = raw.iter().map(|s| s.position.as_ref().map(&resolve)).collect();

  if positions[0].is_none() {
    positions[0] = Some(0.0);
  }
  if positions[n - 1].is_none() {
    positions[n - 1] = Some(1.0);
  }

  let mut i = 0;
  while i < n {
    if positions[i].is_none() {
      let start = i - 1;
      let mut end = i;
      while end < n && positions[end].is_none() {
        end += 1;
      }
      let from = positions[start].unwrap();
      let to = positions[end].unwrap();
      let count = (end - start) as f32;
      for j in i..end {
        positions[j] = Some(from + (to - from) * ((j - start) as f32) / count);
      }
      i = end + 1;
    } else {
      i += 1;
    }
  }

  let mut last = f32::NEG_INFINITY;
  raw
    .iter()
    .zip(positions)
    .map(|(stop, pos)| {
      let p = pos.unwrap_or(0.0).max(last);
      last = p;
      ColorStop { color: stop.color, position: p }
    })
    .collect()
}

// ─── Sampling ────────────────────────────────────────────────────────────

fn sample(stops: &[ColorStop], mut t: f32, repeating: bool) -> Color {
  if stops.is_empty() {
    return [0.0; 4];
  }

  if repeating && stops.len() >= 2 {
    let first = stops[0].position;
    let last = stops[stops.len() - 1].position;
    let range = last - first;
    if range > 0.0 {
      t = first + (t - first).rem_euclid(range);
    }
  }

  if t <= stops[0].position {
    return stops[0].color;
  }
  if t >= stops[stops.len() - 1].position {
    return stops[stops.len() - 1].color;
  }

  for i in 1..stops.len() {
    if t <= stops[i].position {
      let range = stops[i].position - stops[i - 1].position;
      if range <= 0.0 {
        return stops[i].color;
      }
      let local_t = (t - stops[i - 1].position) / range;
      return lerp(stops[i - 1].color, stops[i].color, local_t);
    }
  }

  stops[stops.len() - 1].color
}

fn lerp(a: Color, b: Color, t: f32) -> Color {
  [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]
}

// ─── Linear rasterization ────────────────────────────────────────────────

fn rasterize_linear(
  dir: &LinearDirection,
  raw_stops: &[RawColorStop],
  repeating: bool,
  w: u32,
  h: u32,
  out: &mut Vec<u8>,
) {
  let fw = w as f32;
  let fh = h as f32;
  let angle = resolve_linear_angle(dir, fw, fh);
  let dx = angle.sin();
  let dy = -angle.cos();
  let gradient_length = fw * dx.abs() + fh * dy.abs();
  let half_len = gradient_length / 2.0;
  let cx = fw / 2.0;
  let cy = fh / 2.0;

  let stops = resolve_positions(raw_stops, gradient_length);

  for y in 0..h {
    for x in 0..w {
      let px = x as f32 + 0.5;
      let py = y as f32 + 0.5;
      let dot = (px - cx) * dx + (py - cy) * dy;
      let t = if half_len > 0.0 { dot / half_len * 0.5 + 0.5 } else { 0.5 };
      let color = sample(&stops, t, repeating);
      out.extend_from_slice(&color_to_srgb_u8(color));
    }
  }
}

fn resolve_linear_angle(dir: &LinearDirection, w: f32, h: f32) -> f32 {
  match dir {
    LinearDirection::Angle(a) => *a,
    LinearDirection::ToTop => 0.0,
    LinearDirection::ToBottom => PI,
    LinearDirection::ToRight => PI / 2.0,
    LinearDirection::ToLeft => 3.0 * PI / 2.0,
    LinearDirection::ToTopRight => (w * 0.5).atan2(h * 0.5),
    LinearDirection::ToTopLeft => (-w * 0.5).atan2(h * 0.5),
    LinearDirection::ToBottomRight => (w * 0.5).atan2(-h * 0.5),
    LinearDirection::ToBottomLeft => (-w * 0.5).atan2(-h * 0.5),
  }
}

// ─── Radial rasterization ────────────────────────────────────────────────

fn rasterize_radial(
  shape: &RadialShape,
  size: &RadialSize,
  cx_frac: f32,
  cy_frac: f32,
  raw_stops: &[RawColorStop],
  repeating: bool,
  w: u32,
  h: u32,
  out: &mut Vec<u8>,
) {
  let fw = w as f32;
  let fh = h as f32;
  let cx = cx_frac * fw;
  let cy = cy_frac * fh;

  let (rx, ry) = compute_radial_radii(shape, size, cx, cy, fw, fh);
  let gradient_length = rx.max(ry);
  let stops = resolve_positions(raw_stops, gradient_length);

  for y in 0..h {
    for x in 0..w {
      let px = x as f32 + 0.5;
      let py = y as f32 + 0.5;
      let ndx = if rx > 0.0 { (px - cx) / rx } else { 0.0 };
      let ndy = if ry > 0.0 { (py - cy) / ry } else { 0.0 };
      let t = (ndx * ndx + ndy * ndy).sqrt();
      let color = sample(&stops, t, repeating);
      out.extend_from_slice(&color_to_srgb_u8(color));
    }
  }
}

fn compute_radial_radii(shape: &RadialShape, size: &RadialSize, cx: f32, cy: f32, w: f32, h: f32) -> (f32, f32) {
  let left = cx;
  let right = w - cx;
  let top = cy;
  let bottom = h - cy;

  match size {
    RadialSize::ClosestSide => match shape {
      RadialShape::Circle => {
        let r = left.min(right).min(top).min(bottom).max(1.0);
        (r, r)
      }
      RadialShape::Ellipse => (left.min(right).max(1.0), top.min(bottom).max(1.0)),
    },
    RadialSize::FarthestSide => match shape {
      RadialShape::Circle => {
        let r = left.max(right).max(top).max(bottom).max(1.0);
        (r, r)
      }
      RadialShape::Ellipse => (left.max(right).max(1.0), top.max(bottom).max(1.0)),
    },
    RadialSize::ClosestCorner => {
      let corners = [
        (left * left + top * top).sqrt(),
        (right * right + top * top).sqrt(),
        (left * left + bottom * bottom).sqrt(),
        (right * right + bottom * bottom).sqrt(),
      ];
      match shape {
        RadialShape::Circle => {
          let r = corners.iter().copied().fold(f32::INFINITY, f32::min).max(1.0);
          (r, r)
        }
        RadialShape::Ellipse => {
          let ratio = if h > 0.0 { w / h } else { 1.0 };
          let closest_x = left.min(right);
          let closest_y = top.min(bottom);
          let ry = ((closest_x * closest_x) / (ratio * ratio) + closest_y * closest_y).sqrt().max(1.0);
          let rx = (ry * ratio).max(1.0);
          (rx, ry)
        }
      }
    }
    RadialSize::FarthestCorner => {
      let corners = [
        (left * left + top * top).sqrt(),
        (right * right + top * top).sqrt(),
        (left * left + bottom * bottom).sqrt(),
        (right * right + bottom * bottom).sqrt(),
      ];
      match shape {
        RadialShape::Circle => {
          let r = corners.iter().copied().fold(0.0f32, f32::max).max(1.0);
          (r, r)
        }
        RadialShape::Ellipse => {
          let ratio = if h > 0.0 { w / h } else { 1.0 };
          let fx = left.max(right);
          let fy = top.max(bottom);
          let ry = ((fx * fx) / (ratio * ratio) + fy * fy).sqrt().max(1.0);
          let rx = (ry * ratio).max(1.0);
          (rx, ry)
        }
      }
    }
    RadialSize::Lengths(r1, r2) => match shape {
      RadialShape::Circle => {
        let r = r1.max(1.0);
        (r, r)
      }
      RadialShape::Ellipse => {
        let ry = r2.unwrap_or(*r1).max(1.0);
        (r1.max(1.0), ry)
      }
    },
  }
}

// ─── Conic rasterization ─────────────────────────────────────────────────

fn rasterize_conic(
  from_angle: f32,
  cx_frac: f32,
  cy_frac: f32,
  raw_stops: &[RawColorStop],
  repeating: bool,
  w: u32,
  h: u32,
  out: &mut Vec<u8>,
) {
  let fw = w as f32;
  let fh = h as f32;
  let cx = cx_frac * fw;
  let cy = cy_frac * fh;

  let stops = resolve_positions(raw_stops, 360.0);

  for y in 0..h {
    for x in 0..w {
      let px = x as f32 + 0.5;
      let py = y as f32 + 0.5;
      let dx = px - cx;
      let dy = py - cy;
      let angle = dx.atan2(-dy) - from_angle;
      let t = angle.rem_euclid(2.0 * PI) / (2.0 * PI);
      let color = sample(&stops, t, repeating);
      out.extend_from_slice(&color_to_srgb_u8(color));
    }
  }
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_linear_simple() {
    let g = parse_gradient("linear-gradient(red, blue)").unwrap();
    assert!(matches!(g.kind, GradientKind::Linear(LinearDirection::ToBottom)));
    assert_eq!(g.raw_stops.len(), 2);
    assert!(!g.repeating);
  }

  #[test]
  fn parse_linear_to_right() {
    let g = parse_gradient("linear-gradient(to right, #ff0000, #0000ff)").unwrap();
    assert!(matches!(g.kind, GradientKind::Linear(LinearDirection::ToRight)));
  }

  #[test]
  fn parse_linear_angle() {
    let g = parse_gradient("linear-gradient(45deg, red, blue)").unwrap();
    match &g.kind {
      GradientKind::Linear(LinearDirection::Angle(a)) => {
        assert!((a - 45.0_f32.to_radians()).abs() < 0.001);
      }
      _ => panic!("expected angle"),
    }
  }

  #[test]
  fn parse_radial_simple() {
    let g = parse_gradient("radial-gradient(circle, white, black)").unwrap();
    assert!(matches!(g.kind, GradientKind::Radial { .. }));
  }

  #[test]
  fn parse_conic_simple() {
    let g = parse_gradient("conic-gradient(red, yellow, green, blue, red)").unwrap();
    assert!(matches!(g.kind, GradientKind::Conic { .. }));
    assert_eq!(g.raw_stops.len(), 5);
  }

  #[test]
  fn parse_repeating_linear() {
    let g = parse_gradient("repeating-linear-gradient(45deg, red 0px, blue 20px)").unwrap();
    assert!(g.repeating);
    assert_eq!(g.raw_stops.len(), 2);
  }

  #[test]
  fn rasterize_linear_2x2() {
    let g = parse_gradient("linear-gradient(to right, black, white)").unwrap();
    let pixels = rasterize(&g, 2, 1);
    assert_eq!(pixels.len(), 8);
    // Left pixel should be darker than right pixel
    assert!(pixels[0] < pixels[4]);
  }

  #[test]
  fn rasterize_radial_1x1() {
    let g = parse_gradient("radial-gradient(circle, white, black)").unwrap();
    let pixels = rasterize(&g, 1, 1);
    assert_eq!(pixels.len(), 4);
  }

  #[test]
  fn color_stops_with_positions() {
    let g = parse_gradient("linear-gradient(to right, red 0%, green 50%, blue 100%)").unwrap();
    assert_eq!(g.raw_stops.len(), 3);
    assert!(matches!(g.raw_stops[1].position, Some(StopPosition::Fraction(f)) if (f - 0.5).abs() < 0.001));
  }

  #[test]
  fn parse_rgb_color_stops() {
    let g = parse_gradient("linear-gradient(to right, rgb(255, 0, 0), rgb(0, 0, 255))").unwrap();
    assert_eq!(g.raw_stops.len(), 2);
  }
}
