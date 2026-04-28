use std::collections::HashMap;

use wgpu_html_models::common::css_enums::*;
use wgpu_html_models::Style;

use crate::style_props::clear_value_for;

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
    pub keywords_normal: HashMap<String, CssWideKeyword>,
    pub keywords_important: HashMap<String, CssWideKeyword>,
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
            let property = property.trim().to_ascii_lowercase();
            let (value, important) = strip_important(value.trim());

            // CSS-wide keywords pre-empt the value parsers. They go
            // into the side-car keyword map and clear any matching
            // value the same bucket may have set earlier in this same
            // declaration block — within a layer, last-write-wins,
            // so a later keyword has to displace any earlier value.
            if let Some(kw) = CssWideKeyword::from_value(value) {
                if important {
                    clear_value_for(&property, &mut decls.important);
                    decls.keywords_important.insert(property, kw);
                } else {
                    clear_value_for(&property, &mut decls.normal);
                    decls.keywords_normal.insert(property, kw);
                }
                continue;
            }

            // Conversely: a value declaration drops any keyword
            // override the same bucket may have recorded earlier in
            // the block.
            if important {
                decls.keywords_important.remove(&property);
                apply_css_property(&mut decls.important, &property, value);
            } else {
                decls.keywords_normal.remove(&property);
                apply_css_property(&mut decls.normal, &property, value);
            }
        }
    }
    decls
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
        opacity,
        visibility,
        z_index,
        flex_direction,
        flex_wrap,
        justify_content,
        align_items,
        align_content,
        gap,
        row_gap,
        column_gap,
        flex,
        flex_grow,
        flex_shrink,
        flex_basis,
        grid_template_columns,
        grid_template_rows,
        grid_column,
        grid_row,
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
}

fn apply_css_property(style: &mut Style, property: &str, value: &str) {
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
        "background" => style.background = Some(value.to_string()),
        "background-color" => style.background_color = parse_css_color(value),
        "background-image" => style.background_image = parse_css_image(value),
        "background-size" => style.background_size = Some(value.to_string()),
        "background-position" => style.background_position = Some(value.to_string()),
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
        "font-family" => style.font_family = Some(value.to_string()),
        "font-size" => style.font_size = parse_css_length(value),
        "font-weight" => style.font_weight = parse_font_weight(value),
        "font-style" => style.font_style = parse_font_style(value),
        "line-height" => style.line_height = parse_css_length(value),
        "letter-spacing" => style.letter_spacing = parse_css_length(value),
        "text-align" => style.text_align = parse_text_align(value),
        "text-decoration" => style.text_decoration = Some(value.to_string()),
        "text-transform" => style.text_transform = parse_text_transform(value),
        "white-space" => style.white_space = parse_white_space(value),
        "overflow" => style.overflow = parse_overflow(value),
        "overflow-x" => style.overflow_x = parse_overflow(value),
        "overflow-y" => style.overflow_y = parse_overflow(value),
        "opacity" => style.opacity = value.parse().ok(),
        "visibility" => style.visibility = parse_visibility(value),
        "z-index" => style.z_index = value.parse().ok(),
        "flex-direction" => style.flex_direction = parse_flex_direction(value),
        "flex-wrap" => style.flex_wrap = parse_flex_wrap(value),
        "justify-content" => style.justify_content = parse_justify_content(value),
        "align-items" => style.align_items = parse_align_items(value),
        "align-content" => style.align_content = parse_align_content(value),
        "align-self" => style.align_self = parse_align_self(value),
        "order" => style.order = value.trim().parse().ok(),
        "gap" => style.gap = parse_css_length(value),
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
        "transform" => style.transform = Some(value.to_string()),
        "transform-origin" => style.transform_origin = Some(value.to_string()),
        "transition" => style.transition = Some(value.to_string()),
        "animation" => style.animation = Some(value.to_string()),
        "cursor" => style.cursor = parse_cursor(value),
        "pointer-events" => style.pointer_events = parse_pointer_events(value),
        "user-select" => style.user_select = parse_user_select(value),
        "box-shadow" => style.box_shadow = Some(value.to_string()),
        "box-sizing" => style.box_sizing = parse_box_sizing(value),
        _ => {} // Unknown CSS properties are silently ignored
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
    style.border = Some(value.to_string());
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
fn apply_corner_radius(
    value: &str,
    h_field: &mut Option<CssLength>,
    v_field: &mut Option<CssLength>,
) {
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
        return Some(CssLength::Raw(v.to_string()));
    }
    if let Some(inner) = strip_func(v, "min") {
        let args: Vec<CssLength> = split_top_level_commas(inner)
            .into_iter()
            .filter_map(parse_css_length)
            .collect();
        if !args.is_empty() {
            return Some(CssLength::Min(args));
        }
        return Some(CssLength::Raw(v.to_string()));
    }
    if let Some(inner) = strip_func(v, "max") {
        let args: Vec<CssLength> = split_top_level_commas(inner)
            .into_iter()
            .filter_map(parse_css_length)
            .collect();
        if !args.is_empty() {
            return Some(CssLength::Max(args));
        }
        return Some(CssLength::Raw(v.to_string()));
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
        return Some(CssLength::Raw(v.to_string()));
    }
    if let Some(inner) = strip_func(v, "fit-content") {
        return parse_css_length(inner).or_else(|| Some(CssLength::Raw(v.to_string())));
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
    Some(CssLength::Raw(v.to_string()))
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
    if parser.is_eof() {
        Some(expr)
    } else {
        None
    }
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
            return Some(CssMathExpr::Sub(
                Box::new(CssMathExpr::Number(0.0)),
                Box::new(rhs),
            ));
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
        return Some(CssColor::Hex(v.to_string()));
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
        return Some(CssColor::Function(v.to_string()));
    }
    // Treat as named color
    Some(CssColor::Named(v.to_string()))
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
        return Some(CssImage::Function(v.to_string()));
    }
    None
}

