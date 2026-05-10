use lui_models::ArcStr;
use lui_models::Style;
use lui_models::common::css_enums::*;

use crate::shorthands::shorthand_members;
use crate::style_props::clear_value_for;

use super::{
    parse_css_color,
    parse_css_length,
    parse_css_image,
    parse_background_repeat,
    parse_background_clip,
    parse_border_style,
    strip_func,
    is_preserved_color_function,
};

fn mark_shorthand_reset(style: &mut Style, property: &str) {
  if let Some(members) = shorthand_members(property) {
    for member in members {
      style.reset_properties.insert(ArcStr::from(*member));
    }
  }
}

fn mark_property_resets(style: &mut Style, props: &[&str]) {
  for prop in props {
    style.reset_properties.insert(ArcStr::from(*prop));
  }
}

// ---------------------------------------------------------------------------
// CSS value parsers
// ---------------------------------------------------------------------------

/// Which side a per-side border helper writes to.
#[derive(Copy, Clone)]
pub(crate) enum Side {
  Top,
  Right,
  Bottom,
  Left,
}

/// Parse the `border` shorthand into width / style / color, in any order.
/// The values fan out to all four sides (per CSS spec: `border` is itself
/// a shorthand for the four `border-<side>-<piece>` longhands).
pub(crate) fn parse_border_shorthand(value: &str, style: &mut Style) {
  mark_shorthand_reset(style, "border");
  style.border = Some(ArcStr::from(value));
  let (w, s, c) = parse_border_pieces(value);
  if let Some(w) = w {
    style.border_top_width = Some(w.clone());
    style.border_right_width = Some(w.clone());
    style.border_bottom_width = Some(w.clone());
    style.border_left_width = Some(w);
  }
  if let Some(s) = s {
    style.border_top_style = Some(s.clone());
    style.border_right_style = Some(s.clone());
    style.border_bottom_style = Some(s.clone());
    style.border_left_style = Some(s);
  }
  if let Some(c) = c {
    style.border_top_color = Some(c.clone());
    style.border_right_color = Some(c.clone());
    style.border_bottom_color = Some(c.clone());
    style.border_left_color = Some(c);
  }
}

