use std::collections::HashMap;

use wgpu_html_models::{ArcStr, Style, common::css_enums::*};

use crate::{
  shorthands::{all_shorthands, is_deferred_longhand, shorthand_contains_member, shorthand_members},
  style_props::{clear_value_for, merge_values_clearing_keywords},
};

/// CSS-wide keyword that any property can take as its value.
/// Resolution against the parent's resolved style happens in the
/// cascade — see `wgpu_html_style::keywords`.
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
/// field on `dst`. Lives here (instead of in `wgpu-html-style::merge`)
/// so the parser is self-contained when folding `!important` back into
/// the legacy `parse_inline_style` API. The full Style cascade still
/// uses `wgpu_html_style::merge` which is identical in behaviour.
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
    "box-shadow" => style.box_shadow = Some(ArcStr::from(value)),
    "box-sizing" => style.box_sizing = parse_box_sizing(value),
    _ if shorthand_members(property).is_some() => apply_generic_shorthand(style, property, value),
    _ if is_deferred_longhand(property) => {
      style.deferred_longhands.insert(ArcStr::from(property), ArcStr::from(value));
    }
    _ => {} // Unknown CSS properties are silently ignored
  }
}

/// Returns `true` if the value string contains a `var(` token that
/// isn't inside a quoted string. This is a conservative check.
fn value_contains_var(value: &str) -> bool {
  let bytes = value.as_bytes();
  let mut i = 0;
  let mut quote: Option<u8> = None;
  while i < bytes.len() {
    match quote {
      Some(q) => {
        if bytes[i] == q {
          quote = None;
        }
      }
      None => match bytes[i] {
        b'"' | b'\'' => quote = Some(bytes[i]),
        b'v' | b'V' => {
          if i + 4 <= bytes.len()
            && bytes[i + 1].to_ascii_lowercase() == b'a'
            && bytes[i + 2].to_ascii_lowercase() == b'r'
            && bytes[i + 3] == b'('
          {
            return true;
          }
        }
        _ => {}
      },
    }
    i += 1;
  }
  false
}

/// Resolve all `var()` references in a fully-cascaded, inherited style.
///
/// Phase 1: resolve `var()` inside custom-property values so that
/// `--a: var(--b)` chains collapse.
/// Phase 2: for every entry in `var_properties`, substitute variables
/// and re-parse the resolved value through `apply_css_property`.
pub fn resolve_var_references(style: &mut Style) {
  // Phase 1 — resolve var() inside custom property values.
  let keys: Vec<ArcStr> = style.custom_properties.keys().cloned().collect();
  let mut resolved_cp = style.custom_properties.clone();
  for key in &keys {
    let mut resolving = std::collections::HashSet::new();
    if let Some(val) = resolved_cp.get(key.as_ref()).cloned() {
      if value_contains_var(&val) {
        resolving.insert(key.to_string());
        let substituted = substitute_vars(&val, &resolved_cp, &mut resolving);
        resolved_cp.insert(key.clone(), ArcStr::from(substituted.as_str()));
      }
    }
  }
  style.custom_properties = resolved_cp;

  // Phase 2 — resolve var() in regular property declarations.
  let pending: Vec<(ArcStr, ArcStr)> = style.var_properties.drain().collect();
  for (prop, raw_value) in pending {
    let mut resolving = std::collections::HashSet::new();
    let resolved = substitute_vars(&raw_value, &style.custom_properties, &mut resolving);
    if !resolved.is_empty() {
      apply_css_property(style, &prop, &resolved);
    }
  }
}

