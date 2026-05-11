use super::{
  apply_css_property, parse_align_content, parse_align_items, parse_align_self, parse_border_style,
  parse_box_shorthand, parse_css_color, parse_css_image, parse_css_length, parse_definite_length, parse_flex_direction,
  parse_flex_wrap, parse_font_style, parse_font_weight, parse_grid_line, parse_justify_content, parse_justify_items,
  parse_justify_self, parse_list_style_position, parse_list_style_type, parse_white_space, split_top_level_commas,
  split_top_level_whitespace, strip_function,
};
use crate::{
  shorthands::{is_deferred_longhand, shorthand_members},
  style::Style,
  values::*,
};

pub(crate) fn apply_generic_shorthand(style: &mut Style, property: &str, value: &str) {
  match property {
    "animation-range" => apply_pair_raw_shorthand(style, value, "animation-range-start", "animation-range-end", false),
    "background-position" => apply_background_position_shorthand(value, style),
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
    "columns" => apply_columns_shorthand(style, value),
    "contain-intrinsic-size" => apply_pair_raw_shorthand(
      style,
      value,
      "contain-intrinsic-width",
      "contain-intrinsic-height",
      true,
    ),
    "container" => apply_pair_raw_shorthand(style, value, "container-name", "container-type", false),
    "cue" => apply_pair_raw_shorthand(style, value, "cue-before", "cue-after", true),
    "flex-flow" => apply_flex_flow_shorthand(value, style),
    "font" => apply_font_shorthand(value, style),
    "font-synthesis" => apply_pair_or_quad_raw_shorthand(
      style,
      value,
      &[
        "font-synthesis-weight",
        "font-synthesis-style",
        "font-synthesis-small-caps",
        "font-synthesis-position",
      ],
    ),
    "font-variant" => apply_font_variant_shorthand(style, value),
    "font-variant-ligatures" => apply_pair_or_quad_raw_shorthand(
      style,
      value,
      &[
        "font-variant-ligatures-common",
        "font-variant-ligatures-discretionary",
        "font-variant-ligatures-historical",
        "font-variant-ligatures-contextual",
      ],
    ),
    "grid" => apply_grid_shorthand(style, value),
    "grid-area" => apply_grid_area_shorthand(value, style),
    "grid-template" => apply_grid_template_shorthand(style, value),
    "inset" => apply_inset_shorthand(value, style),
    "inset-block" => apply_pair_raw_shorthand(style, value, "inset-block-start", "inset-block-end", true),
    "inset-inline" => apply_pair_raw_shorthand(style, value, "inset-inline-start", "inset-inline-end", true),
    "line-clamp" => apply_pair_or_quad_raw_shorthand(style, value, &["max-lines", "block-ellipsis", "continue"]),
    "list-style" => apply_list_style_shorthand(style, value),
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
    "place-content" => apply_place_content_shorthand(value, style),
    "place-items" => apply_place_items_shorthand(value, style),
    "place-self" => apply_place_self_shorthand(value, style),
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
    "transition" => apply_transition_shorthand(value, style),
    "view-timeline" => apply_pair_or_quad_raw_shorthand(
      style,
      value,
      &["view-timeline-name", "view-timeline-axis", "view-timeline-inset"],
    ),
    "white-space" => apply_white_space_property(value, style),
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

fn apply_inset_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["top", "right", "bottom", "left"]);
  let (t, r, b, l) = parse_box_shorthand(value);
  style.top = t;
  style.right = r;
  style.bottom = b;
  style.left = l;
}

pub(crate) fn apply_gap_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["gap", "row-gap", "column-gap"]);
  let parts = split_top_level_whitespace(value);
  let first = parts.first().copied().unwrap_or("");
  let second = parts.get(1).copied().unwrap_or(first);
  style.gap = parse_css_length(first);
  style.row_gap = parse_css_length(first);
  style.column_gap = parse_css_length(second);
}

fn apply_flex_flow_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["flex-direction", "flex-wrap"]);
  for token in split_top_level_whitespace(value) {
    if style.flex_direction.is_none() {
      style.flex_direction = parse_flex_direction(token);
      if style.flex_direction.is_some() {
        continue;
      }
    }
    if style.flex_wrap.is_none() {
      style.flex_wrap = parse_flex_wrap(token);
    }
  }
}

fn apply_place_content_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["align-content", "justify-content"]);
  let parts = split_top_level_whitespace(value);
  let first = parts.first().copied().unwrap_or("");
  let second = parts.get(1).copied().unwrap_or(first);
  style.align_content = parse_align_content(first);
  style.justify_content = parse_justify_content(second);
}

