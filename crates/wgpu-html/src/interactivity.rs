//! Position-based interactivity wrappers.
//!
//! All real dispatch logic — hover-chain diffing, focus state,
//! keyboard delivery, click synthesis, selection updates — lives
//! in `wgpu_html_tree::dispatch` and is also exposed as inherent
//! methods on `Tree` (`tree.focus(…)`, `tree.key_down(…)`,
//! `tree.dispatch_mouse_down(…)`, etc.). New apps should drive the
//! tree directly through those.
//!
//! This module exists for the (still useful) case where the host
//! has a `wgpu_html_layout::LayoutBox` handy and would prefer to
//! pass a position rather than hit-test by hand. Each wrapper
//!
//! 1. resolves the hit path via `LayoutBox::hit_path`,
//! 2. resolves a text cursor via `LayoutBox::hit_text_cursor`,
//! 3. forwards to the matching `tree::dispatch_*` function.
//!
//! For compatibility with the previous public surface, the
//! layout-free entry points are re-exported here too:
//! [`focus`], [`blur`], [`focus_next`], [`key_down`], [`key_up`],
//! [`pointer_leave`].

use wgpu_html_layout::{Cursor, FormControlKind, LayoutBox};
use wgpu_html_tree::{ColorPickerDragTarget, ColorPickerState, DatePickerState, MouseButton, RangeDrag, Tree};

use crate::color_picker_overlay;
use crate::date_picker_overlay;
// Re-exports of the layout-free dispatch entry points — these used
// to live here, now they live in `wgpu_html_tree::dispatch`.
pub use wgpu_html_tree::{
  blur, dispatch_pointer_leave as pointer_leave, focus, focus_next, key_down, key_up, wheel_event,
};

/// Update the hover path to whatever lies under `pos` and fire
/// any `on_mouse_enter` / `on_mouse_leave` callbacks the change
/// implies. Returns `true` if the hover path actually changed.
///
/// Modifier state is read from `tree.interaction.modifiers`;
/// keep it in sync with [`Tree::set_modifier`].
pub fn pointer_move(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> bool {
  // Color picker drag.
  if let Some(ref cp) = tree.interaction.color_picker.clone() {
    if let Some(drag) = cp.drag {
      return update_color_picker_drag(tree, pos, drag);
    }
    if color_picker_overlay::hit_test_color_picker(cp, pos).is_some() {
      return true;
    }
  }
  if let Some(ref dp) = tree.interaction.date_picker {
    if date_picker_overlay::hit_test(dp, pos).is_some() {
      return true;
    }
  }

  // Range slider drag: update value from pointer position.
  if let Some(ref rd) = tree.interaction.range_drag.clone() {
    let frac = if rd.content_w > 0.0 {
      ((pos.0 - rd.content_x) / rd.content_w).clamp(0.0, 1.0)
    } else {
      0.0
    };
    wgpu_html_tree::set_range_value_by_fraction(tree, &rd.path, frac);
  }

  let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets);
  let text_cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets);
  tree.dispatch_pointer_move(target.as_deref(), pos, text_cursor)
}

/// Like [`pointer_move`] but also returns the resolved CSS `cursor`
/// for the hovered element. The host can use this to set the OS
/// pointer icon.
pub fn pointer_move_with_cursor(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> (bool, Cursor) {
  // Color picker drag.
  if let Some(ref cp) = tree.interaction.color_picker.clone() {
    if let Some(drag) = cp.drag {
      let changed = update_color_picker_drag(tree, pos, drag);
      return (changed, Cursor::Default);
    }
    if color_picker_overlay::hit_test_color_picker(cp, pos).is_some() {
      return (true, Cursor::Default);
    }
  }
  if let Some(ref dp) = tree.interaction.date_picker {
    if date_picker_overlay::hit_test(dp, pos).is_some() {
      return (true, Cursor::Default);
    }
  }

  // Range slider drag: update value from pointer position.
  if let Some(ref rd) = tree.interaction.range_drag.clone() {
    let frac = if rd.content_w > 0.0 {
      ((pos.0 - rd.content_x) / rd.content_w).clamp(0.0, 1.0)
    } else {
      0.0
    };
    wgpu_html_tree::set_range_value_by_fraction(tree, &rd.path, frac);
  }

  let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets);
  let text_cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets);
  let css_cursor = target
    .as_deref()
    .map(|path| layout.cursor_at_path(path))
    .unwrap_or(Cursor::Auto);
  let changed = tree.dispatch_pointer_move(target.as_deref(), pos, text_cursor);
  (changed || tree.interaction.range_drag.is_some(), css_cursor)
}

