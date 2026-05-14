use lui_core::display_list::{DisplayList, Rect as DlRect};
use lui_layout::{LayoutBox, LayoutTree, Overflow};

use super::style;

const VIEWPORT_SCROLLBAR_MARGIN: f32 = 2.0;
const VIEWPORT_THUMB_INSET: f32 = 1.0;

pub fn paint_scrollbars(b: &LayoutBox, dx: f32, dy: f32, opacity: f32, dl: &mut DisplayList) {
  let scroll = match &b.scroll {
    Some(s) => s,
    None => return,
  };

  let bar_w = scroll.scrollbar_width;
  if bar_w <= 0.0 {
    return;
  }

  let (track_color, thumb_color) = resolve_scrollbar_colors(b.style.scrollbar_color, opacity);
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

pub fn paint_viewport_scrollbars(
  tree: &LayoutTree<'_>,
  viewport_width: f32,
  viewport_height: f32,
  scroll_x: f32,
  scroll_y: f32,
  dl: &mut DisplayList,
) {
  let style_box = viewport_scrollbar_style_box(tree);
  let bar_w = resolve_scrollbar_width(style_box.style.scrollbar_width);
  if bar_w <= 0.0 || viewport_width <= 0.0 || viewport_height <= 0.0 {
    return;
  }

  let (max_x, max_y) = tree.viewport_scroll_bounds(viewport_width, viewport_height);
  if max_x <= 0.5 && max_y <= 0.5 {
    return;
  }

  let (track_color, thumb_color) = resolve_scrollbar_colors(style_box.style.scrollbar_color, 1.0);
  let show_y = max_y > 0.5;
  let show_x = max_x > 0.5;

  if show_y {
    let track_h = viewport_height - VIEWPORT_SCROLLBAR_MARGIN * 2.0 - if show_x { bar_w } else { 0.0 };
    if track_h > 0.0 {
      let track = DlRect::new(
        viewport_width - bar_w - VIEWPORT_SCROLLBAR_MARGIN,
        VIEWPORT_SCROLLBAR_MARGIN,
        bar_w,
        track_h,
      );
      dl.push_quad(track, track_color);

      let doc_h = viewport_height + max_y;
      let thumb_h = (track.h * viewport_height / doc_h).clamp(24.0, track.h);
      let travel = (track.h - thumb_h).max(0.0);
      let thumb_y = track.y + travel * (scroll_y / max_y.max(1.0));
      let thumb = DlRect::new(
        track.x + VIEWPORT_THUMB_INSET,
        thumb_y + VIEWPORT_THUMB_INSET,
        (bar_w - VIEWPORT_THUMB_INSET * 2.0).max(1.0),
        (thumb_h - VIEWPORT_THUMB_INSET * 2.0).max(1.0),
      );
      let radii = [(thumb.w * 0.5).min(4.0); 4];
      dl.push_quad_rounded(thumb, thumb_color, radii);
    }
  }

  if show_x {
    let track_w = viewport_width - VIEWPORT_SCROLLBAR_MARGIN * 2.0 - if show_y { bar_w } else { 0.0 };
    if track_w > 0.0 {
      let track = DlRect::new(
        VIEWPORT_SCROLLBAR_MARGIN,
        viewport_height - bar_w - VIEWPORT_SCROLLBAR_MARGIN,
        track_w,
        bar_w,
      );
      dl.push_quad(track, track_color);

      let doc_w = viewport_width + max_x;
      let thumb_w = (track.w * viewport_width / doc_w).clamp(24.0, track.w);
      let travel = (track.w - thumb_w).max(0.0);
      let thumb_x = track.x + travel * (scroll_x / max_x.max(1.0));
      let thumb = DlRect::new(
        thumb_x + VIEWPORT_THUMB_INSET,
        track.y + VIEWPORT_THUMB_INSET,
        (thumb_w - VIEWPORT_THUMB_INSET * 2.0).max(1.0),
        (bar_w - VIEWPORT_THUMB_INSET * 2.0).max(1.0),
      );
      let radii = [(thumb.h * 0.5).min(4.0); 4];
      dl.push_quad_rounded(thumb, thumb_color, radii);
    }
  }
}

pub(crate) fn resolve_scrollbar_colors(v: Option<&lui_core::CssValue>, opacity: f32) -> ([f32; 4], [f32; 4]) {
  let default_track = [0.95, 0.95, 0.95, opacity];
  let default_thumb = [0.7, 0.7, 0.7, 0.6 * opacity];

  let Some(v) = v else {
    return (default_track, default_thumb);
  };

  if let Some(single) = style::css_color(Some(v)) {
    return (default_track, apply_opacity(single, opacity));
  }

  let raw = match v {
    lui_core::CssValue::String(s) | lui_core::CssValue::Unknown(s) => s.as_ref(),
    _ => "",
  };

  match raw {
    "auto" | "" => (default_track, default_thumb),
    "dark" => ([0.24, 0.24, 0.24, opacity], [0.72, 0.72, 0.72, opacity]),
    "light" => ([0.92, 0.92, 0.92, opacity], [0.55, 0.55, 0.55, opacity]),
    _ => {
      let values = lui_parse::parse_values(raw).unwrap_or_default();
      let thumb = values.first().and_then(|v| style::css_color(Some(v)));
      let track = values.get(1).and_then(|v| style::css_color(Some(v)));
      match (track, thumb) {
        (Some(track), Some(thumb)) => (apply_opacity(track, opacity), apply_opacity(thumb, opacity)),
        (None, Some(thumb)) => (default_track, apply_opacity(thumb, opacity)),
        _ => (default_track, default_thumb),
      }
    }
  }
}

fn apply_opacity(mut color: [f32; 4], opacity: f32) -> [f32; 4] {
  color[3] *= opacity;
  color
}

fn resolve_scrollbar_width(v: Option<&lui_core::CssValue>) -> f32 {
  lui_core::resolve_scrollbar_width(v)
}

fn viewport_scrollbar_style_box<'a>(tree: &'a LayoutTree<'a>) -> &'a LayoutBox<'a> {
  let root = &tree.root;
  if let Some(body) = root.children.first() {
    if body.style.scrollbar_color.is_some() || body.style.scrollbar_width.is_some() {
      return body;
    }
  }
  root
}
