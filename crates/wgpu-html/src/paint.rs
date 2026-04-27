//! M3: walk a `Tree` and emit a `DisplayList` of solid quads.
//!
//! No real layout yet — every element is painted at the absolute position
//! given by its inline `style` attribute (`top`, `left`, `width`, `height`,
//! `background-color`). Boxes nest: `top` and `left` are interpreted
//! relative to the parent box. Elements without a resolvable size or
//! background are skipped (their children still recurse).

use wgpu_html_models::Style;
use wgpu_html_models::common::css_enums::{CssColor, CssLength};
use wgpu_html_parser::parse_inline_style;
use wgpu_html_renderer::{Color, DisplayList, Rect};
use wgpu_html_tree::{Element, Node, Tree};

/// Build a display list for `tree` against a `viewport_w × viewport_h` viewport.
pub fn paint_tree(tree: &Tree, viewport_w: f32, viewport_h: f32) -> DisplayList {
    let mut list = DisplayList::new();
    if let Some(root) = &tree.root {
        let root_rect = Rect::new(0.0, 0.0, viewport_w, viewport_h);
        paint_node(root, root_rect, viewport_w, viewport_h, &mut list);
    }
    list
}

fn paint_node(node: &Node, parent: Rect, vw: f32, vh: f32, out: &mut DisplayList) {
    let style = element_style(&node.element);

    // Resolve the box rectangle. Defaults: position at parent origin,
    // size = parent size.
    let left = style
        .as_ref()
        .and_then(|s| resolve_len(&s.left, parent.w, vw, vh))
        .unwrap_or(0.0);
    let top = style
        .as_ref()
        .and_then(|s| resolve_len(&s.top, parent.h, vw, vh))
        .unwrap_or(0.0);
    let width = style
        .as_ref()
        .and_then(|s| resolve_len(&s.width, parent.w, vw, vh))
        .unwrap_or(parent.w);
    let height = style
        .as_ref()
        .and_then(|s| resolve_len(&s.height, parent.h, vw, vh))
        .unwrap_or(parent.h);

    let rect = Rect::new(parent.x + left, parent.y + top, width, height);

    // Emit a quad if this element has a background color.
    if let Some(color) = style
        .as_ref()
        .and_then(|s| s.background_color.as_ref())
        .and_then(resolve_color)
    {
        if width > 0.0 && height > 0.0 {
            out.push_quad(rect, color);
        }
    }

    for child in &node.children {
        paint_node(child, rect, vw, vh, out);
    }
}

/// Parse the inline `style` attribute of an element into a `Style`.
/// Returns `None` for `Element::Text` and for elements without a `style` attr.
fn element_style(element: &Element) -> Option<Style> {
    let raw = element_style_attr(element)?;
    Some(parse_inline_style(raw))
}

/// One arm per element variant. Text has no style attribute.
fn element_style_attr(el: &Element) -> Option<&str> {
    macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                $(Element::$v(e) => e.style.as_deref(),)*
            }
        };
    }
    arms!(
        Html, Head, Body, Title, Meta, Link, StyleElement, Script, Noscript,
        H1, H2, H3, H4, H5, H6, P, Br, Hr, Pre, Blockquote, Address,
        Span, A, Strong, B, Em, I, U, S, Small, Mark, Code, Kbd, Samp, Var,
        Abbr, Cite, Dfn, Sub, Sup, Time,
        Ul, Ol, Li, Dl, Dt, Dd,
        Header, Nav, Main, Section, Article, Aside, Footer, Div,
        Img, Picture, Source, Video, Audio, Track, Iframe, Canvas, Svg,
        Table, Caption, Thead, Tbody, Tfoot, Tr, Th, Td, Colgroup, Col,
        Form, Label, Input, Textarea, Button, Select, OptionElement, Optgroup,
        Fieldset, Legend, Datalist, Output, Progress, Meter,
        Details, Summary, Dialog, Template, Slot,
        Del, Ins, Bdi, Bdo, Wbr, Data, Ruby, Rt, Rp,
    )
}

// ---------------------------------------------------------------------------
// CSS length resolution
// ---------------------------------------------------------------------------

const DEFAULT_FONT_PX: f32 = 16.0;