/// Primary-button (or any-button) press at `pos`. Records the
/// active path for click synthesis on the matching release; fires
/// `on_mouse_down` bubbling target → root; on a primary press,
/// also moves keyboard focus to the deepest focusable ancestor of
/// the hit path (or clears focus if none).
pub fn mouse_down(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool {
  mouse_down_with_click_count(tree, layout, pos, button, 1)
}

/// Like [`mouse_down`], but lets hosts pass an already-detected click
/// count. `2` selects the word/token under the pointer, `3+` selects
/// the shaped line.
pub fn mouse_down_with_click_count(
  tree: &mut Tree,
  layout: &LayoutBox,
  pos: (f32, f32),
  button: MouseButton,
  click_count: u8,
) -> bool {
  // Color picker: intercept clicks when open.
  if button == MouseButton::Primary {
    if let Some(ref cp) = tree.interaction.color_picker.clone() {
      if let Some(hit) = color_picker_overlay::hit_test_color_picker(cp, pos) {
        match hit {
          color_picker_overlay::ColorPickerHit::Canvas(s, v) => {
            if let Some(cp) = &mut tree.interaction.color_picker {
              color_picker_overlay::deactivate_field(cp);
              cp.saturation = s;
              cp.value = v;
              cp.drag = Some(ColorPickerDragTarget::Canvas);
              let path = cp.path.clone();
              let (r, g, b) = color_picker_overlay::hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
              let a = cp.alpha;
              wgpu_html_tree::set_color_value(tree, &path, r, g, b, a);
            }
            return true;
          }
          color_picker_overlay::ColorPickerHit::HueBar(frac) => {
            if let Some(cp) = &mut tree.interaction.color_picker {
              color_picker_overlay::deactivate_field(cp);
              cp.hue = frac * 360.0;
              cp.drag = Some(ColorPickerDragTarget::HueBar);
              let path = cp.path.clone();
              let (r, g, b) = color_picker_overlay::hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
              let a = cp.alpha;
              wgpu_html_tree::set_color_value(tree, &path, r, g, b, a);
            }
            return true;
          }
          color_picker_overlay::ColorPickerHit::AlphaBar(frac) => {
            if let Some(cp) = &mut tree.interaction.color_picker {
              color_picker_overlay::deactivate_field(cp);
              cp.alpha = frac;
              cp.drag = Some(ColorPickerDragTarget::AlphaBar);
              let path = cp.path.clone();
              let (r, g, b) = color_picker_overlay::hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
              let a = cp.alpha;
              wgpu_html_tree::set_color_value(tree, &path, r, g, b, a);
            }
            return true;
          }
          color_picker_overlay::ColorPickerHit::Field(field) => {
            if let Some(cp) = &mut tree.interaction.color_picker {
              color_picker_overlay::activate_field(cp, field);
            }
            return true;
          }
          color_picker_overlay::ColorPickerHit::Background => {
            if let Some(cp) = &mut tree.interaction.color_picker {
              color_picker_overlay::deactivate_field(cp);
            }
            return true;
          }
        }
      } else {
        tree.interaction.color_picker = None;
      }
    }

    // Date picker: intercept clicks when open.
    if let Some(ref dp) = tree.interaction.date_picker.clone() {
      if let Some(hit) = date_picker_overlay::hit_test(dp, pos) {
        match hit {
          date_picker_overlay::DatePickerHit::PrevMonth => {
            if let Some(dp) = &mut tree.interaction.date_picker {
              let (y, m) = wgpu_html_tree::date::prev_month(dp.view_year, dp.view_month);
              dp.view_year = y;
              dp.view_month = m;
            }
            return true;
          }
          date_picker_overlay::DatePickerHit::NextMonth => {
            if let Some(dp) = &mut tree.interaction.date_picker {
              let (y, m) = wgpu_html_tree::date::next_month(dp.view_year, dp.view_month);
              dp.view_year = y;
              dp.view_month = m;
            }
            return true;
          }
          date_picker_overlay::DatePickerHit::DayCell(row, col) => {
            let first_dow = tree.locale.first_day_of_week();
            let (y, m, d) = date_picker_overlay::resolve_day_cell(dp, row, col, first_dow);
            if let Some(dp) = &mut tree.interaction.date_picker {
              dp.year = y;
              dp.month = m;
              dp.day = d;
              dp.view_year = y;
              dp.view_month = m;
              let path = dp.path.clone();
              let val = if dp.has_time {
                wgpu_html_tree::date::format_datetime_local(y, m, d, dp.hour, dp.minute)
              } else {
                wgpu_html_tree::date::format_date(y, m, d)
              };
              wgpu_html_tree::set_date_value(tree, &path, &val);
            }
            return true;
          }
          date_picker_overlay::DatePickerHit::HourField => {
            if let Some(dp) = &mut tree.interaction.date_picker {
              dp.hour = (dp.hour + 1) % 24;
              let path = dp.path.clone();
              let val = wgpu_html_tree::date::format_datetime_local(dp.year, dp.month, dp.day, dp.hour, dp.minute);
              wgpu_html_tree::set_date_value(tree, &path, &val);
            }
            return true;
          }
          date_picker_overlay::DatePickerHit::MinuteField => {
            if let Some(dp) = &mut tree.interaction.date_picker {
              dp.minute = (dp.minute + 1) % 60;
              let path = dp.path.clone();
              let val = wgpu_html_tree::date::format_datetime_local(dp.year, dp.month, dp.day, dp.hour, dp.minute);
              wgpu_html_tree::set_date_value(tree, &path, &val);
            }
            return true;
          }
          date_picker_overlay::DatePickerHit::Reset => {
            if let Some(dp) = &mut tree.interaction.date_picker {
              dp.year = 0;
              dp.month = 0;
              dp.day = 0;
              dp.hour = 0;
              dp.minute = 0;
              let path = dp.path.clone();
              wgpu_html_tree::set_date_value(tree, &path, "");
            }
            return true;
          }
          date_picker_overlay::DatePickerHit::Background => {
            return true;
          }
        }
      } else {
        tree.interaction.date_picker = None;
      }
    }
  }

  let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets);
  let cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets);
  let result = tree.dispatch_mouse_down(target.as_deref(), pos, button, cursor.clone());

  // After focus is set on a form control, position the edit caret
  // at the clicked glyph. Walk the layout tree to find the form
  // control's text run and convert glyph_index → byte_offset.
  if button == MouseButton::Primary {
    if tree.interaction.edit_cursor.is_some() {
      if let Some(focus_path) = tree.interaction.focus_path.clone() {
        // Read the actual value length to distinguish
        // placeholder (empty value) from typed content.
        let value = field_value(tree, &focus_path).unwrap_or_default();
        let value_len = value.len();

        let byte_offset = if value_len == 0 {
          // Field is empty (showing placeholder) — caret
          // goes to position 0, not inside the placeholder.
          0
        } else if let Some(text_box) = crate::layout_at_path(layout, &focus_path) {
          if let Some(run) = &text_box.text_run {
            let click_x = pos.0 - text_box.content_rect.x;
            let glyph_idx = run
              .glyphs
              .iter()
              .position(|g| g.x + g.w * 0.5 > click_x)
              .unwrap_or(run.glyphs.len());
            if glyph_idx < run.byte_boundaries.len() {
              run.byte_boundaries[glyph_idx]
            } else {
              value_len
            }
          } else {
            0
          }
        } else {
          0
        };
        tree.interaction.edit_cursor = Some(edit_cursor_for_click_count(&value, byte_offset, click_count));
        tree.interaction.caret_blink_epoch = std::time::Instant::now();
      }
    } else if let Some(cursor) = cursor.as_ref() {
      if click_count >= 3 {
        crate::select_line_at_cursor(tree, layout, cursor);
      } else if click_count == 2 {
        crate::select_word_at_cursor(tree, layout, cursor);
      }
    }

    // Range slider: start drag and set initial value from click position.
    // Color input: open picker popup on click.
    if let Some(target_path) = &target {
      if let Some(lb) = crate::layout_at_path(layout, target_path) {
        if let Some(ref fc) = lb.form_control {
          match fc.kind {
            FormControlKind::Range { min, max, .. } => {
              let cr = lb.content_rect;
              let frac = if cr.w > 0.0 {
                ((pos.0 - cr.x) / cr.w).clamp(0.0, 1.0)
              } else {
                0.0
              };
              wgpu_html_tree::set_range_value_by_fraction(tree, target_path, frac);
              tree.interaction.range_drag = Some(RangeDrag {
                path: target_path.clone(),
                content_x: cr.x,
                content_w: cr.w,
                min,
                max,
              });
            }
            FormControlKind::Color { r, g, b, a } => {
              let already_open = tree.interaction.color_picker.as_ref()
                .is_some_and(|cp| cp.path == *target_path);
              if already_open {
                tree.interaction.color_picker = None;
              } else {
                let (sr, sg, sb) = (
                  wgpu_html_layout::color::linear_to_srgb(r),
                  wgpu_html_layout::color::linear_to_srgb(g),
                  wgpu_html_layout::color::linear_to_srgb(b),
                );
                let (h, s, v) = color_picker_overlay::srgb_to_hsv(sr, sg, sb);
                let br = lb.border_rect;
                let mut cp = ColorPickerState {
                  path: target_path.clone(),
                  hue: h,
                  saturation: s,
                  value: v,
                  alpha: a,
                  drag: None,
                  popup_rect: [0.0; 4],
                  canvas_rect: [0.0; 4],
                  hue_rect: [0.0; 4],
                  alpha_rect: [0.0; 4],
                  rgba_field_rect: [0.0; 4],
                  hex_field_rect: [0.0; 4],
                  style_bg: lb.lui.picker_bg,
                  style_border: lb.lui.picker_border,
                  style_indicator: lb.lui.picker_indicator,
                  style_label: lb.lui.picker_label,
                  active_field: None,
                  field_text: String::new(),
                  field_cursor: wgpu_html_tree::EditCursor::collapsed(0),
                  field_blink_epoch: std::time::Instant::now(),
                };
                let vw = layout.border_rect.w;
                let vh = layout.border_rect.h;
                color_picker_overlay::compute_popup_rects(
                  &mut cp, br.x, br.y, br.h, 1.0, vw, vh,
                );
                tree.interaction.color_picker = Some(cp);
              }
            }
            FormControlKind::Date { year, month, day } | FormControlKind::DatetimeLocal { year, month, day, .. } => {
              let already_open = tree.interaction.date_picker.as_ref()
                .is_some_and(|dp| dp.path == *target_path);
              if already_open {
                tree.interaction.date_picker = None;
              } else {
                let has_time = matches!(fc.kind, FormControlKind::DatetimeLocal { .. });
                let (hour, minute) = if let FormControlKind::DatetimeLocal { hour, minute, .. } = fc.kind {
                  (hour, minute)
                } else {
                  (0, 0)
                };
                let today = date_picker_overlay::today_ymd_pub();
                let (vy, vm) = if month >= 1 && month <= 12 { (year, month) } else { (today.0, today.1) };
                let br = lb.border_rect;
                let mut dp = DatePickerState {
                  path: target_path.clone(),
                  year, month, day,
                  hour, minute,
                  has_time,
                  view_year: vy, view_month: vm,
                  popup_rect: [0.0; 4],
                  header_rect: [0.0; 4],
                  prev_btn_rect: [0.0; 4],
                  next_btn_rect: [0.0; 4],
                  grid_rect: [0.0; 4],
                  hour_rect: [0.0; 4],
                  minute_rect: [0.0; 4],
                  reset_btn_rect: [0.0; 4],
                  style_bg: lb.lui.calendar_bg,
                  style_border: lb.lui.calendar_border,
                  style_text: lb.lui.calendar_text,
                  style_dim: lb.lui.calendar_dim,
                  style_selected: lb.lui.calendar_selected,
                  style_today: lb.lui.calendar_today,
                };
                let vw = layout.border_rect.w;
                let vh = layout.border_rect.h;
                date_picker_overlay::compute_popup_rects(&mut dp, br.x, br.y, br.h, vw, vh);
                tree.interaction.date_picker = Some(dp);
              }
            }
            _ => {}
          }
        }
      }
    }
  }

  result
}

