//! Single source of truth for the parser ↔ model property mapping.
//!
//! Every supported CSS property gets one row: `(field, name, inherited)`.
//! From that table we generate four helpers used by both the parser
//! (within-layer mutual exclusion of values vs CSS-wide keywords) and
//! the cascade (per-property keyword resolution against the parent's
//! resolved style):
//!
//! - `clear_value_for(prop, &mut Style)`
//! - `merge_values_clearing_keywords(&mut Style, &mut HashMap, &Style)`
//! - `apply_keyword(&mut Style, parent, prop, keyword)`
//! - `is_inherited(prop)`
//!
//! Keep the table aligned with `apply_css_property` in `css_parser.rs`
//! — same property names, same field names. The compiler enforces the
//! field-name half (each row references `Style::$field`); the
//! kebab-case CSS names are checked at runtime by the parser's match
//! and by tests.

use std::collections::HashMap;

use wgpu_html_models::Style;

use crate::{
  css_parser::CssWideKeyword,
  shorthands::{
    all_deferred_longhands, all_shorthands, is_deferred_longhand, is_inherited_deferred_longhand,
    shorthand_contains_member, shorthand_members,
  },
};

fn clear_background_values(values: &mut Style) {
  values.background = None;
  values.background_color = None;
  values.background_image = None;
  values.background_size = None;
  values.background_position = None;
  values.background_repeat = None;
  values.background_clip = None;
}

fn clear_background_keywords(keywords: &mut HashMap<String, CssWideKeyword>) {
  keywords.remove("background");
  keywords.remove("background-color");
  keywords.remove("background-image");
  keywords.remove("background-size");
  keywords.remove("background-position");
  keywords.remove("background-repeat");
  keywords.remove("background-clip");
}

fn clear_all_values(values: &mut Style) {
  *values = Style::default();
}

fn clear_dynamic_value_for(prop: &str, values: &mut Style) {
  if prop == "all" {
    clear_all_values(values);
    return;
  }
  if let Some(members) = shorthand_members(prop) {
    for member in members {
      if *member == prop {
        continue;
      }
      clear_value_for(member, values);
    }
    return;
  }
  if is_deferred_longhand(prop) {
    values.deferred_longhands.remove(prop);
    values.reset_properties.remove(prop);
  }
}

fn apply_deferred_keyword(values: &mut Style, parent: Option<&Style>, prop: &str, kw: CssWideKeyword) {
  match kw {
    CssWideKeyword::Inherit => {
      if let Some(value) = parent.and_then(|p| p.deferred_longhands.get(prop)) {
        values.deferred_longhands.insert(prop.to_string(), value.clone());
      } else {
        values.deferred_longhands.remove(prop);
      }
    }
    CssWideKeyword::Initial => {
      values.deferred_longhands.remove(prop);
    }
    CssWideKeyword::Unset => {
      if is_inherited_deferred_longhand(prop) {
        if let Some(value) = parent.and_then(|p| p.deferred_longhands.get(prop)) {
          values.deferred_longhands.insert(prop.to_string(), value.clone());
        } else {
          values.deferred_longhands.remove(prop);
        }
      } else {
        values.deferred_longhands.remove(prop);
      }
    }
  }
}

fn apply_all_keyword(values: &mut Style, parent: Option<&Style>, kw: CssWideKeyword) {
  let mut next = Style::default();
  match kw {
    CssWideKeyword::Inherit => {
      if let Some(parent) = parent {
        next = parent.clone();
      }
    }
    CssWideKeyword::Initial => {}
    CssWideKeyword::Unset => {
      if let Some(parent) = parent {
        for prop in all_deferred_longhands() {
          if is_inherited_deferred_longhand(prop) {
            if let Some(value) = parent.deferred_longhands.get(*prop) {
              next.deferred_longhands.insert((*prop).to_string(), value.clone());
            }
          }
        }
      }
    }
  }
  next.reset_properties.clear();
  next.keyword_reset_properties.clear();
  *values = next;
}

fn clear_keywords_covered_by_value(prop: &str, keywords: &mut HashMap<String, CssWideKeyword>) {
  keywords.remove(prop);
  for shorthand in all_shorthands() {
    if shorthand_contains_member(shorthand, prop) {
      keywords.remove(*shorthand);
    }
  }
}

