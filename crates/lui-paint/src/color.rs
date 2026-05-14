use lui_core::{CssColor, CssValue};

pub fn resolve_color(val: Option<&CssValue>) -> Option<[f32; 4]> {
    match val {
        Some(CssValue::Color(c)) => css_color_to_rgba(c),
        Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => parse_color_string(s),
        _ => None,
    }
}

pub fn css_color_to_rgba(c: &CssColor) -> Option<[f32; 4]> {
    let srgb = match c {
        CssColor::Rgb(r, g, b) => [*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, 1.0],
        CssColor::Rgba(r, g, b, a) => [*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, *a as f32 / 255.0],
        CssColor::Hsl(h, s, l) => hsl_to_rgba(*h as f32, *s as f32 / 100.0, *l as f32 / 100.0, 1.0),
        CssColor::Hsla(h, s, l, a) => hsl_to_rgba(*h as f32, *s as f32 / 100.0, *l as f32 / 100.0, *a as f32 / 255.0),
        CssColor::Hwb(h, w, b) => hwb_to_rgba(*h as f32, *w as f32 / 100.0, *b as f32 / 100.0, 1.0),
        CssColor::Hwba(h, w, b, a) => hwb_to_rgba(*h as f32, *w as f32 / 100.0, *b as f32 / 100.0, *a as f32 / 255.0),
        CssColor::Hex(s) => return parse_hex(s),
        CssColor::Named(s) => return named_color(s),
    };
    Some(srgb_to_linear(srgb))
}

pub(crate) fn parse_color_string_pub(s: &str) -> Option<[f32; 4]> {
    parse_color_string(s)
}

fn parse_color_string(s: &str) -> Option<[f32; 4]> {
    let s = s.trim();
    if s == "transparent" { return Some([0.0; 4]); }
    if s == "currentcolor" || s == "currentColor" { return None; }
    if s.starts_with('#') { return parse_hex(s); }
    named_color(s)
}

fn srgb_component_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_to_linear(c: [f32; 4]) -> [f32; 4] {
    [
        srgb_component_to_linear(c[0]),
        srgb_component_to_linear(c[1]),
        srgb_component_to_linear(c[2]),
        c[3],
    ]
}

fn parse_hex(s: &str) -> Option<[f32; 4]> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    let srgb = match hex.len() {
        3 => {
            let bytes: Vec<u8> = hex.chars().map(|c| { let v = hex_digit(c)?; Some(v * 17) }).collect::<Option<Vec<_>>>()?;
            [bytes[0] as f32 / 255.0, bytes[1] as f32 / 255.0, bytes[2] as f32 / 255.0, 1.0]
        }
        4 => {
            let cs: Vec<char> = hex.chars().collect();
            let r = hex_digit(cs[0])? * 17;
            let g = hex_digit(cs[1])? * 17;
            let b = hex_digit(cs[2])? * 17;
            let a = hex_digit(cs[3])? * 17;
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0]
        }
        _ => return None,
    };
    Some(srgb_to_linear(srgb))
}

fn hex_digit(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

fn hsl_to_rgba(h: f32, s: f32, l: f32, a: f32) -> [f32; 4] {
    let h = ((h % 360.0) + 360.0) % 360.0 / 360.0;
    let (r, g, b) = if s == 0.0 {
        (l, l, l)
    } else {
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        (hue_to_rgb(p, q, h + 1.0 / 3.0), hue_to_rgb(p, q, h), hue_to_rgb(p, q, h - 1.0 / 3.0))
    };
    [r, g, b, a]
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

fn hwb_to_rgba(h: f32, w: f32, b: f32, a: f32) -> [f32; 4] {
    let sum = w + b;
    let (w, b) = if sum > 1.0 { (w / sum, b / sum) } else { (w, b) };
    let [r, g, bl, _] = hsl_to_rgba(h, 1.0, 0.5, 1.0);
    let f = |c: f32| c * (1.0 - w - b) + w;
    [f(r), f(g), f(bl), a]
}

pub fn named_color(name: &str) -> Option<[f32; 4]> {
    let rgb = match name.to_ascii_lowercase().as_str() {
        "black" => [0, 0, 0],
        "white" => [255, 255, 255],
        "red" => [255, 0, 0],
        "green" | "lime" => [0, 128, 0],
        "blue" => [0, 0, 255],
        "yellow" => [255, 255, 0],
        "cyan" | "aqua" => [0, 255, 255],
        "magenta" | "fuchsia" => [255, 0, 255],
        "gray" | "grey" => [128, 128, 128],
        "silver" => [192, 192, 192],
        "maroon" => [128, 0, 0],
        "olive" => [128, 128, 0],
        "navy" => [0, 0, 128],
        "teal" => [0, 128, 128],
        "purple" => [128, 0, 128],
        "orange" => [255, 165, 0],
        "pink" => [255, 192, 203],
        "brown" => [165, 42, 42],
        "coral" => [255, 127, 80],
        "crimson" => [220, 20, 60],
        "gold" => [255, 215, 0],
        "indigo" => [75, 0, 130],
        "ivory" => [255, 255, 240],
        "khaki" => [240, 230, 140],
        "lavender" => [230, 230, 250],
        "lightblue" => [173, 216, 230],
        "lightgray" | "lightgrey" => [211, 211, 211],
        "lightgreen" => [144, 238, 144],
        "lightyellow" => [255, 255, 224],
        "darkblue" => [0, 0, 139],
        "darkgray" | "darkgrey" => [169, 169, 169],
        "darkgreen" => [0, 100, 0],
        "darkred" => [139, 0, 0],
        "tomato" => [255, 99, 71],
        "skyblue" => [135, 206, 235],
        "steelblue" => [70, 130, 180],
        "wheat" => [245, 222, 179],
        "whitesmoke" => [245, 245, 245],
        "transparent" => return Some([0.0; 4]),
        _ => return None,
    };
    Some(srgb_to_linear([rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0]))
}