fn field_value(tree: &Tree, focus_path: &[usize]) -> Option<String> {
  tree
    .root
    .as_ref()
    .and_then(|r| r.at_path(focus_path))
    .and_then(|node| match &node.element {
      wgpu_html_tree::Element::Input(inp) => Some(inp.value.as_deref().unwrap_or_default().to_string()),
      wgpu_html_tree::Element::Textarea(ta) => Some(ta.value.as_deref().unwrap_or_default().to_string()),
      _ => None,
    })
}

pub fn edit_cursor_for_click_count(value: &str, byte_offset: usize, click_count: u8) -> wgpu_html_tree::EditCursor {
  if click_count >= 3 {
    let (start, end) = line_byte_range(value, byte_offset);
    wgpu_html_tree::EditCursor {
      cursor: end,
      selection_anchor: Some(start),
    }
  } else if click_count == 2 {
    let (start, end) = word_byte_range(value, byte_offset);
    wgpu_html_tree::EditCursor {
      cursor: end,
      selection_anchor: Some(start),
    }
  } else {
    wgpu_html_tree::EditCursor::collapsed(byte_offset)
  }
}

fn word_byte_range(value: &str, byte_offset: usize) -> (usize, usize) {
  let chars: Vec<(usize, usize, char)> = value
    .char_indices()
    .map(|(start, ch)| (start, start + ch.len_utf8(), ch))
    .collect();
  if chars.is_empty() {
    return (0, 0);
  }
  let mut idx = chars
    .iter()
    .position(|(_, end, _)| *end >= byte_offset)
    .unwrap_or(chars.len() - 1);
  if idx > 0 && chars[idx].0 >= byte_offset {
    idx -= 1;
  }
  let kind = edit_token_kind(chars[idx].2);
  let mut start = idx;
  while start > 0 && edit_token_kind(chars[start - 1].2) == kind {
    start -= 1;
  }
  let mut end = idx + 1;
  while end < chars.len() && edit_token_kind(chars[end].2) == kind {
    end += 1;
  }
  (chars[start].0, chars[end - 1].1)
}