fn apply_place_items_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["align-items", "justify-items"]);
  let parts = split_top_level_whitespace(value);
  let first = parts.first().copied().unwrap_or("");
  let second = parts.get(1).copied().unwrap_or(first);
  style.align_items = parse_align_items(first);
  style.justify_items = parse_justify_items(second);
}

fn apply_place_self_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["align-self", "justify-self"]);
  let parts = split_top_level_whitespace(value);
  let first = parts.first().copied().unwrap_or("");
  let second = parts.get(1).copied().unwrap_or(first);
  style.align_self = parse_align_self(first);
  style.justify_self = parse_justify_self(second);
}

pub(crate) fn apply_background_position_shorthand(value: &str, style: &mut Style) {
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

pub(crate) fn apply_text_decoration_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "text-decoration",
      "text-decoration-line",
      "text-decoration-style",
      "text-decoration-color",
      "text-decoration-thickness",
    ],
  );
  style.text_decoration = Some(ArcStr::from(value));
  let mut lines = Vec::new();
  for token in split_top_level_whitespace(value) {
    match token.to_ascii_lowercase().as_str() {
      "underline" | "overline" | "line-through" | "none" => lines.push(token),
      "solid" | "double" | "dotted" | "dashed" | "wavy" => set_deferred(style, "text-decoration-style", token),
      "auto" | "from-font" => set_deferred(style, "text-decoration-thickness", token),
      _ if parse_css_color(token).is_some() => set_deferred(style, "text-decoration-color", token),
      _ if parse_css_length(token).is_some() => set_deferred(style, "text-decoration-thickness", token),
      _ => {}
    }
  }
  if !lines.is_empty() {
    set_deferred(style, "text-decoration-line", lines.join(" "));
  }
}

pub(crate) fn apply_white_space_property(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "white-space",
      "white-space-collapse",
      "text-wrap-mode",
      "white-space-trim",
    ],
  );
  style.white_space = parse_white_space(value);
  let lower = value.trim().to_ascii_lowercase();
  let (collapse, wrap, trim) = match lower.as_str() {
    "normal" => ("collapse", "wrap", "none"),
    "nowrap" => ("collapse", "nowrap", "none"),
    "pre" => ("preserve", "nowrap", "none"),
    "pre-wrap" => ("preserve", "wrap", "none"),
    "pre-line" => ("preserve-breaks", "wrap", "none"),
    "break-spaces" => ("break-spaces", "wrap", "none"),
    other => (other, other, "none"),
  };
  set_deferred(style, "white-space-collapse", collapse);
  set_deferred(style, "text-wrap-mode", wrap);
  set_deferred(style, "white-space-trim", trim);
}

