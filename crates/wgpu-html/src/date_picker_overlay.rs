use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{DatePickerState, Tree};
use wgpu_html_tree::date;

fn resolve_lui_color(c: &Option<wgpu_html_models::common::css_enums::CssColor>) -> Option<[f32; 4]> {
  c.as_ref().and_then(|c| wgpu_html_layout::color::resolve_color(c))
}

const POPUP_PAD: f32 = 8.0;
const HEADER_H: f32 = 28.0;
const DAY_HEADER_H: f32 = 20.0;
const CELL_SIZE: f32 = 30.0;
const TIME_ROW_H: f32 = 24.0;
const RESET_BTN_H: f32 = 24.0;
const GAP: f32 = 4.0;
const CORNER_R: f32 = 6.0;

const BG: [f32; 4] = [0.15, 0.15, 0.15, 0.96];
const BORDER: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
const TEXT_COLOR: [f32; 4] = [0.85, 0.85, 0.85, 1.0];
const DIM_COLOR: [f32; 4] = [0.45, 0.45, 0.45, 1.0];
const SELECTED_BG: [f32; 4] = [0.23, 0.51, 0.96, 1.0];
const TODAY_BORDER: [f32; 4] = [0.4, 0.6, 1.0, 1.0];
const TIME_FIELD_BG: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

const FONT_SIZE: f32 = 12.0;
const SMALL_FONT: f32 = 10.0;

fn grid_w() -> f32 { CELL_SIZE * 7.0 }
fn popup_h(has_time: bool) -> f32 {
  POPUP_PAD + HEADER_H + GAP + DAY_HEADER_H + CELL_SIZE * 6.0 + GAP
    + if has_time { TIME_ROW_H + GAP } else { 0.0 }
    + RESET_BTN_H + POPUP_PAD
}

pub fn compute_popup_rects(
  dp: &mut DatePickerState,
  swatch_x: f32, swatch_y: f32, swatch_h: f32,
  viewport_w: f32, viewport_h: f32,
) {
  let pw = POPUP_PAD * 2.0 + grid_w();
  let ph = popup_h(dp.has_time);

  let mut px = swatch_x;
  let mut py = swatch_y + swatch_h + 4.0;
  if px + pw > viewport_w { px = (viewport_w - pw - 4.0).max(0.0); }
  if py + ph > viewport_h { py = (swatch_y - ph - 4.0).max(0.0); }

  dp.popup_rect = [px, py, pw, ph];

  let cx = px + POPUP_PAD;
  let cy = py + POPUP_PAD;

  let btn_w = 28.0;
  dp.prev_btn_rect = [cx, cy, btn_w, HEADER_H];
  dp.next_btn_rect = [cx + grid_w() - btn_w, cy, btn_w, HEADER_H];
  dp.header_rect = [cx + btn_w, cy, grid_w() - btn_w * 2.0, HEADER_H];

  let gy = cy + HEADER_H + GAP + DAY_HEADER_H;
  dp.grid_rect = [cx, gy, grid_w(), CELL_SIZE * 6.0];

  let mut bottom_y = gy + CELL_SIZE * 6.0 + GAP;

  if dp.has_time {
    let fw = 40.0;
    let total = fw * 2.0 + 12.0;
    let tx = cx + (grid_w() - total) / 2.0;
    dp.hour_rect = [tx, bottom_y, fw, TIME_ROW_H];
    dp.minute_rect = [tx + fw + 12.0, bottom_y, fw, TIME_ROW_H];
    bottom_y += TIME_ROW_H + GAP;
  }

  dp.reset_btn_rect = [cx, bottom_y, grid_w(), RESET_BTN_H];
}