pub fn parse_css_url(value: &str) -> Option<String> {
    let inner = strip_function(value, "url")?;
    let inner = inner.trim();
    if inner.is_empty() {
        return None;
    }
    let unquoted = if (inner.starts_with('"') && inner.ends_with('"'))
        || (inner.starts_with('\'') && inner.ends_with('\''))
    {
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
        Some(trimmed.to_string())
    }
}

fn looks_like_function(value: &str) -> bool {
    let trimmed = value.trim();
    let Some(open) = trimmed.find('(') else {
        return false;
    };
    trimmed.ends_with(')')
        && trimmed[..open]
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '-')
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
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(Display::None),
        "block" => Some(Display::Block),
        "inline" => Some(Display::Inline),
        "inline-block" => Some(Display::InlineBlock),
        "flex" => Some(Display::Flex),
        "inline-flex" => Some(Display::InlineFlex),
        "grid" => Some(Display::Grid),
        "inline-grid" => Some(Display::InlineGrid),
        "contents" => Some(Display::Contents),
        _ => None,
    }
}

fn parse_position(value: &str) -> Option<Position> {
    match value.to_ascii_lowercase().as_str() {
        "static" => Some(Position::Static),
        "relative" => Some(Position::Relative),
        "absolute" => Some(Position::Absolute),
        "fixed" => Some(Position::Fixed),
        "sticky" => Some(Position::Sticky),
        _ => None,
    }
}

fn parse_background_clip(value: &str) -> Option<BackgroundClip> {
    match value.to_ascii_lowercase().as_str() {
        "border-box" => Some(BackgroundClip::BorderBox),
        "padding-box" => Some(BackgroundClip::PaddingBox),
        "content-box" => Some(BackgroundClip::ContentBox),
        _ => None,
    }
}

