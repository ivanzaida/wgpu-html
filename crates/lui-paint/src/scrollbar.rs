use lui_display_list::{DisplayList, Rect as DlRect};
use lui_layout::{LayoutBox, Overflow};

pub fn paint_scrollbars(
    b: &LayoutBox,
    dx: f32,
    dy: f32,
    opacity: f32,
    dl: &mut DisplayList,
) {
    let scroll = match &b.scroll {
        Some(s) => s,
        None => return,
    };

    let bar_w = scroll.scrollbar_width;
    if bar_w <= 0.0 { return; }

    let track_color = [0.95, 0.95, 0.95, opacity];
    let thumb_color = [0.7, 0.7, 0.7, 0.6 * opacity];
    let thumb_radius = (bar_w * 0.5).min(4.0);

    let pad_rect = b.padding_rect();
    let px = pad_rect.x + dx;
    let py = pad_rect.y + dy;
    let pw = pad_rect.width;
    let ph = pad_rect.height;

    if matches!(b.overflow_y, Overflow::Scroll | Overflow::Auto) && scroll.scroll_height > b.content.height {
        let track = DlRect::new(px + pw - bar_w, py, bar_w, ph);
        dl.push_quad(track, track_color);

        let ratio = b.content.height / scroll.scroll_height;
        let thumb_h = (ph * ratio).max(20.0).min(ph);
        let max_scroll = scroll.scroll_height - b.content.height;
        let thumb_y = if max_scroll > 0.0 {
            py + (ph - thumb_h) * (scroll.scroll_y / max_scroll)
        } else {
            py
        };
        let thumb = DlRect::new(px + pw - bar_w, thumb_y, bar_w, thumb_h);
        let radii = [thumb_radius; 4];
        dl.push_quad_rounded(thumb, thumb_color, radii);
    }

    if matches!(b.overflow_x, Overflow::Scroll | Overflow::Auto) && scroll.scroll_width > b.content.width {
        let track = DlRect::new(px, py + ph - bar_w, pw, bar_w);
        dl.push_quad(track, track_color);

        let ratio = b.content.width / scroll.scroll_width;
        let thumb_w = (pw * ratio).max(20.0).min(pw);
        let max_scroll = scroll.scroll_width - b.content.width;
        let thumb_x = if max_scroll > 0.0 {
            px + (pw - thumb_w) * (scroll.scroll_x / max_scroll)
        } else {
            px
        };
        let thumb = DlRect::new(thumb_x, py + ph - bar_w, thumb_w, bar_w);
        let radii = [thumb_radius; 4];
        dl.push_quad_rounded(thumb, thumb_color, radii);
    }
}