pub fn paint_date_picker_overlay(
  list: &mut DisplayList,
  _root: &LayoutBox,
  tree: &Tree,
  text_ctx: &mut TextContext,
) {
  let dp = match &tree.interaction.date_picker {
    Some(dp) => dp,
    None => return,
  };

  let [px, py, pw, ph] = dp.popup_rect;

  let bg = resolve_lui_color(&dp.popup_style.background_color).unwrap_or(BG);
  let border = resolve_lui_color(&dp.popup_style.border_top_color).unwrap_or(BORDER);
  let text_c = resolve_lui_color(&dp.popup_style.color).unwrap_or(TEXT_COLOR);
  let dim_c = DIM_COLOR;
  let selected_c = SELECTED_BG;
  let today_c = TODAY_BORDER;

  // Background
  list.push_quad_rounded(Rect::new(px, py, pw, ph), bg, [CORNER_R; 4]);
  list.push_quad_stroke(Rect::new(px, py, pw, ph), border, [CORNER_R; 4], [1.0; 4]);

  let locale = &tree.locale;
  let cx = px + POPUP_PAD;
  let cy = py + POPUP_PAD;

  // Navigation arrows (chevrons drawn from quads)
  paint_chevron(list, dp.prev_btn_rect, dim_c, true);
  paint_chevron(list, dp.next_btn_rect, dim_c, false);

  // Month/year header
  let month_name = locale.month_name(dp.view_month);
  let header = format!("{month_name} {}", dp.view_year);
  paint_centered_text(list, text_ctx, &header, dp.header_rect, text_c, FONT_SIZE);

  // Weekday headers
  let day_y = cy + HEADER_H + GAP;
  let first_dow = locale.first_day_of_week();
  for i in 0..7u8 {
    let dow = (first_dow + i) % 7;
    let label = locale.weekday_short(dow);
    let dx = cx + i as f32 * CELL_SIZE;
    paint_centered_text(list, text_ctx, label, [dx, day_y, CELL_SIZE, DAY_HEADER_H], dim_c, SMALL_FONT);
  }

  // Day grid
  let [gx, gy, _, _] = dp.grid_rect;
  let first_dow_of_month = date::day_of_week(dp.view_year, dp.view_month, 1);
  let offset = ((first_dow_of_month as i32 - first_dow as i32) + 7) % 7;
  let days_this = date::days_in_month(dp.view_year, dp.view_month);
  let (prev_y, prev_m) = date::prev_month(dp.view_year, dp.view_month);
  let days_prev = date::days_in_month(prev_y, prev_m);

  let today = today_ymd();

  for row in 0..6u8 {
    for col in 0..7u8 {
      let cell_idx = row as i32 * 7 + col as i32 - offset as i32;
      let (d_year, d_month, d_day, is_current_month) = if cell_idx < 0 {
        let d = days_prev as i32 + cell_idx + 1;
        (prev_y, prev_m, d as u8, false)
      } else if cell_idx >= days_this as i32 {
        let (ny, nm) = date::next_month(dp.view_year, dp.view_month);
        let d = cell_idx - days_this as i32 + 1;
        (ny, nm, d as u8, false)
      } else {
        (dp.view_year, dp.view_month, cell_idx as u8 + 1, true)
      };

      let cell_x = gx + col as f32 * CELL_SIZE;
      let cell_y = gy + row as f32 * CELL_SIZE;
      let r = CELL_SIZE / 2.0;

      let is_selected = d_year == dp.year && d_month == dp.month && d_day == dp.day;
      let is_today = d_year == today.0 && d_month == today.1 && d_day == today.2;

      if is_selected {
        list.push_quad_rounded(Rect::new(cell_x + 1.0, cell_y + 1.0, CELL_SIZE - 2.0, CELL_SIZE - 2.0), selected_c, [r; 4]);
      } else if is_today {
        list.push_quad_stroke(Rect::new(cell_x + 1.0, cell_y + 1.0, CELL_SIZE - 2.0, CELL_SIZE - 2.0), today_c, [r; 4], [1.0; 4]);
      }

      let color = if is_selected { [1.0, 1.0, 1.0, 1.0] }
        else if is_current_month { text_c }
        else { dim_c };

      let label = format!("{d_day}");
      paint_centered_text(list, text_ctx, &label, [cell_x, cell_y, CELL_SIZE, CELL_SIZE], color, SMALL_FONT);
    }
  }

  // Time fields (datetime-local)
  if dp.has_time {
    let hour_str = format!("{:02}", dp.hour);
    let min_str = format!("{:02}", dp.minute);

    paint_time_field(list, text_ctx, &hour_str, dp.hour_rect, text_c, border);
    paint_time_field(list, text_ctx, &min_str, dp.minute_rect, text_c, border);

    let colon_x = dp.hour_rect[0] + dp.hour_rect[2];
    paint_centered_text(list, text_ctx, ":", [colon_x, dp.hour_rect[1], 12.0, TIME_ROW_H], text_c, FONT_SIZE);
  }

  // Reset button
  let [rbx, rby, rbw, rbh] = dp.reset_btn_rect;
  list.push_quad_stroke(Rect::new(rbx, rby, rbw, rbh), border, [3.0; 4], [1.0; 4]);
  let reset_label = locale.date_picker_reset_label();
  paint_centered_text(list, text_ctx, reset_label, dp.reset_btn_rect, dim_c, SMALL_FONT);
}

