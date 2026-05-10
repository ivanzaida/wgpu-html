use std::collections::HashMap;

use lui_models::{ArcStr, Style};

use crate::{
  shorthands::{all_shorthands, is_deferred_longhand, shorthand_contains_member, shorthand_members},
  style_props::{clear_value_for, merge_values_clearing_keywords},
};

// ── Submodules ───────────────────────────────────────────────────────────────

mod border;
mod lui_resolve;
mod shorthand;
pub mod values;
pub mod var;

// ── Re-exports (crate-internal) ──────────────────────────────────────────────
// Make all pub(crate) items from submodules available in this module's
// namespace so that `apply_css_property` and the inline-style parsers
// can call them without qualifying the submodule name.

pub(crate) use border::*;
pub(crate) use shorthand::*;
pub(crate) use values::*;

pub(crate) use var::value_contains_var;

// ── Public re-exports ────────────────────────────────────────────────────────
// These items are accessed by `lib.rs` and external consumers at the
// `css_parser` module level. They must stay `pub`.

pub use border::parse_box_shorthand;
pub use lui_resolve::{
  resolve_lui_calendar_style, resolve_lui_color_picker_style, resolve_lui_popup_style,
};
pub use values::{parse_css_color, parse_css_image, parse_css_length, parse_css_url};

// ── CSS-wide keyword ─────────────────────────────────────────────────────────

/// CSS-wide keyword that any property can take as its value.
/// Resolution against the parent's resolved style happens in the
/// cascade — see `lui_style::keywords`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CssWideKeyword {
  /// Use the parent's resolved value for this property, regardless
  /// of whether the property is normally inherited.
  Inherit,
  /// Use the property's initial value (no UA defaults are tracked,
  /// so this resolves to "unset" in the typed `Style`).
  Initial,
  /// Behaves like `inherit` for inherited properties, like `initial`
  /// for non-inherited ones.
  Unset,
}

impl CssWideKeyword {
  pub fn from_value(v: &str) -> Option<Self> {
    let trimmed = v.trim();
    if trimmed.eq_ignore_ascii_case("inherit") {
      Some(Self::Inherit)
    } else if trimmed.eq_ignore_ascii_case("initial") {
      Some(Self::Initial)
    } else if trimmed.eq_ignore_ascii_case("unset") {
      Some(Self::Unset)
    } else {
      None
    }
  }
}

// ── Style declarations ───────────────────────────────────────────────────────

/// One block's worth of CSS declarations, partitioned by importance.
/// Per CSS-Cascade-3 the cascade applies normal declarations first
/// (with rule + inline ordering) and then important declarations on
/// top, so we keep the two `Style` payloads separate from parse time.
///
/// CSS-wide keywords (`inherit / initial / unset`) live in side-car
/// hash maps keyed by the property's CSS name (kebab-case). They
/// override any value the same property might have on `normal` /
/// `important` from a different layer; the cascade resolves them
/// against the parent's resolved style.
#[derive(Debug, Clone, Default)]
pub struct StyleDecls {
  pub normal: Style,
  pub important: Style,
  pub keywords_normal: HashMap<ArcStr, CssWideKeyword>,
  pub keywords_important: HashMap<ArcStr, CssWideKeyword>,
}

// ── Inline style parsing ─────────────────────────────────────────────────────

/// Parse an inline CSS style string (e.g. `"display: flex; color: red;"`)
/// into a `Style` struct. `!important` is recognised and stripped from
/// values so they parse correctly, but its effect is folded back in:
/// when a property is given as `!important`, it overrides the same
/// property declared as normal in the *same* string. CSS-wide
/// keywords (`inherit / initial / unset`) are recognised but dropped
/// — this back-compat surface returns `Style` only, so a keyword
/// resolution would need a parent. Use [`parse_inline_style_decls`]
/// to preserve the keywords for the cascade to handle.
pub fn parse_inline_style(css: &str) -> Style {
  let decls = parse_inline_style_decls(css);
  let mut out = decls.normal;
  overlay(&mut out, &decls.important);
  out
}