pub(crate) fn apply_animation_shorthand(value: &str, style: &mut Style) {
  mark_shorthand_reset(style, "animation");
  style.animation = Some(ArcStr::from(value));
  let mut names = Vec::new();
  let mut durations = Vec::new();
  let mut timing = Vec::new();
  let mut delays = Vec::new();
  let mut iterations = Vec::new();
  let mut directions = Vec::new();
  let mut fills = Vec::new();
  let mut states = Vec::new();
  let mut compositions = Vec::new();
  let mut timelines = Vec::new();
  let mut range_starts = Vec::new();
  let mut range_ends = Vec::new();
  for layer in split_top_level_commas(value) {
    let mut name: Option<String> = None;
    let mut duration: Option<String> = None;
    let mut timing_fn: Option<String> = None;
    let mut delay: Option<String> = None;
    let mut iteration: Option<String> = None;
    let mut direction: Option<String> = None;
    let mut fill: Option<String> = None;
    let mut state: Option<String> = None;
    let mut composition: Option<String> = None;
    let mut timeline: Option<String> = None;
    let mut range_start: Option<String> = None;
    let mut range_end: Option<String> = None;
    let mut unknown: Vec<String> = Vec::new();
    let mut seen_duration = false;
    for token in split_top_level_whitespace(layer) {
      let lower = token.to_ascii_lowercase();
      if is_time_token(token) {
        if !seen_duration {
          duration = Some(token.to_string());
          seen_duration = true;
        } else if delay.is_none() {
          delay = Some(token.to_string());
        } else {
          unknown.push(token.to_string());
        }
      } else {
        match () {
          _ if timeline.is_none() && is_animation_timeline_token(token) => timeline = Some(token.to_string()),
          _ if timing_fn.is_none() && is_animation_timing_function(token) => timing_fn = Some(token.to_string()),
          _ if iteration.is_none() && is_animation_iteration_count_token(token) => iteration = Some(token.to_string()),
          _ if direction.is_none() && is_animation_direction_token(token) => direction = Some(token.to_string()),
          _ if fill.is_none() && lower != "none" && is_animation_fill_mode_token(token) => {
            fill = Some(token.to_string())
          }
          _ if state.is_none() && is_animation_play_state_token(token) => state = Some(token.to_string()),
          _ if composition.is_none() && is_animation_composition_token(token) => composition = Some(token.to_string()),
          _ => unknown.push(token.to_string()),
        }
      }
    }
    for token in unknown {
      let lower = token.to_ascii_lowercase();
      if lower == "none" {
        if name.is_none() {
          name = Some(token);
          continue;
        }
        if fill.is_none() {
          fill = Some(token);
          continue;
        }
        if timeline.is_none() {
          timeline = Some(token);
          continue;
        }
      }
      if lower == "auto" && timeline.is_none() && name.is_some() {
        timeline = Some(token);
        continue;
      }
      if name.is_none() && !is_css_wide_keyword(&lower) {
        name = Some(token);
        continue;
      }
      if range_start.is_none() {
        range_start = Some(token);
        continue;
      }
      if range_end.is_none() {
        range_end = Some(token);
      }
    }
    names.push(name.unwrap_or_else(|| "none".to_string()));
    durations.push(duration.unwrap_or_else(|| "0s".to_string()));
    timing.push(timing_fn.unwrap_or_else(|| "ease".to_string()));
    delays.push(delay.unwrap_or_else(|| "0s".to_string()));
    iterations.push(iteration.unwrap_or_else(|| "1".to_string()));
    directions.push(direction.unwrap_or_else(|| "normal".to_string()));
    fills.push(fill.unwrap_or_else(|| "none".to_string()));
    states.push(state.unwrap_or_else(|| "running".to_string()));
    compositions.push(composition.unwrap_or_else(|| "replace".to_string()));
    timelines.push(timeline.unwrap_or_else(|| "auto".to_string()));
    range_starts.push(range_start.unwrap_or_else(|| "normal".to_string()));
    range_ends.push(range_end.unwrap_or_else(|| "normal".to_string()));
  }
  set_deferred(style, "animation-name", names.join(", "));
  set_deferred(style, "animation-duration", durations.join(", "));
  set_deferred(style, "animation-timing-function", timing.join(", "));
  set_deferred(style, "animation-delay", delays.join(", "));
  set_deferred(style, "animation-iteration-count", iterations.join(", "));
  set_deferred(style, "animation-direction", directions.join(", "));
  set_deferred(style, "animation-fill-mode", fills.join(", "));
  set_deferred(style, "animation-play-state", states.join(", "));
  set_deferred(style, "animation-composition", compositions.join(", "));
  set_deferred(style, "animation-timeline", timelines.join(", "));
  set_deferred(style, "animation-range-start", range_starts.join(", "));
  set_deferred(style, "animation-range-end", range_ends.join(", "));
}

pub(crate) fn apply_transition_shorthand(value: &str, style: &mut Style) {
  mark_shorthand_reset(style, "transition");
  style.transition = Some(ArcStr::from(value));
  let mut properties = Vec::new();
  let mut durations = Vec::new();
  let mut timing = Vec::new();
  let mut delays = Vec::new();
  let mut behaviors = Vec::new();
  for layer in split_top_level_commas(value) {
    let mut property: Option<String> = None;
    let mut duration: Option<String> = None;
    let mut timing_fn: Option<String> = None;
    let mut delay: Option<String> = None;
    let mut behavior: Option<String> = None;
    let mut unknown: Vec<String> = Vec::new();
    let mut seen_duration = false;
    for token in split_top_level_whitespace(layer) {
      if is_time_token(token) {
        if !seen_duration {
          duration = Some(token.to_string());
          seen_duration = true;
        } else if delay.is_none() {
          delay = Some(token.to_string());
        } else {
          unknown.push(token.to_string());
        }
      } else if timing_fn.is_none() && is_transition_timing_function(token) {
        timing_fn = Some(token.to_string());
      } else if behavior.is_none() && is_transition_behavior_token(token) {
        behavior = Some(token.to_string());
      } else {
        unknown.push(token.to_string());
      }
    }
    for token in unknown {
      if property.is_none() {
        property = Some(token);
      }
    }
    properties.push(property.unwrap_or_else(|| "all".to_string()));
    durations.push(duration.unwrap_or_else(|| "0s".to_string()));
    timing.push(timing_fn.unwrap_or_else(|| "ease".to_string()));
    delays.push(delay.unwrap_or_else(|| "0s".to_string()));
    behaviors.push(behavior.unwrap_or_else(|| "normal".to_string()));
  }
  set_deferred(style, "transition-property", properties.join(", "));
  set_deferred(style, "transition-duration", durations.join(", "));
  set_deferred(style, "transition-timing-function", timing.join(", "));
  set_deferred(style, "transition-delay", delays.join(", "));
  set_deferred(style, "transition-behavior", behaviors.join(", "));
}

