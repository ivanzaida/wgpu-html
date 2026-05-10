//! Form-control helpers: value/placeholder text shaping, appearance
//! detection, default line-height, vertical centering, and per-input
//! FormControlInfo construction.

use lui_models::common::css_enums;
use lui_style::CascadedNode;
use lui_text;
use lui_tree::Element;

use crate::{
    color,
    color::Color,
    text_shaping::shape_text_run,
    types::{FormControlInfo, FormControlKind, Rect},
    Ctx,
};

// ---------------------------------------------------------------------------
// Text shaping helpers for form controls
// ---------------------------------------------------------------------------

pub(crate) fn shape_input_text(
    text: &str,
    wraps: bool,
    content_rect: Rect,
    node: &CascadedNode,
    ctx: &mut Ctx,
) -> (Option<lui_text::ShapedRun>, Option<Color>) {
    if text.is_empty() {
        return (None, None);
    }
    let max_width = if wraps { Some(content_rect.w) } else { None };
    let (run, _w, _h, _a) = shape_text_run(text, &node.style, max_width, false, ctx);
    let fg = node.style.color.as_ref().and_then(|c| color::resolve_color(c));
    (run, fg)
}

/// Shape the current `value` of an `<input>` or `<textarea>` so the
/// field renders the user-entered text. Returns `(None, None)` for
/// non-form-control elements, hidden inputs, or fields with an empty
/// / absent value.
///
/// For `<input type="password">`, every character is replaced with
/// U+2022 BULLET before shaping so the underlying value stays clear
/// but the display shows dots.
pub(crate) fn compute_value_run(
    node: &CascadedNode,
    content_rect: Rect,
    ctx: &mut Ctx,
) -> (Option<lui_text::ShapedRun>, Option<Color>) {
    use lui_models::common::html_enums::InputType;

    let (value, is_password, wraps_multiline) = match &node.element {
        Element::Input(inp) => {
            if matches!(
                inp.r#type,
                Some(
                    InputType::Hidden
                        | InputType::Checkbox
                        | InputType::Radio
                        | InputType::Range
                        | InputType::Color
                )
            ) {
                return (None, None);
            }
            if matches!(inp.r#type, Some(InputType::File)) {
                let label = if let Some(first) = inp.files.first() {
                    if inp.files.len() > 1 {
                        format!("{} files", inp.files.len())
                    } else {
                        first.name.to_string()
                    }
                } else {
                    ctx.locale.file_no_file_label().to_string()
                };
                return shape_input_text(&label, false, content_rect, node, ctx);
            }
            // Date inputs: show locale-formatted value (or display value while editing).
            if matches!(inp.r#type, Some(InputType::Date) | Some(InputType::DatetimeLocal)) {
                let is_focused = ctx.date_display_value.is_some()
                    && ctx.date_focus_iso.as_deref() == inp.value.as_deref();
                let val = if is_focused {
                    ctx.date_display_value.as_ref().unwrap().clone()
                } else {
                    let iso = inp.value.as_deref().unwrap_or("");
                    if matches!(inp.r#type, Some(InputType::DatetimeLocal)) {
                        if let Some((y, m, d, h, min)) = lui_tree::date::parse_datetime_local(iso) {
                            ctx.locale.format_datetime(y, m, d, h, min)
                        } else if iso.is_empty() {
                            return (None, None);
                        } else {
                            iso.to_string()
                        }
                    } else if let Some((y, m, d)) = lui_tree::date::parse_date(iso) {
                        ctx.locale.format_date(y, m, d)
                    } else if iso.is_empty() {
                        return (None, None);
                    } else {
                        iso.to_string()
                    }
                };
                return shape_input_text(&val, false, content_rect, node, ctx);
            }
            let default_label = match inp.r#type {
                Some(InputType::Submit) => "Submit",
                Some(InputType::Reset) => "Reset",
                _ => "",
            };
            let val = inp.value.as_deref().unwrap_or(default_label);
            if val.is_empty() {
                return (None, None);
            }
            let is_pw = matches!(inp.r#type, Some(InputType::Password));
            (val.to_string(), is_pw, false)
        }
        Element::Textarea(ta) => {
            // `value` field (set by editing) takes priority over RAWTEXT children.
            let val = ta.value.as_deref().map(|v| v.to_string()).or_else(|| {
                let mut s = String::new();
                for child in &node.children {
                    if let Element::Text(t) = &child.element {
                        s.push_str(t);
                    }
                }
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            });
            let Some(val) = val else {
                return (None, None);
            };
            (val, false, true)
        }
        _ => return (None, None),
    };

    // Password masking: replace every char with bullet.
    let display_text = if is_password {
        "\u{2022}".repeat(value.chars().count())
    } else {
        value.clone()
    };

    let max_width = if wraps_multiline {
        Some(content_rect.w)
    } else {
        None
    };

    let (mut run, _w, _h, _ascent) = shape_text_run(&display_text, &node.style, max_width, false, ctx);

    // For password inputs, replace byte_boundaries with the original
    // value's char boundaries so the caret maps correctly. The shaped
    // run's boundaries correspond to the bullet string (3 bytes per
    // U+2022), but EditCursor.cursor is a byte offset into the
    // cleartext value (1 byte per ASCII char, variable for UTF-8).
    // The `text` field keeps the bullet string (no cleartext leak).
    if is_password {
        if let Some(run) = run.as_mut() {
            run.byte_boundaries = lui_text::utf8_boundaries(&value);
        }
    }

    // Single-line inputs: vertical centering. Glyphs are kept in full
    // (not truncated) — the paint pass clips to the content rect, and
    // a per-input scroll offset keeps the caret visible.
    if !wraps_multiline {
        if let Some(run) = run.as_mut() {
            vcenter_run_in_rect(run, content_rect.h);
            if matches!(
                node.style.text_align,
                Some(css_enums::TextAlign::Center)
            ) {
                let dx = (content_rect.w - run.width).max(0.0) * 0.5;
                if dx > 0.01 {
                    for g in run.glyphs.iter_mut() {
                        g.x += dx;
                    }
                }
            }
        }
    }

    let color = node
        .style
        .color
        .as_ref()
        .and_then(color::resolve_color)
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);

    (run, Some(color))
}

