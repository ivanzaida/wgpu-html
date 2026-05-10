//! CSS 2D transform parsing and matrix composition.

/// 2D affine transform matrix stored as `[a, b, c, d, tx, ty]`.
///
/// Corresponds to the CSS matrix:
/// ```text
/// | a  c  tx |
/// | b  d  ty |
/// | 0  0  1  |
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform2D {
  pub a: f32,
  pub b: f32,
  pub c: f32,
  pub d: f32,
  pub tx: f32,
  pub ty: f32,
}

impl Transform2D {
  pub const IDENTITY: Self = Self {
    a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx: 0.0, ty: 0.0,
  };

  pub fn translate(tx: f32, ty: f32) -> Self {
    Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, tx, ty }
  }

  pub fn scale(sx: f32, sy: f32) -> Self {
    Self { a: sx, b: 0.0, c: 0.0, d: sy, tx: 0.0, ty: 0.0 }
  }

  pub fn rotate(radians: f32) -> Self {
    let (s, c) = radians.sin_cos();
    Self { a: c, b: s, c: -s, d: c, tx: 0.0, ty: 0.0 }
  }

  pub fn skew(ax: f32, ay: f32) -> Self {
    Self { a: 1.0, b: ay.tan(), c: ax.tan(), d: 1.0, tx: 0.0, ty: 0.0 }
  }

  pub fn then(&self, other: &Self) -> Self {
    Self {
      a:  self.a * other.a  + self.c * other.b,
      b:  self.b * other.a  + self.d * other.b,
      c:  self.a * other.c  + self.c * other.d,
      d:  self.b * other.c  + self.d * other.d,
      tx: self.a * other.tx + self.c * other.ty + self.tx,
      ty: self.b * other.tx + self.d * other.ty + self.ty,
    }
  }

  pub fn is_identity(&self) -> bool {
    *self == Self::IDENTITY
  }

  pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
    (
      self.a * x + self.c * y + self.tx,
      self.b * x + self.d * y + self.ty,
    )
  }

  pub fn is_translate_only(&self) -> bool {
    self.a == 1.0 && self.b == 0.0 && self.c == 0.0 && self.d == 1.0
  }
}

/// Parse a CSS `transform` value into a composed 2D matrix.
/// `ref_w` / `ref_h` are the element's border-box dimensions,
/// used to resolve percentage-based `translate()` values.
/// Returns `None` for `none` or unparseable values.
pub fn parse_transform(value: &str, ref_w: f32, ref_h: f32) -> Option<Transform2D> {
  let value = value.trim();
  if value.eq_ignore_ascii_case("none") || value.is_empty() {
    return None;
  }

  let mut result = Transform2D::IDENTITY;
  let mut rest = value;

  while !rest.is_empty() {
    rest = rest.trim_start();
    if rest.is_empty() {
      break;
    }

    let paren = rest.find('(')?;
    let func = rest[..paren].trim().to_ascii_lowercase();
    let close = rest.find(')')?;
    let args_str = &rest[paren + 1..close];
    rest = &rest[close + 1..];

    let args = parse_args(args_str);
    let t = match func.as_str() {
      "translate" => {
        let x = parse_length_or_pct(&args, 0, ref_w);
        let y = parse_length_or_pct(&args, 1, ref_h);
        Transform2D::translate(x, y)
      }
      "translatex" => Transform2D::translate(parse_length_or_pct(&args, 0, ref_w), 0.0),
      "translatey" => Transform2D::translate(0.0, parse_length_or_pct(&args, 0, ref_h)),
      "scale" => {
        let sx = parse_num(&args, 0).unwrap_or(1.0);
        let sy = parse_num(&args, 1).unwrap_or(sx);
        Transform2D::scale(sx, sy)
      }
      "scalex" => Transform2D::scale(parse_num(&args, 0).unwrap_or(1.0), 1.0),
      "scaley" => Transform2D::scale(1.0, parse_num(&args, 0).unwrap_or(1.0)),
      "rotate" => {
        let angle = parse_angle(&args, 0);
        Transform2D::rotate(angle)
      }
      "skew" => {
        let ax = parse_angle(&args, 0);
        let ay = parse_angle(&args, 1);
        Transform2D::skew(ax, ay)
      }
      "skewx" => Transform2D::skew(parse_angle(&args, 0), 0.0),
      "skewy" => Transform2D::skew(0.0, parse_angle(&args, 0)),
      "matrix" => {
        if args.len() >= 6 {
          let a = parse_num(&args, 0).unwrap_or(1.0);
          let b = parse_num(&args, 1).unwrap_or(0.0);
          let c = parse_num(&args, 2).unwrap_or(0.0);
          let d = parse_num(&args, 3).unwrap_or(1.0);
          let tx = parse_num(&args, 4).unwrap_or(0.0);
          let ty = parse_num(&args, 5).unwrap_or(0.0);
          Transform2D { a, b, c, d, tx, ty }
        } else {
          continue;
        }
      }
      _ => continue,
    };
    result = result.then(&t);
  }

  if result.is_identity() {
    None
  } else {
    Some(result)
  }
}