/// Replace all `var(--name)` and `var(--name, fallback)` occurrences
/// in `value` with the corresponding custom-property value. Detects
/// cycles via `resolving` and falls back gracefully.
fn substitute_vars(
  value: &str,
  custom_props: &std::collections::HashMap<ArcStr, ArcStr>,
  resolving: &mut std::collections::HashSet<String>,
) -> String {
  let bytes = value.as_bytes();
  let mut out = String::with_capacity(value.len());
  let mut i = 0;
  let mut quote: Option<u8> = None;

  while i < bytes.len() {
    match quote {
      Some(q) => {
        out.push(bytes[i] as char);
        if bytes[i] == q {
          quote = None;
        }
        i += 1;
      }
      None => {
        if (bytes[i] == b'v' || bytes[i] == b'V')
          && i + 4 <= bytes.len()
          && bytes[i + 1].to_ascii_lowercase() == b'a'
          && bytes[i + 2].to_ascii_lowercase() == b'r'
          && bytes[i + 3] == b'('
        {
          // Found `var(` — parse the contents.
          let start = i;
          i += 4; // skip "var("
          // Find matching `)`.
          let mut depth = 1i32;
          let inner_start = i;
          while i < bytes.len() && depth > 0 {
            match bytes[i] {
              b'(' => depth += 1,
              b')' => depth -= 1,
              b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                  i += 1;
                }
              }
              _ => {}
            }
            if depth > 0 {
              i += 1;
            }
          }
          if depth != 0 {
            // Unbalanced — emit raw.
            out.push_str(&value[start..]);
            break;
          }
          let inner = value[inner_start..i].trim();
          i += 1; // skip ')'

          // Split inner into name and optional fallback.
          let (name, fallback) = split_var_args(inner);
          let name = name.trim();

          if resolving.contains(name) {
            // Cycle — use fallback.
            if let Some(fb) = fallback {
              out.push_str(&substitute_vars(fb.trim(), custom_props, resolving));
            }
          } else if let Some(cp_val) = custom_props.get(name) {
            let mut resolved = cp_val.to_string();
            if value_contains_var(&resolved) {
              resolving.insert(name.to_owned());
              resolved = substitute_vars(&resolved, custom_props, resolving);
              resolving.remove(name);
            }
            out.push_str(&resolved);
          } else if let Some(fb) = fallback {
            out.push_str(&substitute_vars(fb.trim(), custom_props, resolving));
          }
          // If no value and no fallback, nothing is appended
          // (guaranteed-invalid per CSS spec).
        } else {
          if bytes[i] == b'"' || bytes[i] == b'\'' {
            quote = Some(bytes[i]);
          }
          out.push(bytes[i] as char);
          i += 1;
        }
      }
    }
  }
  out
}

/// Split the argument inside `var(...)` into (name, optional_fallback).
/// The fallback is everything after the first top-level `,`.
fn split_var_args(s: &str) -> (&str, Option<&str>) {
  let bytes = s.as_bytes();
  let mut depth = 0i32;
  let mut i = 0;
  while i < bytes.len() {
    match bytes[i] {
      b'(' => depth += 1,
      b')' => depth -= 1,
      b',' if depth == 0 => {
        return (&s[..i], Some(&s[i + 1..]));
      }
      _ => {}
    }
    i += 1;
  }
  (s, None)
}