/// Shape the `placeholder` attribute on an empty `<input>` /
/// `<textarea>` so the field renders the hint text. Returns
/// `(None, None)` for non-form-control elements, hidden inputs,
/// fields with a non-empty value/content, or empty placeholders.
///
/// Color: the cascaded `color` with alpha multiplied by 0.5,
/// approximating the browser default `::placeholder` styling.
/// Falls back to mid-gray if `color` doesn't resolve.
pub(crate) fn compute_placeholder_run(
    node: &CascadedNode,
    content_rect: Rect,
    ctx: &mut Ctx,
) -> (Option<lui_text::ShapedRun>, Option<Color>) {
    use lui_models::common::html_enums::InputType;

    // Pull the placeholder string (if any) and the "is this a
    // wrapping multiline field?" hint up front.
    let (text, wraps_multiline) = match &node.element {
        Element::Input(inp) => {
            // A non-empty `value` overrides the placeholder.
            // (We don't render the value yet — that lands with
            // typing — but we shouldn't paint placeholder text
            // on top of a real value either.)
            if inp.value.as_deref().is_some_and(|v| !v.is_empty()) {
                return (None, None);
            }
            if matches!(
                inp.r#type,
                Some(
                    InputType::Hidden
                        | InputType::Checkbox
                        | InputType::Radio
                        | InputType::Range
                        | InputType::Color
                )
            ) {
                return (None, None);
            }
            if matches!(inp.r#type, Some(InputType::File)) {
                return (None, None);
            }
            (inp.placeholder.as_deref(), false)
        }
        Element::Textarea(ta) => {
            // `value` field (set by editing) suppresses placeholder.
            if ta.value.as_deref().is_some_and(|v| !v.is_empty()) {
                return (None, None);
            }
            // RAWTEXT children of a `<textarea>` are its content;
            // if any are present, suppress the placeholder.
            if !node.children.is_empty() {
                return (None, None);
            }
            (ta.placeholder.as_deref(), true)
        }
        _ => return (None, None),
    };
    let Some(text) = text else {
        return (None, None);
    };
    if text.is_empty() {
        return (None, None);
    }

    // For textareas, soft-wrap inside the content-box width.
    // For single-line inputs, pass `None` so the shaper produces
    // a single line at its natural width — we clip overflow after
    // the fact (see below).
    let max_width = if wraps_multiline {
        Some(content_rect.w)
    } else {
        None
    };
    let (mut run, _w, _h, _ascent) = shape_text_run(text, &node.style, max_width, false, ctx);

    // Single-line inputs:
    //   1. Truncate any trailing glyphs whose right edge crosses the content edge, so the placeholder never paints into
    //      the right padding or past the input's border. Mirrors browsers' built-in clip on `<input>` content.
    //   2. Vertically centre the run inside the content box, as browsers do for form-control line boxes.
    // Textareas skip both: long lines wrap (already handled at
    // shape time) and content flows from the top down.
    if !wraps_multiline {
        if let Some(run) = run.as_mut() {
            // 1. Horizontal clip.
            let max_x = content_rect.w;
            if max_x > 0.0 {
                let cutoff = run
                    .glyphs
                    .iter()
                    .position(|g| g.x + g.w > max_x)
                    .unwrap_or(run.glyphs.len());
                if cutoff < run.glyphs.len() {
                    run.glyphs.truncate(cutoff);
                    for line in run.lines.iter_mut() {
                        let (start, end) = line.glyph_range;
                        line.glyph_range = (start, end.min(cutoff).max(start));
                    }
                    run.width = run.glyphs.last().map(|g| g.x + g.w).unwrap_or(0.0);
                }
            }
            // 2. Vertical centering — centre within the padding box so text
            //    appears centered in the full input, not just the content area.
            vcenter_run_in_rect(run, content_rect.h);
        }
    }

    // ::placeholder style: use CSS ::placeholder color if specified,
    // otherwise fall back to cascaded `color` with alpha halved.
    let color = node
        .placeholder
        .as_ref()
        .and_then(|ps| ps.color.as_ref())
        .and_then(color::resolve_color)
        .or_else(|| {
            node.style
                .color
                .as_ref()
                .and_then(color::resolve_color)
                .map(|[r, g, b, a]| [r, g, b, a * 0.5])
        })
        .unwrap_or([0.0, 0.0, 0.0, 0.5]);

    // Override per-glyph colors to the placeholder color. Glyphs
    // carry their own color from shaping (the cascaded `color` at
    // full opacity); paint uses `g.color`, not `text_color`, so
    // without this override the dimmed placeholder alpha is ignored.
    if let Some(run) = run.as_mut() {
        for g in run.glyphs.iter_mut() {
            g.color = color;
        }
    }

    (run, Some(color))
}

