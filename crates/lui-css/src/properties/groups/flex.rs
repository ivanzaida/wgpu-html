use super::property_group::PropertyGroup;
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, values::*};

pub struct FlexGroup;

impl PropertyGroup for FlexGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      match &*decl.property {
        "flex" => apply_flex_shorthand(&decl.value, style),
        "flex-direction" => style.flex_direction = parse_flex_direction(&decl.value),
        "flex-wrap" => style.flex_wrap = parse_flex_wrap(&decl.value),
        "flex-grow" => style.flex_grow = decl.value.parse().ok(),
        "flex-shrink" => style.flex_shrink = decl.value.parse().ok(),
        "flex-basis" => style.flex_basis = parse_css_length(&decl.value),
        "flex-flow" => apply_flex_flow_shorthand(&decl.value, style),
        "justify-content" => style.justify_content = parse_justify_content(&decl.value),
        "align-items" => style.align_items = parse_align_items(&decl.value),
        "align-content" => style.align_content = parse_align_content(&decl.value),
        "align-self" => style.align_self = parse_align_self(&decl.value),
        "place-content" => apply_place_content_shorthand(&decl.value, style),
        "place-items" => apply_place_items_shorthand(&decl.value, style),
        "place-self" => apply_place_self_shorthand(&decl.value, style),
        "gap" => apply_gap_shorthand(&decl.value, style),
        "row-gap" => style.row_gap = parse_css_length(&decl.value),
        "column-gap" => style.column_gap = parse_css_length(&decl.value),
        "order" => style.order = decl.value.trim().parse().ok(),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "flex",
      "flex-direction",
      "flex-wrap",
      "flex-grow",
      "flex-shrink",
      "flex-basis",
      "flex-flow",
      "justify-content",
      "align-items",
      "align-content",
      "align-self",
      "place-content",
      "place-items",
      "place-self",
      "gap",
      "row-gap",
      "column-gap",
      "order",
    ]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Expand the `flex` shorthand into the three longhands per CSS-Flex-1
/// S7.2 (`flex` shorthand).
///
/// Recognized forms:
/// - `none`    -> 0 0 auto
/// - `auto`    -> 1 1 auto
/// - `initial` -> 0 1 auto
/// - `<number>`            -> grow=<n>, shrink=1, basis=0%
/// - `<basis>`             -> grow=1, shrink=1, basis=<basis>
/// - `<grow> <shrink>`     -> grow, shrink, basis=0%
/// - `<grow> <basis>`      -> grow, shrink=1, basis
/// - `<grow> <shrink> <basis>` (full form)
///
/// Token classification:
/// - A bare positive number (`1`, `0.5`) is a flex factor.
/// - Anything else (`100px`, `30%`, `auto`) is treated as basis.
fn apply_flex_shorthand(value: &str, style: &mut Style) {
  style.flex = Some(ArcStr::from(value));
  let trimmed = value.trim();
  let lower = trimmed.to_ascii_lowercase();
  match lower.as_str() {
    "none" => {
      style.flex_grow = Some(0.0);
      style.flex_shrink = Some(0.0);
      style.flex_basis = Some(CssLength::Auto);
      return;
    }
    "auto" => {
      style.flex_grow = Some(1.0);
      style.flex_shrink = Some(1.0);
      style.flex_basis = Some(CssLength::Auto);
      return;
    }
    "initial" => {
      style.flex_grow = Some(0.0);
      style.flex_shrink = Some(1.0);
      style.flex_basis = Some(CssLength::Auto);
      return;
    }
    _ => {}
  }

  let tokens: Vec<&str> = trimmed.split_whitespace().collect();
  let is_number = |t: &str| t.parse::<f32>().is_ok();
  let mut grow: Option<f32> = None;
  let mut shrink: Option<f32> = None;
  let mut basis: Option<CssLength> = None;

  match tokens.len() {
    0 => return,
    1 => {
      let t = tokens[0];
      if is_number(t) {
        grow = t.parse().ok();
        shrink = Some(1.0);
        basis = Some(CssLength::Percent(0.0));
      } else if let Some(b) = parse_css_length(t) {
        grow = Some(1.0);
        shrink = Some(1.0);
        basis = Some(b);
      }
    }
    2 => {
      let (a, b) = (tokens[0], tokens[1]);
      if is_number(a) && is_number(b) {
        grow = a.parse().ok();
        shrink = b.parse().ok();
        basis = Some(CssLength::Percent(0.0));
      } else if is_number(a) {
        grow = a.parse().ok();
        shrink = Some(1.0);
        basis = parse_css_length(b);
      }
    }
    _ => {
      // Three (or more -- extra ignored) tokens: grow shrink basis.
      grow = tokens[0].parse().ok();
      shrink = tokens[1].parse().ok();
      basis = parse_css_length(tokens[2]);
    }
  }

  if let Some(g) = grow {
    style.flex_grow = Some(g);
  }
  if let Some(s) = shrink {
    style.flex_shrink = Some(s);
  }
  if let Some(b) = basis {
    style.flex_basis = Some(b);
  }
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

fn apply_gap_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["gap", "row-gap", "column-gap"]);
  let parts = split_top_level_whitespace(value);
  let first = parts.first().copied().unwrap_or("");
  let second = parts.get(1).copied().unwrap_or(first);
  style.gap = parse_css_length(first);
  style.row_gap = parse_css_length(first);
  style.column_gap = parse_css_length(second);
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