fn apply_generic_shorthand(style: &mut Style, property: &str, value: &str) {
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

fn mark_shorthand_reset(style: &mut Style, property: &str) {
  if let Some(members) = shorthand_members(property) {
    for member in members {
      style.reset_properties.insert(ArcStr::from(*member));
    }
  }
}

fn set_deferred(style: &mut Style, property: &str, value: impl AsRef<str>) {
  style.deferred_longhands.insert(ArcStr::from(property), ArcStr::from(value.as_ref()));
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
  let second_value = tokens
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

fn apply_gap_shorthand(value: &str, style: &mut Style) {
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

fn apply_text_decoration_shorthand(value: &str, style: &mut Style) {
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

fn apply_white_space_property(value: &str, style: &mut Style) {
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
      "inside" | "outside" => set_deferred(style, "list-style-position", token),
      _ if parse_css_image(token).is_some() || token.eq_ignore_ascii_case("none") => {
        set_deferred(style, "list-style-image", token)
      }
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

fn mark_property_resets(style: &mut Style, props: &[&str]) {
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

// ---------------------------------------------------------------------------
// CSS value parsers
// ---------------------------------------------------------------------------

/// Which side a per-side border helper writes to.
#[derive(Copy, Clone)]
enum Side {
  Top,
  Right,
  Bottom,
  Left,
}

/// Parse the `border` shorthand into width / style / color, in any order.
/// The values fan out to all four sides (per CSS spec: `border` is itself
/// a shorthand for the four `border-<side>-<piece>` longhands).
pub fn parse_border_shorthand(value: &str, style: &mut Style) {
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
fn parse_border_side_shorthand(value: &str, style: &mut Style, side: Side) {
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
fn split_top_level_whitespace(value: &str) -> Vec<&str> {
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
fn apply_border_widths(value: &str, style: &mut Style) {
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
fn apply_border_styles(value: &str, style: &mut Style) {
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
fn apply_border_colors(value: &str, style: &mut Style) {
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
fn apply_border_radii(value: &str, style: &mut Style) {
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

/// `parse_css_length` minus its catch-all `Raw` / `Auto` returns — used
/// when matching one piece of a shorthand against multiple value kinds.
fn parse_definite_length(token: &str) -> Option<CssLength> {
  match parse_css_length(token)? {
    CssLength::Raw(_) | CssLength::Auto => None,
    other => Some(other),
  }
}

/// Parse a CSS length value (e.g. "10px", "50%", "1.5em", "auto").
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

pub fn parse_css_length(value: &str) -> Option<CssLength> {
  let v = value.trim();
  if v.is_empty() {
    return None;
  }
  if v.eq_ignore_ascii_case("auto") {
    return Some(CssLength::Auto);
  }
  if v == "0" {
    return Some(CssLength::Zero);
  }
  if let Some(inner) = strip_func(v, "calc") {
    if let Some(expr) = parse_css_math_expr(inner) {
      return Some(CssLength::Calc(Box::new(expr)));
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "min") {
    let args: Vec<CssLength> = split_top_level_commas(inner)
      .into_iter()
      .filter_map(parse_css_length)
      .collect();
    if !args.is_empty() {
      return Some(CssLength::Min(args));
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "max") {
    let args: Vec<CssLength> = split_top_level_commas(inner)
      .into_iter()
      .filter_map(parse_css_length)
      .collect();
    if !args.is_empty() {
      return Some(CssLength::Max(args));
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "clamp") {
    let args: Vec<CssLength> = split_top_level_commas(inner)
      .into_iter()
      .filter_map(parse_css_length)
      .collect();
    if args.len() == 3 {
      return Some(CssLength::Clamp {
        min: Box::new(args[0].clone()),
        preferred: Box::new(args[1].clone()),
        max: Box::new(args[2].clone()),
      });
    }
    return Some(CssLength::Raw(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "fit-content") {
    return parse_css_length(inner).or_else(|| Some(CssLength::Raw(ArcStr::from(v))));
  }
  if is_numeric_function_value(v) {
    if let Some(expr) = parse_css_math_expr(v) {
      return Some(CssLength::Calc(Box::new(expr)));
    }
  }
  if let Some(s) = v.strip_suffix("px") {
    return s.trim().parse::<f32>().ok().map(CssLength::Px);
  }
  if let Some(s) = v.strip_suffix('%') {
    return s.trim().parse::<f32>().ok().map(CssLength::Percent);
  }
  if let Some(s) = v.strip_suffix("rem") {
    return s.trim().parse::<f32>().ok().map(CssLength::Rem);
  }
  if let Some(s) = v.strip_suffix("em") {
    return s.trim().parse::<f32>().ok().map(CssLength::Em);
  }
  if let Some(s) = v.strip_suffix("vw") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vw);
  }
  if let Some(s) = v.strip_suffix("vh") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vh);
  }
  if let Some(s) = v.strip_suffix("vmin") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vmin);
  }
  if let Some(s) = v.strip_suffix("vmax") {
    return s.trim().parse::<f32>().ok().map(CssLength::Vmax);
  }
  // Bare number (treat as raw)
  Some(CssLength::Raw(ArcStr::from(v)))
}

fn is_numeric_function_value(v: &str) -> bool {
  let Some(open) = v.find('(') else {
    return false;
  };
  let name = v[..open].trim();
  numeric_function_from_name(name).is_some() && v.ends_with(')')
}

fn parse_css_math_expr(input: &str) -> Option<CssMathExpr> {
  let mut parser = MathParser::new(input);
  let expr = parser.parse_sum()?;
  parser.skip_ws();
  if parser.is_eof() { Some(expr) } else { None }
}

struct MathParser<'a> {
  input: &'a str,
  pos: usize,
}

impl<'a> MathParser<'a> {
  fn new(input: &'a str) -> Self {
    Self { input, pos: 0 }
  }

  fn parse_sum(&mut self) -> Option<CssMathExpr> {
    let mut lhs = self.parse_product()?;
    loop {
      self.skip_ws();
      if self.consume_char('+') {
        let rhs = self.parse_product()?;
        lhs = CssMathExpr::Add(Box::new(lhs), Box::new(rhs));
      } else if self.consume_char('-') {
        let rhs = self.parse_product()?;
        lhs = CssMathExpr::Sub(Box::new(lhs), Box::new(rhs));
      } else {
        return Some(lhs);
      }
    }
  }

  fn parse_product(&mut self) -> Option<CssMathExpr> {
    let mut lhs = self.parse_unary()?;
    loop {
      self.skip_ws();
      if self.consume_char('*') {
        let rhs = self.parse_unary()?;
        lhs = CssMathExpr::Mul(Box::new(lhs), Box::new(rhs));
      } else if self.consume_char('/') {
        let rhs = self.parse_unary()?;
        lhs = CssMathExpr::Div(Box::new(lhs), Box::new(rhs));
      } else {
        return Some(lhs);
      }
    }
  }

  fn parse_unary(&mut self) -> Option<CssMathExpr> {
    self.skip_ws();
    if self.consume_char('+') {
      return self.parse_unary();
    }
    if self.consume_char('-') {
      let rhs = self.parse_unary()?;
      return Some(CssMathExpr::Sub(Box::new(CssMathExpr::Number(0.0)), Box::new(rhs)));
    }
    self.parse_primary()
  }

  fn parse_primary(&mut self) -> Option<CssMathExpr> {
    self.skip_ws();
    if self.consume_char('(') {
      let inner = self.parse_sum()?;
      self.skip_ws();
      return self.consume_char(')').then_some(inner);
    }

    let start = self.pos;
    let ch = self.peek_char()?;
    if ch.is_ascii_alphabetic() || ch == '_' {
      let name = self.consume_ident();
      self.skip_ws();
      if self.consume_char('(') {
        let args_start = self.pos;
        let mut depth = 1i32;
        while let Some(c) = self.next_char() {
          match c {
            '(' => depth += 1,
            ')' => {
              depth -= 1;
              if depth == 0 {
                let args = &self.input[args_start..self.pos - 1];
                return self.parse_function(&name, args);
              }
            }
            _ => {}
          }
        }
        return None;
      }
      self.pos = start;
    }

    self.parse_numeric_or_length()
  }

  fn parse_function(&self, name: &str, args: &str) -> Option<CssMathExpr> {
    let fn_kind = numeric_function_from_name(name)?;
    let parsed: Vec<CssMathExpr> = split_top_level_commas(args)
      .into_iter()
      .map(parse_css_math_expr)
      .collect::<Option<Vec<_>>>()?;
    Some(CssMathExpr::Function(fn_kind, parsed))
  }

  fn parse_numeric_or_length(&mut self) -> Option<CssMathExpr> {
    let start = self.pos;
    let _number = self.consume_number_text()?;
    let unit_start = self.pos;
    while let Some(c) = self.peek_char() {
      if c.is_ascii_alphabetic() || c == '%' {
        self.next_char();
      } else {
        break;
      }
    }
    let token = &self.input[start..self.pos];
    if self.pos > unit_start {
      return parse_css_length(token).map(CssMathExpr::Length);
    }
    token.parse::<f32>().ok().map(CssMathExpr::Number)
  }

  fn consume_number_text(&mut self) -> Option<&'a str> {
    let start = self.pos;
    if matches!(self.peek_char(), Some('+') | Some('-')) {
      self.next_char();
    }
    let mut saw_digit = false;
    while let Some(c) = self.peek_char() {
      if c.is_ascii_digit() {
        saw_digit = true;
        self.next_char();
      } else {
        break;
      }
    }
    if self.consume_char('.') {
      while let Some(c) = self.peek_char() {
        if c.is_ascii_digit() {
          saw_digit = true;
          self.next_char();
        } else {
          break;
        }
      }
    }
    if !saw_digit {
      self.pos = start;
      return None;
    }
    if matches!(self.peek_char(), Some('e') | Some('E')) {
      let exp_start = self.pos;
      self.next_char();
      if matches!(self.peek_char(), Some('+') | Some('-')) {
        self.next_char();
      }
      let mut saw_exp_digit = false;
      while let Some(c) = self.peek_char() {
        if c.is_ascii_digit() {
          saw_exp_digit = true;
          self.next_char();
        } else {
          break;
        }
      }
      if !saw_exp_digit {
        self.pos = exp_start;
      }
    }
    Some(&self.input[start..self.pos])
  }

  fn consume_ident(&mut self) -> String {
    let start = self.pos;
    while let Some(c) = self.peek_char() {
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        self.next_char();
      } else {
        break;
      }
    }
    self.input[start..self.pos].to_string()
  }

  fn skip_ws(&mut self) {
    while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
      self.next_char();
    }
  }

  fn consume_char(&mut self, expected: char) -> bool {
    if self.peek_char() == Some(expected) {
      self.next_char();
      true
    } else {
      false
    }
  }

  fn peek_char(&self) -> Option<char> {
    self.input[self.pos..].chars().next()
  }

  fn next_char(&mut self) -> Option<char> {
    let c = self.peek_char()?;
    self.pos += c.len_utf8();
    Some(c)
  }

  fn is_eof(&self) -> bool {
    self.pos >= self.input.len()
  }
}

fn numeric_function_from_name(name: &str) -> Option<CssNumericFunction> {
  match name.to_ascii_lowercase().as_str() {
    "sin" => Some(CssNumericFunction::Sin),
    "cos" => Some(CssNumericFunction::Cos),
    "tan" => Some(CssNumericFunction::Tan),
    "asin" => Some(CssNumericFunction::Asin),
    "acos" => Some(CssNumericFunction::Acos),
    "atan" => Some(CssNumericFunction::Atan),
    "atan2" => Some(CssNumericFunction::Atan2),
    "pow" => Some(CssNumericFunction::Pow),
    "sqrt" => Some(CssNumericFunction::Sqrt),
    "hypot" => Some(CssNumericFunction::Hypot),
    "log" => Some(CssNumericFunction::Log),
    "exp" => Some(CssNumericFunction::Exp),
    "abs" => Some(CssNumericFunction::Abs),
    "sign" => Some(CssNumericFunction::Sign),
    "mod" => Some(CssNumericFunction::Mod),
    "rem" => Some(CssNumericFunction::Rem),
    "round" => Some(CssNumericFunction::Round),
    _ => None,
  }
}

/// Parse a CSS color value.
pub fn parse_css_color(value: &str) -> Option<CssColor> {
  let v = value.trim();
  if v.eq_ignore_ascii_case("transparent") {
    return Some(CssColor::Transparent);
  }
  if v.eq_ignore_ascii_case("currentcolor") || v.eq_ignore_ascii_case("currentColor") {
    return Some(CssColor::CurrentColor);
  }
  if v.starts_with('#') {
    return Some(CssColor::Hex(ArcStr::from(v)));
  }
  if let Some(inner) = strip_func(v, "rgba").or_else(|| strip_func(v, "rgb")) {
    let parts = split_color_function_args(inner);
    if parts.len() >= 3 {
      let r = parse_color_component(parts[0]);
      let g = parse_color_component(parts[1]);
      let b = parse_color_component(parts[2]);
      if let Some(alpha) = parts.get(3).map(|s| parse_alpha_component(s)) {
        return Some(CssColor::Rgba(r, g, b, alpha));
      }
      return Some(CssColor::Rgb(r, g, b));
    }
  }
  if let Some(inner) = strip_func(v, "hsla").or_else(|| strip_func(v, "hsl")) {
    let parts = split_color_function_args(inner);
    if parts.len() >= 3 {
      let h = parse_hue_component(parts[0]);
      let s = parts[1].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
      let l = parts[2].trim_end_matches('%').parse::<f32>().unwrap_or(0.0);
      if let Some(alpha) = parts.get(3).map(|s| parse_alpha_component(s)) {
        return Some(CssColor::Hsla(h, s, l, alpha));
      }
      return Some(CssColor::Hsl(h, s, l));
    }
  }
  if is_preserved_color_function(v) {
    return Some(CssColor::Function(ArcStr::from(v)));
  }
  // Treat as named color
  Some(CssColor::Named(ArcStr::from(v)))
}

pub fn parse_css_image(value: &str) -> Option<CssImage> {
  let v = value.trim();
  if v.is_empty() || v.eq_ignore_ascii_case("none") {
    return None;
  }
  if let Some(url) = parse_css_url(v) {
    return Some(CssImage::Url(url));
  }
  if looks_like_function(v) {
    return Some(CssImage::Function(ArcStr::from(v)));
  }
  None
}

pub fn parse_css_url(value: &str) -> Option<ArcStr> {
  let inner = strip_function(value, "url")?;
  let inner = inner.trim();
  if inner.is_empty() {
    return None;
  }
  let unquoted =
    if (inner.starts_with('"') && inner.ends_with('"')) || (inner.starts_with('\'') && inner.ends_with('\'')) {
      if inner.len() < 2 {
        return None;
      }
      &inner[1..inner.len() - 1]
    } else {
      inner
    };
  let trimmed = unquoted.trim();
  if trimmed.is_empty() {
    None
  } else {
    Some(ArcStr::from(trimmed))
  }
}

fn looks_like_function(value: &str) -> bool {
  let trimmed = value.trim();
  let Some(open) = trimmed.find('(') else {
    return false;
  };
  trimmed.ends_with(')') && trimmed[..open].chars().all(|c| c.is_ascii_alphabetic() || c == '-')
}

fn strip_func<'a>(value: &'a str, func_name: &str) -> Option<&'a str> {
  let v = value.trim();
  if v.len() > func_name.len() + 2
    && v[..func_name.len()].eq_ignore_ascii_case(func_name)
    && v.as_bytes()[func_name.len()] == b'('
    && v.ends_with(')')
  {
    Some(&v[func_name.len() + 1..v.len() - 1])
  } else {
    None
  }
}

fn parse_color_component(s: &str) -> u8 {
  let s = s.trim();
  if let Some(pct) = s.strip_suffix('%') {
    let pct_val: f32 = pct.parse().unwrap_or(0.0);
    (pct_val * 2.55).round().clamp(0.0, 255.0) as u8
  } else {
    s.parse::<f32>().unwrap_or(0.0).round().clamp(0.0, 255.0) as u8
  }
}

fn parse_alpha_component(s: &str) -> f32 {
  let s = s.trim();
  if let Some(pct) = s.strip_suffix('%') {
    pct.parse::<f32>().unwrap_or(100.0) / 100.0
  } else {
    s.parse::<f32>().unwrap_or(1.0)
  }
  .clamp(0.0, 1.0)
}

fn parse_hue_component(s: &str) -> f32 {
  let s = s.trim();
  if let Some(v) = s.strip_suffix("deg") {
    v.trim().parse::<f32>().unwrap_or(0.0)
  } else if let Some(v) = s.strip_suffix("rad") {
    v.trim().parse::<f32>().unwrap_or(0.0).to_degrees()
  } else if let Some(v) = s.strip_suffix("turn") {
    v.trim().parse::<f32>().unwrap_or(0.0) * 360.0
  } else {
    s.parse::<f32>().unwrap_or(0.0)
  }
}

fn split_color_function_args(inner: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let mut start: Option<usize> = None;
  for (i, ch) in inner.char_indices() {
    if ch == ',' || ch == '/' || ch.is_whitespace() {
      if let Some(s) = start.take() {
        out.push(inner[s..i].trim());
      }
    } else if start.is_none() {
      start = Some(i);
    }
  }
  if let Some(s) = start {
    out.push(inner[s..].trim());
  }
  out.into_iter().filter(|s| !s.is_empty()).collect()
}

fn is_preserved_color_function(v: &str) -> bool {
  [
    "color",
    "color-mix",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "light-dark",
  ]
  .iter()
  .any(|name| strip_func(v, name).is_some())
}

fn parse_display(value: &str) -> Option<Display> {
  value.parse().ok()
}

fn parse_position(value: &str) -> Option<Position> {
  value.parse().ok()
}

fn parse_background_clip(value: &str) -> Option<BackgroundClip> {
  value.parse().ok()
}

fn parse_background_repeat(value: &str) -> Option<BackgroundRepeat> {
  value.parse().ok()
}

fn parse_border_style(value: &str) -> Option<BorderStyle> {
  value.parse().ok()
}

fn parse_font_weight(value: &str) -> Option<FontWeight> {
  value.parse().ok()
}

fn parse_font_style(value: &str) -> Option<FontStyle> {
  value.parse().ok()
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
  value.parse().ok()
}

fn parse_text_transform(value: &str) -> Option<TextTransform> {
  value.parse().ok()
}

fn parse_white_space(value: &str) -> Option<WhiteSpace> {
  value.parse().ok()
}

fn parse_overflow(value: &str) -> Option<Overflow> {
  value.parse().ok()
}

fn parse_resize(value: &str) -> Option<Resize> {
  value.parse().ok()
}

fn parse_scrollbar_color(value: &str) -> Option<ScrollbarColor> {
  let v = value.trim();
  if v.eq_ignore_ascii_case("auto") {
    return Some(ScrollbarColor::Auto);
  }
  let mut parts = v.split_whitespace();
  let thumb = parts.next().and_then(parse_css_color)?;
  let track = parts.next().and_then(parse_css_color)?;
  if parts.next().is_some() {
    return None;
  }
  Some(ScrollbarColor::Custom { thumb, track })
}

fn parse_scrollbar_width(value: &str) -> Option<ScrollbarWidth> {
  value.parse().ok()
}

fn apply_overflow_shorthand(value: &str, style: &mut Style) {
  let mut parts = value.split_whitespace();
  let Some(first) = parts.next().and_then(parse_overflow) else {
    return;
  };
  let second = match parts.next() {
    Some(value) => match parse_overflow(value) {
      Some(parsed) => parsed,
      None => return,
    },
    None => first,
  };
  if parts.next().is_some() {
    return;
  }

  style.overflow = Some(first);
  style.overflow_x = Some(first);
  style.overflow_y = Some(second);
}

fn parse_visibility(value: &str) -> Option<Visibility> {
  value.parse().ok()
}

fn parse_flex_direction(value: &str) -> Option<FlexDirection> {
  value.parse().ok()
}

fn parse_flex_wrap(value: &str) -> Option<FlexWrap> {
  value.parse().ok()
}

fn parse_justify_content(value: &str) -> Option<JustifyContent> {
  value.parse().ok()
}

fn parse_align_items(value: &str) -> Option<AlignItems> {
  value.parse().ok()
}

fn parse_align_content(value: &str) -> Option<AlignContent> {
  value.parse().ok()
}

fn parse_align_self(value: &str) -> Option<AlignSelf> {
  value.parse().ok()
}

fn parse_justify_items(value: &str) -> Option<JustifyItems> {
  value.parse().ok()
}

fn parse_justify_self(value: &str) -> Option<JustifySelf> {
  value.parse().ok()
}

/// `grid-auto-flow: row | column | row dense | column dense | dense`.
/// `dense` packing isn't honoured at layout time yet; we accept the
/// keyword for cascade fidelity.
fn parse_grid_auto_flow(value: &str) -> Option<GridAutoFlow> {
  value.parse().ok()
}

/// Parse a single grid track size token: `auto`, `<flex>` (`1fr`), or
/// any `CssLength`. Returns `None` for unrecognized input.
fn parse_grid_track_size(value: &str) -> Option<GridTrackSize> {
  let trimmed = value.trim();
  if trimmed.eq_ignore_ascii_case("auto") {
    return Some(GridTrackSize::Auto);
  }
  if let Some(stripped) = strip_suffix_ci(trimmed, "fr") {
    if let Ok(n) = stripped.trim().parse::<f32>() {
      if n.is_finite() && n >= 0.0 {
        return Some(GridTrackSize::Fr(n));
      }
    }
  }
  parse_css_length(trimmed).map(GridTrackSize::Length)
}

/// Parse `grid-template-columns` / `grid-template-rows` as a flat list
/// of typed track sizes. Expands `repeat(<int>, <list>)` inline; leaves
/// `repeat(auto-fill, ...)` / `repeat(auto-fit, ...)` as a single `Auto`
/// track for now (still parses but doesn't auto-fit). Skips
/// `minmax()` / `fit-content()` (returns the inner length when
/// recognizable, otherwise `Auto`).
fn parse_grid_track_list(value: &str) -> Vec<GridTrackSize> {
  let trimmed = value.trim();
  if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
    return Vec::new();
  }
  let tokens = split_track_tokens(trimmed);
  let mut out: Vec<GridTrackSize> = Vec::new();
  for tok in tokens {
    if let Some(rest) = strip_function(&tok, "repeat") {
      // `repeat(<count>, <track-list>)`. Top-level comma split.
      let parts: Vec<&str> = split_top_level_commas(&rest);
      if parts.len() >= 2 {
        let count_tok = parts[0].trim();
        let inner = parts[1..].join(",");
        if let Ok(n) = count_tok.parse::<u32>() {
          let inner_list = parse_grid_track_list(&inner);
          for _ in 0..n {
            out.extend(inner_list.iter().cloned());
          }
          continue;
        }
        // `auto-fill` / `auto-fit` — single Auto placeholder
        // for now. Track-count resolution is a future job.
        if count_tok.eq_ignore_ascii_case("auto-fill") || count_tok.eq_ignore_ascii_case("auto-fit") {
          out.push(GridTrackSize::Auto);
          continue;
        }
      }
      continue;
    }
    if let Some(rest) = strip_function(&tok, "minmax") {
      // `minmax(<min>, <max>)` — for v1 we use the max as the
      // track size. Real two-bound clamping is deferred.
      let parts: Vec<&str> = split_top_level_commas(&rest);
      if let Some(max_tok) = parts.get(1) {
        if let Some(s) = parse_grid_track_size(max_tok.trim()) {
          out.push(s);
          continue;
        }
      }
      out.push(GridTrackSize::Auto);
      continue;
    }
    if let Some(rest) = strip_function(&tok, "fit-content") {
      // `fit-content(<length>)` — degrade to the inner length
      // for v1; the ceiling-by-content semantics are deferred.
      if let Some(s) = parse_grid_track_size(rest.trim()) {
        out.push(s);
        continue;
      }
      out.push(GridTrackSize::Auto);
      continue;
    }
    // Plain tokens.
    if let Some(size) = parse_grid_track_size(&tok) {
      out.push(size);
    }
  }
  out
}

/// Tokenize a track list into whitespace-separated entries while
/// keeping `repeat(...)`, `minmax(...)`, and `fit-content(...)` calls
/// intact.
fn split_track_tokens(s: &str) -> Vec<String> {
  let mut out = Vec::new();
  let mut buf = String::new();
  let mut depth: i32 = 0;
  for ch in s.chars() {
    match ch {
      '(' => {
        depth += 1;
        buf.push(ch);
      }
      ')' => {
        depth -= 1;
        buf.push(ch);
      }
      c if c.is_whitespace() && depth == 0 => {
        if !buf.is_empty() {
          out.push(std::mem::take(&mut buf));
        }
      }
      c => buf.push(c),
    }
  }
  if !buf.is_empty() {
    out.push(buf);
  }
  out
}

/// Split a string on commas at parenthesis depth 0. Used inside
/// `repeat(…)` / `minmax(…)` argument lists.
fn split_top_level_commas(s: &str) -> Vec<&str> {
  let mut out = Vec::new();
  let bytes = s.as_bytes();
  let mut depth: i32 = 0;
  let mut start = 0;
  for (i, &b) in bytes.iter().enumerate() {
    match b {
      b'(' => depth += 1,
      b')' => depth -= 1,
      b',' if depth == 0 => {
        out.push(&s[start..i]);
        start = i + 1;
      }
      _ => {}
    }
  }
  out.push(&s[start..]);
  out
}

/// If `s` looks like `<name>(<inside>)`, return `<inside>` (trimmed).
/// Case-insensitive on the function name. Returns `None` otherwise.
fn strip_function(s: &str, name: &str) -> Option<String> {
  let trimmed = s.trim();
  let lower = trimmed.to_ascii_lowercase();
  let prefix = format!("{name}(");
  if !lower.starts_with(&prefix) || !trimmed.ends_with(')') {
    return None;
  }
  let inside = &trimmed[prefix.len()..trimmed.len() - 1];
  Some(inside.to_string())
}

/// Strip a case-insensitive suffix; returns the prefix when matched.
fn strip_suffix_ci<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
  if s.len() < suffix.len() {
    return None;
  }
  let cut = s.len() - suffix.len();
  if s[cut..].eq_ignore_ascii_case(suffix) {
    Some(&s[..cut])
  } else {
    None
  }
}

/// Parse one end of a `grid-row` / `grid-column` placement.
/// Recognized: `auto`, a positive integer line number, `span <n>`.
/// Negative line numbers and named lines fall through to `None`.
fn parse_grid_line(value: &str) -> Option<GridLine> {
  value.parse().ok()
}

#[derive(Copy, Clone)]
enum GridAxis {
  Column,
  Row,
}

/// Expand `grid-column` / `grid-row` shorthand into the start / end
/// longhands. Accepts:
/// - `<line>` → start=line, end=auto
/// - `<line> / <line>` → start, end
/// - `span <n> / <line>` (and the reverse), etc.
fn apply_grid_axis_shorthand(value: &str, style: &mut Style, axis: GridAxis) {
  // Round-trip the raw value for cascade introspection.
  match axis {
    GridAxis::Column => style.grid_column = Some(ArcStr::from(value)),
    GridAxis::Row => style.grid_row = Some(ArcStr::from(value)),
  }
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return;
  }
  let parts: Vec<&str> = trimmed.split('/').map(|p| p.trim()).collect();
  let (start_tok, end_tok) = match parts.len() {
    1 => (parts[0], "auto"),
    _ => (parts[0], parts[1]),
  };
  let start = parse_grid_line(start_tok).unwrap_or(GridLine::Auto);
  let end = parse_grid_line(end_tok).unwrap_or(GridLine::Auto);
  match axis {
    GridAxis::Column => {
      style.grid_column_start = Some(start);
      style.grid_column_end = Some(end);
    }
    GridAxis::Row => {
      style.grid_row_start = Some(start);
      style.grid_row_end = Some(end);
    }
  }
}

/// Expand the `flex` shorthand into the three longhands per CSS-Flex-1
/// §7.2 (`flex` shorthand).
///
/// Recognized forms:
/// - `none`    → 0 0 auto
/// - `auto`    → 1 1 auto
/// - `initial` → 0 1 auto
/// - `<number>`            → grow=<n>, shrink=1, basis=0%
/// - `<basis>`             → grow=1, shrink=1, basis=<basis>
/// - `<grow> <shrink>`     → grow, shrink, basis=0%
/// - `<grow> <basis>`      → grow, shrink=1, basis
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
      // Three (or more — extra ignored) tokens: grow shrink basis.
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

fn parse_cursor(value: &str) -> Option<Cursor> {
  value.parse().ok()
}

fn parse_pointer_events(value: &str) -> Option<PointerEvents> {
  value.parse().ok()
}

fn parse_user_select(value: &str) -> Option<UserSelect> {
  value.parse().ok()
}

fn parse_box_sizing(value: &str) -> Option<BoxSizing> {
  value.parse().ok()
}
