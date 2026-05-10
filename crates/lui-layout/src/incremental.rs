//! Incremental layout — re-lay-out only dirty subtrees and shift clean
//! siblings. Also contains paint-only colour patching and LUI pseudo-element
//! helpers.

use std::collections::HashMap;
use std::sync::Arc;

use lui_models::{
  common::css_enums::{CssColor, CssLength, Display, FlexDirection, Position},
  ArcStr,
  LuiCalendarStyle,
  LuiColorPickerStyle,
  LuiPopupStyle,
};
use lui_style::{CascadedNode, CascadedTree};
use lui_tree::{Node, Tree};

use super::*;

// ---------------------------------------------------------------------------
// Incremental layout
// ---------------------------------------------------------------------------

/// Walk a laid-out `LayoutBox` tree and its matching `Tree` in lockstep,
/// patching only the form-control metadata from the updated element data.
/// The geometry (positions, sizes) is NOT recomputed — this is O(n) simple
/// field writes, not a full relayout.
///
/// Use this after `cascade_incremental` returns `true` when all
/// pseudo-class rules are known to be paint-only.
pub fn patch_form_controls(layout: &mut LayoutBox, tree: &Tree) {
  if let Some(root) = &tree.root {
    patch_fc_recursive(layout, root);
  }
}

fn patch_fc_recursive(b: &mut LayoutBox, node: &Node) {
  b.form_control = form_control_info_from_element(&node.element);
  for (child_box, child_node) in b.children.iter_mut().zip(node.children.iter()) {
    patch_fc_recursive(child_box, child_node);
  }
}

/// Incrementally update a cached LayoutBox tree, re-laying-out only
/// dirty subtrees and shifting clean siblings. Falls back to full
/// relayout when dirty_paths is empty or the root dimensions change.
pub fn layout_incremental(
  cascaded: &CascadedTree,
  prev: &mut LayoutBox,
  dirty_paths: &[Vec<usize>],
  text_ctx: &mut lui_text::TextContext,
  image_cache: &mut ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  locale: &dyn lui_tree::Locale,
  date_display_value: Option<String>,
  date_focus_iso: Option<String>,
) -> bool {
  let Some(root) = cascaded.root.as_ref() else {
    return false;
  };
  let mut ctx = Ctx {
    viewport_w,
    viewport_h,
    scale,
    text: TextCtx { ctx: text_ctx },
    images: image_cache,
    locale,
    date_display_value,
    date_focus_iso,
    profiler: None,
  };
  let path = Vec::new();
  let dy = relayout_children(prev, root, dirty_paths, &path, viewport_w, viewport_h, &mut ctx);
  if dy.abs() > 0.01 {
    prev.content_rect.h += dy;
    prev.border_rect.h += dy;
    prev.margin_rect.h += dy;
    prev.background_rect.h += dy;
  }
  dy.abs() > 0.01
}

fn path_is_dirty(dirty_paths: &[Vec<usize>], path: &[usize]) -> bool {
  dirty_paths.iter().any(|dp| dp.as_slice() == path)
}

fn path_is_ancestor_of_dirty(dirty_paths: &[Vec<usize>], path: &[usize]) -> bool {
  dirty_paths.iter().any(|dp| dp.len() > path.len() && dp.starts_with(path))
}

fn needs_full_relayout(node: &CascadedNode) -> bool {
  let style = &node.style;
  match style.display.as_ref() {
    Some(Display::Table) => true,
    Some(Display::Grid | Display::InlineGrid) => true,
    Some(Display::Flex | Display::InlineFlex) => {
      if matches!(
        style.flex_direction,
        Some(FlexDirection::Column | FlexDirection::ColumnReverse)
      ) {
        return false;
      }
      // Flex-row has cross-item width dependencies. However, when
      // every direct child has an explicit CSS width, content
      // changes inside one child cannot affect sibling sizing —
      // safe to recurse into the dirty child only.
      let children = effective_children(node);
      !children.iter().all(|c| c.style.width.is_some())
    }
    _ => false,
  }
}

