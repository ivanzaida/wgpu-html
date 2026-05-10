//! Text shaping, whitespace processing, font resolution, and text-leaf
//! construction.  These helpers are shared across block, inline, flex, and
//! grid layout paths.

use lui_models::{
    common::css_enums::{
        CssLength, FontWeight, FontStyle, TextTransform, WhiteSpace, WordBreak,
        Cursor, PointerEvents, Resize, UserSelect,
    },
    Style,
};
use lui_text::{FontHandle, FontStyleAxis, ShapedRun};

use crate::{
    color::resolve_color,
    resolved_opacity, resolved_cursor, resolved_user_select, resolved_z_index,
    types::*,
    Ctx,
};

// ---------------------------------------------------------------------------
// Text shaping
// ---------------------------------------------------------------------------

/// Shape a text-node string against the current `TextContext`. Reads
/// `font-size` and `line-height` from the cascaded style (which the
/// inheritance pass filled in from the nearest ancestor that set
/// them); falls back to 16px / 1.25× when unset. Picks the first
/// registered font for now — proper `font-family` matching is T4.
pub(crate) fn shape_text_run(
    text: &str,
    style: &Style,
    max_width_px: Option<f32>,
    trim_edges: bool,
    ctx: &mut Ctx,
) -> (Option<ShapedRun>, f32, f32, f32) {
    crate::layout_profile::count_text_shape(&mut ctx.profiler);
    if text.is_empty() {
        return (None, 0.0, 0.0, 0.0);
    }

    // CSS `white-space: normal` (the default) collapses runs of
    // whitespace — including embedded `\n` from raw HTML source —
    // into a single space. Apply that first so cosmic-text doesn't
    // see a leading newline and produce an empty first layout run
    // (we only consume `layout_runs().next()` until line breaking
    // lands in T7). Without this, a text leaf like "\n    Plain, "
    // shapes to width 0 and paints nothing.
    let mut prev_collapsed_space = false;
    let normalized = normalize_text_for_style(text, style, Some(&mut prev_collapsed_space));
    let normalized = if trim_edges && style_collapses_whitespace(style) {
        trim_collapsed_whitespace_edges(&normalized, true, true).to_string()
    } else {
        normalized
    };
    if normalized.is_empty() {
        return (None, 0.0, 0.0, 0.0);
    }

    // `text-transform` re-cases the *visible* text before shaping. Do
    // it once here so `font-feature` style ligatures still apply to
    // the transformed forms.
    let transformed = apply_text_transform(&normalized, style.text_transform.as_ref());
    let display_text: &str = match transformed.as_ref() {
        Some(s) => s.as_str(),
        None => &normalized,
    };

    // CSS `word-break: break-all`: insert U+200B between every
    // character so the shaper treats every boundary as a break opportunity.
    let break_all_buf;
    if style_breaks_all(style) && display_text.len() > 1 {
        let mut buf = String::with_capacity(display_text.len() * 2);
        let mut chars = display_text.chars();
        if let Some(first) = chars.next() {
            buf.push(first);
            for ch in chars {
                buf.push('\u{200B}');
                buf.push(ch);
            }
        }
        break_all_buf = Some(buf);
    } else {
        break_all_buf = None;
    }
    let display_text: &str = match break_all_buf.as_ref() {
        Some(s) => s.as_str(),
        None => display_text,
    };

    // Family / weight / style come from the cascaded style. The text
    // node itself has no rules applied, but cascade inheritance has
    // already pulled these from the nearest ancestor that set them
    // (or from UA defaults for `<b>`, `<strong>`, `<em>`, …).
    let families = parse_family_list(style.font_family.as_deref());
    let family_refs: Vec<&str> = families.iter().map(String::as_str).collect();
    let weight = font_weight_value(style.font_weight.as_ref());
    let axis = font_style_axis(style.font_style.as_ref());

    let Some(handle) = ctx.text.ctx.pick_font(&family_refs, weight, axis) else {
        return (None, 0.0, 0.0, 0.0);
    };

    let size_css = font_size_px(style).unwrap_or(16.0);
    let line_h_css = line_height_px_for_font(style, size_css, &ctx.text.ctx, handle);
    let size_px = size_css * ctx.scale;
    let line_height = line_h_css * ctx.scale;
    let letter_spacing = letter_spacing_px(style, size_css) * ctx.scale;
    let color = style
        .color
        .as_ref()
        .and_then(resolve_color)
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let wrap_enabled = style_wraps_text(style);
    match ctx.text.ctx.shape_and_pack(
        display_text,
        handle,
        size_px,
        line_height,
        letter_spacing,
        weight,
        axis,
        if wrap_enabled { max_width_px } else { None },
        color,
    ) {
        Some(run) => {
            let w = run.width;
            let h = run.height;
            let a = run.ascent;
            (Some(run), w, h, a)
        }
        None => (None, 0.0, 0.0, 0.0),
    }
}

