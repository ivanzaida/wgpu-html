//! CSS color → linear RGBA.
//!
//! The render surface is sRGB; the GPU does linear→sRGB on write, so the
//! values produced here are already linearised.

use wgpu_html_models::common::css_enums::CssColor;

/// Linear RGBA in 0..1.
pub type Color = [f32; 4];

pub fn resolve_color(c: &CssColor) -> Option<Color> {
    let srgb = match c {
        CssColor::Transparent => return Some([0.0, 0.0, 0.0, 0.0]),
        CssColor::CurrentColor => return None, // not tracked yet
        CssColor::Rgb(r, g, b) => [*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, 1.0],
        CssColor::Rgba(r, g, b, a) => [*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, *a],
        CssColor::Hex(s) => parse_hex(s)?,
        CssColor::Named(name) => named_color(name)?,
        CssColor::Hsl(h, s, l) => hsl_to_rgb(*h, *s, *l, 1.0),
        CssColor::Hsla(h, s, l, a) => hsl_to_rgb(*h, *s, *l, *a),
        // Modern colour functions (`color()`, `lab()`, `oklch()`, …)
        // are parsed but not yet resolved here. Treat as
        // `None` (skip the paint) until a resolver is wired in.
        CssColor::Function(_) => return None,
    };
    Some([
        srgb_to_linear(srgb[0]),
        srgb_to_linear(srgb[1]),
        srgb_to_linear(srgb[2]),
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
        // CSS Color Module Level 4 system colors. Values follow the
        // CSS-Color-4 §17 light-mode defaults; we don't track
        // `prefers-color-scheme` yet, so dark-mode UAs would just
        // pick different RGB. Author CSS routinely overrides these,
        // so the exact values matter less than not failing the
        // cascade.
        "canvas" => (255, 255, 255),
        "canvastext" => (0, 0, 0),
        "linktext" => (0, 0, 238),
        "visitedtext" => (85, 26, 139),
        "activetext" => (255, 0, 0),
        "buttonface" => (221, 221, 221),
        "buttontext" => (0, 0, 0),
        "buttonborder" => (111, 111, 111),
        "field" => (255, 255, 255),
        "fieldtext" => (0, 0, 0),
        "highlight" => (51, 136, 255),
        "highlighttext" => (255, 255, 255),
        "selecteditem" => (51, 136, 255),
        "selecteditemtext" => (255, 255, 255),
        "mark" => (255, 255, 0),
        "marktext" => (0, 0, 0),
        "graytext" => (128, 128, 128),
        "accentcolor" => (51, 136, 255),
        "accentcolortext" => (255, 255, 255),
        _ => return None,
    };
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
}

fn hsl_to_rgb(h: f32, s: f32, l: f32, a: f32) -> [f32; 4] {
    let s = (s / 100.0).clamp(0.0, 1.0);
    let l = (l / 100.0).clamp(0.0, 1.0);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h6 = h.rem_euclid(360.0) / 60.0;
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

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
