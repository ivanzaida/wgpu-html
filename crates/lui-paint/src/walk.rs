use lui_display_list::{DisplayList, Rect as DlRect};
use lui_glyph::TextContext;
use lui_layout::LayoutBox;

use crate::{background, border, clip, convert, scrollbar, shadow, style, text};

pub fn paint_box(
    b: &LayoutBox,
    dl: &mut DisplayList,
    clip_stack: &mut Vec<clip::ClipFrame>,
    text_ctx: &mut TextContext,
    paint_offset_x: f32,
    paint_offset_y: f32,
    parent_opacity: f32,
) {
    if !style::is_visible(b.style) { return; }

    let opacity = parent_opacity * style::css_opacity(b.style);
    if opacity <= 0.0 { return; }

    let border_rect = {
        let br = b.border_rect();
        DlRect::new(br.x + paint_offset_x, br.y + paint_offset_y, br.width, br.height)
    };
    let (radii_h, radii_v) = style::border_radii(b.style, border_rect.w, border_rect.h);

    shadow::paint_box_shadows(b.style.box_shadow, border_rect, radii_h, radii_v, opacity, dl);
    background::paint_background(b, border_rect, radii_h, radii_v, opacity, dl);
    border::paint_borders(b, border_rect, radii_h, radii_v, opacity, dl);

    if b.node.element.is_text() {
        text::paint_text(
            b,
            b.content.x + paint_offset_x,
            b.content.y + paint_offset_y,
            opacity,
            text_ctx,
            dl,
        );
    }

    let font_size = style::css_f32(b.style.font_size).max(1.0);
    let line_height = font_size * 1.2;
    text::paint_text_decoration(
        b,
        b.content.x + paint_offset_x,
        b.content.y + paint_offset_y,
        b.content.width,
        line_height,
        opacity,
        dl,
    );

    text::paint_list_marker(
        b,
        b.content.x + paint_offset_x,
        b.content.y + paint_offset_y,
        opacity,
        text_ctx,
        dl,
    );

    let clipped = clip::should_clip(b);
    let parent_clip = if clipped {
        Some(clip::push_overflow_clip(b, paint_offset_x, paint_offset_y, clip_stack, dl))
    } else {
        None
    };

    let (scroll_x, scroll_y) = clip::scroll_offset(b);
    let child_dx = paint_offset_x - scroll_x;
    let child_dy = paint_offset_y - scroll_y;

    let mut child_order: Vec<usize> = (0..b.children.len()).collect();
    child_order.sort_by_key(|&i| z_index_sort_key(&b.children[i]));

    for &i in &child_order {
        paint_box(&b.children[i], dl, clip_stack, text_ctx, child_dx, child_dy, opacity);
    }

    if let Some(ref parent) = parent_clip {
        clip::pop_overflow_clip(parent, clip_stack, dl);
    }

    scrollbar::paint_scrollbars(b, paint_offset_x, paint_offset_y, opacity, dl);
}

fn z_index_sort_key(b: &LayoutBox) -> (i32, i32) {
    match b.z_index {
        Some(z) if z < 0 => (-1, z),
        Some(z) => (1, z),
        None => (0, 0),
    }
}
