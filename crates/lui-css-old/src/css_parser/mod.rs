use std::collections::HashMap;

use crate::{
  shorthands::{all_shorthands, shorthand_contains_member, shorthand_members},
  style::Style,
  style_props::{clear_value_for, merge_values_clearing_keywords},
  values::ArcStr,
};

// -- Submodules -----------------------------------------------------------

mod border;
mod lui_resolve;
mod shorthand;
pub mod values;
pub mod var;

// -- Re-exports (crate-internal) ------------------------------------------
// Make all pub(crate) items from submodules available in this module's
// namespace so that `apply_css_property` and the inline-style parsers
// can call them without qualifying the submodule name.

// -- Public re-exports ----------------------------------------------------
// These items are accessed by `lib.rs` and external consumers at the
// `css_parser` module level. They must stay `pub`.
pub use border::parse_box_shorthand;
pub(crate) use border::*;
pub use lui_resolve::{resolve_lui_calendar_style, resolve_lui_color_picker_style, resolve_lui_popup_style};
pub(crate) use shorthand::*;
pub(crate) use values::*;
pub use values::{parse_css_color, parse_css_image, parse_css_length, parse_css_url};
pub(crate) use var::value_contains_var;

// -- CSS-wide keyword -----------------------------------------------------
pub use crate::declaration::CssWideKeyword;

// -- Style declarations ---------------------------------------------------

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

// -- Inline style parsing -------------------------------------------------

/// Parse an inline CSS style string (e.g. `"display: flex; color: red;"`)
/// into a `Style` struct. `!important` is recognised and stripped from
/// values so they parse correctly, but its effect is folded back in:
/// when a property is given as `!important`, it overrides the same
/// property declared as normal in the *same* string. CSS-wide
/// keywords (`inherit / initial / unset`) are recognised but dropped
/// -- this back-compat surface returns `Style` only, so a keyword
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
      // declaration block -- within a layer, last-write-wins,
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

// -- Property application -------------------------------------------------

pub fn apply_css_property(style: &mut Style, property: &str, value: &str) {
  if property.starts_with("--") {
    style
      .custom_properties
      .insert(ArcStr::from(property), ArcStr::from(value));
    return;
  }
  if value_contains_var(value) {
    style.var_properties.insert(ArcStr::from(property), ArcStr::from(value));
    return;
  }
  let mut block = crate::declaration::DeclarationBlock::new();
  block.push(
    ArcStr::from(property),
    ArcStr::from(value),
    crate::declaration::Importance::Normal,
  );
  crate::properties::groups::apply_declarations(&block, style);
}