/// Parse `transform-origin` into `(ox, oy)` in pixels, relative to
/// the element's border box. Defaults to `(w/2, h/2)` (center).
pub fn parse_transform_origin(value: Option<&str>, w: f32, h: f32) -> (f32, f32) {
  let value = match value {
    Some(v) if !v.is_empty() => v.trim(),
    _ => return (w * 0.5, h * 0.5),
  };

  let parts: Vec<&str> = value.split_whitespace().collect();
  let ox = resolve_origin_axis(parts.first().copied().unwrap_or("50%"), w);
  let oy = resolve_origin_axis(parts.get(1).copied().unwrap_or("50%"), h);
  (ox, oy)
}

fn resolve_origin_axis(token: &str, extent: f32) -> f32 {
  let t = token.trim().to_ascii_lowercase();
  match t.as_str() {
    "left" | "top" => 0.0,
    "center" => extent * 0.5,
    "right" | "bottom" => extent,
    _ => {
      if let Some(pct) = t.strip_suffix('%') {
        pct.trim().parse::<f32>().unwrap_or(50.0) / 100.0 * extent
      } else {
        let num = t.strip_suffix("px").unwrap_or(&t);
        num.trim().parse::<f32>().unwrap_or(extent * 0.5)
      }
    }
  }
}

fn parse_args(s: &str) -> Vec<String> {
  s.split(',')
    .map(|a| a.trim().to_string())
    .collect()
}

fn parse_num(args: &[String], idx: usize) -> Option<f32> {
  args.get(idx)?.trim().parse::<f32>().ok()
}

fn parse_length_or_pct(args: &[String], idx: usize, reference: f32) -> f32 {
  let s = match args.get(idx) {
    Some(s) => s.trim(),
    None => return 0.0,
  };
  if let Some(pct) = s.strip_suffix('%') {
    return pct.trim().parse::<f32>().unwrap_or(0.0) / 100.0 * reference;
  }
  let num = s.strip_suffix("px").unwrap_or(s);
  num.trim().parse::<f32>().unwrap_or(0.0)
}

fn parse_px_or_zero(args: &[String], idx: usize) -> f32 {
  let s = match args.get(idx) {
    Some(s) => s.trim(),
    None => return 0.0,
  };
  let num = s.strip_suffix("px").unwrap_or(s);
  num.trim().parse::<f32>().unwrap_or(0.0)
}

fn parse_angle(args: &[String], idx: usize) -> f32 {
  let s = match args.get(idx) {
    Some(s) => s.trim(),
    None => return 0.0,
  };
  if let Some(deg) = s.strip_suffix("deg") {
    return deg.trim().parse::<f32>().unwrap_or(0.0).to_radians();
  }
  if let Some(rad) = s.strip_suffix("rad") {
    return rad.trim().parse::<f32>().unwrap_or(0.0);
  }
  if let Some(turn) = s.strip_suffix("turn") {
    return turn.trim().parse::<f32>().unwrap_or(0.0) * std::f32::consts::TAU;
  }
  if let Some(grad) = s.strip_suffix("grad") {
    return grad.trim().parse::<f32>().unwrap_or(0.0) * std::f32::consts::PI / 200.0;
  }
  s.parse::<f32>().unwrap_or(0.0).to_radians()
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::f32::consts::PI;

  fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
  }

  #[test]
  fn parse_none() {
    assert!(parse_transform("none", 100.0, 100.0).is_none());
  }

  #[test]
  fn parse_translate() {
    let t = parse_transform("translate(10px, 20px)", 200.0, 100.0).unwrap();
    assert!(approx(t.tx, 10.0));
    assert!(approx(t.ty, 20.0));
    assert!(t.is_translate_only());
  }

  #[test]
  fn parse_scale() {
    let t = parse_transform("scale(2)", 200.0, 100.0).unwrap();
    assert!(approx(t.a, 2.0));
    assert!(approx(t.d, 2.0));
  }

  #[test]
  fn parse_rotate_deg() {
    let t = parse_transform("rotate(90deg)", 200.0, 100.0).unwrap();
    assert!(approx(t.a, 0.0));
    assert!(approx(t.b, 1.0));
  }

  #[test]
  fn parse_chained() {
    let t = parse_transform("translate(100px, 0) scale(2)", 200.0, 100.0).unwrap();
    let (x, y) = t.apply(0.0, 0.0);
    assert!(approx(x, 100.0));
    assert!(approx(y, 0.0));
  }

  #[test]
  fn parse_matrix() {
    let t = parse_transform("matrix(1, 0, 0, 1, 50, 60)", 200.0, 100.0).unwrap();
    assert!(approx(t.tx, 50.0));
    assert!(approx(t.ty, 60.0));
  }

  #[test]
  fn origin_defaults_to_center() {
    let (ox, oy) = parse_transform_origin(None, 200.0, 100.0);
    assert!(approx(ox, 100.0));
    assert!(approx(oy, 50.0));
  }

  #[test]
  fn origin_keywords() {
    let (ox, oy) = parse_transform_origin(Some("left top"), 200.0, 100.0);
    assert!(approx(ox, 0.0));
    assert!(approx(oy, 0.0));
  }

  #[test]
  fn origin_percentage() {
    let (ox, oy) = parse_transform_origin(Some("25% 75%"), 200.0, 100.0);
    assert!(approx(ox, 50.0));
    assert!(approx(oy, 75.0));
  }

  #[test]
  fn translate_percentage() {
    let t = parse_transform("translate(-50%, -50%)", 200.0, 100.0).unwrap();
    assert!(approx(t.tx, -100.0));
    assert!(approx(t.ty, -50.0));
  }
}
