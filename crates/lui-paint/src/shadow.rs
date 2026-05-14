use lui_core::display_list::{DisplayList, Quad, Rect as DlRect, DisplayCommand, DisplayCommandKind};
use lui_core::CssValue;

use crate::style;

pub fn paint_box_shadows(
    val: Option<&CssValue>,
    border_rect: DlRect,
    radii_h: [f32; 4],
    radii_v: [f32; 4],
    opacity: f32,
    dl: &mut DisplayList,
) {
    let s = style::css_str(Some(val.unwrap_or(&CssValue::Number(0.0))));
    if s.is_empty() || s == "none" { return; }

    for shadow in parse_shadows(s) {
        if shadow.inset { continue; }
        let mut color = shadow.color;
        color[3] *= opacity;
        if color[3] <= 0.0 { continue; }

        let spread = shadow.spread;
        let rect = DlRect::new(
            border_rect.x + shadow.offset_x - spread - shadow.blur,
            border_rect.y + shadow.offset_y - spread - shadow.blur,
            border_rect.w + 2.0 * (spread + shadow.blur),
            border_rect.h + 2.0 * (spread + shadow.blur),
        );
        let sr = [
            (radii_h[0] + spread).max(0.0),
            (radii_h[1] + spread).max(0.0),
            (radii_h[2] + spread).max(0.0),
            (radii_h[3] + spread).max(0.0),
        ];
        let sv = [
            (radii_v[0] + spread).max(0.0),
            (radii_v[1] + spread).max(0.0),
            (radii_v[2] + spread).max(0.0),
            (radii_v[3] + spread).max(0.0),
        ];

        let index = dl.quads.len() as u32;
        dl.quads.push(Quad {
            rect,
            color,
            radii_h: sr,
            radii_v: sv,
            stroke: [0.0; 4],
            pattern: [0.0; 4],
            shadow_sigma: shadow.blur,
            transform: [1.0, 0.0, 0.0, 1.0],
            transform_origin: [rect.w * 0.5, rect.h * 0.5],
        });
        dl.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: dl.clips.len().saturating_sub(1) as u32,
        });
    }
}

struct Shadow {
    offset_x: f32,
    offset_y: f32,
    blur: f32,
    spread: f32,
    color: [f32; 4],
    inset: bool,
}

fn parse_shadows(s: &str) -> Vec<Shadow> {
    let mut result = Vec::new();
    for part in s.split(',') {
        if let Some(shadow) = parse_one_shadow(part.trim()) {
            result.push(shadow);
        }
    }
    result
}

fn parse_one_shadow(s: &str) -> Option<Shadow> {
    let mut tokens: Vec<&str> = s.split_whitespace().collect();
    let inset = tokens.iter().position(|t| *t == "inset").map(|i| { tokens.remove(i); true }).unwrap_or(false);

    let mut nums = Vec::new();
    let mut color_str = None;
    for t in &tokens {
        if let Some(v) = parse_px(t) {
            nums.push(v);
        } else {
            color_str = Some(*t);
        }
    }

    let offset_x = *nums.first().unwrap_or(&0.0);
    let offset_y = *nums.get(1).unwrap_or(&0.0);
    let blur = nums.get(2).copied().unwrap_or(0.0).max(0.0);
    let spread = nums.get(3).copied().unwrap_or(0.0);
    let color = color_str
        .and_then(|s| crate::color::parse_color_string_pub(s))
        .unwrap_or([0.0, 0.0, 0.0, 1.0]);

    Some(Shadow { offset_x, offset_y, blur, spread, color, inset })
}

fn parse_px(s: &str) -> Option<f32> {
    s.strip_suffix("px").and_then(|v| v.parse().ok())
        .or_else(|| s.parse::<f32>().ok().filter(|v| *v == 0.0))
}