fn parse_background_repeat(value: &str) -> Option<BackgroundRepeat> {
    match value.to_ascii_lowercase().as_str() {
        "repeat" => Some(BackgroundRepeat::Repeat),
        "repeat-x" => Some(BackgroundRepeat::RepeatX),
        "repeat-y" => Some(BackgroundRepeat::RepeatY),
        "no-repeat" => Some(BackgroundRepeat::NoRepeat),
        "space" => Some(BackgroundRepeat::Space),
        "round" => Some(BackgroundRepeat::Round),
        _ => None,
    }
}

fn parse_border_style(value: &str) -> Option<BorderStyle> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(BorderStyle::None),
        "hidden" => Some(BorderStyle::Hidden),
        "solid" => Some(BorderStyle::Solid),
        "dashed" => Some(BorderStyle::Dashed),
        "dotted" => Some(BorderStyle::Dotted),
        "double" => Some(BorderStyle::Double),
        "groove" => Some(BorderStyle::Groove),
        "ridge" => Some(BorderStyle::Ridge),
        "inset" => Some(BorderStyle::Inset),
        "outset" => Some(BorderStyle::Outset),
        _ => None,
    }
}

fn parse_font_weight(value: &str) -> Option<FontWeight> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(FontWeight::Normal),
        "bold" => Some(FontWeight::Bold),
        "bolder" => Some(FontWeight::Bolder),
        "lighter" => Some(FontWeight::Lighter),
        _ => value.parse::<u16>().ok().map(FontWeight::Weight),
    }
}

fn parse_font_style(value: &str) -> Option<FontStyle> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(FontStyle::Normal),
        "italic" => Some(FontStyle::Italic),
        "oblique" => Some(FontStyle::Oblique),
        _ => None,
    }
}

fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value.to_ascii_lowercase().as_str() {
        "left" => Some(TextAlign::Left),
        "right" => Some(TextAlign::Right),
        "center" => Some(TextAlign::Center),
        "justify" => Some(TextAlign::Justify),
        "start" => Some(TextAlign::Start),
        "end" => Some(TextAlign::End),
        _ => None,
    }
}

fn parse_text_transform(value: &str) -> Option<TextTransform> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextTransform::None),
        "capitalize" => Some(TextTransform::Capitalize),
        "uppercase" => Some(TextTransform::Uppercase),
        "lowercase" => Some(TextTransform::Lowercase),
        _ => None,
    }
}

fn parse_white_space(value: &str) -> Option<WhiteSpace> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(WhiteSpace::Normal),
        "nowrap" => Some(WhiteSpace::Nowrap),
        "pre" => Some(WhiteSpace::Pre),
        "pre-wrap" => Some(WhiteSpace::PreWrap),
        "pre-line" => Some(WhiteSpace::PreLine),
        "break-spaces" => Some(WhiteSpace::BreakSpaces),
        _ => None,
    }
}

fn parse_overflow(value: &str) -> Option<Overflow> {
    match value.to_ascii_lowercase().as_str() {
        "visible" => Some(Overflow::Visible),
        "hidden" => Some(Overflow::Hidden),
        "clip" => Some(Overflow::Clip),
        "scroll" => Some(Overflow::Scroll),
        "auto" => Some(Overflow::Auto),
        _ => None,
    }
}

fn parse_visibility(value: &str) -> Option<Visibility> {
    match value.to_ascii_lowercase().as_str() {
        "visible" => Some(Visibility::Visible),
        "hidden" => Some(Visibility::Hidden),
        "collapse" => Some(Visibility::Collapse),
        _ => None,
    }
}

fn parse_flex_direction(value: &str) -> Option<FlexDirection> {
    match value.to_ascii_lowercase().as_str() {
        "row" => Some(FlexDirection::Row),
        "row-reverse" => Some(FlexDirection::RowReverse),
        "column" => Some(FlexDirection::Column),
        "column-reverse" => Some(FlexDirection::ColumnReverse),
        _ => None,
    }
}

