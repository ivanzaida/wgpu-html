use super::{property_group::PropertyGroup, warn_none};
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, style_props::clear_value_for, values::*};

pub struct BackgroundGroup;

impl PropertyGroup for BackgroundGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      if !decl.property.starts_with("background") {
        continue;
      }
      let (p, v) = (&*decl.property, &*decl.value);
      match p {
        "background" => apply_background_shorthand(v, style),
        "background-color" => style.background_color = parse_css_color(v),
        "background-image" => style.background_image = parse_css_image(v),
        "background-size" => style.background_size = Some(ArcStr::from(v)),
        "background-position" => apply_background_position_shorthand(v, style),
        "background-repeat" => style.background_repeat = warn_none(p, v, parse_background_repeat(v)),
        "background-clip" => style.background_clip = warn_none(p, v, parse_background_clip(v)),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "background",
      "background-color",
      "background-image",
      "background-size",
      "background-position",
      "background-repeat",
      "background-clip",
    ]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn apply_background_shorthand(value: &str, style: &mut Style) {
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
            // stylesheet for form controls (`buttonface`, `field`, ...).
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

fn apply_background_position_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &["background-position", "background-position-x", "background-position-y"],
  );
  style.background_position = Some(ArcStr::from(value));
  let parts = split_top_level_whitespace(value);
  if let Some(x) = parts.first() {
    set_deferred(style, "background-position-x", *x);
  }
  if let Some(y) = parts.get(1).or_else(|| parts.first()) {
    set_deferred(style, "background-position-y", *y);
  }
}