fn apply_columns_shorthand(style: &mut Style, value: &str) {
  mark_property_resets(style, &["column-width", "column-count"]);
  for token in split_top_level_whitespace(value) {
    if parse_css_length(token).is_some() && !matches!(token.trim(), "auto") {
      set_deferred(style, "column-width", token);
    } else {
      set_deferred(style, "column-count", token);
    }
  }
}

fn apply_font_variant_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "font-variant");
  set_deferred(style, "font-variant-ligatures", value);
  set_deferred(style, "font-variant-caps", value);
  set_deferred(style, "font-variant-numeric", value);
  set_deferred(style, "font-variant-east-asian", value);
  set_deferred(style, "font-variant-alternates", value);
  set_deferred(style, "font-variant-position", value);
  set_deferred(style, "font-variant-emoji", value);
}

fn apply_font_shorthand(value: &str, style: &mut Style) {
  mark_shorthand_reset(style, "font");
  set_deferred(style, "font-variant", "normal");
  set_deferred(style, "font-stretch", "normal");
  style.font_style = Some(FontStyle::Normal);
  style.font_weight = Some(FontWeight::Normal);
  style.line_height = Some(CssLength::Raw(ArcStr::from("normal")));

  let tokens = split_top_level_whitespace(value);
  let mut size_idx = None;
  for (idx, token) in tokens.iter().enumerate() {
    if token.contains('/') || is_font_size_token(token) {
      size_idx = Some(idx);
      break;
    }
    match token.to_ascii_lowercase().as_str() {
      "italic" | "oblique" | "normal" => style.font_style = parse_font_style(token),
      "bold" | "bolder" | "lighter" => style.font_weight = parse_font_weight(token),
      "small-caps" => set_deferred(style, "font-variant", *token),
      _ => {
        if let Some(weight) = parse_font_weight(token) {
          style.font_weight = Some(weight);
        } else if matches!(
          token.to_ascii_lowercase().as_str(),
          "ultra-condensed"
            | "extra-condensed"
            | "condensed"
            | "semi-condensed"
            | "semi-expanded"
            | "expanded"
            | "extra-expanded"
            | "ultra-expanded"
        ) {
          set_deferred(style, "font-stretch", *token);
        }
      }
    }
  }
  if let Some(size_idx) = size_idx {
    let size_token = tokens[size_idx];
    if let Some((size_part, line_part)) = size_token.split_once('/') {
      style.font_size = parse_css_length(size_part);
      style.line_height = parse_css_length(line_part);
    } else {
      style.font_size = parse_css_length(size_token);
      if let Some(next) = tokens.get(size_idx + 1) {
        if let Some(line) = next.strip_prefix('/') {
          style.line_height = parse_css_length(line);
        }
      }
    }
    let family_start = if tokens.get(size_idx + 1).is_some_and(|t| t.starts_with('/')) {
      size_idx + 2
    } else {
      size_idx + 1
    };
    if family_start < tokens.len() {
      style.font_family = Some(ArcStr::from(tokens[family_start..].join(" ").as_str()));
    }
  }
}

fn is_font_size_token(token: &str) -> bool {
  matches!(
    parse_css_length(token),
    Some(
      CssLength::Px(_)
        | CssLength::Percent(_)
        | CssLength::Em(_)
        | CssLength::Rem(_)
        | CssLength::Vw(_)
        | CssLength::Vh(_)
        | CssLength::Vmin(_)
        | CssLength::Vmax(_)
        | CssLength::Zero
        | CssLength::Calc(_)
        | CssLength::Min(_)
        | CssLength::Max(_)
        | CssLength::Clamp { .. }
    )
  )
}

fn apply_list_style_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "list-style");
  for token in split_top_level_whitespace(value) {
    match token.to_ascii_lowercase().as_str() {
      "inside" | "outside" => style.list_style_position = parse_list_style_position(token),
      _ if parse_list_style_type(token).is_some() => style.list_style_type = parse_list_style_type(token),
      _ if parse_css_image(token).is_some() => set_deferred(style, "list-style-image", token),
      "none" => style.list_style_type = Some(ListStyleType::None),
      _ => set_deferred(style, "list-style-type", token),
    }
  }
}