/// Per-side `border-<side>` shorthand. Sets only that side's width / style /
/// color, falling back to whatever was set previously (so the cascade order
/// `border: 2px solid red; border-top: 4px dashed blue;` works correctly).
pub(crate) fn parse_border_side_shorthand(value: &str, style: &mut Style, side: Side) {
  let resets = match side {
    Side::Top => ["border-top-width", "border-top-style", "border-top-color"],
    Side::Right => ["border-right-width", "border-right-style", "border-right-color"],
    Side::Bottom => ["border-bottom-width", "border-bottom-style", "border-bottom-color"],
    Side::Left => ["border-left-width", "border-left-style", "border-left-color"],
  };
  mark_property_resets(style, &resets);
  let (w, s, c) = parse_border_pieces(value);
  match side {
    Side::Top => {
      if let Some(w) = w {
        style.border_top_width = Some(w);
      }
      if let Some(s) = s {
        style.border_top_style = Some(s);
      }
      if let Some(c) = c {
        style.border_top_color = Some(c);
      }
    }
    Side::Right => {
      if let Some(w) = w {
        style.border_right_width = Some(w);
      }
      if let Some(s) = s {
        style.border_right_style = Some(s);
      }
      if let Some(c) = c {
        style.border_right_color = Some(c);
      }
    }
    Side::Bottom => {
      if let Some(w) = w {
        style.border_bottom_width = Some(w);
      }
      if let Some(s) = s {
        style.border_bottom_style = Some(s);
      }
      if let Some(c) = c {
        style.border_bottom_color = Some(c);
      }
    }
    Side::Left => {
      if let Some(w) = w {
        style.border_left_width = Some(w);
      }
      if let Some(s) = s {
        style.border_left_style = Some(s);
      }
      if let Some(c) = c {
        style.border_left_color = Some(c);
      }
    }
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

pub(crate) fn apply_background_shorthand(value: &str, style: &mut Style) {
  clear_value_for("background", style);
  style.background = Some(ArcStr::from(value));

  for token in split_top_level_whitespace(value) {
    if token == "/" {
      continue;
    }
    if style.background_repeat.is_none() {
      if let Some(repeat) = parse_background_repeat(token) {
        style.background_repeat = Some(repeat);
        continue;
      }
    }
    if style.background_clip.is_none() {
      if let Some(clip) = parse_background_clip(token) {
        style.background_clip = Some(clip);
        continue;
      }
    }
    if style.background_color.is_none() {
      if let Some(color) = parse_background_shorthand_color(token) {
        style.background_color = Some(color);
        continue;
      }
    }
    if let Some(image) = parse_background_shorthand_image(token) {
      style.background_image = image;
    }
  }
}

fn parse_background_shorthand_color(value: &str) -> Option<CssColor> {
  let v = value.trim();
  if v.starts_with('#')
    || v.eq_ignore_ascii_case("transparent")
    || v.eq_ignore_ascii_case("currentcolor")
    || strip_func(v, "rgb").is_some()
    || strip_func(v, "rgba").is_some()
    || strip_func(v, "hsl").is_some()
    || strip_func(v, "hsla").is_some()
    || is_preserved_color_function(v)
    || is_supported_named_color(v)
  {
    parse_css_color(v)
  } else {
    None
  }
}

fn parse_background_shorthand_image(value: &str) -> Option<Option<CssImage>> {
  let v = value.trim();
  if v.eq_ignore_ascii_case("none") {
    return Some(None);
  }
  parse_css_image(v).map(Some)
}

fn is_supported_named_color(value: &str) -> bool {
  matches!(
    value.to_ascii_lowercase().as_str(),
    "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "cyan"
            | "aqua"
            | "magenta"
            | "fuchsia"
            | "gray"
            | "grey"
            | "lightgray"
            | "lightgrey"
            | "darkgray"
            | "darkgrey"
            | "silver"
            | "maroon"
            | "olive"
            | "lime"
            | "teal"
            | "navy"
            | "purple"
            | "orange"
            | "pink"
            // CSS Color Module Level 4 system colors. Used by the UA
            // stylesheet for form controls (`buttonface`, `field`, …).
            | "canvas"
            | "canvastext"
            | "linktext"
            | "visitedtext"
            | "activetext"
            | "buttonface"
            | "buttontext"
            | "buttonborder"
            | "field"
            | "fieldtext"
            | "highlight"
            | "highlighttext"
            | "selecteditem"
            | "selecteditemtext"
            | "mark"
            | "marktext"
            | "graytext"
            | "accentcolor"
            | "accentcolortext"
  )
}

/// Split `value` on whitespace, but only at parenthesis depth 0 — so
/// `rgb(1, 2, 3)`, `hsl(...)`, `calc(...)` survive intact as a single
/// token. Used by shorthand parsers (`border`, …) where the value
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

/// `border-width: 1 / 2 / 3 / 4 values` → fans into the four per-side widths.
pub(crate) fn apply_border_widths(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
    ],
  );
  let (t, r, b, l) = parse_box_shorthand(value);
  if let Some(t) = t {
    style.border_top_width = Some(t);
  }
  if let Some(r) = r {
    style.border_right_width = Some(r);
  }
  if let Some(b) = b {
    style.border_bottom_width = Some(b);
  }
  if let Some(l) = l {
    style.border_left_width = Some(l);
  }
}

/// `border-style: 1 / 2 / 3 / 4 values` → fans into the four per-side styles.
pub(crate) fn apply_border_styles(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "border-top-style",
      "border-right-style",
      "border-bottom-style",
      "border-left-style",
    ],
  );
  let (t, r, b, l) = parse_keyword_box_shorthand(value, parse_border_style);
  if let Some(t) = t {
    style.border_top_style = Some(t);
  }
  if let Some(r) = r {
    style.border_right_style = Some(r);
  }
  if let Some(b) = b {
    style.border_bottom_style = Some(b);
  }
  if let Some(l) = l {
    style.border_left_style = Some(l);
  }
}

/// `border-color: 1 / 2 / 3 / 4 values` → fans into the four per-side colors.
pub(crate) fn apply_border_colors(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "border-top-color",
      "border-right-color",
      "border-bottom-color",
      "border-left-color",
    ],
  );
  let (t, r, b, l) = parse_keyword_box_shorthand(value, parse_css_color);
  if let Some(t) = t {
    style.border_top_color = Some(t);
  }
  if let Some(r) = r {
    style.border_right_color = Some(r);
  }
  if let Some(b) = b {
    style.border_bottom_color = Some(b);
  }
  if let Some(l) = l {
    style.border_left_color = Some(l);
  }
}

