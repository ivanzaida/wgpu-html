//! CSS `box-shadow` parsing.

use crate::color::{self, parse_color_str};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxShadow {
  pub offset_x: f32,
  pub offset_y: f32,
  pub blur: f32,
  pub spread: f32,
  pub color: [f32; 4],
  pub inset: bool,
}

pub fn parse_box_shadows(value: &str) -> Vec<BoxShadow> {
  if value.trim().eq_ignore_ascii_case("none") || value.trim().is_empty() {
    return Vec::new();
  }
  let mut results = Vec::new();
  for part in split_shadow_list(value) {
    if let Some(s) = parse_single(part.trim()) {
      results.push(s);
    }
  }
  results
}

fn split_shadow_list(value: &str) -> Vec<&str> {
  let mut parts = Vec::new();
  let mut depth = 0i32;
  let mut start = 0;
  for (i, ch) in value.char_indices() {
    match ch {
      '(' => depth += 1,
      ')' => depth -= 1,
      ',' if depth == 0 => {
        parts.push(&value[start..i]);
        start = i + 1;
      }
      _ => {}
    }
  }
  parts.push(&value[start..]);
  parts
}

fn parse_single(s: &str) -> Option<BoxShadow> {
  let mut inset = false;
  let mut lengths: Vec<f32> = Vec::new();
  let mut color_str: Option<String> = None;

  let s = s.trim();
  let mut rest = s;

  while !rest.is_empty() {
    rest = rest.trim_start();
    if rest.is_empty() {
      break;
    }

    if rest.starts_with("inset") && rest[5..].starts_with(|c: char| c.is_whitespace() || c == ',') || rest == "inset" {
      inset = true;
      rest = &rest[5..];
      continue;
    }

    if rest.starts_with("rgb") || rest.starts_with("hsl") {
      if let Some(end) = rest.find(')') {
        let token = &rest[..=end];
        color_str = Some(token.to_string());
        rest = &rest[end + 1..];
        continue;
      }
    }

    if rest.starts_with('#') {
      let end = rest[1..].find(|c: char| c.is_whitespace() || c == ',').map(|i| i + 1).unwrap_or(rest.len());
      color_str = Some(rest[..end].to_string());
      rest = &rest[end..];
      continue;
    }

    let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
    let token = &rest[..end];
    rest = &rest[end..];

    if let Some(px) = parse_px(token) {
      lengths.push(px);
    } else if parse_color_str(token).is_some() {
      color_str = Some(token.to_string());
    }
  }

  if lengths.len() < 2 {
    return None;
  }

  let resolved_color = color_str
    .and_then(|c| parse_color_str(&c))
    .unwrap_or([0.0, 0.0, 0.0, 1.0]);

  Some(BoxShadow {
    offset_x: lengths[0],
    offset_y: lengths[1],
    blur: lengths.get(2).copied().unwrap_or(0.0).max(0.0),
    spread: lengths.get(3).copied().unwrap_or(0.0),
    color: resolved_color,
    inset,
  })
}

fn parse_px(s: &str) -> Option<f32> {
  let s = s.trim();
  if s == "0" {
    return Some(0.0);
  }
  let num = s.strip_suffix("px").unwrap_or(s);
  num.parse::<f32>().ok()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
  }

  #[test]
  fn parse_simple_shadow() {
    let shadows = parse_box_shadows("2px 4px 8px rgba(0,0,0,0.5)");
    assert_eq!(shadows.len(), 1);
    let s = &shadows[0];
    assert!(approx(s.offset_x, 2.0));
    assert!(approx(s.offset_y, 4.0));
    assert!(approx(s.blur, 8.0));
    assert!(approx(s.spread, 0.0));
    assert!(!s.inset);
  }

  #[test]
  fn parse_with_spread() {
    let shadows = parse_box_shadows("0 0 10px 5px black");
    assert_eq!(shadows.len(), 1);
    let s = &shadows[0];
    assert!(approx(s.blur, 10.0));
    assert!(approx(s.spread, 5.0));
  }

  #[test]
  fn parse_inset() {
    let shadows = parse_box_shadows("inset 0 2px 4px rgba(0,0,0,0.3)");
    assert_eq!(shadows.len(), 1);
    assert!(shadows[0].inset);
  }

  #[test]
  fn parse_multiple_shadows() {
    let shadows = parse_box_shadows("2px 2px 4px black, 0 0 10px red");
    assert_eq!(shadows.len(), 2);
    assert!(approx(shadows[0].offset_x, 2.0));
    assert!(approx(shadows[1].blur, 10.0));
  }

  #[test]
  fn parse_none() {
    assert!(parse_box_shadows("none").is_empty());
  }

  #[test]
  fn parse_hex_color() {
    let shadows = parse_box_shadows("0 4px 6px #00000040");
    assert_eq!(shadows.len(), 1);
    assert!(shadows[0].color[3] < 1.0, "alpha should be < 1 for #00000040");
  }
}