// ---------------------------------------------------------------------------
// Form-control detection helpers
// ---------------------------------------------------------------------------

/// Whether `node` is a form control whose empty content box
/// should default to `line-height` tall (the browser-side
/// behaviour: an empty `<input>` doesn't collapse to 0px).
///
/// Skips `<input type="hidden">` since the UA stylesheet sets
/// `display: none` on it.
pub(crate) fn form_control_default_line_height(node: &CascadedNode) -> bool {
    use lui_models::common::html_enums::InputType;
    match &node.element {
        Element::Input(inp) => !matches!(inp.r#type, Some(InputType::Hidden)),
        Element::Textarea(_) | Element::Select(_) => true,
        Element::Button(_) => node.children.is_empty(),
        _ => false,
    }
}

pub(crate) fn has_native_appearance(node: &CascadedNode) -> bool {
    use lui_models::common::html_enums::InputType;
    matches!(
        &node.element,
        Element::Input(inp) if matches!(
            inp.r#type,
            Some(InputType::Checkbox | InputType::Radio | InputType::Range)
        )
    )
}

// ---------------------------------------------------------------------------
// Vertical centering
// ---------------------------------------------------------------------------

fn vcenter_run_in_rect(run: &mut lui_text::ShapedRun, box_h: f32) {
    if run.glyphs.is_empty() {
        return;
    }
    let line_h = run.height;
    let dy = (box_h - line_h) * 0.5;
    if dy.abs() > 0.01 {
        for g in run.glyphs.iter_mut() {
            g.y += dy;
        }
        for line in run.lines.iter_mut() {
            line.top += dy;
        }
    }
}

// ---------------------------------------------------------------------------
// FormControlInfo construction
// ---------------------------------------------------------------------------

pub(crate) fn form_control_info(node: &CascadedNode) -> Option<FormControlInfo> {
    form_control_info_from_element(&node.element)
}

pub(crate) fn form_control_info_from_element(element: &Element) -> Option<FormControlInfo> {
    use lui_models::common::html_enums::InputType;
    let inp = match element {
        Element::Input(inp) => inp,
        _ => return None,
    };
    let kind = match inp.r#type {
        Some(InputType::Checkbox) => FormControlKind::Checkbox {
            checked: inp.checked.unwrap_or(false),
        },
        Some(InputType::Radio) => FormControlKind::Radio {
            checked: inp.checked.unwrap_or(false),
        },
        Some(InputType::Range) => {
            let min: f32 = inp.min.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0);
            let max: f32 = inp.max.as_deref().and_then(|s| s.parse().ok()).unwrap_or(100.0);
            let value: f32 = inp
                .value
                .as_deref()
                .and_then(|s| s.parse().ok())
                .unwrap_or((min + max) / 2.0);
            FormControlKind::Range {
                value: value.clamp(min, max),
                min,
                max,
            }
        }
        Some(InputType::Color) => {
            let hex = inp.value.as_deref().unwrap_or("#000000");
            let srgb = color::parse_hex(hex).unwrap_or([0.0, 0.0, 0.0, 1.0]);
            FormControlKind::Color {
                r: color::srgb_to_linear(srgb[0]),
                g: color::srgb_to_linear(srgb[1]),
                b: color::srgb_to_linear(srgb[2]),
                a: srgb[3],
            }
        }
        Some(InputType::Date) => {
            let val = inp.value.as_deref().unwrap_or("");
            let (y, m, d) = lui_tree::date::parse_date(val).unwrap_or((0, 0, 0));
            FormControlKind::Date {
                year: y,
                month: m,
                day: d,
            }
        }
        Some(InputType::DatetimeLocal) => {
            let val = inp.value.as_deref().unwrap_or("");
            let (y, m, d, hour, minute) =
                lui_tree::date::parse_datetime_local(val).unwrap_or((0, 0, 0, 0, 0));
            FormControlKind::DatetimeLocal {
                year: y,
                month: m,
                day: d,
                hour,
                minute,
            }
        }
        Some(InputType::File) => FormControlKind::File {
            file_name: inp.files.first().map(|f| f.name.to_string()),
            file_count: inp.files.len(),
            disabled: inp.disabled.unwrap_or(false),
        },
        _ => return None,
    };
    Some(FormControlInfo { kind })
}
