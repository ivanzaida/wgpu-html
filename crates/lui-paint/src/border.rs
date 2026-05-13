use lui_display_list::{DisplayList, Rect as DlRect};
use lui_layout::LayoutBox;

use crate::style;

pub fn paint_borders(
    b: &LayoutBox,
    border_rect: DlRect,
    radii_h: [f32; 4],
    radii_v: [f32; 4],
    opacity: f32,
    dl: &mut DisplayList,
) {
    let bw = b.border;
    if bw.top == 0.0 && bw.right == 0.0 && bw.bottom == 0.0 && bw.left == 0.0 {
        return;
    }

    let top_color = style::css_color(b.style.border_top_color);
    let right_color = style::css_color(b.style.border_right_color);
    let bottom_color = style::css_color(b.style.border_bottom_color);
    let left_color = style::css_color(b.style.border_left_color);

    let top_style = style::css_str(b.style.border_top_style);
    let right_style = style::css_str(b.style.border_right_style);
    let bottom_style = style::css_str(b.style.border_bottom_style);
    let left_style = style::css_str(b.style.border_left_style);

    if top_style == "none" && right_style == "none" && bottom_style == "none" && left_style == "none" {
        return;
    }

    let uniform_color = top_color == right_color && right_color == bottom_color && bottom_color == left_color;
    let all_solid = matches!((top_style, right_style, bottom_style, left_style), ("solid"|"", "solid"|"", "solid"|"", "solid"|""));

    if uniform_color && all_solid {
        if let Some(mut color) = top_color {
            color[3] *= opacity;
            if color[3] <= 0.0 { return; }
            let stroke = [bw.top, bw.right, bw.bottom, bw.left];
            dl.push_quad_stroke_ellipse(border_rect, color, radii_h, radii_v, stroke);
        }
        return;
    }

    paint_edge(dl, border_rect, bw.top, top_color, top_style, opacity, Edge::Top);
    paint_edge(dl, border_rect, bw.right, right_color, right_style, opacity, Edge::Right);
    paint_edge(dl, border_rect, bw.bottom, bottom_color, bottom_style, opacity, Edge::Bottom);
    paint_edge(dl, border_rect, bw.left, left_color, left_style, opacity, Edge::Left);
}

enum Edge { Top, Right, Bottom, Left }

fn paint_edge(
    dl: &mut DisplayList,
    border_rect: DlRect,
    thickness: f32,
    color: Option<[f32; 4]>,
    border_style: &str,
    opacity: f32,
    edge: Edge,
) {
    if thickness <= 0.0 { return; }
    if border_style == "none" || border_style == "hidden" { return; }
    let mut color = match color {
        Some(c) => c,
        None => return,
    };
    color[3] *= opacity;
    if color[3] <= 0.0 { return; }

    let rect = match edge {
        Edge::Top => DlRect::new(border_rect.x, border_rect.y, border_rect.w, thickness),
        Edge::Right => DlRect::new(border_rect.x + border_rect.w - thickness, border_rect.y, thickness, border_rect.h),
        Edge::Bottom => DlRect::new(border_rect.x, border_rect.y + border_rect.h - thickness, border_rect.w, thickness),
        Edge::Left => DlRect::new(border_rect.x, border_rect.y, thickness, border_rect.h),
    };
    dl.push_quad(rect, color);
}