/// Resolve `letter-spacing` to CSS pixels. `Px` is literal; `Em` /
/// `Rem` multiply against `font_size`. Anything else (percent,
/// unset) is treated as zero.
fn letter_spacing_px(style: &Style, font_size: f32) -> f32 {
    match style.letter_spacing.as_ref() {
        Some(CssLength::Px(v)) => *v,
        Some(CssLength::Em(v)) | Some(CssLength::Rem(v)) => v * font_size,
        _ => 0.0,
    }
}

/// Apply CSS `text-transform`. Returns `None` when no transform is
/// set (caller can keep using the original `&str` without an extra
/// allocation), otherwise the transformed string.
pub(crate) fn apply_text_transform(text: &str, tt: Option<&TextTransform>) -> Option<String> {
    match tt {
        Some(TextTransform::Uppercase) => Some(text.to_uppercase()),
        Some(TextTransform::Lowercase) => Some(text.to_lowercase()),
        Some(TextTransform::Capitalize) => Some(capitalize_words(text)),
        // None / FullWidth / FullSizeKana — pass through unchanged.
        _ => None,
    }
}

/// `text-transform: capitalize` — uppercase the first letter of each
/// run of non-whitespace characters; pass everything else through.
fn capitalize_words(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for ch in s.chars() {
        if ch.is_whitespace() {
            at_word_start = true;
            out.push(ch);
        } else if at_word_start {
            for u in ch.to_uppercase() {
                out.push(u);
            }
            at_word_start = false;
        } else {
            out.push(ch);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// White-space collapsing
// ---------------------------------------------------------------------------

/// CSS `white-space: normal` whitespace collapsing: every run of
/// ASCII / Unicode whitespace (including `\n`, `\t`, `\r`) becomes a
/// single ASCII space. Callers decide whether block / paragraph edges
/// should then trim those collapsed spaces away. Returns an owned
/// `String` because the typical input differs from the output.
#[cfg(test)]
pub(crate) fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut prev_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out
}

fn collapse_whitespace_with_state(text: &str, prev_space: &mut bool) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !*prev_space {
                out.push(' ');
                *prev_space = true;
            }
        } else {
            out.push(ch);
            *prev_space = false;
        }
    }
    out
}

fn collapse_preserving_newlines_with_state(text: &str, prev_space: &mut bool) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\n' => {
                if out.ends_with(' ') {
                    out.pop();
                }
                out.push('\n');
                *prev_space = false;
            }
            ' ' | '\t' | '\r' | '\u{000C}' => {
                if !*prev_space {
                    out.push(' ');
                    *prev_space = true;
                }
            }
            _ => {
                out.push(ch);
                *prev_space = false;
            }
        }
    }
    out
}

fn style_white_space(style: &Style) -> WhiteSpace {
    style.white_space.clone().unwrap_or(WhiteSpace::Normal)
}

pub(crate) fn style_collapses_whitespace(style: &Style) -> bool {
    matches!(
        style_white_space(style),
        WhiteSpace::Normal | WhiteSpace::Nowrap | WhiteSpace::PreLine
    )
}