fn apply_grid_template_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "grid-template");
  if let Some((rows, cols)) = split_once_top_level(value, '/') {
    apply_css_property(style, "grid-template-rows", rows.trim());
    apply_css_property(style, "grid-template-columns", cols.trim());
  } else {
    apply_css_property(style, "grid-template-rows", value);
  }
  set_deferred(style, "grid-template-areas", value);
}

fn apply_grid_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "grid");
  if let Some((template, auto)) = split_once_top_level(value, '/') {
    apply_grid_template_shorthand(style, template.trim());
    set_deferred(style, "grid-auto-flow", auto.trim());
    set_deferred(style, "grid-auto-rows", auto.trim());
    set_deferred(style, "grid-auto-columns", auto.trim());
  } else {
    apply_grid_template_shorthand(style, value);
  }
}

fn apply_grid_area_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &["grid-row-start", "grid-column-start", "grid-row-end", "grid-column-end"],
  );
  let parts: Vec<&str> = value.split('/').map(str::trim).filter(|p| !p.is_empty()).collect();
  match parts.as_slice() {
    [a] => {
      let line = parse_grid_line(a);
      style.grid_row_start = line.clone();
      style.grid_column_start = line.clone();
      style.grid_row_end = line.clone();
      style.grid_column_end = line;
    }
    [a, b] => {
      style.grid_row_start = parse_grid_line(a);
      style.grid_column_start = parse_grid_line(b);
      style.grid_row_end = parse_grid_line(a);
      style.grid_column_end = parse_grid_line(b);
    }
    [a, b, c] => {
      style.grid_row_start = parse_grid_line(a);
      style.grid_column_start = parse_grid_line(b);
      style.grid_row_end = parse_grid_line(c);
      style.grid_column_end = parse_grid_line(b);
    }
    [a, b, c, d] => {
      style.grid_row_start = parse_grid_line(a);
      style.grid_column_start = parse_grid_line(b);
      style.grid_row_end = parse_grid_line(c);
      style.grid_column_end = parse_grid_line(d);
    }
    _ => {}
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

fn is_time_token(token: &str) -> bool {
  let t = token.trim().to_ascii_lowercase();
  t.ends_with("ms") || t.ends_with('s')
}

fn is_animation_timing_function(token: &str) -> bool {
  is_transition_timing_function(token)
}

fn is_transition_timing_function(token: &str) -> bool {
  let lower = token.trim().to_ascii_lowercase();
  matches!(
    lower.as_str(),
    "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end"
  ) || strip_function(token, "cubic-bezier").is_some()
    || strip_function(token, "steps").is_some()
    || strip_function(token, "linear").is_some()
}

fn is_transition_behavior_token(token: &str) -> bool {
  matches!(token.trim().to_ascii_lowercase().as_str(), "normal" | "allow-discrete")
}

fn is_animation_timeline_token(token: &str) -> bool {
  strip_function(token, "scroll").is_some() || strip_function(token, "view").is_some()
}

fn is_animation_iteration_count_token(token: &str) -> bool {
  let lower = token.trim().to_ascii_lowercase();
  lower == "infinite" || lower.parse::<f32>().is_ok()
}

fn is_animation_direction_token(token: &str) -> bool {
  matches!(
    token.trim().to_ascii_lowercase().as_str(),
    "normal" | "reverse" | "alternate" | "alternate-reverse"
  )
}

fn is_animation_fill_mode_token(token: &str) -> bool {
  matches!(
    token.trim().to_ascii_lowercase().as_str(),
    "none" | "forwards" | "backwards" | "both"
  )
}

fn is_animation_play_state_token(token: &str) -> bool {
  matches!(token.trim().to_ascii_lowercase().as_str(), "running" | "paused")
}

fn is_animation_composition_token(token: &str) -> bool {
  matches!(
    token.trim().to_ascii_lowercase().as_str(),
    "replace" | "add" | "accumulate"
  )
}

fn is_css_wide_keyword(token: &str) -> bool {
  matches!(token, "initial" | "inherit" | "unset" | "revert" | "revert-layer")
}

fn split_once_top_level<'a>(value: &'a str, delim: char) -> Option<(&'a str, &'a str)> {
  let mut depth = 0i32;
  for (idx, ch) in value.char_indices() {
    match ch {
      '(' => depth += 1,
      ')' => depth -= 1,
      _ if ch == delim && depth == 0 => {
        return Some((&value[..idx], &value[idx + ch.len_utf8()..]));
      }
      _ => {}
    }
  }
  None
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
