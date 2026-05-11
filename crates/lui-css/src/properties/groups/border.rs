use super::property_group::PropertyGroup;
use crate::{css_parser::*, declaration::DeclarationBlock, shorthands::shorthand_members, style::Style, values::*};
pub struct BorderGroup;

impl PropertyGroup for BorderGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      if !decl.property.starts_with("border") {
        continue;
      }
      match &*decl.property {
        "border" => parse_border_shorthand(&decl.value, style),
        "border-top" => parse_border_side_shorthand(&decl.value, style, Side::Top),
        "border-right" => parse_border_side_shorthand(&decl.value, style, Side::Right),
        "border-bottom" => parse_border_side_shorthand(&decl.value, style, Side::Bottom),
        "border-left" => parse_border_side_shorthand(&decl.value, style, Side::Left),
        "border-width" => apply_border_widths(&decl.value, style),
        "border-style" => apply_border_styles(&decl.value, style),
        "border-color" => apply_border_colors(&decl.value, style),
        "border-top-width" => style.border_top_width = parse_css_length(&decl.value),
        "border-right-width" => style.border_right_width = parse_css_length(&decl.value),
        "border-bottom-width" => style.border_bottom_width = parse_css_length(&decl.value),
        "border-left-width" => style.border_left_width = parse_css_length(&decl.value),
        "border-top-style" => style.border_top_style = parse_border_style(&decl.value),
        "border-right-style" => style.border_right_style = parse_border_style(&decl.value),
        "border-bottom-style" => style.border_bottom_style = parse_border_style(&decl.value),
        "border-left-style" => style.border_left_style = parse_border_style(&decl.value),
        "border-top-color" => style.border_top_color = parse_css_color(&decl.value),
        "border-right-color" => style.border_right_color = parse_css_color(&decl.value),
        "border-bottom-color" => style.border_bottom_color = parse_css_color(&decl.value),
        "border-left-color" => style.border_left_color = parse_css_color(&decl.value),
        "border-radius" => apply_border_radii(&decl.value, style),
        "border-top-left-radius" => apply_corner_radius(
          &decl.value,
          &mut style.border_top_left_radius,
          &mut style.border_top_left_radius_v,
        ),
        "border-top-right-radius" => apply_corner_radius(
          &decl.value,
          &mut style.border_top_right_radius,
          &mut style.border_top_right_radius_v,
        ),
        "border-bottom-right-radius" => apply_corner_radius(
          &decl.value,
          &mut style.border_bottom_right_radius,
          &mut style.border_bottom_right_radius_v,
        ),
        "border-bottom-left-radius" => apply_corner_radius(
          &decl.value,
          &mut style.border_bottom_left_radius,
          &mut style.border_bottom_left_radius_v,
        ),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "border",
      "border-top",
      "border-right",
      "border-bottom",
      "border-left",
      "border-width",
      "border-style",
      "border-color",
      "border-top-width",
      "border-right-width",
      "border-bottom-width",
      "border-left-width",
      "border-top-style",
      "border-right-style",
      "border-bottom-style",
      "border-left-style",
      "border-top-color",
      "border-right-color",
      "border-bottom-color",
      "border-left-color",
      "border-radius",
      "border-top-left-radius",
      "border-top-right-radius",
      "border-bottom-right-radius",
      "border-bottom-left-radius",
    ]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Which side a per-side border helper writes to.
#[derive(Copy, Clone)]
enum Side {
  Top,
  Right,
  Bottom,
  Left,
}

fn local_mark_shorthand_reset(style: &mut Style, property: &str) {
  if let Some(members) = shorthand_members(property) {
    for member in members {
      style.reset_properties.insert(ArcStr::from(*member));
    }
  }
}

fn local_mark_property_resets(style: &mut Style, props: &[&str]) {
  for prop in props {
    style.reset_properties.insert(ArcStr::from(*prop));
  }
}

/// Parse the `border` shorthand into width / style / color, in any order.
/// The values fan out to all four sides (per CSS spec: `border` is itself
/// a shorthand for the four `border-<side>-<piece>` longhands).
fn parse_border_shorthand(value: &str, style: &mut Style) {
  local_mark_shorthand_reset(style, "border");
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
fn parse_border_side_shorthand(value: &str, style: &mut Style, side: Side) {
  let resets = match side {
    Side::Top => ["border-top-width", "border-top-style", "border-top-color"],
    Side::Right => ["border-right-width", "border-right-style", "border-right-color"],
    Side::Bottom => ["border-bottom-width", "border-bottom-style", "border-bottom-color"],
    Side::Left => ["border-left-width", "border-left-style", "border-left-color"],
  };
  local_mark_property_resets(style, &resets);
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
fn parse_border_pieces(value: &str) -> (Option<CssLength>, Option<BorderStyle>, Option<CssColor>) {
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

/// `border-width: 1 / 2 / 3 / 4 values` -> fans into the four per-side widths.
fn apply_border_widths(value: &str, style: &mut Style) {
  local_mark_property_resets(
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

/// `border-style: 1 / 2 / 3 / 4 values` -> fans into the four per-side styles.
fn apply_border_styles(value: &str, style: &mut Style) {
  local_mark_property_resets(
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

/// `border-color: 1 / 2 / 3 / 4 values` -> fans into the four per-side colors.
fn apply_border_colors(value: &str, style: &mut Style) {
  local_mark_property_resets(
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

/// `border-radius: <h-list> [ / <v-list> ]` -- each list 1..4 values in
/// CSS per-corner order TL, TR, BR, BL. Without the slash both axes
/// share the same list. Each axis uses the standard 1/2/3/4-value
/// expansion rules.
fn apply_border_radii(value: &str, style: &mut Style) {
  local_mark_shorthand_reset(style, "border-radius");
  local_mark_property_resets(
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
fn apply_corner_radius(value: &str, h_field: &mut Option<CssLength>, v_field: &mut Option<CssLength>) {
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
