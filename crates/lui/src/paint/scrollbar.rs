use lui_core::{
  CssUnit, CssValue,
  display_list::{DisplayList, Rect as DlRect},
  resolve_scrollbar_inset, resolve_scrollbar_min_thumb_size,
};
use lui_layout::{LayoutBox, LayoutTree, Overflow};

use super::style;

const VIEWPORT_SCROLLBAR_MARGIN: f32 = 2.0;
const VIEWPORT_THUMB_INSET: f32 = 1.0;

struct ResolvedPart {
  bg: [f32; 4],
  radii: [f32; 4],
  opacity: f32,
}

fn resolve_part(ps_style: &lui_cascade::ComputedStyle, default_bg: [f32; 4]) -> ResolvedPart {
  let bg = style::css_color(ps_style.background_color).unwrap_or(default_bg);
  let opacity = match ps_style.opacity {
    Some(CssValue::Number(n)) => (*n as f32).clamp(0.0, 1.0),
    _ => 1.0,
  };
  let resolve_radius = |v: Option<&CssValue>| -> f32 {
    match v {
      Some(CssValue::Dimension {
        value,
        unit: CssUnit::Px,
      }) => *value as f32,
      Some(CssValue::Number(n)) => *n as f32,
      Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s
        .as_ref()
        .strip_suffix("px")
        .unwrap_or(s.as_ref())
        .parse::<f32>()
        .unwrap_or(0.0),
      _ => 0.0,
    }
  };
  let radii = [
    resolve_radius(ps_style.border_top_left_radius),
    resolve_radius(ps_style.border_top_right_radius),
    resolve_radius(ps_style.border_bottom_right_radius),
    resolve_radius(ps_style.border_bottom_left_radius),
  ];
  ResolvedPart { bg, radii, opacity }
}

pub fn paint_scrollbars(b: &LayoutBox, dx: f32, dy: f32, opacity: f32, dl: &mut DisplayList) {
  let scroll = match &b.scroll {
    Some(s) => s,
    None => return,
  };

  let bar_w = scroll.scrollbar_width;
  if bar_w <= 0.0 {
    return;
  }

  if let Some(ps) = &b.scrollbar_pseudo {
    paint_scrollbars_styled(b, scroll, ps, dx, dy, opacity, bar_w, dl);
  } else {
    let (track_color, thumb_color) = resolve_scrollbar_colors(b.style.scrollbar_color, opacity);
    paint_scrollbars_legacy(b, scroll, dx, dy, bar_w, track_color, thumb_color, dl);
  }
}

