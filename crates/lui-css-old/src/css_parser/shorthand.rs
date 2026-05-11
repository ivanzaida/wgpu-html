use super::{
  parse_border_style, parse_box_shorthand, parse_css_color, parse_definite_length, split_top_level_whitespace,
};
use crate::{
  shorthands::{is_deferred_longhand, shorthand_members},
  style::Style,
  values::*,
};

pub(crate) fn apply_generic_shorthand(style: &mut Style, property: &str, value: &str) {
  match property {
    "animation-range" => apply_pair_raw_shorthand(style, value, "animation-range-start", "animation-range-end", false),
    "border-block" => apply_three_part_borderish_deferred(
      style,
      value,
      "border-block-width",
      "border-block-style",
      "border-block-color",
    ),
    "border-block-color" => {
      apply_pair_raw_shorthand(style, value, "border-block-start-color", "border-block-end-color", true)
    }
    "border-block-end" => apply_three_part_borderish_deferred(
      style,
      value,
      "border-block-end-width",
      "border-block-end-style",
      "border-block-end-color",
    ),
    "border-block-start" => apply_three_part_borderish_deferred(
      style,
      value,
      "border-block-start-width",
      "border-block-start-style",
      "border-block-start-color",
    ),
    "border-block-style" => {
      apply_pair_raw_shorthand(style, value, "border-block-start-style", "border-block-end-style", true)
    }
    "border-block-width" => {
      apply_pair_raw_shorthand(style, value, "border-block-start-width", "border-block-end-width", true)
    }
    "border-image" => apply_placeholder_shorthand(style, property, value),
    "border-inline" => apply_three_part_borderish_deferred(
      style,
      value,
      "border-inline-width",
      "border-inline-style",
      "border-inline-color",
    ),
    "border-inline-color" => apply_pair_raw_shorthand(
      style,
      value,
      "border-inline-start-color",
      "border-inline-end-color",
      true,
    ),
    "border-inline-end" => apply_three_part_borderish_deferred(
      style,
      value,
      "border-inline-end-width",
      "border-inline-end-style",
      "border-inline-end-color",
    ),
    "border-inline-start" => apply_three_part_borderish_deferred(
      style,
      value,
      "border-inline-start-width",
      "border-inline-start-style",
      "border-inline-start-color",
    ),
    "border-inline-style" => apply_pair_raw_shorthand(
      style,
      value,
      "border-inline-start-style",
      "border-inline-end-style",
      true,
    ),
    "border-inline-width" => apply_pair_raw_shorthand(
      style,
      value,
      "border-inline-start-width",
      "border-inline-end-width",
      true,
    ),
    "column-rule" => apply_three_part_borderish_deferred(
      style,
      value,
      "column-rule-width",
      "column-rule-style",
      "column-rule-color",
    ),
    "contain-intrinsic-size" => apply_pair_raw_shorthand(
      style,
      value,
      "contain-intrinsic-width",
      "contain-intrinsic-height",
      true,
    ),
    "container" => apply_pair_raw_shorthand(style, value, "container-name", "container-type", false),
    "cue" => apply_pair_raw_shorthand(style, value, "cue-before", "cue-after", true),
    "inset-block" => apply_pair_raw_shorthand(style, value, "inset-block-start", "inset-block-end", true),
    "inset-inline" => apply_pair_raw_shorthand(style, value, "inset-inline-start", "inset-inline-end", true),
    "line-clamp" => apply_pair_or_quad_raw_shorthand(style, value, &["max-lines", "block-ellipsis", "continue"]),
    "margin-block" => apply_pair_raw_shorthand(style, value, "margin-block-start", "margin-block-end", true),
    "margin-inline" => apply_pair_raw_shorthand(style, value, "margin-inline-start", "margin-inline-end", true),
    "marker" => apply_pair_or_quad_raw_shorthand(style, value, &["marker-start", "marker-mid", "marker-end"]),
    "mask" | "mask-border" | "offset" => apply_placeholder_shorthand(style, property, value),
    "outline" => apply_three_part_borderish_deferred(style, value, "outline-width", "outline-style", "outline-color"),
    "overscroll-behavior" => {
      apply_pair_raw_shorthand(style, value, "overscroll-behavior-x", "overscroll-behavior-y", true)
    }
    "overscroll-behavior-block" => apply_pair_raw_shorthand(
      style,
      value,
      "overscroll-behavior-block-start",
      "overscroll-behavior-block-end",
      true,
    ),
    "overscroll-behavior-inline" => apply_pair_raw_shorthand(
      style,
      value,
      "overscroll-behavior-inline-start",
      "overscroll-behavior-inline-end",
      true,
    ),
    "padding-block" => apply_pair_raw_shorthand(style, value, "padding-block-start", "padding-block-end", true),
    "padding-inline" => apply_pair_raw_shorthand(style, value, "padding-inline-start", "padding-inline-end", true),
    "pause" => apply_pair_raw_shorthand(style, value, "pause-before", "pause-after", true),
    "rest" => apply_pair_raw_shorthand(style, value, "rest-before", "rest-after", true),
    "scroll-margin" => apply_box_raw_shorthand(
      style,
      value,
      &[
        "scroll-margin-top",
        "scroll-margin-right",
        "scroll-margin-bottom",
        "scroll-margin-left",
      ],
    ),
    "scroll-margin-block" => apply_pair_raw_shorthand(
      style,
      value,
      "scroll-margin-block-start",
      "scroll-margin-block-end",
      true,
    ),
    "scroll-margin-inline" => apply_pair_raw_shorthand(
      style,
      value,
      "scroll-margin-inline-start",
      "scroll-margin-inline-end",
      true,
    ),
    "scroll-padding" => apply_box_raw_shorthand(
      style,
      value,
      &[
        "scroll-padding-top",
        "scroll-padding-right",
        "scroll-padding-bottom",
        "scroll-padding-left",
      ],
    ),
    "scroll-padding-block" => apply_pair_raw_shorthand(
      style,
      value,
      "scroll-padding-block-start",
      "scroll-padding-block-end",
      true,
    ),
    "scroll-padding-inline" => apply_pair_raw_shorthand(
      style,
      value,
      "scroll-padding-inline-start",
      "scroll-padding-inline-end",
      true,
    ),
    "scroll-timeline" => apply_pair_raw_shorthand(style, value, "scroll-timeline-name", "scroll-timeline-axis", false),
    "text-box" => apply_pair_raw_shorthand(style, value, "text-box-trim", "text-box-edge", false),
    "text-emphasis" => apply_pair_raw_shorthand(style, value, "text-emphasis-style", "text-emphasis-color", false),
    "view-timeline" => apply_pair_or_quad_raw_shorthand(
      style,
      value,
      &["view-timeline-name", "view-timeline-axis", "view-timeline-inset"],
    ),
    _ => apply_placeholder_shorthand(style, property, value),
  }
}