pub(crate) fn style_wraps_text(style: &Style) -> bool {
    if let Some(mode) = style.deferred_longhands.get("text-wrap-mode") {
        match mode.trim().to_ascii_lowercase().as_str() {
            "nowrap" => return false,
            "wrap" => return true,
            _ => {}
        }
    }
    if let Some(mode) = style.deferred_longhands.get("overflow-wrap") {
        match mode.trim().to_ascii_lowercase().as_str() {
            "break-word" | "anywhere" => return true,
            _ => {}
        }
    }
    !matches!(style_white_space(style), WhiteSpace::Nowrap | WhiteSpace::Pre)
}

pub(crate) fn style_breaks_all(style: &Style) -> bool {
    matches!(style.word_break, Some(WordBreak::BreakAll))
}

pub(crate) fn normalize_text_for_style(text: &str, style: &Style, prev_space: Option<&mut bool>) -> String {
    match style_white_space(style) {
        WhiteSpace::Normal | WhiteSpace::Nowrap => {
            let mut local_prev = false;
            let state = match prev_space {
                Some(state) => state,
                None => &mut local_prev,
            };
            collapse_whitespace_with_state(text, state)
        }
        WhiteSpace::PreLine => {
            let mut local_prev = false;
            let state = match prev_space {
                Some(state) => state,
                None => &mut local_prev,
            };
            collapse_preserving_newlines_with_state(text, state)
        }
        WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::BreakSpaces => {
            if let Some(state) = prev_space {
                *state = false;
            }
            text.to_string()
        }
    }
}

pub(crate) fn split_collapsed_first_word_prefix_and_tail(text: &str, style: &Style) -> Option<(String, String)> {
    if !style_collapses_whitespace(style) {
        return None;
    }
    let mut prev_space = false;
    let normalized = normalize_text_for_style(text, style, Some(&mut prev_space));
    let trimmed = normalized.trim_start_matches(' ');
    if trimmed.is_empty() {
        return None;
    }
    let lead = normalized.len().saturating_sub(trimmed.len());
    let word_end_rel = trimmed.find(' ').unwrap_or(trimmed.len());
    let split_at = lead + word_end_rel;
    if split_at == 0 || split_at >= normalized.len() {
        return None;
    }
    Some((normalized[..split_at].to_string(), normalized[split_at..].to_string()))
}

pub(crate) fn trim_collapsed_whitespace_edges(text: &str, trim_start: bool, trim_end: bool) -> &str {
    let text = if trim_start { text.trim_start_matches(' ') } else { text };
    if trim_end { text.trim_end_matches(' ') } else { text }
}

// ---------------------------------------------------------------------------
// Box construction
// ---------------------------------------------------------------------------

/// Zero-area `LayoutBox` for elements whose effective `display` is
/// `none`. The parent treats it as contributing no width / height /
/// children, so the subtree disappears from the box tree completely.
pub(crate) fn empty_box(origin_x: f32, origin_y: f32) -> LayoutBox {
    let r = Rect::new(origin_x, origin_y, 0.0, 0.0);
    LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: r,
        background: None,
        background_rect: r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Block,
        text_run: None,
        text_color: None,
        text_unselectable: false,
        text_decorations: Vec::new(),
        overflow: OverflowAxes::visible(),
        resize: Resize::None,
        text_overflow: None,
    transform: None,
    transform_origin: (0.0, 0.0),
    box_shadows: Vec::new(),
        opacity: 1.0,
        pointer_events: PointerEvents::Auto,
        user_select: UserSelect::Auto,
        cursor: Cursor::Auto,
        z_index: None,
        image: None,
        background_image: None,
        first_line_color: None,
        first_letter_color: None,
        selection_bg: None,
        selection_fg: None,
        accent_color: None,
        lui: LuiProperties::default(),
        lui_popup: None,
        lui_color_picker: None,
        lui_calendar: None,
        file_button: None,
        children: Vec::new(),
        is_fixed: false,
        form_control: None,
    }
}