fn line_byte_range(value: &str, byte_offset: usize) -> (usize, usize) {
  let pos = byte_offset.min(value.len());
  let start = value[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
  let end = value[pos..].find('\n').map(|i| pos + i).unwrap_or(value.len());
  (start, end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditTokenKind {
  Word,
  Whitespace,
  Punctuation(char),
}

fn edit_token_kind(ch: char) -> EditTokenKind {
  if ch.is_alphanumeric() || ch == '_' {
    EditTokenKind::Word
  } else if ch.is_whitespace() {
    EditTokenKind::Whitespace
  } else {
    EditTokenKind::Punctuation(ch)
  }
}

/// Mouse-up at `pos`. Fires `on_mouse_up`; then, if `button` is
/// `Primary` and the release path shares its root with the press
/// path, synthesises a click and fires `on_click` bubbling.
pub fn mouse_up(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool {
  if button == MouseButton::Primary {
    tree.interaction.range_drag = None;
    if let Some(cp) = &mut tree.interaction.color_picker {
      cp.drag = None;
    }
  }
  let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets);
  let cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets);
  tree.dispatch_mouse_up(target.as_deref(), pos, button, cursor)
}

fn update_color_picker_drag(tree: &mut Tree, pos: (f32, f32), drag: ColorPickerDragTarget) -> bool {
  let cp = match &mut tree.interaction.color_picker {
    Some(cp) => cp,
    None => return false,
  };
  match drag {
    ColorPickerDragTarget::Canvas => {
      let [cx, cy, cw, ch] = cp.canvas_rect;
      cp.saturation = ((pos.0 - cx) / cw).clamp(0.0, 1.0);
      cp.value = (1.0 - (pos.1 - cy) / ch).clamp(0.0, 1.0);
    }
    ColorPickerDragTarget::HueBar => {
      let [hx, _, hw, _] = cp.hue_rect;
      cp.hue = ((pos.0 - hx) / hw).clamp(0.0, 1.0) * 360.0;
    }
    ColorPickerDragTarget::AlphaBar => {
      let [ax, _, aw, _] = cp.alpha_rect;
      cp.alpha = ((pos.0 - ax) / aw).clamp(0.0, 1.0);
    }
  }
  let path = cp.path.clone();
  let (r, g, b) = color_picker_overlay::hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
  let a = cp.alpha;
  wgpu_html_tree::set_color_value(tree, &path, r, g, b, a);
  true
}