pub(crate) fn mark_shorthand_reset(style: &mut Style, property: &str) {
  if let Some(members) = shorthand_members(property) {
    for member in members {
      style.reset_properties.insert(ArcStr::from(*member));
    }
  }
}

pub(crate) fn set_deferred(style: &mut Style, property: &str, value: impl AsRef<str>) {
  style
    .deferred_longhands
    .insert(ArcStr::from(property), ArcStr::from(value.as_ref()));
}

fn apply_placeholder_shorthand(style: &mut Style, property: &str, value: &str) {
  mark_shorthand_reset(style, property);
  if let Some(members) = shorthand_members(property) {
    for member in members {
      if is_deferred_longhand(member) {
        set_deferred(style, member, value);
      }
    }
  }
}

fn apply_pair_raw_shorthand(
  style: &mut Style,
  value: &str,
  first: &str,
  second: &str,
  duplicate_second_when_missing: bool,
) {
  let tokens = split_top_level_whitespace(value);
  mark_property_resets(style, &[first, second]);
  let first_value = tokens.first().copied().unwrap_or("");
  let second_value =
    tokens
      .get(1)
      .copied()
      .unwrap_or_else(|| if duplicate_second_when_missing { first_value } else { "" });
  if !first_value.is_empty() {
    set_deferred(style, first, first_value);
  }
  if !second_value.is_empty() {
    set_deferred(style, second, second_value);
  }
}

fn apply_pair_or_quad_raw_shorthand(style: &mut Style, value: &str, props: &[&str]) {
  let tokens = split_top_level_whitespace(value);
  mark_property_resets(style, props);
  for (idx, prop) in props.iter().enumerate() {
    let resolved = tokens
      .get(idx)
      .copied()
      .or_else(|| tokens.last().copied())
      .unwrap_or("");
    if !resolved.is_empty() {
      set_deferred(style, prop, resolved);
    }
  }
}

fn apply_box_raw_shorthand(style: &mut Style, value: &str, props: &[&str; 4]) {
  let (t, r, b, l) = parse_box_shorthand(value);
  mark_property_resets(style, props);
  for (prop, parsed) in props.iter().zip([t, r, b, l]) {
    if let Some(parsed) = parsed {
      set_deferred(style, prop, css_length_to_string(&parsed));
    }
  }
}

fn apply_three_part_borderish_deferred(
  style: &mut Style,
  value: &str,
  width_prop: &str,
  style_prop: &str,
  color_prop: &str,
) {
  mark_property_resets(style, &[width_prop, style_prop, color_prop]);
  for token in split_top_level_whitespace(value) {
    if parse_definite_length(token).is_some() {
      set_deferred(style, width_prop, token);
    } else if parse_border_style(token).is_some() {
      set_deferred(style, style_prop, token);
    } else if parse_css_color(token).is_some() {
      set_deferred(style, color_prop, token);
    }
  }
}

pub(crate) fn mark_property_resets(style: &mut Style, props: &[&str]) {
  for prop in props {
    style.reset_properties.insert(ArcStr::from(*prop));
  }
}

fn css_length_to_string(len: &CssLength) -> String {
  match len {
    CssLength::Px(v) => format!("{v}px"),
    CssLength::Percent(v) => format!("{v}%"),
    CssLength::Em(v) => format!("{v}em"),
    CssLength::Rem(v) => format!("{v}rem"),
    CssLength::Vw(v) => format!("{v}vw"),
    CssLength::Vh(v) => format!("{v}vh"),
    CssLength::Vmin(v) => format!("{v}vmin"),
    CssLength::Vmax(v) => format!("{v}vmax"),
    CssLength::Auto => "auto".into(),
    CssLength::Zero => "0".into(),
    CssLength::Raw(v) => v.to_string(),
    CssLength::Calc(v) => format!("calc({v:?})"),
    CssLength::Min(v) => format!("min({v:?})"),
    CssLength::Max(v) => format!("max({v:?})"),
    CssLength::Clamp { min, preferred, max } => {
      format!("clamp({min:?}, {preferred:?}, {max:?})")
    }
  }
}