macro_rules! style_props {
    (
        $(
            $field:ident => $name:literal $(, $is_inh:ident)?
        );* $(;)?
    ) => {
        /// Wipe the field for one named property. Also clears any
        /// pending `var_properties` entry for the same name.
        pub fn clear_value_for(prop: &str, values: &mut Style) {
            if prop == "all" {
                clear_all_values(values);
                values.var_properties.clear();
                return;
            }
            if let Some(members) = shorthand_members(prop) {
                for member in members {
                    if *member != prop {
                        clear_value_for(member, values);
                    }
                }
            }
            clear_direct_value_for(prop, values);
            values.var_properties.remove(prop);
        }

        fn clear_direct_value_for(prop: &str, values: &mut Style) {
            match prop {
                "background" => clear_background_values(values),
                $(
                    $name => values.$field = None,
                )*
                _ => clear_dynamic_value_for(prop, values),
            }
        }

        /// Per-field copy of `Some` values from `src` into `dst`. For
        /// every property the source actually set (`Some`), drop any
        /// previously-recorded keyword override on the same property
        /// — within a layer, a value displaces an earlier keyword for
        /// the same property.
        pub fn merge_values_clearing_keywords(
            dst: &mut Style,
            keywords: &mut HashMap<String, CssWideKeyword>,
            src: &Style,
        ) {
            for prop in &src.reset_properties {
                clear_value_for(prop, dst);
                clear_keywords_covered_by_value(prop, keywords);
            }
            // Merge var_properties from src BEFORE typed fields so that
            // a typed field in src can override a var_property.
            for (prop, val) in &src.var_properties {
                clear_value_for(prop, dst);
                clear_keywords_covered_by_value(prop, keywords);
                dst.var_properties.insert(prop.clone(), val.clone());
            }
            // Merge custom_properties (simple override).
            for (prop, val) in &src.custom_properties {
                dst.custom_properties.insert(prop.clone(), val.clone());
            }
            if src.background.is_some() {
                clear_background_values(dst);
                clear_background_keywords(keywords);
                dst.background = src.background.clone();
                keywords.remove("background");
                dst.var_properties.remove("background");
            }
            $(
                if src.$field.is_some() {
                    dst.$field = src.$field.clone();
                    clear_keywords_covered_by_value($name, keywords);
                    dst.var_properties.remove($name);
                }
            )*
            for (prop, value) in &src.deferred_longhands {
                dst.deferred_longhands.insert(prop.clone(), value.clone());
                clear_keywords_covered_by_value(prop, keywords);
            }
        }

        /// Resolve one CSS-wide keyword against the parent's resolved
        /// style. With no parent (root element) `Inherit` and the
        /// inherited-flavoured `Unset` fall through to `None`, the
        /// "no UA default" stand-in for `Initial`.
        pub fn apply_keyword(
            values: &mut Style,
            parent: Option<&Style>,
            prop: &str,
            kw: CssWideKeyword,
        ) {
            if prop == "all" {
                apply_all_keyword(values, parent, kw);
                return;
            }
            if let Some(members) = shorthand_members(prop) {
                for member in members {
                    if *member != prop {
                        apply_keyword(values, parent, member, kw);
                    }
                }
            }
            apply_direct_keyword(values, parent, prop, kw);
        }

        fn apply_direct_keyword(
            values: &mut Style,
            parent: Option<&Style>,
            prop: &str,
            kw: CssWideKeyword,
        ) {
            match prop {
                "background" => {
                    match kw {
                        CssWideKeyword::Inherit => {
                            values.background = parent.and_then(|p| p.background.clone());
                        }
                        CssWideKeyword::Initial | CssWideKeyword::Unset => {
                            values.background = None;
                        }
                    }
                }
                $(
                    $name => {
                        match kw {
                            CssWideKeyword::Inherit => {
                                values.$field = parent.and_then(|p| p.$field.clone());
                            }
                            CssWideKeyword::Initial => {
                                values.$field = None;
                            }
                            CssWideKeyword::Unset => {
                                if is_inherited($name) {
                                    values.$field = parent.and_then(|p| p.$field.clone());
                                } else {
                                    values.$field = None;
                                }
                            }
                        }
                    }
                )*
                _ => {
                    if is_deferred_longhand(prop) {
                        apply_deferred_keyword(values, parent, prop, kw);
                    }
                }
            }
        }

        /// True if a property is in the standard inherited set.
        /// Custom properties (`--*`) are always inherited per CSS spec.
        pub fn is_inherited(prop: &str) -> bool {
            if prop.starts_with("--") {
                return true;
            }
            match prop {
                $(
                    $name => style_props!(@is_inh $($is_inh)?),
                )*
                _ => is_inherited_deferred_longhand(prop),
            }
        }
    };

    // Helper arms: the `$(, $is_inh:ident)?` matcher above either
    // captured one ident (e.g. `inherited`) or none. Re-emit the
    // capture into one of these arms.
    (@is_inh) => { false };
    (@is_inh inherited) => { true };
}

