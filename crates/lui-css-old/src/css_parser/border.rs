use super::{parse_border_style, parse_css_color, parse_css_length};
use crate::values::*;

/// Split `value` on whitespace, but only at parenthesis depth 0 -- so
/// `rgb(1, 2, 3)`, `hsl(...)`, `calc(...)` survive intact as a single
/// token. Used by shorthand parsers (`border`, ...) where the value
/// can mix bare keywords / lengths and functional values.
pub(crate) fn split_top_level_whitespace(value: &str) -> Vec<&str> {
  let bytes = value.as_bytes();
  let mut out: Vec<&str> = Vec::new();
  let mut start: Option<usize> = None;
  let mut depth: i32 = 0;
  for (i, &b) in bytes.iter().enumerate() {
    match b {
      b'(' => {
        if start.is_none() {
          start = Some(i);
        }
        depth += 1;
      }
      b')' => {
        if depth > 0 {
          depth -= 1;
        }
      }
      _ if (b as char).is_ascii_whitespace() && depth == 0 => {
        if let Some(s_idx) = start.take() {
          out.push(&value[s_idx..i]);
        }
      }
      _ => {
        if start.is_none() {
          start = Some(i);
        }
      }
    }
  }
  if let Some(s_idx) = start {
    out.push(&value[s_idx..]);
  }
  out
}

/// `parse_css_length` minus its catch-all `Raw` / `Auto` returns -- used
/// when matching one piece of a shorthand against multiple value kinds.
pub(crate) fn parse_definite_length(token: &str) -> Option<CssLength> {
  match parse_css_length(token)? {
    CssLength::Raw(_) | CssLength::Auto => None,
    other => Some(other),
  }
}

/// Tokenise a `border` / `border-<side>` value into (width, style, color)
/// pieces. Each top-level-whitespace-separated token is tried first as a
/// length (rejecting the `Raw` / `Auto` fallback so non-numeric tokens
/// don't get gobbled), then as a border-style keyword, and finally as a
/// color. "Top-level" means the splitter ignores spaces inside `(...)`,
/// so functional values like `rgb(212, 175, 55)` stay intact.
pub(crate) fn parse_border_pieces(value: &str) -> (Option<CssLength>, Option<BorderStyle>, Option<CssColor>) {
  let mut w = None;
  let mut s = None;
  let mut c = None;
  for token in split_top_level_whitespace(value) {
    if w.is_none() {
      if let Some(v) = parse_definite_length(token) {
        w = Some(v);
        continue;
      }
    }
    if s.is_none() {
      if let Some(v) = parse_border_style(token) {
        s = Some(v);
        continue;
      }
    }
    if c.is_none() {
      if let Some(v) = parse_css_color(token) {
        c = Some(v);
        continue;
      }
    }
  }
  (w, s, c)
}

/// Parse the CSS box shorthand (`padding` / `margin`) into per-side lengths.
/// Accepts 1, 2, 3, or 4 whitespace-separated values per CSS spec:
/// - 1: all sides
/// - 2: vertical, horizontal
/// - 3: top, horizontal, bottom
/// - 4: top, right, bottom, left
///
/// Returns `(top, right, bottom, left)`. Any unparseable token in a slot
/// becomes `None` for that side.
pub fn parse_box_shorthand(
  value: &str,
) -> (
  Option<CssLength>,
  Option<CssLength>,
  Option<CssLength>,
  Option<CssLength>,
) {
  let parts = split_top_level_whitespace(value);
  match parts.len() {
    1 => {
      let v = parse_css_length(parts[0]);
      (v.clone(), v.clone(), v.clone(), v)
    }
    2 => {
      let v = parse_css_length(parts[0]);
      let h = parse_css_length(parts[1]);
      (v.clone(), h.clone(), v, h)
    }
    3 => {
      let t = parse_css_length(parts[0]);
      let h = parse_css_length(parts[1]);
      let b = parse_css_length(parts[2]);
      (t, h.clone(), b, h)
    }
    4 => (
      parse_css_length(parts[0]),
      parse_css_length(parts[1]),
      parse_css_length(parts[2]),
      parse_css_length(parts[3]),
    ),
    _ => (None, None, None, None),
  }
}
