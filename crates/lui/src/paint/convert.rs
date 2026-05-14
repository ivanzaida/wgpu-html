#![allow(dead_code)]

use lui_core::display_list::Rect as DlRect;

pub(crate) fn to_dl_rect(r: lui_core::Rect) -> DlRect {
    DlRect::new(r.x, r.y, r.width, r.height)
}

pub(crate) fn to_dl_rect_offset(r: lui_core::Rect, dx: f32, dy: f32) -> DlRect {
    DlRect::new(r.x + dx, r.y + dy, r.width, r.height)
}
