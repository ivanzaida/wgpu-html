//! Positioned layout helpers, overflow resolution, and out-of-flow
//! block placement.  Extracted from `lib.rs` to keep the block-layout
//! file focused.

use lui_models::{
  common::css_enums::{Overflow, Position, ScrollbarColor, ScrollbarWidth},
  Style,
};
use lui_style::CascadedNode;

use crate::{
  color::{resolve_color, Color},
  length,
  BlockOverrides, Ctx, LayoutBox, OverflowAxes, Rect,
};

// ---------------------------------------------------------------------------
// Simple style accessors
// ---------------------------------------------------------------------------

pub(crate) fn is_out_of_flow_position(position: Position) -> bool {
  matches!(position, Position::Absolute | Position::Fixed)
}

pub(crate) fn resolved_opacity(style: &Style) -> f32 {
  style.opacity.unwrap_or(1.0).clamp(0.0, 1.0)
}

pub(crate) fn resolved_pointer_events(style: &Style) -> lui_models::common::css_enums::PointerEvents {
  style.pointer_events.unwrap_or(lui_models::common::css_enums::PointerEvents::Auto)
}

pub(crate) fn resolved_user_select(style: &Style) -> lui_models::common::css_enums::UserSelect {
  style.user_select.unwrap_or(lui_models::common::css_enums::UserSelect::Auto)
}

pub(crate) fn resolved_cursor(style: &Style) -> lui_models::common::css_enums::Cursor {
  style.cursor.clone().unwrap_or(lui_models::common::css_enums::Cursor::Auto)
}

pub(crate) fn resolved_z_index(style: &Style) -> Option<i32> {
  style.z_index
}

pub(crate) fn establishes_containing_block(style: &Style) -> bool {
  !matches!(style.position, None | Some(Position::Static))
}

// ---------------------------------------------------------------------------
// Out-of-flow block layout
// ---------------------------------------------------------------------------

pub(crate) fn layout_out_of_flow_block(
  node: &CascadedNode,
  static_x: f32,
  static_y: f32,
  _container_w: f32,
  _container_h: f32,
  containing_block: Rect,
  ctx: &mut Ctx,
) -> LayoutBox {
  let style = &node.style;
  let cb = if matches!(style.position, Some(Position::Fixed)) {
    Rect::new(0.0, 0.0, ctx.viewport_w, ctx.viewport_h)
  } else {
    containing_block
  };
  let left = length::resolve(style.left.as_ref(), cb.w, ctx);
  let right = length::resolve(style.right.as_ref(), cb.w, ctx);
  let top = length::resolve(style.top.as_ref(), cb.h, ctx);
  let bottom = length::resolve(style.bottom.as_ref(), cb.h, ctx);
  let overrides = positioned_overrides(node, cb.w, cb.h, left, right, top, bottom, ctx);

  let origin_x = left.map(|v| cb.x + v).unwrap_or(static_x);
  let origin_y = top.map(|v| cb.y + v).unwrap_or(static_y);
  let mut box_ = crate::layout_block(node, origin_x, origin_y, cb.w, cb.h, cb, overrides, ctx);
  box_.is_fixed = matches!(style.position, Some(Position::Fixed));

  if left.is_none()
    && let Some(right) = right
  {
    let target_x = cb.x + cb.w - right - box_.margin_rect.w;
    let dx = target_x - box_.margin_rect.x;
    crate::translate_box_x_in_place(&mut box_, dx);
  }
  if top.is_none()
    && let Some(bottom) = bottom
  {
    let target_y = cb.y + cb.h - bottom - box_.margin_rect.h;
    let dy = target_y - box_.margin_rect.y;
    crate::translate_box_y_in_place(&mut box_, dy);
  }
  box_
}

// ---------------------------------------------------------------------------
// Positioned overrides & shrink-to-fit
// ---------------------------------------------------------------------------

fn positioned_overrides(
  node: &CascadedNode,
  cb_w: f32,
  cb_h: f32,
  left: Option<f32>,
  right: Option<f32>,
  top: Option<f32>,
  bottom: Option<f32>,
  ctx: &mut Ctx,
) -> BlockOverrides {
  let style = &node.style;
  let margin = crate::resolve_insets_margin(style, cb_w, ctx);
  let border = crate::resolve_border_widths(style, cb_w, ctx);
  let padding = crate::resolve_insets_padding(style, cb_w, ctx);
  let width = if style.width.is_none() {
    match left.zip(right) {
      Some((left, right)) => {
        Some((cb_w - left - right - margin.horizontal() - border.horizontal() - padding.horizontal()).max(0.0))
      }
      None => Some(shrink_to_fit_content_width(node, cb_w, ctx)),
    }
  } else {
    None
  };
  let height = if style.height.is_none() {
    top
      .zip(bottom)
      .map(|(top, bottom)| (cb_h - top - bottom - margin.vertical() - border.vertical() - padding.vertical()).max(0.0))
  } else {
    None
  };
  BlockOverrides {
    width,
    height,
    ignore_style_width: false,
    ignore_style_height: false,
  }
}