/// Build a text-leaf `LayoutBox` for an `Element::Text`. Used both
/// from block-flow (text as a degenerate "block" of one line) and
/// from the inline-formatting context.
pub(crate) fn make_text_leaf(
    text: &str,
    style: &Style,
    origin_x: f32,
    origin_y: f32,
    max_width_px: Option<f32>,
    trim_edges: bool,
    ctx: &mut Ctx,
) -> (LayoutBox, f32, f32, f32) {
    let (run, w, h, ascent) = shape_text_run(text, style, max_width_px, trim_edges, ctx);
    // Clamp box width to the container so text that exceeds its flex
    // item (e.g. when flex-shrink reduces the item below text width)
    // doesn't extend past the box bounds. The paint pass clips glyphs
    // to the box rect, preventing visual overflow into adjacent items.
    let box_w = match max_width_px {
        Some(max_w) => w.min(max_w),
        None => w,
    };
    let text_color = style
        .color
        .as_ref()
        .and_then(resolve_color)
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);
    let decorations = resolve_text_decorations(style);
    // The box height is determined by the line-height per CSS, but glyph
    // bitmaps (especially round glyphs and descenders) routinely extend
    // past the line box.  We keep the content_rect tall enough to cover
    // the full glyph quads so that no downstream clip / scissor can
    // accidentally cut off the bottom.  margin_rect / border_rect stay
    // at the CSS line-height so that sibling spacing isn't blown out.
    // Add 1px safety margin to the glyph quads so boundary conditions
    // (subpixel alignment, GPU rasterization, atlas placement) don't clip
    // the bottom or right edge of any glyph.
    let content_h = run.as_ref().map_or(h, |r| {
        let max_g = r.glyphs.iter().map(|g| g.y + g.h).fold(0.0f32, f32::max);
        h.max(max_g).ceil()
    });
    let r = Rect::new(origin_x, origin_y, box_w, h);
    let content_r = Rect::new(origin_x, origin_y, box_w, content_h);
    let box_ = LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: content_r,
        background: None,
        background_rect: content_r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Text,
        text_run: run,
        text_color: Some(text_color),
        text_unselectable: false,
        text_decorations: decorations,
        overflow: OverflowAxes::visible(),
        resize: Resize::None,
        text_overflow: None,
    transform: None,
    transform_origin: (0.0, 0.0),
    box_shadows: Vec::new(),
        opacity: resolved_opacity(style),
        pointer_events: PointerEvents::Auto,
        user_select: resolved_user_select(style),
        cursor: resolved_cursor(style),
        z_index: resolved_z_index(style),
        image: None,
        background_image: None,
        first_line_color: None,
        first_letter_color: None,
        selection_bg: None,
        selection_fg: None,
        accent_color: None,
        lui: LuiProperties::default(),
        lui_popup: None,
        lui_color_picker: None,
        lui_calendar: None,
        file_button: None,
        children: Vec::new(),
        is_fixed: false,
        form_control: None,
    };
    (box_, w, h, ascent)
}

pub(crate) fn measure_text_leaf(text: &str, style: &Style, ctx: &mut Ctx) -> (f32, f32) {
    // Delegate to shape_text_run so the measurement uses the exact same
    // code path (shape_and_pack) that produces the final glyphs. The old
    // measure_only path returned slightly different widths, causing ~1-2px
    // glyph overlap between adjacent flex items.
    let (_run, w, h, _ascent) = shape_text_run(text, style, None, true, ctx);
    (w, h)
}

// ---------------------------------------------------------------------------
// Text decoration
// ---------------------------------------------------------------------------

/// Parse the raw `text-decoration` shorthand string into the set of
/// active decoration lines. Whitespace-separated tokens; `none`
/// resets any previously-collected lines.
fn parse_text_decorations(s: &str) -> Vec<TextDecorationLine> {
    let mut out = Vec::new();
    for tok in s.split_ascii_whitespace() {
        match tok.to_ascii_lowercase().as_str() {
            "underline" => out.push(TextDecorationLine::Underline),
            "line-through" => out.push(TextDecorationLine::LineThrough),
            "overline" => out.push(TextDecorationLine::Overline),
            "none" => out.clear(),
            // Other tokens (colour names, "wavy", "solid", …) — the
            // value parser hands them along as part of the same raw
            // string; we ignore everything but the line keywords for
            // now.
            _ => {}
        }
    }
    out
}