/// Resolve a CSS length to physical pixels.
///
/// - `Px` is taken verbatim.
/// - `Percent` is resolved against `parent_size_px`.
/// - `Vw` / `Vh` / `Vmin` / `Vmax` against the viewport.
/// - `Em` / `Rem` against `DEFAULT_FONT_PX` (real font metrics come later).
/// - `Auto` and `Raw(_)` return `None` (the caller picks a default).
fn resolve_len(len: &Option<CssLength>, parent_size_px: f32, vw: f32, vh: f32) -> Option<f32> {
    match len.as_ref()? {
        CssLength::Px(v) => Some(*v),
        CssLength::Percent(v) => Some(parent_size_px * v / 100.0),
        CssLength::Em(v) | CssLength::Rem(v) => Some(*v * DEFAULT_FONT_PX),
        CssLength::Vw(v) => Some(vw * v / 100.0),
        CssLength::Vh(v) => Some(vh * v / 100.0),
        CssLength::Vmin(v) => Some(vw.min(vh) * v / 100.0),
        CssLength::Vmax(v) => Some(vw.max(vh) * v / 100.0),
        CssLength::Zero => Some(0.0),
        CssLength::Auto | CssLength::Raw(_) => None,
    }
}

// ---------------------------------------------------------------------------
// CSS color resolution → linear RGBA
// ---------------------------------------------------------------------------

fn resolve_color(c: &CssColor) -> Option<Color> {
    let srgb = match c {
        CssColor::Transparent => return Some([0.0, 0.0, 0.0, 0.0]),
        CssColor::CurrentColor => return None, // not tracked yet
        CssColor::Rgb(r, g, b) => [
            *r as f32 / 255.0,
            *g as f32 / 255.0,
            *b as f32 / 255.0,
            1.0,
        ],
        CssColor::Rgba(r, g, b, a) => [
            *r as f32 / 255.0,
            *g as f32 / 255.0,
            *b as f32 / 255.0,
            *a,
        ],
        CssColor::Hex(s) => parse_hex(s)?,
        CssColor::Named(name) => named_color(name)?,
        CssColor::Hsl(h, s, l) => hsl_to_rgb(*h, *s, *l, 1.0),
        CssColor::Hsla(h, s, l, a) => hsl_to_rgb(*h, *s, *l, *a),
    };
    Some([
        srgb_channel_to_linear(srgb[0]),
        srgb_channel_to_linear(srgb[1]),
        srgb_channel_to_linear(srgb[2]),
        srgb[3],
    ])
}

/// `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` → sRGB-encoded RGBA in 0..1.
fn parse_hex(s: &str) -> Option<[f32; 4]> {
    let s = s.strip_prefix('#').unwrap_or(s);
    let to_u8 = |hi: u8, lo: u8| -> Option<u8> {
        let h = (hi as char).to_digit(16)?;
        let l = (lo as char).to_digit(16)?;
        Some((h * 16 + l) as u8)
    };
    let bytes = s.as_bytes();
    let (r, g, b, a) = match bytes.len() {
        3 => {
            let r = to_u8(bytes[0], bytes[0])?;
            let g = to_u8(bytes[1], bytes[1])?;
            let b = to_u8(bytes[2], bytes[2])?;
            (r, g, b, 255)
        }
        4 => {
            let r = to_u8(bytes[0], bytes[0])?;
            let g = to_u8(bytes[1], bytes[1])?;
            let b = to_u8(bytes[2], bytes[2])?;
            let a = to_u8(bytes[3], bytes[3])?;
            (r, g, b, a)
        }
        6 => {
            let r = to_u8(bytes[0], bytes[1])?;
            let g = to_u8(bytes[2], bytes[3])?;
            let b = to_u8(bytes[4], bytes[5])?;
            (r, g, b, 255)
        }
        8 => {
            let r = to_u8(bytes[0], bytes[1])?;
            let g = to_u8(bytes[2], bytes[3])?;
            let b = to_u8(bytes[4], bytes[5])?;
            let a = to_u8(bytes[6], bytes[7])?;
            (r, g, b, a)
        }
        _ => return None,
    };
    Some([
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ])
}