pub(crate) fn shrink_to_fit_content_width(node: &CascadedNode, available_w: f32, ctx: &mut Ctx) -> f32 {
  if crate::all_children_inline_level(node) {
    let (_children, width, _height) = crate::layout_inline_block_children(node, 0.0, 0.0, available_w, ctx);
    return width.min(available_w).max(0.0);
  }

  node
    .children
    .iter()
    .filter(|child| !is_out_of_flow_position(child.style.position.clone().unwrap_or(Position::Static)))
    .map(|child| {
      let measured = crate::layout_block(
        child,
        0.0,
        0.0,
        available_w,
        f32::INFINITY,
        Rect::new(0.0, 0.0, available_w, f32::INFINITY),
        BlockOverrides::default(),
        ctx,
      );
      measured.margin_rect.w
    })
    .fold(0.0_f32, f32::max)
    .min(available_w)
    .max(0.0)
}

// ---------------------------------------------------------------------------
// Relative position
// ---------------------------------------------------------------------------

pub(crate) fn apply_relative_position(
  box_: &mut LayoutBox,
  style: &Style,
  container_w: f32,
  container_h: f32,
  ctx: &mut Ctx,
) {
  let left = length::resolve(style.left.as_ref(), container_w, ctx);
  let right = length::resolve(style.right.as_ref(), container_w, ctx);
  let top = length::resolve(style.top.as_ref(), container_h, ctx);
  let bottom = length::resolve(style.bottom.as_ref(), container_h, ctx);
  let dx = left.or_else(|| right.map(|v| -v)).unwrap_or(0.0);
  let dy = top.or_else(|| bottom.map(|v| -v)).unwrap_or(0.0);
  if dx != 0.0 {
    crate::translate_box_x_in_place(box_, dx);
  }
  if dy != 0.0 {
    crate::translate_box_y_in_place(box_, dy);
  }
}

// ---------------------------------------------------------------------------
// Overflow resolution
// ---------------------------------------------------------------------------

/// Resolve `overflow` / `overflow-x` / `overflow-y` to computed axes.
///
/// CSS computes `visible` to `auto` and `clip` to `hidden` when the
/// opposite axis is scrollable (`hidden`, `scroll`, or `auto`). That
/// avoids one visible axis leaking out of an actual scroll container.
pub(crate) fn effective_overflow(style: &Style) -> OverflowAxes {
  let base = style.overflow.unwrap_or(Overflow::Visible);
  let mut x = style.overflow_x.unwrap_or(base);
  let mut y = style.overflow_y.unwrap_or(base);

  if overflow_forces_cross_axis(x) {
    y = coerce_cross_axis_overflow(y);
  }
  if overflow_forces_cross_axis(y) {
    x = coerce_cross_axis_overflow(x);
  }

  OverflowAxes {
    x,
    y,
    scrollbar_width: effective_scrollbar_width(style),
    scrollbar_thumb: effective_scrollbar_thumb(style),
    scrollbar_track: effective_scrollbar_track(style),
  }
}

pub(crate) fn effective_scrollbar_width(style: &Style) -> f32 {
  match style.scrollbar_width.unwrap_or(ScrollbarWidth::Auto) {
    ScrollbarWidth::Auto => 10.0,
    ScrollbarWidth::Thin => 6.0,
    ScrollbarWidth::None => 0.0,
    ScrollbarWidth::Px(v) => v.max(0.0),
  }
}

pub(crate) fn effective_scrollbar_thumb(style: &Style) -> Option<Color> {
  match style.scrollbar_color.as_ref()? {
    ScrollbarColor::Auto => None,
    ScrollbarColor::Custom { thumb, track: _ } => resolve_color(thumb),
  }
}

pub(crate) fn effective_scrollbar_track(style: &Style) -> Option<Color> {
  match style.scrollbar_color.as_ref()? {
    ScrollbarColor::Auto => None,
    ScrollbarColor::Custom { thumb: _, track } => resolve_color(track),
  }
}

fn overflow_forces_cross_axis(value: Overflow) -> bool {
  matches!(value, Overflow::Hidden | Overflow::Scroll | Overflow::Auto)
}

fn coerce_cross_axis_overflow(value: Overflow) -> Overflow {
  match value {
    Overflow::Visible => Overflow::Auto,
    Overflow::Clip => Overflow::Hidden,
    other => other,
  }
}