/// `border-radius: <h-list> [ / <v-list> ]` — each list 1..4 values in
/// CSS per-corner order TL, TR, BR, BL. Without the slash both axes
/// share the same list. Each axis uses the standard 1/2/3/4-value
/// expansion rules.
pub(crate) fn apply_border_radii(value: &str, style: &mut Style) {
  mark_shorthand_reset(style, "border-radius");
  mark_property_resets(
    style,
    &[
      "border-top-left-radius-v",
      "border-top-right-radius-v",
      "border-bottom-right-radius-v",
      "border-bottom-left-radius-v",
    ],
  );
  let (h_part, v_part) = match value.split_once('/') {
    Some((h, v)) => (h.trim(), Some(v.trim())),
    None => (value.trim(), None),
  };

  let (h_tl, h_tr, h_br, h_bl) = expand_corner_list(h_part);
  if let Some(v) = h_tl.clone() {
    style.border_top_left_radius = Some(v);
  }
  if let Some(v) = h_tr.clone() {
    style.border_top_right_radius = Some(v);
  }
  if let Some(v) = h_br.clone() {
    style.border_bottom_right_radius = Some(v);
  }
  if let Some(v) = h_bl.clone() {
    style.border_bottom_left_radius = Some(v);
  }

  if let Some(v_str) = v_part {
    let (v_tl, v_tr, v_br, v_bl) = expand_corner_list(v_str);
    if let Some(v) = v_tl {
      style.border_top_left_radius_v = Some(v);
    }
    if let Some(v) = v_tr {
      style.border_top_right_radius_v = Some(v);
    }
    if let Some(v) = v_br {
      style.border_bottom_right_radius_v = Some(v);
    }
    if let Some(v) = v_bl {
      style.border_bottom_left_radius_v = Some(v);
    }
  } else {
    // Without a slash, the vertical axis equals the horizontal one.
    if let Some(v) = h_tl {
      style.border_top_left_radius_v = Some(v);
    }
    if let Some(v) = h_tr {
      style.border_top_right_radius_v = Some(v);
    }
    if let Some(v) = h_br {
      style.border_bottom_right_radius_v = Some(v);
    }
    if let Some(v) = h_bl {
      style.border_bottom_left_radius_v = Some(v);
    }
  }
}

/// Expand a 1..4-value space-separated CSS length list to per-corner
/// `(TL, TR, BR, BL)`.
fn expand_corner_list(
  value: &str,
) -> (
  Option<CssLength>,
  Option<CssLength>,
  Option<CssLength>,
  Option<CssLength>,
) {
  let parts: Vec<&str> = value.split_whitespace().collect();
  match parts.len() {
    0 => (None, None, None, None),
    1 => {
      let v = parse_css_length(parts[0]);
      (v.clone(), v.clone(), v.clone(), v)
    }
    2 => {
      let a = parse_css_length(parts[0]);
      let b = parse_css_length(parts[1]);
      (a.clone(), b.clone(), a, b)
    }
    3 => {
      let a = parse_css_length(parts[0]);
      let b = parse_css_length(parts[1]);
      let c = parse_css_length(parts[2]);
      (a, b.clone(), c, b)
    }
    _ => (
      parse_css_length(parts[0]),
      parse_css_length(parts[1]),
      parse_css_length(parts[2]),
      parse_css_length(parts[3]),
    ),
  }
}

/// Per-corner longhand: `border-<corner>-radius: <h>` (v defaults to h)
/// or `border-<corner>-radius: <h> <v>`.
pub(crate) fn apply_corner_radius(value: &str, h_field: &mut Option<CssLength>, v_field: &mut Option<CssLength>) {
  let parts = split_top_level_whitespace(value);
  match parts.len() {
    0 => {}
    1 => {
      let v = parse_css_length(parts[0]);
      *h_field = v.clone();
      *v_field = v;
    }
    _ => {
      *h_field = parse_css_length(parts[0]);
      *v_field = parse_css_length(parts[1]);
    }
  }
}

/// Generic 1/2/3/4-value box shorthand for properties whose values are
/// keyword-typed (border-style) or color-typed (border-color). Returns
/// `(top, right, bottom, left)`.
fn parse_keyword_box_shorthand<T: Clone>(
  value: &str,
  parse_one: fn(&str) -> Option<T>,
) -> (Option<T>, Option<T>, Option<T>, Option<T>) {
  let parts = split_top_level_whitespace(value);
  match parts.len() {
    0 => (None, None, None, None),
    1 => {
      let v = parse_one(parts[0]);
      (v.clone(), v.clone(), v.clone(), v)
    }
    2 => {
      let v = parse_one(parts[0]);
      let h = parse_one(parts[1]);
      (v.clone(), h.clone(), v, h)
    }
    3 => {
      let t = parse_one(parts[0]);
      let h = parse_one(parts[1]);
      let b = parse_one(parts[2]);
      (t, h.clone(), b, h)
    }
    _ => (
      parse_one(parts[0]),
      parse_one(parts[1]),
      parse_one(parts[2]),
      parse_one(parts[3]),
    ),
  }
}

/// `parse_css_length` minus its catch-all `Raw` / `Auto` returns — used
/// when matching one piece of a shorthand against multiple value kinds.
pub(crate) fn parse_definite_length(token: &str) -> Option<CssLength> {
  match parse_css_length(token)? {
    CssLength::Raw(_) | CssLength::Auto => None,
    other => Some(other),
  }
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
