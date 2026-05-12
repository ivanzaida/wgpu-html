use rustc_hash::{FxHashMap, FxHashSet};

use bumpalo::Bump;
use lui_css_parser::{ArcStr, CssValue, CssProperty};

use crate::style::ComputedStyle;

/// Resolve all `var()` references in a computed style.
///
/// Phase 1: resolve custom property chains (`--a: var(--b)` where `--b: 10px`).
/// Phase 2: substitute `CssValue::Var` nodes in regular property values.
///
/// Synthesized values are allocated in `arena`.
pub fn resolve_vars<'a>(style: &mut ComputedStyle<'a>, arena: &'a Bump) {
    let resolved_cp = resolve_custom_property_chains(style, arena);

    resolve_property_vars(style, &resolved_cp, arena);

    if !resolved_cp.is_empty() {
        let cp = style.custom_properties.get_or_insert_with(Default::default);
        for (name, value) in resolved_cp {
            cp.insert(name, value);
        }
    }
}

/// Phase 1: walk custom properties and resolve any var() references
/// within their values, collapsing chains like `--a: var(--b)`.
fn resolve_custom_property_chains<'a>(
    style: &ComputedStyle<'a>,
    arena: &'a Bump,
) -> FxHashMap<ArcStr, &'a CssValue> {
    let cp = match &style.custom_properties {
        Some(cp) => cp,
        None => return FxHashMap::default(),
    };

    let mut resolved: FxHashMap<ArcStr, &'a CssValue> = FxHashMap::default();

    for (name, value) in cp.iter() {
        if !contains_var(value) {
            resolved.insert(name.clone(), *value);
        }
    }

    for (name, value) in cp.iter() {
        if contains_var(value) {
            let mut resolving = FxHashSet::default();
            resolving.insert(name.clone());
            let substituted = substitute(value, &resolved, cp, &mut resolving, arena);
            resolved.insert(name.clone(), substituted);
        }
    }

    resolved
}

/// Phase 2: walk every property value in the style and substitute
/// any `Var` nodes with their resolved custom property values.
fn resolve_property_vars<'a>(
    style: &mut ComputedStyle<'a>,
    resolved_cp: &FxHashMap<ArcStr, &'a CssValue>,
    arena: &'a Bump,
) {
    macro_rules! resolve_field {
        ($($field:ident),* $(,)?) => {
            $(
                if let Some(val) = style.$field {
                    if contains_var(val) {
                        let empty = FxHashMap::default();
                        let mut resolving = FxHashSet::default();
                        style.$field = Some(substitute(val, resolved_cp, &empty, &mut resolving, arena));
                    }
                }
            )*
        };
    }

    resolve_field! {
        display, position, top, right, bottom, left, float, clear,
        width, height, min_width, min_height, max_width, max_height, box_sizing, aspect_ratio,
        margin_top, margin_right, margin_bottom, margin_left,
        padding_top, padding_right, padding_bottom, padding_left,
        border_top_width, border_right_width, border_bottom_width, border_left_width,
        border_top_style, border_right_style, border_bottom_style, border_left_style,
        border_top_color, border_right_color, border_bottom_color, border_left_color,
        border_top_left_radius, border_top_right_radius, border_bottom_right_radius, border_bottom_left_radius,
        background_color, background_image, background_size, background_position, background_repeat, background_clip,
        color, opacity, visibility,
        font_family, font_size, font_weight, font_style, line_height,
        letter_spacing, word_spacing, text_align,
        text_decoration_line, text_decoration_color, text_decoration_style,
        text_transform, white_space, word_break, text_overflow, vertical_align,
        flex_direction, flex_wrap, justify_content, align_items, align_content, align_self,
        flex_grow, flex_shrink, flex_basis, order, row_gap, column_gap,
        grid_template_columns, grid_template_rows, grid_auto_columns, grid_auto_rows, grid_auto_flow,
        grid_column_start, grid_column_end, grid_row_start, grid_row_end,
        justify_items, justify_self,
        overflow_x, overflow_y, scrollbar_color, scrollbar_width,
        transform, transform_origin, box_shadow, z_index,
        cursor, pointer_events, user_select, resize, accent_color,
        list_style_type, list_style_position, list_style_image,
        content,
        fill, fill_opacity, fill_rule, stroke, stroke_width, stroke_opacity,
        stroke_linecap, stroke_linejoin, stroke_dasharray, stroke_dashoffset,
    }

    if let Some(extra) = &mut style.extra {
        let needs_resolve: Vec<CssProperty> = extra.iter()
            .filter(|(_, v)| contains_var(v))
            .map(|(k, _)| k.clone())
            .collect();

        for prop in needs_resolve {
            if let Some(val) = extra.get(&prop) {
                let empty = FxHashMap::default();
                let mut resolving = FxHashSet::default();
                let resolved = substitute(val, resolved_cp, &empty, &mut resolving, arena);
                extra.insert(prop, resolved);
            }
        }
    }
}

fn contains_var(value: &CssValue) -> bool {
    match value {
        CssValue::Var { .. } => true,
        CssValue::Function { args, .. } => args.iter().any(contains_var),
        _ => false,
    }
}

fn substitute<'a>(
    value: &CssValue,
    resolved: &FxHashMap<ArcStr, &'a CssValue>,
    raw_cp: &FxHashMap<ArcStr, &CssValue>,
    resolving: &mut FxHashSet<ArcStr>,
    arena: &'a Bump,
) -> &'a CssValue {
    match value {
        CssValue::Var { name, fallback } => {
            if resolving.contains(name) {
                return match fallback {
                    Some(fb) => substitute(fb, resolved, raw_cp, resolving, arena),
                    None => arena.alloc(CssValue::Unknown("".into())),
                };
            }

            if let Some(resolved_val) = resolved.get(name) {
                if contains_var(resolved_val) {
                    resolving.insert(name.clone());
                    let result = substitute(resolved_val, resolved, raw_cp, resolving, arena);
                    resolving.remove(name);
                    result
                } else {
                    *resolved_val
                }
            } else if let Some(raw_val) = raw_cp.get(name.as_ref()) {
                resolving.insert(name.clone());
                let result = substitute(raw_val, resolved, raw_cp, resolving, arena);
                resolving.remove(name);
                result
            } else {
                match fallback {
                    Some(fb) => substitute(fb, resolved, raw_cp, resolving, arena),
                    None => arena.alloc(CssValue::Unknown("".into())),
                }
            }
        }
        CssValue::Function { function, args } => {
            if !args.iter().any(contains_var) {
                return arena.alloc(value.clone());
            }
            let new_args: Vec<CssValue> = args.iter()
                .map(|a| substitute(a, resolved, raw_cp, resolving, arena).clone())
                .collect();
            arena.alloc(CssValue::Function { function: function.clone(), args: new_args })
        }
        _ => arena.alloc(value.clone()),
    }
}