fn relayout_children(
  parent_box: &mut LayoutBox,
  parent_node: &CascadedNode,
  dirty_paths: &[Vec<usize>],
  current_path: &[usize],
  container_w: f32,
  container_h: f32,
  ctx: &mut Ctx,
) -> f32 {
  let effective = effective_children(parent_node);
  if effective.len() != parent_box.children.len() {
    return 0.0;
  }

  let mut cursor_dy = 0.0_f32;

  for (i, (child_box, child_node)) in parent_box
    .children
    .iter_mut()
    .zip(effective.iter())
    .enumerate()
  {
    let mut child_path = current_path.to_vec();
    child_path.push(i);

    if cursor_dy.abs() > 0.01 {
      translate_box_y_in_place(child_box, cursor_dy);
    }

    let is_dirty = path_is_dirty(dirty_paths, &child_path);
    let is_ancestor = path_is_ancestor_of_dirty(dirty_paths, &child_path);

    if is_dirty {
      let old_h = child_box.margin_rect.h;
      let style = &child_node.style;
      let child_position = style.position.clone().unwrap_or(Position::Static);
      let containing_block = Rect::new(
        parent_box.content_rect.x,
        parent_box.content_rect.y,
        container_w,
        container_h,
      );
      if is_out_of_flow_position(child_position.clone()) {
        *child_box = layout_out_of_flow_block(
          child_node,
          child_box.margin_rect.x,
          child_box.margin_rect.y,
          container_w,
          container_h,
          containing_block,
          ctx,
        );
      } else {
        *child_box = layout_block(
          child_node,
          child_box.margin_rect.x,
          child_box.margin_rect.y,
          container_w,
          container_h,
          containing_block,
          BlockOverrides::default(),
          ctx,
        );
      }
      let new_h = child_box.margin_rect.h;
      if !is_out_of_flow_position(child_position) {
        cursor_dy += new_h - old_h;
      }
    } else if is_ancestor {
      if needs_full_relayout(child_node) {
        let old_h = child_box.margin_rect.h;
        let containing_block = Rect::new(
          parent_box.content_rect.x,
          parent_box.content_rect.y,
          container_w,
          container_h,
        );
        *child_box = layout_block(
          child_node,
          child_box.margin_rect.x,
          child_box.margin_rect.y,
          container_w,
          container_h,
          containing_block,
          BlockOverrides::default(),
          ctx,
        );
        let new_h = child_box.margin_rect.h;
        let child_position = child_node.style.position.clone().unwrap_or(
          lui_models::common::css_enums::Position::Static,
        );
        if !is_out_of_flow_position(child_position) {
          cursor_dy += new_h - old_h;
        }
      } else {
        let inner_w = child_box.content_rect.w;
        let inner_h = child_box.content_rect.h;
        let dy = relayout_children(child_box, child_node, dirty_paths, &child_path, inner_w, inner_h, ctx);
        if dy.abs() > 0.01 {
          let has_explicit_h = child_node.style.height.is_some();
          if !has_explicit_h {
            child_box.content_rect.h += dy;
            child_box.border_rect.h += dy;
            child_box.margin_rect.h += dy;
            child_box.background_rect.h += dy;
            let child_position = child_node.style.position.clone().unwrap_or(
              lui_models::common::css_enums::Position::Static,
            );
            if !is_out_of_flow_position(child_position) {
              cursor_dy += dy;
            }
          }
        }
      }
    }
    // else: clean + not ancestor → skip (already shifted if needed)
  }

  cursor_dy
}

// ---------------------------------------------------------------------------
// Colour patching
// ---------------------------------------------------------------------------

/// Walk a laid-out `LayoutBox` tree and its matching `CascadedTree` in
/// lockstep, patching only paint-relevant properties (background color,
/// text color, border colors, opacity) from the updated cascade. The
/// geometry (positions, sizes) is NOT recomputed — this is O(n) simple
/// field writes, not a full relayout.
pub fn patch_layout_colors(layout: &mut LayoutBox, cascaded: &CascadedTree) {
  if let Some(root) = &cascaded.root {
    patch_node_colors(layout, root, color::BLACK);
  }
}