fn paint_chevron(list: &mut DisplayList, rect: [f32; 4], color: [f32; 4], left: bool) {
  let [rx, ry, rw, rh] = rect;
  let sz = rh.min(rw) * 0.3;
  let t = (sz * 0.18).max(1.2);
  let cx = rx + rw / 2.0;
  let cy = ry + rh / 2.0;
  let half = sz / 2.0;
  let steps = (sz / (t * 0.4)).ceil().max(4.0) as usize;
  let r = t / 2.0;

  let (tip_x, wing_x) = if left { (cx - half * 0.4, cx + half * 0.4) } else { (cx + half * 0.4, cx - half * 0.4) };
  let top_y = cy - half;
  let bot_y = cy + half;

  for i in 0..=steps {
    let frac = i as f32 / steps as f32;
    let px = wing_x + (tip_x - wing_x) * frac;
    let py = top_y + (cy - top_y) * frac;
    list.push_quad_rounded(Rect::new(px - r, py - r, t, t), color, [r; 4]);
  }
  for i in 0..=steps {
    let frac = i as f32 / steps as f32;
    let px = tip_x + (wing_x - tip_x) * frac;
    let py = cy + (bot_y - cy) * frac;
    list.push_quad_rounded(Rect::new(px - r, py - r, t, t), color, [r; 4]);
  }
}

fn paint_time_field(list: &mut DisplayList, text_ctx: &mut TextContext, text: &str, rect: [f32; 4], text_c: [f32; 4], border_c: [f32; 4]) {
  let [x, y, w, h] = rect;
  list.push_quad_rounded(Rect::new(x, y, w, h), TIME_FIELD_BG, [3.0; 4]);
  list.push_quad_stroke(Rect::new(x, y, w, h), border_c, [3.0; 4], [1.0; 4]);
  paint_centered_text(list, text_ctx, text, rect, text_c, FONT_SIZE);
}

fn paint_centered_text(
  list: &mut DisplayList,
  text_ctx: &mut TextContext,
  text: &str,
  rect: [f32; 4],
  color: [f32; 4],
  font_size: f32,
) {
  let [rx, ry, rw, rh] = rect;
  let families = ["sans-serif"];
  let font = text_ctx.pick_font(&families, 400, wgpu_html_tree::FontStyleAxis::Normal);
  let Some(fh) = font else { return };
  let shaped = text_ctx.shape_and_pack(
    text, fh, font_size, font_size * 1.3, 0.0, 400,
    wgpu_html_tree::FontStyleAxis::Normal, None, color,
  );
  let Some(run) = shaped else { return };
  let text_x = rx + (rw - run.width) / 2.0;
  let text_y = ry + (rh - run.height) / 2.0;
  for g in &run.glyphs {
    list.push_glyph(
      Rect::new((text_x + g.x).round(), (text_y + g.y).round(), g.w, g.h),
      g.color, g.uv_min, g.uv_max,
    );
  }
}

