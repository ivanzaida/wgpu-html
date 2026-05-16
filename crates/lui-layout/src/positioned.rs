//! Positioned layout — `position: relative/absolute/fixed`.
//!
//! - `relative`: laid out in-flow, then offset by top/left/bottom/right
//! - `absolute`: removed from flow, positioned against nearest positioned ancestor
//! - `fixed`: removed from flow, positioned against viewport

use bumpalo::Bump;
use lui_core::Rect;
use lui_parse::HtmlNode;

use crate::{
  box_tree::{BoxKind, LayoutBox},
  context::LayoutContext,
  geometry::Point,
  sides, sizes,
  text::TextContext,
};

fn css_str(v: Option<&lui_core::CssValue>) -> &str {
  match v {
    Some(lui_core::CssValue::String(s)) | Some(lui_core::CssValue::Unknown(s)) => s.as_ref(),
    _ => "",
  }
}

/// True if the element is out-of-flow (absolute or fixed).
pub fn is_out_of_flow(style: &lui_cascade::ComputedStyle) -> bool {
  matches!(css_str(style.position), "absolute" | "fixed")
}

/// True if the element establishes a containing block for positioned descendants.
pub fn is_positioned(style: &lui_cascade::ComputedStyle) -> bool {
  matches!(css_str(style.position), "relative" | "absolute" | "fixed" | "sticky") || has_transform(style)
}

fn has_transform(style: &lui_cascade::ComputedStyle) -> bool {
  match css_str(style.transform) {
    "" | "none" => false,
    _ => true,
  }
}

/// Layout an absolutely or fixed-positioned box.
///
/// `static_pos` is where the element would be if it were in normal flow.
/// `containing_block` is the padding box of the nearest positioned ancestor
/// (or viewport for fixed).
pub fn layout_out_of_flow<'a>(
  b: &mut LayoutBox<'a>,
  ctx: &LayoutContext,
  static_pos: Point,
  containing_block: Rect,
  text_ctx: &mut TextContext,
  rects: &mut Vec<(&'a HtmlNode, Rect)>,
  cache: &crate::incremental::CacheView,
  bump: &'a Bump,
) {
  let is_fixed = css_str(b.style.position) == "fixed";
  let cb = if is_fixed {
    Rect::new(0.0, 0.0, ctx.viewport_width, ctx.viewport_height)
  } else {
    containing_block
  };

  let margin = sides::resolve_margin(b.style);
  let border = sides::resolve_border(b.style);
  let padding = sides::resolve_padding(b.style);
  b.margin = margin.edges;
  b.border = border;
  b.padding = padding;

  let inset_left = sizes::resolve_length(b.style.left, cb.width);
  let inset_right = sizes::resolve_length(b.style.right, cb.width);
  let inset_top = sizes::resolve_length(b.style.top, cb.height);
  let inset_bottom = sizes::resolve_length(b.style.bottom, cb.height);

  let frame_h = margin.edges.horizontal() + border.horizontal() + padding.horizontal();
  let frame_v = margin.edges.vertical() + border.vertical() + padding.vertical();

  // Width resolution
  let explicit_w = sizes::resolve_length(b.style.width, cb.width);
  let content_w = if let Some(w) = explicit_w {
    w
  } else if inset_left.is_some() && inset_right.is_some() {
    (cb.width - inset_left.unwrap() - inset_right.unwrap() - frame_h).max(0.0)
  } else {
    let available = (cb.width - frame_h).max(0.0);
    let shrink = crate::flex::measure_max_content_width_pub(b, text_ctx);
    let content_only = (shrink - border.horizontal() - padding.horizontal()).max(0.0);
    content_only.min(available)
  };
  b.content.width = content_w;

  // Height resolution
  let explicit_h = sizes::resolve_length(b.style.height, cb.height);
  let content_h_from_style = if let Some(h) = explicit_h {
    Some(h)
  } else if inset_top.is_some() && inset_bottom.is_some() {
    Some((cb.height - inset_top.unwrap() - inset_bottom.unwrap() - frame_v).max(0.0))
  } else {
    None
  };

  // Horizontal position
  let x = if let Some(left) = inset_left {
    cb.x + left + margin.edges.left + border.left + padding.left
  } else if let Some(right) = inset_right {
    cb.x + cb.width - right - margin.edges.right - border.right - padding.right - content_w
  } else {
    static_pos.x + margin.edges.left + border.left + padding.left
  };

  // Vertical position
  let y = if let Some(top) = inset_top {
    cb.y + top + margin.edges.top + border.top + padding.top
  } else if let Some(bottom) = inset_bottom {
    let h = content_h_from_style.unwrap_or(0.0);
    cb.y + cb.height - bottom - margin.edges.bottom - border.bottom - padding.bottom - h
  } else {
    static_pos.y + margin.edges.top + border.top + padding.top
  };

  b.content.x = x;
  b.content.y = y;

  // Layout children as block
  let child_ctx = LayoutContext {
    containing_width: content_w,
    ..*ctx
  };
  let mut cursor_y = b.content.y;
  for child in b.children.iter_mut() {
    let placeholder = LayoutBox::new(BoxKind::Block, child.node, child.style, bump);
    let old = std::mem::replace(child, placeholder);
    let result = crate::engine::layout_node(
      old,
      &child_ctx,
      Point::new(b.content.x, cursor_y),
      text_ctx,
      rects,
      cache,
      bump,
    );
    *child = result;
    cursor_y += child.outer_height();
  }

  b.content.height = content_h_from_style.unwrap_or((cursor_y - b.content.y).max(0.0));
}

/// Apply `position: relative` offset after normal layout.
pub fn apply_relative_offset(b: &mut LayoutBox, containing_width: f32, containing_height: f32) {
  let pos = css_str(b.style.position);
  if pos == "relative" {
    let dx = sizes::resolve_length(b.style.left, containing_width)
      .or_else(|| sizes::resolve_length(b.style.right, containing_width).map(|v| -v))
      .unwrap_or(0.0);
    let dy = sizes::resolve_length(b.style.top, containing_height)
      .or_else(|| sizes::resolve_length(b.style.bottom, containing_height).map(|v| -v))
      .unwrap_or(0.0);

    if dx.abs() > 0.001 || dy.abs() > 0.001 {
      translate_recursive(b, dx, dy);
    }
  } else if pos == "sticky" {
    b.sticky = Some(crate::box_tree::StickyInsets {
      top: sizes::resolve_length(b.style.top, containing_height),
      right: sizes::resolve_length(b.style.right, containing_width),
      bottom: sizes::resolve_length(b.style.bottom, containing_height),
      left: sizes::resolve_length(b.style.left, containing_width),
    });
  }
}

/// Populate z-index on a LayoutBox from its computed style.
pub fn apply_z_index(b: &mut LayoutBox) {
  if let Some(lui_core::CssValue::Number(n)) = b.style.z_index {
    b.z_index = Some(*n as i32);
  }
}

pub fn translate_recursive_pub(b: &mut LayoutBox, dx: f32, dy: f32) {
  translate_recursive(b, dx, dy);
}

fn translate_recursive(b: &mut LayoutBox, dx: f32, dy: f32) {
  b.content.x += dx;
  b.content.y += dy;
  for child in &mut b.children {
    translate_recursive(child, dx, dy);
  }
}