/// Full-fidelity parse: separate `normal` and `important` `Style`
/// payloads plus per-property CSS-wide keyword side-cars. Cascade-3
/// uses all four to compose final values.
pub fn parse_inline_style_decls(css: &str) -> StyleDecls {
  let mut decls = StyleDecls::default();
  for declaration in css.split(';') {
    let declaration = declaration.trim();
    if declaration.is_empty() {
      continue;
    }
    if let Some((property, value)) = declaration.split_once(':') {
      let raw_prop = property.trim();
      // CSS custom properties (--*) are case-sensitive; everything
      // else is ASCII-lowercased per the CSS spec.
      let property: ArcStr = if raw_prop.starts_with("--") {
        ArcStr::from(raw_prop)
      } else {
        ArcStr::from(raw_prop.to_ascii_lowercase().as_str())
      };
      let (value, important) = strip_important(value.trim());

      // CSS-wide keywords pre-empt the value parsers. They go
      // into the side-car keyword map and clear any matching
      // value the same bucket may have set earlier in this same
      // declaration block — within a layer, last-write-wins,
      // so a later keyword has to displace any earlier value.
      if let Some(kw) = CssWideKeyword::from_value(value) {
        if important {
          mark_keyword_resets(&mut decls.important, &property);
          clear_value_for(&property, &mut decls.important);
          clear_keywords_for_property(&property, &mut decls.keywords_important);
          decls.keywords_important.insert(property, kw);
        } else {
          mark_keyword_resets(&mut decls.normal, &property);
          clear_value_for(&property, &mut decls.normal);
          clear_keywords_for_property(&property, &mut decls.keywords_normal);
          decls.keywords_normal.insert(property, kw);
        }
        continue;
      }

      // Conversely: a value declaration drops any keyword
      // override the same bucket may have recorded earlier in
      // the block.
      if important {
        let mut parsed = Style::default();
        apply_css_property(&mut parsed, &property, value);
        merge_values_clearing_keywords(&mut decls.important, &mut decls.keywords_important, &parsed);
      } else {
        let mut parsed = Style::default();
        apply_css_property(&mut parsed, &property, value);
        merge_values_clearing_keywords(&mut decls.normal, &mut decls.keywords_normal, &parsed);
      }
    }
  }
  decls
}

fn clear_keywords_for_property(prop: &str, keywords: &mut HashMap<ArcStr, CssWideKeyword>) {
  keywords.remove(prop);
  for shorthand in all_shorthands() {
    if shorthand_contains_member(shorthand, prop) {
      keywords.remove(*shorthand);
    }
  }
  if let Some(members) = shorthand_members(prop) {
    for member in members {
      if *member != prop {
        clear_keywords_for_property(member, keywords);
      }
    }
  }
}

fn mark_keyword_resets(style: &mut Style, prop: &str) {
  if let Some(members) = shorthand_members(prop) {
    for member in members {
      if *member != prop {
        mark_keyword_resets(style, member);
        style.keyword_reset_properties.insert(ArcStr::from(*member));
      }
    }
  } else {
    style.keyword_reset_properties.insert(ArcStr::from(prop));
  }
}

/// Recognise a trailing `!important` (or `! important`, with arbitrary
/// whitespace between the bang and the keyword, per CSS spec). Returns
/// the cleaned value and a flag.
fn strip_important(value: &str) -> (&str, bool) {
  let trimmed = value.trim_end();
  // Take the trailing alphabetic word; if it's `important` (case-
  // insensitive), look back for the `!` allowing whitespace between.
  let bytes = trimmed.as_bytes();
  let mut i = bytes.len();
  while i > 0 && bytes[i - 1].is_ascii_alphabetic() {
    i -= 1;
  }
  let word = &trimmed[i..];
  if !word.eq_ignore_ascii_case("important") {
    return (trimmed, false);
  }
  // Walk back over whitespace, then expect `!`.
  let mut j = i;
  while j > 0 && bytes[j - 1].is_ascii_whitespace() {
    j -= 1;
  }
  if j == 0 || bytes[j - 1] != b'!' {
    return (trimmed, false);
  }
  let cleaned = trimmed[..j - 1].trim_end();
  (cleaned, true)
}