pub fn hit_test(dp: &DatePickerState, pos: (f32, f32)) -> Option<DatePickerHit> {
  let (mx, my) = pos;
  let [px, py, pw, ph] = dp.popup_rect;
  if mx < px || mx > px + pw || my < py || my > py + ph {
    return None;
  }

  let [bx, by, bw, bh] = dp.prev_btn_rect;
  if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
    return Some(DatePickerHit::PrevMonth);
  }
  let [bx, by, bw, bh] = dp.next_btn_rect;
  if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh {
    return Some(DatePickerHit::NextMonth);
  }

  let [gx, gy, gw, gh] = dp.grid_rect;
  if mx >= gx && mx <= gx + gw && my >= gy && my <= gy + gh {
    let col = ((mx - gx) / CELL_SIZE) as u8;
    let row = ((my - gy) / CELL_SIZE) as u8;
    if col < 7 && row < 6 {
      return Some(DatePickerHit::DayCell(row, col));
    }
  }

  if dp.has_time {
    let [hx, hy, hw, hh] = dp.hour_rect;
    if mx >= hx && mx <= hx + hw && my >= hy && my <= hy + hh {
      return Some(DatePickerHit::HourField);
    }
    let [mx2, my2, mw, mh] = dp.minute_rect;
    if mx >= mx2 && mx <= mx2 + mw && my >= my2 && my <= my2 + mh {
      return Some(DatePickerHit::MinuteField);
    }
  }

  let [rbx, rby, rbw, rbh] = dp.reset_btn_rect;
  if mx >= rbx && mx <= rbx + rbw && my >= rby && my <= rby + rbh {
    return Some(DatePickerHit::Reset);
  }

  Some(DatePickerHit::Background)
}

pub fn resolve_day_cell(dp: &DatePickerState, row: u8, col: u8, first_dow: u8) -> (i32, u8, u8) {
  let first_dow_of_month = date::day_of_week(dp.view_year, dp.view_month, 1);
  let offset = ((first_dow_of_month as i32 - first_dow as i32) + 7) % 7;
  let cell_idx = row as i32 * 7 + col as i32 - offset as i32;
  let days_this = date::days_in_month(dp.view_year, dp.view_month);

  if cell_idx < 0 {
    let (py, pm) = date::prev_month(dp.view_year, dp.view_month);
    let d = date::days_in_month(py, pm) as i32 + cell_idx + 1;
    (py, pm, d as u8)
  } else if cell_idx >= days_this as i32 {
    let (ny, nm) = date::next_month(dp.view_year, dp.view_month);
    (ny, nm, (cell_idx - days_this as i32 + 1) as u8)
  } else {
    (dp.view_year, dp.view_month, cell_idx as u8 + 1)
  }
}

#[derive(Debug, Clone, Copy)]
pub enum DatePickerHit {
  PrevMonth,
  NextMonth,
  DayCell(u8, u8),
  HourField,
  MinuteField,
  Reset,
  Background,
}

pub fn today_ymd_pub() -> (i32, u8, u8) { today_ymd() }

fn today_ymd() -> (i32, u8, u8) {
  // std::time only gives duration since UNIX_EPOCH; compute date from days.
  let secs = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0);
  let days = (secs / 86400) as i32;
  civil_from_days(days)
}

fn civil_from_days(mut z: i32) -> (i32, u8, u8) {
  z += 719468;
  let era = if z >= 0 { z } else { z - 146096 } / 146097;
  let doe = (z - era * 146097) as u32;
  let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
  let y = yoe as i32 + era * 400;
  let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
  let mp = (5 * doy + 2) / 153;
  let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
  let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
  let y = if m <= 2 { y + 1 } else { y };
  (y, m, d)
}