fn paint_scrollbars_styled(
  b: &LayoutBox,
  scroll: &lui_layout::ScrollInfo,
  ps: &lui_cascade::ScrollbarPseudoStyles,
  dx: f32,
  dy: f32,
  opacity: f32,
  bar_w: f32,
  dl: &mut DisplayList,
) {
  let default_track_bg = [0.95, 0.95, 0.95, 1.0];
  let default_thumb_bg = [0.7, 0.7, 0.7, 0.6];

  let inset = resolve_scrollbar_inset(ps.scrollbar.scrollbar_inset);
  let min_thumb = resolve_scrollbar_min_thumb_size(ps.scrollbar.scrollbar_min_thumb_size);
  let track = resolve_part(&ps.track, default_track_bg);
  let thumb = resolve_part(&ps.thumb, default_thumb_bg);
  let corner = resolve_part(&ps.corner, default_track_bg);

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
      dl.push_quad_rounded(
        DlRect::new(track_x, track_y, track_w, track_h),
        track_color,
        track.radii,
      );
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
    dl.push_quad_rounded(
      DlRect::new(track_x, thumb_y, track_w, thumb_h),
      thumb_color,
      thumb_radii,
    );
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
      dl.push_quad_rounded(
        DlRect::new(track_x, track_y, track_w, track_h),
        track_color,
        track.radii,
      );
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
    dl.push_quad_rounded(
      DlRect::new(thumb_x, track_y, thumb_w, track_h),
      thumb_color,
      thumb_radii,
    );
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

fn paint_scrollbars_legacy(
  b: &LayoutBox,
  scroll: &lui_layout::ScrollInfo,
  dx: f32,
  dy: f32,
  bar_w: f32,
  track_color: [f32; 4],
  thumb_color: [f32; 4],
  dl: &mut DisplayList,
) {
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

  let show_y = max_y > 0.5;
  let show_x = max_x > 0.5;
  if let Some(ps) = &style_box.scrollbar_pseudo {
    paint_viewport_scrollbars_styled(
      ps,
      viewport_width,
      viewport_height,
      scroll_x,
      scroll_y,
      max_x,
      max_y,
      bar_w,
      show_x,
      show_y,
      dl,
    );
    return;
  }

  let (track_color, thumb_color) = resolve_scrollbar_colors(style_box.style.scrollbar_color, 1.0);

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

fn paint_viewport_scrollbars_styled(
  ps: &lui_cascade::ScrollbarPseudoStyles,
  viewport_width: f32,
  viewport_height: f32,
  scroll_x: f32,
  scroll_y: f32,
  max_x: f32,
  max_y: f32,
  bar_w: f32,
  show_x: bool,
  show_y: bool,
  dl: &mut DisplayList,
) {
  let default_track_bg = [0.95, 0.95, 0.95, 1.0];
  let default_thumb_bg = [0.7, 0.7, 0.7, 0.6];
  let default_corner_bg = default_track_bg;

  let inset = resolve_scrollbar_inset(ps.scrollbar.scrollbar_inset);
  let min_thumb = resolve_scrollbar_min_thumb_size(ps.scrollbar.scrollbar_min_thumb_size);
  let track_part = resolve_part(&ps.track, default_track_bg);
  let thumb_part = resolve_part(&ps.thumb, default_thumb_bg);
  let corner_part = resolve_part(&ps.corner, default_corner_bg);

  if show_y {
    let track_x = viewport_width - bar_w - VIEWPORT_SCROLLBAR_MARGIN + inset[1];
    let track_y = VIEWPORT_SCROLLBAR_MARGIN + inset[0];
    let track_w = (bar_w - inset[1] - inset[3]).max(1.0);
    let track_h =
      viewport_height - VIEWPORT_SCROLLBAR_MARGIN * 2.0 - inset[0] - inset[2] - if show_x { bar_w } else { 0.0 };
    if track_h > 0.0 {
      let track_color = apply_opacity(track_part.bg, track_part.opacity);
      if track_part.radii == [0.0; 4] {
        dl.push_quad(DlRect::new(track_x, track_y, track_w, track_h), track_color);
      } else {
        dl.push_quad_rounded(
          DlRect::new(track_x, track_y, track_w, track_h),
          track_color,
          track_part.radii,
        );
      }

      let doc_h = viewport_height + max_y;
      let thumb_h = (track_h * viewport_height / doc_h).max(min_thumb).min(track_h);
      let travel = (track_h - thumb_h).max(0.0);
      let thumb_y = track_y + travel * (scroll_y / max_y.max(1.0));
      let thumb_color = apply_opacity(thumb_part.bg, thumb_part.opacity);
      let thumb_radii = if thumb_part.radii == [0.0; 4] {
        [(track_w * 0.5).min(4.0); 4]
      } else {
        thumb_part.radii
      };
      dl.push_quad_rounded(
        DlRect::new(track_x, thumb_y, track_w, thumb_h),
        thumb_color,
        thumb_radii,
      );
    }
  }

  if show_x {
    let track_x = VIEWPORT_SCROLLBAR_MARGIN + inset[3];
    let track_y = viewport_height - bar_w - VIEWPORT_SCROLLBAR_MARGIN + inset[2];
    let track_w =
      viewport_width - VIEWPORT_SCROLLBAR_MARGIN * 2.0 - inset[1] - inset[3] - if show_y { bar_w } else { 0.0 };
    let track_h = (bar_w - inset[0] - inset[2]).max(1.0);
    if track_w > 0.0 {
      let track_color = apply_opacity(track_part.bg, track_part.opacity);
      if track_part.radii == [0.0; 4] {
        dl.push_quad(DlRect::new(track_x, track_y, track_w, track_h), track_color);
      } else {
        dl.push_quad_rounded(
          DlRect::new(track_x, track_y, track_w, track_h),
          track_color,
          track_part.radii,
        );
      }

      let doc_w = viewport_width + max_x;
      let thumb_w = (track_w * viewport_width / doc_w).max(min_thumb).min(track_w);
      let travel = (track_w - thumb_w).max(0.0);
      let thumb_x = track_x + travel * (scroll_x / max_x.max(1.0));
      let thumb_color = apply_opacity(thumb_part.bg, thumb_part.opacity);
      let thumb_radii = if thumb_part.radii == [0.0; 4] {
        [(track_h * 0.5).min(4.0); 4]
      } else {
        thumb_part.radii
      };
      dl.push_quad_rounded(
        DlRect::new(thumb_x, track_y, thumb_w, track_h),
        thumb_color,
        thumb_radii,
      );
    }
  }

  if show_x && show_y {
    let corner_color = apply_opacity(corner_part.bg, corner_part.opacity);
    dl.push_quad(
      DlRect::new(
        viewport_width - bar_w - VIEWPORT_SCROLLBAR_MARGIN + inset[1],
        viewport_height - bar_w - VIEWPORT_SCROLLBAR_MARGIN + inset[2],
        (bar_w - inset[1] - inset[3]).max(1.0),
        (bar_w - inset[0] - inset[2]).max(1.0),
      ),
      corner_color,
    );
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

pub(crate) fn resolve_scrollbar_width(v: Option<&lui_core::CssValue>) -> f32 {
  lui_core::resolve_scrollbar_width(v)
}

pub(crate) fn viewport_scrollbar_style_box<'a>(tree: &'a LayoutTree<'a>) -> &'a LayoutBox<'a> {
  let root = &tree.root;
  if let Some(body) = root.children.first() {
    if body.style.scrollbar_color.is_some() || body.style.scrollbar_width.is_some() || body.scrollbar_pseudo.is_some() {
      return body;
    }
  }
  root
}

pub(crate) fn viewport_scrollbar_style_path(tree: &LayoutTree<'_>) -> Vec<usize> {
  let root = &tree.root;
  if let Some(body) = root.children.first() {
    if body.style.scrollbar_color.is_some() || body.style.scrollbar_width.is_some() || body.scrollbar_pseudo.is_some() {
      return vec![0];
    }
  }
  Vec::new()
}