fn parse_flex_wrap(value: &str) -> Option<FlexWrap> {
    match value.to_ascii_lowercase().as_str() {
        "nowrap" => Some(FlexWrap::Nowrap),
        "wrap" => Some(FlexWrap::Wrap),
        "wrap-reverse" => Some(FlexWrap::WrapReverse),
        _ => None,
    }
}

fn parse_justify_content(value: &str) -> Option<JustifyContent> {
    match value.to_ascii_lowercase().as_str() {
        "start" => Some(JustifyContent::Start),
        "end" => Some(JustifyContent::End),
        "center" => Some(JustifyContent::Center),
        "flex-start" => Some(JustifyContent::FlexStart),
        "flex-end" => Some(JustifyContent::FlexEnd),
        "left" => Some(JustifyContent::Left),
        "right" => Some(JustifyContent::Right),
        "space-between" => Some(JustifyContent::SpaceBetween),
        "space-around" => Some(JustifyContent::SpaceAround),
        "space-evenly" => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}

fn parse_align_items(value: &str) -> Option<AlignItems> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(AlignItems::Normal),
        "stretch" => Some(AlignItems::Stretch),
        "center" => Some(AlignItems::Center),
        "start" => Some(AlignItems::Start),
        "end" => Some(AlignItems::End),
        "flex-start" => Some(AlignItems::FlexStart),
        "flex-end" => Some(AlignItems::FlexEnd),
        "baseline" => Some(AlignItems::Baseline),
        _ => None,
    }
}

fn parse_align_content(value: &str) -> Option<AlignContent> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(AlignContent::Normal),
        "stretch" => Some(AlignContent::Stretch),
        "center" => Some(AlignContent::Center),
        "start" => Some(AlignContent::Start),
        "end" => Some(AlignContent::End),
        "flex-start" => Some(AlignContent::FlexStart),
        "flex-end" => Some(AlignContent::FlexEnd),
        "space-between" => Some(AlignContent::SpaceBetween),
        "space-around" => Some(AlignContent::SpaceAround),
        "space-evenly" => Some(AlignContent::SpaceEvenly),
        _ => None,
    }
}

fn parse_align_self(value: &str) -> Option<AlignSelf> {
    match value.to_ascii_lowercase().as_str() {
        "auto" => Some(AlignSelf::Auto),
        "normal" => Some(AlignSelf::Normal),
        "stretch" => Some(AlignSelf::Stretch),
        "center" => Some(AlignSelf::Center),
        "start" => Some(AlignSelf::Start),
        "end" => Some(AlignSelf::End),
        "flex-start" => Some(AlignSelf::FlexStart),
        "flex-end" => Some(AlignSelf::FlexEnd),
        "baseline" => Some(AlignSelf::Baseline),
        _ => None,
    }
}

fn parse_justify_items(value: &str) -> Option<JustifyItems> {
    match value.to_ascii_lowercase().as_str() {
        "normal" => Some(JustifyItems::Normal),
        "stretch" => Some(JustifyItems::Stretch),
        "center" => Some(JustifyItems::Center),
        "start" => Some(JustifyItems::Start),
        "end" => Some(JustifyItems::End),
        "flex-start" => Some(JustifyItems::FlexStart),
        "flex-end" => Some(JustifyItems::FlexEnd),
        "left" => Some(JustifyItems::Left),
        "right" => Some(JustifyItems::Right),
        "baseline" => Some(JustifyItems::Baseline),
        _ => None,
    }
}

