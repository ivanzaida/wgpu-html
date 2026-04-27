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

use crate::css_parser::CssWideKeyword;

macro_rules! style_props {
    (
        $(
            $field:ident => $name:literal $(, $is_inh:ident)?
        );* $(;)?
    ) => {
        /// Wipe the field for one named property.
        pub fn clear_value_for(prop: &str, values: &mut Style) {
            match prop {
                $(
                    $name => values.$field = None,
                )*
                _ => {}
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
            $(
                if src.$field.is_some() {
                    dst.$field = src.$field.clone();
                    keywords.remove($name);
                }
            )*
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
            match prop {
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
                _ => {}
            }
        }

        /// True if a property is in the standard inherited set.
        pub fn is_inherited(prop: &str) -> bool {
            match prop {
                $(
                    $name => style_props!(@is_inh $($is_inh)?),
                )*
                _ => false,
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
    background => "background";
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
    opacity => "opacity";
    z_index => "z-index";

    // Flex / grid / gap -----------------------------------------------
    flex_direction => "flex-direction";
    flex_wrap => "flex-wrap";
    justify_content => "justify-content";
    align_items => "align-items";
    align_content => "align-content";
    gap => "gap";
    row_gap => "row-gap";
    column_gap => "column-gap";
    flex => "flex";
    flex_grow => "flex-grow";
    flex_shrink => "flex-shrink";
    flex_basis => "flex-basis";
    grid_template_columns => "grid-template-columns";
    grid_template_rows => "grid-template-rows";
    grid_column => "grid-column";
    grid_row => "grid-row";

    // Effects / interaction -------------------------------------------
    transform => "transform";
    transform_origin => "transform-origin";
    transition => "transition";
    animation => "animation";
    pointer_events => "pointer-events";
    user_select => "user-select";
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu_html_models::common::css_enums::{CssColor, CssLength};

    #[test]
    fn inherited_block_is_marked() {
        assert!(is_inherited("color"));
        assert!(is_inherited("font-family"));
        assert!(is_inherited("line-height"));
        assert!(is_inherited("visibility"));
        assert!(is_inherited("cursor"));
        // Non-inherited reference checks
        assert!(!is_inherited("background-color"));
        assert!(!is_inherited("margin"));
        assert!(!is_inherited("display"));
        assert!(!is_inherited("z-index"));
    }

    #[test]
    fn clear_value_for_unsets_named_field() {
        let mut s = Style::default();
        s.color = Some(CssColor::Named("red".into()));
        s.width = Some(CssLength::Px(10.0));
        clear_value_for("color", &mut s);
        assert!(s.color.is_none());
        assert!(s.width.is_some());
    }

    #[test]
    fn apply_inherit_uses_parent_value() {
        let mut child = Style::default();
        let mut parent = Style::default();
        parent.color = Some(CssColor::Named("white".into()));
        apply_keyword(&mut child, Some(&parent), "color", CssWideKeyword::Inherit);
        assert!(matches!(
            child.color.as_ref().unwrap(),
            CssColor::Named(s) if s == "white"
        ));
    }

    #[test]
    fn apply_initial_clears_field() {
        let mut child = Style::default();
        child.color = Some(CssColor::Named("red".into()));
        let parent = Style::default();
        apply_keyword(&mut child, Some(&parent), "color", CssWideKeyword::Initial);
        assert!(child.color.is_none());
    }

    #[test]
    fn apply_unset_inherits_for_inherited_props() {
        let mut child = Style::default();
        child.color = Some(CssColor::Named("red".into()));
        let mut parent = Style::default();
        parent.color = Some(CssColor::Named("blue".into()));
        apply_keyword(&mut child, Some(&parent), "color", CssWideKeyword::Unset);
        assert!(matches!(
            child.color.as_ref().unwrap(),
            CssColor::Named(s) if s == "blue"
        ));
    }

    #[test]
    fn apply_unset_clears_for_non_inherited_props() {
        let mut child = Style::default();
        child.background_color = Some(CssColor::Named("red".into()));
        let mut parent = Style::default();
        parent.background_color = Some(CssColor::Named("blue".into()));
        apply_keyword(
            &mut child,
            Some(&parent),
            "background-color",
            CssWideKeyword::Unset,
        );
        assert!(child.background_color.is_none());
    }

    #[test]
    fn merge_values_clears_keywords_for_touched_fields() {
        let mut dst = Style::default();
        let mut kw: HashMap<String, CssWideKeyword> = HashMap::new();
        kw.insert("color".into(), CssWideKeyword::Inherit);
        kw.insert("width".into(), CssWideKeyword::Initial);
        let mut src = Style::default();
        src.color = Some(CssColor::Named("red".into()));
        merge_values_clearing_keywords(&mut dst, &mut kw, &src);
        assert!(dst.color.is_some());
        assert!(!kw.contains_key("color"));
        assert!(kw.contains_key("width"));
    }

    #[test]
    fn root_inherit_with_no_parent_clears() {
        let mut child = Style::default();
        child.color = Some(CssColor::Named("red".into()));
        apply_keyword(&mut child, None, "color", CssWideKeyword::Inherit);
        assert!(child.color.is_none());
    }
}