/// Right-merge: copy each `Some` field from `src` over the matching
/// field on `dst`. Lives here (instead of in `lui-style::merge`)
/// so the parser is self-contained when folding `!important` back into
/// the legacy `parse_inline_style` API. The full Style cascade still
/// uses `lui_style::merge` which is identical in behaviour.
fn overlay(dst: &mut Style, src: &Style) {
  for prop in &src.reset_properties {
    clear_value_for(prop, dst);
  }
  for prop in &src.keyword_reset_properties {
    clear_value_for(prop, dst);
  }
  macro_rules! overlay_fields {
        ($($field:ident),* $(,)?) => {
            $(
                if src.$field.is_some() {
                    dst.$field = src.$field.clone();
                }
            )*
        };
    }
  overlay_fields!(
    display,
    position,
    top,
    right,
    bottom,
    left,
    width,
    height,
    min_width,
    min_height,
    max_width,
    max_height,
    margin,
    margin_top,
    margin_right,
    margin_bottom,
    margin_left,
    padding,
    padding_top,
    padding_right,
    padding_bottom,
    padding_left,
    color,
    background,
    background_color,
    background_image,
    background_size,
    background_position,
    background_repeat,
    background_clip,
    border,
    border_top_width,
    border_right_width,
    border_bottom_width,
    border_left_width,
    border_top_style,
    border_right_style,
    border_bottom_style,
    border_left_style,
    border_top_color,
    border_right_color,
    border_bottom_color,
    border_left_color,
    border_top_left_radius,
    border_top_right_radius,
    border_bottom_right_radius,
    border_bottom_left_radius,
    border_top_left_radius_v,
    border_top_right_radius_v,
    border_bottom_right_radius_v,
    border_bottom_left_radius_v,
    font_family,
    font_size,
    font_weight,
    font_style,
    line_height,
    letter_spacing,
    text_align,
    text_decoration,
    text_transform,
    white_space,
    word_break,
    vertical_align,
    text_overflow,
    overflow,
    overflow_x,
    overflow_y,
    resize,
    opacity,
    visibility,
    z_index,
    flex_direction,
    flex_wrap,
    justify_content,
    align_items,
    align_content,
    align_self,
    order,
    gap,
    row_gap,
    column_gap,
    flex,
    flex_grow,
    flex_shrink,
    flex_basis,
    grid_template_columns,
    grid_template_rows,
    grid_auto_columns,
    grid_auto_rows,
    grid_auto_flow,
    grid_column,
    grid_column_start,
    grid_column_end,
    grid_row,
    grid_row_start,
    grid_row_end,
    justify_items,
    justify_self,
    transform,
    transform_origin,
    transition,
    animation,
    cursor,
    pointer_events,
    user_select,
    box_shadow,
    box_sizing,
  );
  for (prop, value) in &src.deferred_longhands {
    dst.deferred_longhands.insert(prop.clone(), value.clone());
  }
  dst.reset_properties.extend(src.reset_properties.iter().cloned());
  dst
    .keyword_reset_properties
    .extend(src.keyword_reset_properties.iter().cloned());
}

// ── Property application ─────────────────────────────────────────────────────