/// Subset of CSS named colors. Returns sRGB-encoded RGBA in 0..1.
fn named_color(name: &str) -> Option<[f32; 4]> {
    let n = name.to_ascii_lowercase();
    let (r, g, b) = match n.as_str() {
        "black" => (0, 0, 0),
        "white" => (255, 255, 255),
        "red" => (255, 0, 0),
        "green" => (0, 128, 0),
        "blue" => (0, 0, 255),
        "yellow" => (255, 255, 0),
        "cyan" | "aqua" => (0, 255, 255),
        "magenta" | "fuchsia" => (255, 0, 255),
        "gray" | "grey" => (128, 128, 128),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "silver" => (192, 192, 192),
        "maroon" => (128, 0, 0),
        "olive" => (128, 128, 0),
        "lime" => (0, 255, 0),
        "teal" => (0, 128, 128),
        "navy" => (0, 0, 128),
        "purple" => (128, 0, 128),
        "orange" => (255, 165, 0),
        "pink" => (255, 192, 203),
        "transparent" => return Some([0.0, 0.0, 0.0, 0.0]),
        _ => return None,
    };
    Some([
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        1.0,
    ])
}

/// HSL → sRGB-encoded RGBA in 0..1. `h` in degrees, `s`/`l` in 0..100.
fn hsl_to_rgb(h: f32, s: f32, l: f32, a: f32) -> [f32; 4] {
    let s = (s / 100.0).clamp(0.0, 1.0);
    let l = (l / 100.0).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h6 = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h6 as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    [r1 + m, g1 + m, b1 + m, a]
}

/// sRGB component (0..1) → linear (0..1). The surface is sRGB so the GPU
/// does linear→sRGB on write; we encode our colors in linear here.
fn srgb_channel_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_short_form() {
        let c = parse_hex("#f0a").unwrap();
        assert_eq!(c, [1.0, 0.0, 170.0 / 255.0, 1.0]);
    }

    #[test]
    fn hex_long_form_with_alpha() {
        let c = parse_hex("#ff8000aa").unwrap();
        assert_eq!(c[0], 1.0);
        assert!((c[3] - 170.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn named_red() {
        let c = named_color("red").unwrap();
        assert_eq!(c, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn px_resolves_directly() {
        let v = resolve_len(&Some(CssLength::Px(42.0)), 100.0, 800.0, 600.0).unwrap();
        assert_eq!(v, 42.0);
    }

    #[test]
    fn percent_resolves_against_parent() {
        let v = resolve_len(&Some(CssLength::Percent(50.0)), 200.0, 800.0, 600.0).unwrap();
        assert_eq!(v, 100.0);
    }

    #[test]
    fn auto_returns_none() {
        assert!(resolve_len(&Some(CssLength::Auto), 100.0, 800.0, 600.0).is_none());
        assert!(resolve_len(&None, 100.0, 800.0, 600.0).is_none());
    }

    #[test]
    fn paint_emits_quad_for_styled_div() {
        let html = r#"<div style="width: 100px; height: 50px; background-color: red;"></div>"#;
        let tree = wgpu_html_parser::parse(html);
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 1);
        let q = list.quads[0];
        assert_eq!(q.rect.w, 100.0);
        assert_eq!(q.rect.h, 50.0);
        // sRGB red → linear: r ≈ 1.0, g = 0, b = 0
        assert!((q.color[0] - 1.0).abs() < 1e-6);
        assert_eq!(q.color[1], 0.0);
        assert_eq!(q.color[2], 0.0);
    }

    #[test]
    fn paint_skips_elements_without_background() {
        let tree = wgpu_html_parser::parse("<div><p>hi</p></div>");
        let list = paint_tree(&tree, 800.0, 600.0);
        assert!(list.quads.is_empty());
    }

    #[test]
    fn paint_nests_positions() {
        let html = r#"<div style="width: 200px; height: 200px; background-color: blue;">
            <div style="left: 10px; top: 20px; width: 50px; height: 50px; background-color: red;"></div>
        </div>"#;
        let tree = wgpu_html_parser::parse(html);
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 2);
        let inner = list.quads[1];
        assert_eq!(inner.rect.x, 10.0);
        assert_eq!(inner.rect.y, 20.0);
    }
}
