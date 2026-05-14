use lui_core::display_list::{DisplayList, Rect as DlRect};
use lui_layout::{LayoutBox, Overflow};

use super::style;

#[derive(Clone)]
pub struct ClipFrame {
  pub rect: Option<DlRect>,
  pub radii_h: [f32; 4],
  pub radii_v: [f32; 4],
}

pub fn should_clip(b: &LayoutBox) -> bool {
  matches!(
    b.overflow_x,
    Overflow::Hidden | Overflow::Scroll | Overflow::Auto | Overflow::Clip
  ) || matches!(
    b.overflow_y,
    Overflow::Hidden | Overflow::Scroll | Overflow::Auto | Overflow::Clip
  )
}

pub fn push_overflow_clip(
  b: &LayoutBox,
  dx: f32,
  dy: f32,
  clip_stack: &mut Vec<ClipFrame>,
  dl: &mut DisplayList,
) -> ClipFrame {
  let padding_rect = b.padding_rect();
  let clip_rect = DlRect::new(
    padding_rect.x + dx,
    padding_rect.y + dy,
    padding_rect.width,
    padding_rect.height,
  );

  let border_rect = b.border_rect();
  let (outer_h, outer_v) = style::border_radii(b.style, border_rect.width, border_rect.height);
  let (inner_h, inner_v) = style::padding_box_radii(outer_h, outer_v, &b.border);

  let parent = clip_stack.last().cloned().unwrap_or(ClipFrame {
    rect: None,
    radii_h: [0.0; 4],
    radii_v: [0.0; 4],
  });

  let clipped = match parent.rect {
    Some(pr) => Some(intersect(clip_rect, pr)),
    None => Some(clip_rect),
  };

  let frame = ClipFrame {
    rect: clipped,
    radii_h: inner_h,
    radii_v: inner_v,
  };
  clip_stack.push(frame.clone());
  dl.push_clip(clipped, inner_h, inner_v);
  parent
}

pub fn pop_overflow_clip(parent: &ClipFrame, clip_stack: &mut Vec<ClipFrame>, dl: &mut DisplayList) {
  clip_stack.pop();
  dl.pop_clip(parent.rect, parent.radii_h, parent.radii_v);
}

pub fn scroll_offset(b: &LayoutBox) -> (f32, f32) {
  b.scroll.map(|s| (s.scroll_x, s.scroll_y)).unwrap_or((0.0, 0.0))
}

fn intersect(a: DlRect, b: DlRect) -> DlRect {
  let x = a.x.max(b.x);
  let y = a.y.max(b.y);
  let r = (a.x + a.w).min(b.x + b.w);
  let bot = (a.y + a.h).min(b.y + b.h);
  DlRect::new(x, y, (r - x).max(0.0), (bot - y).max(0.0))
}
