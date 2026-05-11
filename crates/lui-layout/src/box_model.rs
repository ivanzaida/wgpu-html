//! Box model helpers: margin, border, padding resolution, background-clip,
//! border-radius clamping, and min/max clamping.

use lui_models::{
  Style,
  common::css_enums::{BoxSizing, CssLength},
};

use crate::{CornerRadii, Ctx, Insets, Radius, Rect, length};

/// Pick the rectangle and corner radii that the background fills, based
/// on `background-clip`. The default `border-box` keeps the outer
/// rectangle and radii. `padding-box` shrinks by the border thickness;
/// `content-box` shrinks by border + padding. Inner radii are reduced
/// in step so the curvature stays concentric with the outer edge.
pub(crate) fn compute_background_box(
  style: &Style,
  border_rect: Rect,
  content_rect: Rect,
  border: Insets,
  padding: Insets,
  radii: &CornerRadii,
) -> (Rect, CornerRadii) {
  use lui_models::common::css_enums::BackgroundClip;
  match style.background_clip.clone().unwrap_or(BackgroundClip::BorderBox) {
    BackgroundClip::BorderBox => (border_rect, radii.clone()),
    BackgroundClip::PaddingBox => {
      let inset_top = border.top;
      let inset_right = border.right;
      let inset_bottom = border.bottom;
      let inset_left = border.left;
      let r = inset_radii(radii, inset_top, inset_right, inset_bottom, inset_left);
      let rect = Rect::new(
        border_rect.x + inset_left,
        border_rect.y + inset_top,
        (border_rect.w - inset_left - inset_right).max(0.0),
        (border_rect.h - inset_top - inset_bottom).max(0.0),
      );
      (rect, r)
    }
    BackgroundClip::ContentBox => {
      let inset_top = border.top + padding.top;
      let inset_right = border.right + padding.right;
      let inset_bottom = border.bottom + padding.bottom;
      let inset_left = border.left + padding.left;
      let r = inset_radii(radii, inset_top, inset_right, inset_bottom, inset_left);
      (content_rect, r)
    }
  }
}

/// Reduce each corner's radius by the matching adjacent insets,
/// clamped at zero. The horizontal component shrinks by the inset of
/// the side it shares an x-edge with; the vertical component shrinks
/// by the inset of its y-edge. A tight border eats into the curvature
/// until the inner edge is straight.
pub(crate) fn inset_radii(r: &CornerRadii, top: f32, right: f32, bottom: f32, left: f32) -> CornerRadii {
  let shrink = |corner: Radius, dh: f32, dv: f32| Radius {
    h: (corner.h - dh).max(0.0),
    v: (corner.v - dv).max(0.0),
  };
  CornerRadii {
    top_left: shrink(r.top_left, left, top),
    top_right: shrink(r.top_right, right, top),
    bottom_right: shrink(r.bottom_right, right, bottom),
    bottom_left: shrink(r.bottom_left, left, bottom),
  }
}

pub(crate) fn resolve_insets_margin(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
  Insets {
    top: side(&style.margin_top, &style.margin, container_w, ctx),
    right: side(&style.margin_right, &style.margin, container_w, ctx),
    bottom: side(&style.margin_bottom, &style.margin, container_w, ctx),
    left: side(&style.margin_left, &style.margin, container_w, ctx),
  }
}

pub(crate) fn resolve_border_widths(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
  use lui_models::common::css_enums::BorderStyle;
  let w = |width: &Option<CssLength>, bstyle: &Option<BorderStyle>| -> f32 {
    if matches!(bstyle, Some(BorderStyle::None | BorderStyle::Hidden)) {
      return 0.0;
    }
    length::resolve(width.as_ref(), container_w, ctx).unwrap_or(0.0)
  };
  Insets {
    top: w(&style.border_top_width, &style.border_top_style),
    right: w(&style.border_right_width, &style.border_right_style),
    bottom: w(&style.border_bottom_width, &style.border_bottom_style),
    left: w(&style.border_left_width, &style.border_left_style),
  }
}

