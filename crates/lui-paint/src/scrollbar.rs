use lui_cascade::ComputedStyle;
use lui_core::display_list::{DisplayList, Rect as DlRect};
use lui_core::{CssUnit, CssValue, resolve_scrollbar_inset, resolve_scrollbar_min_thumb_size, resolve_scrollbar_width};
use lui_layout::{LayoutBox, Overflow};

use crate::color::resolve_color;
use crate::style::css_str;

struct ResolvedPart {
  bg: [f32; 4],
  radii: [f32; 4],
  opacity: f32,
}

fn resolve_part(style: &ComputedStyle, default_bg: [f32; 4]) -> ResolvedPart {
  let bg = resolve_color(style.background_color).unwrap_or(default_bg);
  let opacity = match style.opacity {
    Some(CssValue::Number(n)) => (*n as f32).clamp(0.0, 1.0),
    _ => 1.0,
  };
  let resolve_radius = |v: Option<&CssValue>| -> f32 {
    match v {
      Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
      Some(CssValue::Number(n)) => *n as f32,
      Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => {
        s.as_ref().strip_suffix("px").unwrap_or(s.as_ref()).parse::<f32>().unwrap_or(0.0)
      }
      _ => 0.0,
    }
  };
  let radii = [
    resolve_radius(style.border_top_left_radius),
    resolve_radius(style.border_top_right_radius),
    resolve_radius(style.border_bottom_right_radius),
    resolve_radius(style.border_bottom_left_radius),
  ];
  ResolvedPart { bg, radii, opacity }
}

fn resolve_width(style: &ComputedStyle) -> f32 {
  match style.scrollbar_width {
    Some(CssValue::Dimension { value, .. }) => *value as f32,
    _ => resolve_scrollbar_width(style.scrollbar_width),
  }
}

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

  let default_track_bg = [0.95, 0.95, 0.95, 1.0];
  let default_thumb_bg = [0.7, 0.7, 0.7, 0.6];

  let (track, thumb, corner, inset, min_thumb) = if let Some(ps) = &b.scrollbar_pseudo {
    let sb_width_override = resolve_width(&ps.scrollbar);
    let _ = sb_width_override;
    let inset = resolve_scrollbar_inset(ps.scrollbar.scrollbar_inset);
    let min_thumb = resolve_scrollbar_min_thumb_size(ps.scrollbar.scrollbar_min_thumb_size);
    let track = resolve_part(&ps.track, default_track_bg);
    let thumb = resolve_part(&ps.thumb, default_thumb_bg);
    let corner = resolve_part(&ps.corner, default_track_bg);
    (track, thumb, corner, inset, min_thumb)
  } else {
    let track = ResolvedPart { bg: default_track_bg, radii: [0.0; 4], opacity: 1.0 };
    let thumb = ResolvedPart {
      bg: default_thumb_bg,
      radii: [(bar_w * 0.5).min(4.0); 4],
      opacity: 1.0,
    };
    let corner = ResolvedPart { bg: default_track_bg, radii: [0.0; 4], opacity: 1.0 };
    (track, thumb, corner, [0.0; 4], 20.0_f32)
  };

  let pad_rect = b.padding_rect();
  let px = pad_rect.x + dx;
  let py = pad_rect.y + dy;
  let pw = pad_rect.width;
  let ph = pad_rect.height;

  let has_v = matches!(b.overflow_y, Overflow::Scroll | Overflow::Auto) && scroll.scroll_height > b.content.height;
  let has_h = matches!(b.overflow_x, Overflow::Scroll | Overflow::Auto) && scroll.scroll_width > b.content.width;

  if has_v {
    let track_x = px + pw - bar_w + inset[1];
    let track_y = py + inset[0];
    let track_w = bar_w - inset[1] - inset[3];
    let track_h = ph - inset[0] - inset[2] - if has_h { bar_w } else { 0.0 };

    let track_color = apply_opacity(track.bg, track.opacity * opacity);
    if track.radii == [0.0; 4] {
      dl.push_quad(DlRect::new(track_x, track_y, track_w, track_h), track_color);
    } else {
      dl.push_quad_rounded(DlRect::new(track_x, track_y, track_w, track_h), track_color, track.radii);
    }

    let ratio = b.content.height / scroll.scroll_height;
    let thumb_h = (track_h * ratio).max(min_thumb).min(track_h);
    let max_scroll = scroll.scroll_height - b.content.height;
    let thumb_y = if max_scroll > 0.0 {
      track_y + (track_h - thumb_h) * (scroll.scroll_y / max_scroll)
    } else {
      track_y
    };
    let thumb_color = apply_opacity(thumb.bg, thumb.opacity * opacity);
    let thumb_radii = if thumb.radii == [0.0; 4] {
      [(track_w * 0.5).min(4.0); 4]
    } else {
      thumb.radii
    };
    dl.push_quad_rounded(DlRect::new(track_x, thumb_y, track_w, thumb_h), thumb_color, thumb_radii);
  }

  if has_h {
    let track_x = px + inset[3];
    let track_y = py + ph - bar_w + inset[2];
    let track_w = pw - inset[1] - inset[3] - if has_v { bar_w } else { 0.0 };
    let track_h = bar_w - inset[0] - inset[2];

    let track_color = apply_opacity(track.bg, track.opacity * opacity);
    if track.radii == [0.0; 4] {
      dl.push_quad(DlRect::new(track_x, track_y, track_w, track_h), track_color);
    } else {
      dl.push_quad_rounded(DlRect::new(track_x, track_y, track_w, track_h), track_color, track.radii);
    }

    let ratio = b.content.width / scroll.scroll_width;
    let thumb_w = (track_w * ratio).max(min_thumb).min(track_w);
    let max_scroll = scroll.scroll_width - b.content.width;
    let thumb_x = if max_scroll > 0.0 {
      track_x + (track_w - thumb_w) * (scroll.scroll_x / max_scroll)
    } else {
      track_x
    };
    let thumb_color = apply_opacity(thumb.bg, thumb.opacity * opacity);
    let thumb_radii = if thumb.radii == [0.0; 4] {
      [(track_h * 0.5).min(4.0); 4]
    } else {
      thumb.radii
    };
    dl.push_quad_rounded(DlRect::new(thumb_x, track_y, thumb_w, track_h), thumb_color, thumb_radii);
  }

  if has_v && has_h {
    let cx = px + pw - bar_w + inset[1];
    let cy = py + ph - bar_w + inset[2];
    let cw = bar_w - inset[1] - inset[3];
    let ch = bar_w - inset[0] - inset[2];
    let corner_color = apply_opacity(corner.bg, corner.opacity * opacity);
    dl.push_quad(DlRect::new(cx, cy, cw, ch), corner_color);
  }
}

fn apply_opacity(color: [f32; 4], opacity: f32) -> [f32; 4] {
  [color[0], color[1], color[2], color[3] * opacity]
}