fn patch_node_colors(b: &mut LayoutBox, node: &CascadedNode, inherited_color: Color) {
  use color::{resolve_foreground, resolve_with_current};
  let style = &node.style;

  let fg = resolve_foreground(style.color.as_ref(), inherited_color);

  b.background = style.background_color.as_ref().and_then(|c| resolve_with_current(c, fg));
  b.opacity = resolved_opacity(style);
  b.pointer_events = resolved_pointer_events(style);
  b.user_select = resolved_user_select(style);

  if b.text_color.is_some() || matches!(b.kind, BoxKind::Text) || b.form_control.is_some() {
    b.text_color = Some(fg);
  }

  b.accent_color = style.accent_color.as_ref().and_then(|c| resolve_with_current(c, fg));
  b.lui = resolve_lui_properties(&style.custom_properties, fg);

  let resolve_border = |c: &CssColor| resolve_with_current(c, fg);
  b.border_colors = BorderColors {
    top: style.border_top_color.as_ref().and_then(resolve_border).or(Some(fg)),
    right: style.border_right_color.as_ref().and_then(resolve_border).or(Some(fg)),
    bottom: style.border_bottom_color.as_ref().and_then(resolve_border).or(Some(fg)),
    left: style.border_left_color.as_ref().and_then(resolve_border).or(Some(fg)),
  };

  b.first_line_color = node.first_line.as_ref().and_then(|s| s.color.as_ref()).and_then(resolve_color);
  b.first_letter_color = node.first_letter.as_ref().and_then(|s| s.color.as_ref()).and_then(resolve_color);
  b.selection_bg = node
    .selection
    .as_ref()
    .and_then(|s| s.background_color.as_ref())
    .and_then(resolve_color);
  b.selection_fg = node
    .selection
    .as_ref()
    .and_then(|s| s.color.as_ref())
    .and_then(resolve_color);

  for (child_box, child_node) in b.children.iter_mut().zip(node.children.iter()) {
    patch_node_colors(child_box, child_node, fg);
  }
}

// ---------------------------------------------------------------------------
// LUI pseudo-element helpers
// ---------------------------------------------------------------------------

fn resolve_lui_color(
  custom_properties: &HashMap<ArcStr, ArcStr>,
  name: &str,
  current: Color,
) -> Option<Color> {
  let val = custom_properties.get(name)?;
  let css_color = lui_parser::parse_css_color(val.trim())?;
  color::resolve_with_current(&css_color, current)
}

pub fn resolve_lui_properties(
  cp: &HashMap<ArcStr, ArcStr>,
  fg: Color,
) -> LuiProperties {
  LuiProperties {
    track_color: resolve_lui_color(cp, "--lui-track-color", fg),
    thumb_color: resolve_lui_color(cp, "--lui-thumb-color", fg),
  }
}

pub fn lui_popup_from_pseudo(node: &CascadedNode) -> Option<Arc<LuiPopupStyle>> {
  let s = node.lui_style(lui_tree::PseudoElement::LuiPopup)?;
  Some(Arc::new(LuiPopupStyle {
    width: s.width.clone(),
    height: s.height.clone(),
    background_color: s.background_color.clone(),
    color: s.color.clone(),
    border_top_width: s.border_top_width.clone(),
    border_right_width: s.border_right_width.clone(),
    border_bottom_width: s.border_bottom_width.clone(),
    border_left_width: s.border_left_width.clone(),
    border_top_style: s.border_top_style.clone(),
    border_right_style: s.border_right_style.clone(),
    border_bottom_style: s.border_bottom_style.clone(),
    border_left_style: s.border_left_style.clone(),
    border_top_color: s.border_top_color.clone(),
    border_right_color: s.border_right_color.clone(),
    border_bottom_color: s.border_bottom_color.clone(),
    border_left_color: s.border_left_color.clone(),
    border_radius: s.border_top_left_radius.clone(),
    font_size: s.font_size.clone(),
    font_family: s.font_family.clone(),
    font_weight: s.font_weight.clone(),
  }))
}

pub fn lui_color_from_pseudo(node: &CascadedNode) -> Option<Arc<LuiColorPickerStyle>> {
  use lui_tree::PseudoElement;
  let mut p = LuiColorPickerStyle::default();
  let mut any = false;
  if let Some(s) = node.lui_style(PseudoElement::LuiCanvas) {
    p.canvas_width = s.width.clone();
    p.canvas_height = s.height.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiRange) {
    p.range_height = s.height.clone();
    p.range_border_radius = s.border_top_left_radius.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiThumb) {
    p.thumb_width = s.width.clone();
    p.thumb_height = s.height.clone();
    p.thumb_color = s.background_color.clone().or_else(|| s.color.clone());
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiInput) {
    p.input_height = s.height.clone();
    p.input_background = s.background_color.clone();
    p.input_border_color = s.border_top_color.clone();
    p.input_border_width = s.border_top_width.clone();
    p.input_border_radius = s.border_top_left_radius.clone();
    p.input_font_size = s.font_size.clone();
    any = true;
  }
  if any { Some(Arc::new(p)) } else { None }
}