pub(crate) fn resolve_insets_padding(style: &Style, container_w: f32, ctx: &mut Ctx) -> Insets {
  Insets {
    top: side(&style.padding_top, &style.padding, container_w, ctx),
    right: side(&style.padding_right, &style.padding, container_w, ctx),
    bottom: side(&style.padding_bottom, &style.padding, container_w, ctx),
    left: side(&style.padding_left, &style.padding, container_w, ctx),
  }
}

pub(crate) fn side(
  specific: &Option<CssLength>,
  shorthand: &Option<CssLength>,
  container_w: f32,
  ctx: &mut Ctx,
) -> f32 {
  length::resolve(specific.as_ref(), container_w, ctx)
    .or_else(|| length::resolve(shorthand.as_ref(), container_w, ctx))
    .unwrap_or(0.0)
}

/// True when the effective value of a margin side (specific longhand
/// falling through to shorthand) is `auto`. Used by the flex layer to
/// detect items that want to absorb free space on the main / cross
/// axis.
pub(crate) fn is_auto_margin(specific: &Option<CssLength>, shorthand: &Option<CssLength>) -> bool {
  fn is_auto(v: &Option<CssLength>) -> bool {
    matches!(v, Some(CssLength::Auto))
  }
  if specific.is_some() {
    is_auto(specific)
  } else {
    is_auto(shorthand)
  }
}

/// Apply CSS `min-*` / `max-*` clamping to a content-box dimension.
///
/// Both bounds are interpreted in `box_sizing` semantics (so a
/// `border-box` `min-width: 100px` clamps the *border-box* width to
/// 100px just like browsers do). `frame` is the matching axis's
/// border + padding, used to convert between content-box and
/// border-box. `Auto` resolves to "no constraint", matching CSS.
///
/// `max` is applied first, then `min` (per CSS-Sizing-3 §5.2: "min
/// wins ties"), so an over-eager `min` always wins against `max`.
pub(crate) fn clamp_axis(
  size: f32,
  min: Option<&CssLength>,
  max: Option<&CssLength>,
  container: f32,
  frame: f32,
  box_sizing: BoxSizing,
  ctx: &mut Ctx,
) -> f32 {
  let convert = |raw: f32| -> f32 {
    match box_sizing {
      BoxSizing::ContentBox => raw,
      BoxSizing::BorderBox => (raw - frame).max(0.0),
    }
  };
  let mut out = size;
  if let Some(m) = length::resolve(max, container, ctx) {
    out = out.min(convert(m));
  }
  if let Some(m) = length::resolve(min, container, ctx) {
    out = out.max(convert(m));
  }
  out.max(0.0)
}

/// Per CSS 3 border-radius spec: when the sum of radii on any side
/// exceeds that side's length, *all* radii are scaled down by the
/// same factor (the smallest of the per-side ratios) so adjacent
/// corners no longer overlap. The horizontal and vertical components
/// are checked independently — overflow on either axis triggers a
/// uniform scale of every corner on every axis.
pub(crate) fn clamp_corner_radii(r: &mut CornerRadii, width: f32, height: f32) {
  let mut scale: f32 = 1.0;
  let limit = |edge_len: f32, sum: f32, scale: &mut f32| {
    if sum > 0.0 && edge_len > 0.0 && sum > edge_len {
      *scale = scale.min(edge_len / sum);
    }
  };
  // Horizontal axis: the h-components on each horizontal edge.
  limit(width, r.top_left.h + r.top_right.h, &mut scale);
  limit(width, r.bottom_left.h + r.bottom_right.h, &mut scale);
  // Vertical axis: the v-components on each vertical edge.
  limit(height, r.top_left.v + r.bottom_left.v, &mut scale);
  limit(height, r.top_right.v + r.bottom_right.v, &mut scale);
  if scale < 1.0 {
    for c in [
      &mut r.top_left,
      &mut r.top_right,
      &mut r.bottom_right,
      &mut r.bottom_left,
    ] {
      c.h *= scale;
      c.v *= scale;
    }
  }
}