fn parse_justify_self(value: &str) -> Option<JustifySelf> {
    match value.to_ascii_lowercase().as_str() {
        "auto" => Some(JustifySelf::Auto),
        "normal" => Some(JustifySelf::Normal),
        "stretch" => Some(JustifySelf::Stretch),
        "center" => Some(JustifySelf::Center),
        "start" => Some(JustifySelf::Start),
        "end" => Some(JustifySelf::End),
        "flex-start" => Some(JustifySelf::FlexStart),
        "flex-end" => Some(JustifySelf::FlexEnd),
        "left" => Some(JustifySelf::Left),
        "right" => Some(JustifySelf::Right),
        "baseline" => Some(JustifySelf::Baseline),
        _ => None,
    }
}

/// `grid-auto-flow: row | column | row dense | column dense | dense`.
/// `dense` packing isn't honoured at layout time yet; we accept the
/// keyword for cascade fidelity.
fn parse_grid_auto_flow(value: &str) -> Option<GridAutoFlow> {
    let lower = value.to_ascii_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().collect();
    let dense = tokens.iter().any(|t| *t == "dense");
    let column = tokens.iter().any(|t| *t == "column");
    match (column, dense) {
        (true, true) => Some(GridAutoFlow::ColumnDense),
        (true, false) => Some(GridAutoFlow::Column),
        (false, true) => Some(GridAutoFlow::RowDense),
        (false, false) => {
            // Empty token list is invalid; otherwise default to Row.
            if tokens.is_empty() {
                None
            } else {
                Some(GridAutoFlow::Row)
            }
        }
    }
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
                if count_tok.eq_ignore_ascii_case("auto-fill")
                    || count_tok.eq_ignore_ascii_case("auto-fit")
                {
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
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("auto") || trimmed.is_empty() {
        return Some(GridLine::Auto);
    }
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if tokens.len() == 2 && tokens[0].eq_ignore_ascii_case("span") {
        if let Ok(n) = tokens[1].parse::<u32>() {
            if n >= 1 {
                return Some(GridLine::Span(n));
            }
        }
        return None;
    }
    if tokens.len() == 1 {
        if let Ok(n) = tokens[0].parse::<i32>() {
            if n != 0 {
                return Some(GridLine::Line(n));
            }
        }
    }
    None
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
        GridAxis::Column => style.grid_column = Some(value.to_string()),
        GridAxis::Row => style.grid_row = Some(value.to_string()),
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
    style.flex = Some(value.to_string());
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
                basis = Some(CssLength::Px(0.0));
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
                basis = Some(CssLength::Px(0.0));
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
    match value.to_ascii_lowercase().as_str() {
        "auto" => Some(Cursor::Auto),
        "default" => Some(Cursor::Default),
        "pointer" => Some(Cursor::Pointer),
        "text" => Some(Cursor::Text),
        "move" => Some(Cursor::Move),
        "not-allowed" => Some(Cursor::NotAllowed),
        "grab" => Some(Cursor::Grab),
        "grabbing" => Some(Cursor::Grabbing),
        "crosshair" => Some(Cursor::Crosshair),
        "wait" => Some(Cursor::Wait),
        "help" => Some(Cursor::Help),
        "progress" => Some(Cursor::Progress),
        "none" => Some(Cursor::None),
        "resize" => Some(Cursor::Resize),
        _ => Some(Cursor::Raw(value.to_string())),
    }
}

fn parse_pointer_events(value: &str) -> Option<PointerEvents> {
    match value.to_ascii_lowercase().as_str() {
        "auto" => Some(PointerEvents::Auto),
        "none" => Some(PointerEvents::None),
        _ => None,
    }
}

fn parse_user_select(value: &str) -> Option<UserSelect> {
    match value.to_ascii_lowercase().as_str() {
        "auto" => Some(UserSelect::Auto),
        "none" => Some(UserSelect::None),
        "text" => Some(UserSelect::Text),
        "all" => Some(UserSelect::All),
        _ => None,
    }
}

fn parse_box_sizing(value: &str) -> Option<BoxSizing> {
    match value.to_ascii_lowercase().as_str() {
        "content-box" => Some(BoxSizing::ContentBox),
        "border-box" => Some(BoxSizing::BorderBox),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_style() {
        let style = parse_inline_style("display: flex; color: red; padding: 10px;");
        assert!(matches!(style.display, Some(Display::Flex)));
        assert!(matches!(style.color, Some(CssColor::Named(ref s)) if s == "red"));
        assert!(matches!(style.padding, Some(CssLength::Px(10.0))));
    }

    #[test]
    fn test_parse_css_length() {
        assert!(matches!(parse_css_length("auto"), Some(CssLength::Auto)));
        assert!(matches!(parse_css_length("0"), Some(CssLength::Zero)));
        assert!(
            matches!(parse_css_length("10px"), Some(CssLength::Px(v)) if (v - 10.0).abs() < 0.01)
        );
        assert!(
            matches!(parse_css_length("50%"), Some(CssLength::Percent(v)) if (v - 50.0).abs() < 0.01)
        );
        assert!(
            matches!(parse_css_length("1.5em"), Some(CssLength::Em(v)) if (v - 1.5).abs() < 0.01)
        );
        assert!(
            matches!(parse_css_length("2rem"), Some(CssLength::Rem(v)) if (v - 2.0).abs() < 0.01)
        );
    }

    #[test]
    fn test_parse_css_color_hex() {
        assert!(matches!(parse_css_color("#ff0000"), Some(CssColor::Hex(ref s)) if s == "#ff0000"));
    }

    #[test]
    fn test_parse_css_color_rgb() {
        let c = parse_css_color("rgb(255, 128, 0)");
        assert!(matches!(c, Some(CssColor::Rgb(255, 128, 0))));
    }

    #[test]
    fn test_parse_css_color_rgba() {
        let c = parse_css_color("rgba(255, 128, 0, 0.5)");
        assert!(matches!(c, Some(CssColor::Rgba(255, 128, 0, a)) if (a - 0.5).abs() < 0.01));
    }

    #[test]
    fn test_parse_css_color_transparent() {
        assert!(matches!(
            parse_css_color("transparent"),
            Some(CssColor::Transparent)
        ));
    }

    #[test]
    fn test_font_weight_numeric() {
        assert!(matches!(
            parse_font_weight("700"),
            Some(FontWeight::Weight(700))
        ));
        assert!(matches!(parse_font_weight("bold"), Some(FontWeight::Bold)));
    }

    #[test]
    fn padding_shorthand_one_value() {
        let s = parse_inline_style("padding: 10px;");
        assert!(matches!(s.padding_top, Some(CssLength::Px(10.0))));
        assert!(matches!(s.padding_right, Some(CssLength::Px(10.0))));
        assert!(matches!(s.padding_bottom, Some(CssLength::Px(10.0))));
        assert!(matches!(s.padding_left, Some(CssLength::Px(10.0))));
        // shorthand field stays set so the merge layer's "shorthand clears
        // inherited per-side base" rule still fires.
        assert!(s.padding.is_some());
    }

    #[test]
    fn padding_shorthand_two_values() {
        let s = parse_inline_style("padding: 6px 10px;");
        assert!(matches!(s.padding_top, Some(CssLength::Px(6.0))));
        assert!(matches!(s.padding_bottom, Some(CssLength::Px(6.0))));
        assert!(matches!(s.padding_left, Some(CssLength::Px(10.0))));
        assert!(matches!(s.padding_right, Some(CssLength::Px(10.0))));
    }

    #[test]
    fn padding_shorthand_three_values() {
        let s = parse_inline_style("padding: 1px 2px 3px;");
        assert!(matches!(s.padding_top, Some(CssLength::Px(1.0))));
        assert!(matches!(s.padding_right, Some(CssLength::Px(2.0))));
        assert!(matches!(s.padding_left, Some(CssLength::Px(2.0))));
        assert!(matches!(s.padding_bottom, Some(CssLength::Px(3.0))));
    }

    #[test]
    fn padding_shorthand_four_values() {
        let s = parse_inline_style("padding: 1px 2px 3px 4px;");
        assert!(matches!(s.padding_top, Some(CssLength::Px(1.0))));
        assert!(matches!(s.padding_right, Some(CssLength::Px(2.0))));
        assert!(matches!(s.padding_bottom, Some(CssLength::Px(3.0))));
        assert!(matches!(s.padding_left, Some(CssLength::Px(4.0))));
    }

    #[test]
    fn margin_shorthand_two_values_mixed_units() {
        let s = parse_inline_style("margin: 1em 20px;");
        assert!(matches!(s.margin_top,    Some(CssLength::Em(v)) if (v - 1.0).abs() < 0.01));
        assert!(matches!(s.margin_bottom, Some(CssLength::Em(v)) if (v - 1.0).abs() < 0.01));
        assert!(matches!(s.margin_left, Some(CssLength::Px(20.0))));
        assert!(matches!(s.margin_right, Some(CssLength::Px(20.0))));
    }

    #[test]
    fn padding_shorthand_zero_and_auto() {
        let s = parse_inline_style("margin: 0 auto;");
        assert!(matches!(s.margin_top, Some(CssLength::Zero)));
        assert!(matches!(s.margin_bottom, Some(CssLength::Zero)));
        assert!(matches!(s.margin_left, Some(CssLength::Auto)));
        assert!(matches!(s.margin_right, Some(CssLength::Auto)));
    }

    #[test]
    fn padding_shorthand_too_many_tokens_is_invalid() {
        // 5+ tokens → entire declaration is dropped (per CSS spec); per-side
        // fields remain unset so a previous layer can show through.
        let s = parse_inline_style("padding: 1px 2px 3px 4px 5px;");
        assert!(s.padding_top.is_none());
        assert!(s.padding_right.is_none());
        assert!(s.padding_bottom.is_none());
        assert!(s.padding_left.is_none());
        assert!(s.padding.is_none());
    }

    #[test]
    fn flex_shorthand_extracts_flex_grow() {
        let s = parse_inline_style("flex: 1;");
        assert_eq!(s.flex_grow, Some(1.0));
        assert_eq!(s.flex.as_deref(), Some("1"));

        let s = parse_inline_style("flex: 2.5;");
        assert_eq!(s.flex_grow, Some(2.5));

        let s = parse_inline_style("flex: auto;");
        assert_eq!(s.flex_grow, Some(1.0));

        let s = parse_inline_style("flex: none;");
        assert_eq!(s.flex_grow, Some(0.0));

        // Multi-value form: first token is flex-grow.
        let s = parse_inline_style("flex: 3 1 0%;");
        assert_eq!(s.flex_grow, Some(3.0));
    }

    #[test]
    fn parse_box_shorthand_direct() {
        let (t, r, b, l) = parse_box_shorthand("1px 2px 3px 4px");
        assert!(matches!(t, Some(CssLength::Px(1.0))));
        assert!(matches!(r, Some(CssLength::Px(2.0))));
        assert!(matches!(b, Some(CssLength::Px(3.0))));
        assert!(matches!(l, Some(CssLength::Px(4.0))));

        // 5+ tokens → all None (per-spec: invalid declaration).
        let (t, r, b, l) = parse_box_shorthand("1px 2px 3px 4px 5px");
        assert!(t.is_none() && r.is_none() && b.is_none() && l.is_none());
    }

    #[test]
    fn test_full_style() {
        let css = "display: grid; position: sticky; flex-direction: column; \
                    justify-content: space-between; align-items: center; \
                    font-weight: 600; opacity: 0.8; z-index: 10; \
                    box-sizing: border-box; cursor: pointer;";
        let style = parse_inline_style(css);
        assert!(matches!(style.display, Some(Display::Grid)));
        assert!(matches!(style.position, Some(Position::Sticky)));
        assert!(matches!(style.flex_direction, Some(FlexDirection::Column)));
        assert!(matches!(
            style.justify_content,
            Some(JustifyContent::SpaceBetween)
        ));
        assert!(matches!(style.align_items, Some(AlignItems::Center)));
        assert!(matches!(style.font_weight, Some(FontWeight::Weight(600))));
        assert!(matches!(style.opacity, Some(v) if (v - 0.8).abs() < 0.01));
        assert_eq!(style.z_index, Some(10));
        assert!(matches!(style.box_sizing, Some(BoxSizing::BorderBox)));
        assert!(matches!(style.cursor, Some(Cursor::Pointer)));
    }

    // ---------------------------------------------------------------------
    // !important
    // ---------------------------------------------------------------------

    #[test]
    fn important_routes_to_important_bucket() {
        let decls = parse_inline_style_decls("color: red !important;");
        assert!(decls.normal.color.is_none());
        assert!(decls.important.color.is_some());
    }

    #[test]
    fn normal_and_important_in_same_block_split_by_property() {
        let decls = parse_inline_style_decls("color: red !important; background-color: blue;");
        assert!(decls.normal.color.is_none());
        assert!(decls.normal.background_color.is_some());
        assert!(decls.important.color.is_some());
        assert!(decls.important.background_color.is_none());
    }

    #[test]
    fn important_value_parses_as_if_it_were_normal() {
        // The trailing `!important` must not bleed into the value.
        let decls = parse_inline_style_decls("width: 100px !important;");
        assert!(matches!(decls.important.width, Some(CssLength::Px(v)) if v == 100.0));
    }

    #[test]
    fn important_is_case_insensitive_and_whitespace_tolerant() {
        // CSS spec allows whitespace between `!` and `important`,
        // and the keyword is case-insensitive.
        for css in [
            "color: red !important;",
            "color: red ! important;",
            "color: red  !  IMPORTANT  ;",
            "color: red !IMPORTANT;",
        ] {
            let decls = parse_inline_style_decls(css);
            assert!(
                decls.important.color.is_some(),
                "expected `{css}` to be marked important"
            );
            assert!(decls.normal.color.is_none(), "`{css}` leaked into normal");
        }
    }

    #[test]
    fn parse_inline_style_folds_important_back_in() {
        // Back-compat path: `parse_inline_style` returns a single
        // `Style` with !important values overlaid on top of normal
        // ones, so existing callers see the "winning" value.
        let style = parse_inline_style("color: red; color: blue !important;");
        let c = style.color.expect("color set");
        assert!(matches!(c, CssColor::Named(s) if s == "blue"));
    }

    #[test]
    fn bare_word_important_without_bang_is_not_important() {
        // `important` without the `!` must not flip the !important
        // bit. Whether the value parses into `decls.normal.color` at
        // all depends on the property's own permissiveness; the
        // invariant we're asserting here is just that the important
        // bucket stays untouched.
        let decls = parse_inline_style_decls("color: red important;");
        assert!(decls.important.color.is_none());
    }

    #[test]
    fn border_shorthand_with_rgb_color_keeps_function_intact() {
        // `border: 2px solid rgb(212, 175, 55)` — the shorthand
        // tokenizer must respect parentheses and hand the whole
        // `rgb(...)` chunk to the colour parser as a single token,
        // not split it on the inner whitespace.
        let style = parse_inline_style("border: 2px solid rgb(212, 175, 55);");
        let top_color = style
            .border_top_color
            .as_ref()
            .expect("border-top-color should be set");
        match top_color {
            wgpu_html_models::common::css_enums::CssColor::Rgb(212, 175, 55) => {}
            other => panic!("expected Rgb(212, 175, 55), got {:?}", other),
        }
        assert!(matches!(
            style.border_top_style,
            Some(wgpu_html_models::common::css_enums::BorderStyle::Solid)
        ));
    }
}