pub(crate) fn resolve_text_decorations(style: &Style) -> Vec<TextDecorationLine> {
    style
        .text_decoration
        .as_deref()
        .map(parse_text_decorations)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Font resolution
// ---------------------------------------------------------------------------

/// Split a CSS `font-family` value into individual family names.
/// Each entry is trimmed and stripped of surrounding quotes; empty
/// entries are dropped. The empty list means "no family preference".
pub(crate) fn parse_family_list(s: Option<&str>) -> Vec<String> {
    let Some(raw) = s else { return Vec::new() };
    raw.split(',')
        .map(|p| p.trim().trim_matches('"').trim_matches('\'').trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// CSS `font-weight` → numeric weight in [100, 900]. `None` falls
/// back to 400 (CSS initial). `Lighter` / `Bolder` are treated as
/// fixed shifts here (no parent context yet) — 300 / 700.
pub(crate) fn font_weight_value(fw: Option<&FontWeight>) -> u16 {
    match fw {
        Some(FontWeight::Bold) => 700,
        Some(FontWeight::Lighter) => 300,
        Some(FontWeight::Bolder) => 700,
        Some(FontWeight::Weight(n)) => *n,
        Some(FontWeight::Normal) | None => 400,
    }
}

/// CSS `font-style` → font-registry style axis.
pub(crate) fn font_style_axis(fs: Option<&FontStyle>) -> FontStyleAxis {
    match fs {
        Some(FontStyle::Italic) => FontStyleAxis::Italic,
        Some(FontStyle::Oblique) => FontStyleAxis::Oblique,
        Some(FontStyle::Normal) | None => FontStyleAxis::Normal,
    }
}

/// Resolve `font-size` to CSS pixels. `Em` and `Rem` use 16px as the
/// reference (the T3 placeholder — proper `em` against the parent's
/// computed font size lands in T4 once the cascade tracks computed
/// values). `Percent`, viewport-relative units, and `auto` aren't
/// meaningful here yet and fall through.
pub(crate) fn font_size_px(style: &Style) -> Option<f32> {
    match style.font_size.as_ref()? {
        CssLength::Px(v) => Some(*v),
        CssLength::Em(v) | CssLength::Rem(v) => Some(v * 16.0),
        _ => None,
    }
}

/// Resolve `line-height` to CSS pixels. CSS allows a unitless number
/// (multiplier of font size); we currently parse line-height as
/// `CssLength`, so a `Px` value is the literal height and `Em` /
/// `Rem` multiply against `font_size_px`. Falls back to 1.25× the
/// font size when no font metrics are available.
pub(crate) fn line_height_px(style: &Style, font_size: f32) -> f32 {
    match style.line_height.as_ref() {
        Some(CssLength::Px(v)) => *v,
        Some(CssLength::Em(v)) | Some(CssLength::Rem(v)) => v * font_size,
        _ => font_size * 1.25,
    }
}

/// Like [`line_height_px`] but derives the `normal` default from the
/// actual font metrics instead of the hardcoded 1.25× multiplier.
///
/// Uses the same algorithm as browsers: OS/2 `USE_TYPO_METRICS` →
/// typo metrics; otherwise `usWinAscent + usWinDescent`; hhea as
/// last resort. See [`lui_text::parse_line_height_multiplier`].
fn line_height_px_for_font(
    style: &Style,
    font_size: f32,
    text_ctx: &lui_text::TextContext,
    handle: FontHandle,
) -> f32 {
    match style.line_height.as_ref() {
        Some(CssLength::Px(v)) => *v,
        Some(CssLength::Em(v)) | Some(CssLength::Rem(v)) => v * font_size,
        _ => {
            let multiplier = text_ctx.normal_line_height_multiplier(handle).unwrap_or(1.2);
            font_size * multiplier
        }
    }
}