style_props! {
    // Layout / box ----------------------------------------------------
    display => "display";
    position => "position";
    top => "top";
    right => "right";
    bottom => "bottom";
    left => "left";
    width => "width";
    height => "height";
    min_width => "min-width";
    min_height => "min-height";
    max_width => "max-width";
    max_height => "max-height";
    margin => "margin";
    margin_top => "margin-top";
    margin_right => "margin-right";
    margin_bottom => "margin-bottom";
    margin_left => "margin-left";
    padding => "padding";
    padding_top => "padding-top";
    padding_right => "padding-right";
    padding_bottom => "padding-bottom";
    padding_left => "padding-left";
    box_sizing => "box-sizing";

    // Background / borders -------------------------------------------
    background_color => "background-color";
    background_image => "background-image";
    background_size => "background-size";
    background_position => "background-position";
    background_repeat => "background-repeat";
    background_clip => "background-clip";
    border => "border";
    border_top_width => "border-top-width";
    border_right_width => "border-right-width";
    border_bottom_width => "border-bottom-width";
    border_left_width => "border-left-width";
    border_top_style => "border-top-style";
    border_right_style => "border-right-style";
    border_bottom_style => "border-bottom-style";
    border_left_style => "border-left-style";
    border_top_color => "border-top-color";
    border_right_color => "border-right-color";
    border_bottom_color => "border-bottom-color";
    border_left_color => "border-left-color";
    border_top_left_radius => "border-top-left-radius";
    border_top_right_radius => "border-top-right-radius";
    border_bottom_right_radius => "border-bottom-right-radius";
    border_bottom_left_radius => "border-bottom-left-radius";
    border_top_left_radius_v => "border-top-left-radius-v";
    border_top_right_radius_v => "border-top-right-radius-v";
    border_bottom_right_radius_v => "border-bottom-right-radius-v";
    border_bottom_left_radius_v => "border-bottom-left-radius-v";
    box_shadow => "box-shadow";

    // Typography (inherited block) -----------------------------------
    color => "color" ,inherited;
    font_family => "font-family" ,inherited;
    font_size => "font-size" ,inherited;
    font_weight => "font-weight" ,inherited;
    font_style => "font-style" ,inherited;
    line_height => "line-height" ,inherited;
    letter_spacing => "letter-spacing" ,inherited;
    text_align => "text-align" ,inherited;
    text_decoration => "text-decoration" ,inherited;
    text_transform => "text-transform" ,inherited;
    white_space => "white-space" ,inherited;

    // Visibility / cursor (also inherit) -----------------------------
    visibility => "visibility" ,inherited;
    cursor => "cursor" ,inherited;

    // Misc layout / overflow / opacity --------------------------------
    overflow => "overflow";
    overflow_x => "overflow-x";
    overflow_y => "overflow-y";
    resize => "resize";
    opacity => "opacity";
    z_index => "z-index";

    // SVG presentation properties (inherited within SVG subtrees) -----
    svg_fill => "fill" ,inherited;
    svg_fill_opacity => "fill-opacity" ,inherited;
    svg_fill_rule => "fill-rule" ,inherited;
    svg_stroke => "stroke" ,inherited;
    svg_stroke_width => "stroke-width" ,inherited;
    svg_stroke_opacity => "stroke-opacity" ,inherited;
    svg_stroke_linecap => "stroke-linecap" ,inherited;
    svg_stroke_linejoin => "stroke-linejoin" ,inherited;
    svg_stroke_dasharray => "stroke-dasharray" ,inherited;
    svg_stroke_dashoffset => "stroke-dashoffset" ,inherited;

    // Flex / grid / gap -----------------------------------------------
    flex_direction => "flex-direction";
    flex_wrap => "flex-wrap";
    justify_content => "justify-content";
    align_items => "align-items";
    align_content => "align-content";
    align_self => "align-self";
    order => "order";
    gap => "gap";
    row_gap => "row-gap";
    column_gap => "column-gap";
    flex => "flex";
    flex_grow => "flex-grow";
    flex_shrink => "flex-shrink";
    flex_basis => "flex-basis";
    grid_template_columns => "grid-template-columns";
    grid_template_rows => "grid-template-rows";
    grid_auto_columns => "grid-auto-columns";
    grid_auto_rows => "grid-auto-rows";
    grid_auto_flow => "grid-auto-flow";
    grid_column => "grid-column";
    grid_column_start => "grid-column-start";
    grid_column_end => "grid-column-end";
    grid_row => "grid-row";
    grid_row_start => "grid-row-start";
    grid_row_end => "grid-row-end";
    justify_items => "justify-items";
    justify_self => "justify-self";

    // Effects / interaction -------------------------------------------
    transform => "transform";
    transform_origin => "transform-origin";
    transition => "transition";
    animation => "animation";
    pointer_events => "pointer-events" ,inherited;
    user_select => "user-select" ,inherited;
}

