use lui_core::display_list::{DisplayList, Rect as DlRect};
use lui_layout::LayoutBox;

use super::style;

pub fn paint_background(
    b: &LayoutBox,
    border_rect: DlRect,
    radii_h: [f32; 4],
    radii_v: [f32; 4],
    opacity: f32,
    dl: &mut DisplayList,
) {
    let color = match style::css_color(b.style.background_color) {
        Some(mut c) => { c[3] *= opacity; c }
        None => return,
    };
    if color[3] <= 0.0 { return; }

    let is_rounded = radii_h.iter().any(|r| *r > 0.0);
    if is_rounded {
        dl.push_quad_rounded_ellipse(border_rect, color, radii_h, radii_v);
    } else {
        dl.push_quad(border_rect, color);
    }
}