pub fn apply_css_property(style: &mut Style, property: &str, value: &str) {
  // Custom properties (--*): store in side-car map.
  if property.starts_with("--") {
    style.custom_properties.insert(ArcStr::from(property), ArcStr::from(value));
    return;
  }
  // Values containing var(): defer resolution until computed-value time.
  if value_contains_var(value) {
    style.var_properties.insert(ArcStr::from(property), ArcStr::from(value));
    return;
  }
  match property {
    "display" => style.display = parse_display(value),
    "position" => style.position = parse_position(value),
    "top" => style.top = parse_css_length(value),
    "right" => style.right = parse_css_length(value),
    "bottom" => style.bottom = parse_css_length(value),
    "left" => style.left = parse_css_length(value),
    "width" => style.width = parse_css_length(value),
    "height" => style.height = parse_css_length(value),
    "min-width" => style.min_width = parse_css_length(value),
    "min-height" => style.min_height = parse_css_length(value),
    "max-width" => style.max_width = parse_css_length(value),
    "max-height" => style.max_height = parse_css_length(value),
    "margin" => {
      let (t, r, b, l) = parse_box_shorthand(value);
      if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
        mark_property_resets(style, &["margin-top", "margin-right", "margin-bottom", "margin-left"]);
        style.margin = t.clone();
        style.margin_top = t;
        style.margin_right = r;
        style.margin_bottom = b;
        style.margin_left = l;
      }
    }
    "margin-top" => style.margin_top = parse_css_length(value),
    "margin-right" => style.margin_right = parse_css_length(value),
    "margin-bottom" => style.margin_bottom = parse_css_length(value),
    "margin-left" => style.margin_left = parse_css_length(value),
    "padding" => {
      let (t, r, b, l) = parse_box_shorthand(value);
      if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
        mark_property_resets(
          style,
          &["padding-top", "padding-right", "padding-bottom", "padding-left"],
        );
        style.padding = t.clone();
        style.padding_top = t;
        style.padding_right = r;
        style.padding_bottom = b;
        style.padding_left = l;
      }
    }
    "padding-top" => style.padding_top = parse_css_length(value),
    "padding-right" => style.padding_right = parse_css_length(value),
    "padding-bottom" => style.padding_bottom = parse_css_length(value),
    "padding-left" => style.padding_left = parse_css_length(value),
    "color" => style.color = parse_css_color(value),
    "accent-color" => style.accent_color = parse_css_color(value),
    "background" => apply_background_shorthand(value, style),
    "background-color" => style.background_color = parse_css_color(value),
    "background-image" => style.background_image = parse_css_image(value),
    "background-size" => style.background_size = Some(ArcStr::from(value)),
    "background-position" => apply_background_position_shorthand(value, style),
    "background-repeat" => style.background_repeat = parse_background_repeat(value),
    "background-clip" => style.background_clip = parse_background_clip(value),
    "border" => parse_border_shorthand(value, style),
    "border-top" => parse_border_side_shorthand(value, style, Side::Top),
    "border-right" => parse_border_side_shorthand(value, style, Side::Right),
    "border-bottom" => parse_border_side_shorthand(value, style, Side::Bottom),
    "border-left" => parse_border_side_shorthand(value, style, Side::Left),

    "border-width" => apply_border_widths(value, style),
    "border-style" => apply_border_styles(value, style),
    "border-color" => apply_border_colors(value, style),

    "border-top-width" => style.border_top_width = parse_css_length(value),
    "border-right-width" => style.border_right_width = parse_css_length(value),
    "border-bottom-width" => style.border_bottom_width = parse_css_length(value),
    "border-left-width" => style.border_left_width = parse_css_length(value),

    "border-top-style" => style.border_top_style = parse_border_style(value),
    "border-right-style" => style.border_right_style = parse_border_style(value),
    "border-bottom-style" => style.border_bottom_style = parse_border_style(value),
    "border-left-style" => style.border_left_style = parse_border_style(value),

    "border-top-color" => style.border_top_color = parse_css_color(value),
    "border-right-color" => style.border_right_color = parse_css_color(value),
    "border-bottom-color" => style.border_bottom_color = parse_css_color(value),
    "border-left-color" => style.border_left_color = parse_css_color(value),

    "border-radius" => apply_border_radii(value, style),
    "border-top-left-radius" => apply_corner_radius(
      value,
      &mut style.border_top_left_radius,
      &mut style.border_top_left_radius_v,
    ),
    "border-top-right-radius" => apply_corner_radius(
      value,
      &mut style.border_top_right_radius,
      &mut style.border_top_right_radius_v,
    ),
    "border-bottom-right-radius" => apply_corner_radius(
      value,
      &mut style.border_bottom_right_radius,
      &mut style.border_bottom_right_radius_v,
    ),
    "border-bottom-left-radius" => apply_corner_radius(
      value,
      &mut style.border_bottom_left_radius,
      &mut style.border_bottom_left_radius_v,
    ),
    "font-family" => style.font_family = Some(ArcStr::from(value)),
    "font-size" => style.font_size = parse_css_length(value),
    "font-weight" => style.font_weight = parse_font_weight(value),
    "font-style" => style.font_style = parse_font_style(value),
    "line-height" => style.line_height = parse_css_length(value),
    "letter-spacing" => style.letter_spacing = parse_css_length(value),
    "text-align" => style.text_align = parse_text_align(value),
    "text-decoration" => apply_text_decoration_shorthand(value, style),
    "text-transform" => style.text_transform = parse_text_transform(value),
    "white-space" => apply_white_space_property(value, style),
    "word-break" => { style.word_break = parse_word_break(value); }
    "vertical-align" => { style.vertical_align = parse_vertical_align(value); }
    "text-overflow" => style.text_overflow = parse_text_overflow(value),
    "overflow" => apply_overflow_shorthand(value, style),
    "overflow-x" => style.overflow_x = parse_overflow(value),
    "overflow-y" => style.overflow_y = parse_overflow(value),
    "scrollbar-color" => style.scrollbar_color = parse_scrollbar_color(value),
    "scrollbar-width" => style.scrollbar_width = parse_scrollbar_width(value),
    "resize" => style.resize = parse_resize(value),
    "opacity" => style.opacity = value.parse().ok(),
    "visibility" => style.visibility = parse_visibility(value),
    "z-index" => style.z_index = value.parse().ok(),

    // SVG presentation properties ------------------------------------
    "fill" => style.svg_fill = parse_css_color(value),
    "fill-opacity" => style.svg_fill_opacity = value.trim().parse().ok(),
    "fill-rule" => {
      let v = value.trim();
      if matches!(v, "nonzero" | "evenodd") {
        style.svg_fill_rule = Some(ArcStr::from(v));
      }
    }
    "stroke" => style.svg_stroke = parse_css_color(value),
    "stroke-width" => style.svg_stroke_width = parse_css_length(value),
    "stroke-opacity" => style.svg_stroke_opacity = value.trim().parse().ok(),
    "stroke-linecap" => {
      let v = value.trim();
      if matches!(v, "butt" | "round" | "square") {
        style.svg_stroke_linecap = Some(ArcStr::from(v));
      }
    }
    "stroke-linejoin" => {
      let v = value.trim();
      if matches!(v, "miter" | "round" | "bevel" | "arcs" | "miter-clip") {
        style.svg_stroke_linejoin = Some(ArcStr::from(v));
      }
    }
    "stroke-dasharray" => {
      let v = value.trim();
      if v != "none" {
        style.svg_stroke_dasharray = Some(ArcStr::from(v));
      }
    }
    "stroke-dashoffset" => style.svg_stroke_dashoffset = parse_css_length(value),
    "flex-direction" => style.flex_direction = parse_flex_direction(value),
    "flex-wrap" => style.flex_wrap = parse_flex_wrap(value),
    "justify-content" => style.justify_content = parse_justify_content(value),
    "align-items" => style.align_items = parse_align_items(value),
    "align-content" => style.align_content = parse_align_content(value),
    "align-self" => style.align_self = parse_align_self(value),
    "order" => style.order = value.trim().parse().ok(),
    "gap" => apply_gap_shorthand(value, style),
    "row-gap" => style.row_gap = parse_css_length(value),
    "column-gap" => style.column_gap = parse_css_length(value),
    "flex" => apply_flex_shorthand(value, style),
    "flex-grow" => style.flex_grow = value.parse().ok(),
    "flex-shrink" => style.flex_shrink = value.parse().ok(),
    "flex-basis" => style.flex_basis = parse_css_length(value),
    "grid-template-columns" => {
      let list = parse_grid_track_list(value);
      if !list.is_empty() {
        style.grid_template_columns = Some(list);
      }
    }
    "grid-template-rows" => {
      let list = parse_grid_track_list(value);
      if !list.is_empty() {
        style.grid_template_rows = Some(list);
      }
    }
    "grid-auto-columns" => style.grid_auto_columns = parse_grid_track_size(value),
    "grid-auto-rows" => style.grid_auto_rows = parse_grid_track_size(value),
    "grid-auto-flow" => style.grid_auto_flow = parse_grid_auto_flow(value),
    "grid-column" => apply_grid_axis_shorthand(value, style, GridAxis::Column),
    "grid-column-start" => style.grid_column_start = parse_grid_line(value),
    "grid-column-end" => style.grid_column_end = parse_grid_line(value),
    "grid-row" => apply_grid_axis_shorthand(value, style, GridAxis::Row),
    "grid-row-start" => style.grid_row_start = parse_grid_line(value),
    "grid-row-end" => style.grid_row_end = parse_grid_line(value),
    "justify-items" => style.justify_items = parse_justify_items(value),
    "justify-self" => style.justify_self = parse_justify_self(value),
    "transform" => style.transform = Some(ArcStr::from(value)),
    "transform-origin" => style.transform_origin = Some(ArcStr::from(value)),
    "transition" => apply_transition_shorthand(value, style),
    "animation" => apply_animation_shorthand(value, style),
    "cursor" => style.cursor = parse_cursor(value),
    "pointer-events" => style.pointer_events = parse_pointer_events(value),
    "user-select" => style.user_select = parse_user_select(value),
    "content" => style.content = parse_css_content(value),
    "box-shadow" => style.box_shadow = Some(ArcStr::from(value)),
    "box-sizing" => style.box_sizing = parse_box_sizing(value),
    "list-style-type" => style.list_style_type = parse_list_style_type(value),
    "list-style-position" => style.list_style_position = parse_list_style_position(value),
    _ if shorthand_members(property).is_some() => apply_generic_shorthand(style, property, value),
    _ if is_deferred_longhand(property) => {
      style.deferred_longhands.insert(ArcStr::from(property), ArcStr::from(value));
    }
    _ => {} // Unknown CSS properties are silently ignored
  }
}
