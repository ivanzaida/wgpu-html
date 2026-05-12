use super::property_group::PropertyGroup;
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, values::ArcStr};

pub struct AnimationGroup;

impl PropertyGroup for AnimationGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      match &*decl.property {
        "transition" => apply_transition_shorthand(&decl.value, style),
        "animation" => apply_animation_shorthand(&decl.value, style),
        "transform" => style.transform = Some(ArcStr::from(decl.value.as_ref())),
        "transform-origin" => style.transform_origin = Some(ArcStr::from(decl.value.as_ref())),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &["transition", "animation", "transform", "transform-origin"]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn apply_animation_shorthand(value: &str, style: &mut Style) {
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

fn apply_transition_shorthand(value: &str, style: &mut Style) {
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