pub fn lui_calendar_from_pseudo(node: &CascadedNode) -> Option<Arc<LuiCalendarStyle>> {
  use lui_tree::PseudoElement;
  let mut p = LuiCalendarStyle::default();
  let mut any = false;
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarCell) {
    p.cell_size = s.width.clone();
    p.cell_radius = s.border_top_left_radius.clone();
    p.day_font_size = s.font_size.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarSelected) {
    p.selected_bg = s.background_color.clone();
    p.selected_color = s.color.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarToday) {
    p.today_color = s.border_top_color.clone().or_else(|| s.color.clone());
    p.today_width = s.border_top_width.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarHeader) {
    p.header_font_size = s.font_size.clone();
    p.header_font_weight = s.font_weight.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarWeekday) {
    p.weekday_font_size = s.font_size.clone();
    p.dim = s.color.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarNav) {
    p.nav_size = s.width.clone().or_else(|| s.font_size.clone());
    if p.dim.is_none() { p.dim = s.color.clone(); }
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarTime) {
    p.time_width = s.width.clone();
    p.time_height = s.height.clone();
    p.time_background = s.background_color.clone();
    p.time_border_color = s.border_top_color.clone();
    p.time_border_width = s.border_top_width.clone();
    p.time_border_radius = s.border_top_left_radius.clone();
    p.time_font_size = s.font_size.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarReset) {
    p.reset_height = s.height.clone();
    p.reset_background = s.background_color.clone();
    p.reset_color = s.color.clone();
    p.reset_border_color = s.border_top_color.clone();
    p.reset_border_width = s.border_top_width.clone();
    p.reset_border_radius = s.border_top_left_radius.clone();
    p.reset_font_size = s.font_size.clone();
    any = true;
  }
  if let Some(s) = node.lui_style(PseudoElement::LuiCalendarIcon) {
    if p.dim.is_none() { p.dim = s.color.clone(); }
    p.padding = s.width.clone();
    any = true;
  }
  if any { Some(Arc::new(p)) } else { None }
}

pub fn file_button_from_pseudo(node: &CascadedNode) -> Option<FileButtonStyle> {
  let s = node.lui_style(lui_tree::PseudoElement::FileSelectorButton)?;
  let mut fb = FileButtonStyle::default();
  if let Some(c) = s.background_color.as_ref().and_then(|c| color::resolve_color(c)) {
    fb.background = Some(c);
  }
  if let Some(c) = s.color.as_ref().and_then(|c| color::resolve_color(c)) {
    fb.color = Some(c);
  }
  if let Some(c) = s.border_top_color.as_ref().and_then(|c| color::resolve_color(c)) {
    fb.border_color = Some(c);
  }
  if let Some(r) = resolve_border_radius_single(s.border_top_left_radius.as_ref()) {
    fb.border_radius = r;
  }
  let fs = 16.0;
  if let Some(v) = resolve_length_px(s.padding_top.as_ref(), fs) { fb.padding[0] = v; }
  if let Some(v) = resolve_length_px(s.padding_right.as_ref(), fs) { fb.padding[1] = v; }
  if let Some(v) = resolve_length_px(s.padding_bottom.as_ref(), fs) { fb.padding[2] = v; }
  if let Some(v) = resolve_length_px(s.padding_left.as_ref(), fs) { fb.padding[3] = v; }
  if let Some(c) = s.cursor.clone() {
    fb.cursor = c;
  }
  Some(fb)
}

fn resolve_length_px(val: Option<&CssLength>, font_size: f32) -> Option<f32> {
  match val? {
    CssLength::Px(v) => Some(*v),
    CssLength::Em(v) => Some(*v * font_size),
    CssLength::Rem(v) => Some(*v * 16.0),
    _ => None,
  }
}

fn resolve_border_radius_single(val: Option<&CssLength>) -> Option<f32> {
  resolve_length_px(val, 16.0)
}

// ---------------------------------------------------------------------------
// Shared utility
// ---------------------------------------------------------------------------

pub fn padding_box_rect(b: &LayoutBox) -> Rect {
  Rect::new(
    b.border_rect.x + b.border.left,
    b.border_rect.y + b.border.top,
    (b.border_rect.w - b.border.horizontal()).max(0.0),
    (b.border_rect.h - b.border.vertical()).max(0.0),
  )
}
