use lui_layout::LayoutBox;
use lui_models::common::css_enums::*;
use lui_models::{ArcStr, Div, Span};
use lui_renderer_wgpu::{DisplayList, Rect};
use lui_style::{CascadedNode, CascadedTree};
use lui_text::TextContext;
use lui_tree::date;
use lui_tree::{DatePickerState, Element, Tree};

const CELL_SIZE: f32 = 30.0;


fn cn(element: Element, style: lui_models::Style, children: Vec<CascadedNode>) -> CascadedNode {
  CascadedNode {
    element,
    style,
    children,
    before: None,
    after: None,
    first_line: None,
    first_letter: None,
    placeholder: None,
    selection: None,
    marker: None,
    lui_pseudo: vec![],
  }
}

fn text_node(text: &str, color: &CssColor, font_size: f32) -> CascadedNode {
  let mut s = lui_models::Style::default();
  s.color = Some(color.clone());
  s.font_size = Some(CssLength::Px(font_size));
  cn(Element::Text(ArcStr::from(text)), s, vec![])
}

fn merge_popup_style(base: &mut lui_models::Style, popup: Option<&lui_models::LuiPopupStyle>) {
  let Some(ps) = popup else { return };
  if ps.width.is_some() { base.width = ps.width.clone(); }
  if ps.height.is_some() { base.height = ps.height.clone(); }
  if ps.background_color.is_some() { base.background_color = ps.background_color.clone(); }
  if ps.color.is_some() { base.color = ps.color.clone(); }
  if ps.font_size.is_some() { base.font_size = ps.font_size.clone(); }
  if ps.font_family.is_some() { base.font_family = ps.font_family.clone(); }
  if ps.font_weight.is_some() { base.font_weight = ps.font_weight.clone(); }
  macro_rules! border {
    ($w:ident, $s:ident, $c:ident) => {
      if ps.$w.is_some() { base.$w = ps.$w.clone(); }
      if ps.$s.is_some() { base.$s = ps.$s.clone(); }
      if ps.$c.is_some() { base.$c = ps.$c.clone(); }
    }
  }
  border!(border_top_width, border_top_style, border_top_color);
  border!(border_right_width, border_right_style, border_right_color);
  border!(border_bottom_width, border_bottom_style, border_bottom_color);
  border!(border_left_width, border_left_style, border_left_color);
  if let Some(r) = &ps.border_radius {
    base.border_top_left_radius = Some(r.clone());
    base.border_top_right_radius = Some(r.clone());
    base.border_bottom_left_radius = Some(r.clone());
    base.border_bottom_right_radius = Some(r.clone());
  }
}

fn set_border_all(s: &mut lui_models::Style, w: f32, color: CssColor) {
  let pw = Some(CssLength::Px(w));
  s.border_top_width = pw.clone();
  s.border_right_width = pw.clone();
  s.border_bottom_width = pw.clone();
  s.border_left_width = pw;
  s.border_top_style = Some(BorderStyle::Solid);
  s.border_right_style = Some(BorderStyle::Solid);
  s.border_bottom_style = Some(BorderStyle::Solid);
  s.border_left_style = Some(BorderStyle::Solid);
  s.border_top_color = Some(color.clone());
  s.border_right_color = Some(color.clone());
  s.border_bottom_color = Some(color.clone());
  s.border_left_color = Some(color);
}

fn set_radius_all(s: &mut lui_models::Style, r: f32) {
  let v = Some(CssLength::Px(r));
  s.border_top_left_radius = v.clone();
  s.border_top_right_radius = v.clone();
  s.border_bottom_left_radius = v.clone();
  s.border_bottom_right_radius = v;
}

fn build_date_picker_tree(dp: &DatePickerState, tree: &Tree) -> CascadedTree {
  let popup_ps = dp.popup_style.as_deref();
  let cal_ps = dp.calendar_style.as_deref();
  let locale = &tree.locale;

  let text_color = popup_ps.and_then(|p| p.color.clone())
    .unwrap_or(CssColor::Rgba(217, 217, 217, 1.0));
  let dim_color = cal_ps.and_then(|c| c.dim.clone())
    .unwrap_or(CssColor::Rgba(115, 115, 115, 1.0));
  let font_size = 12.0;
  let small_font = 10.0;

  // ── Popup container ──
  let mut ps = lui_models::Style::default();
  ps.display = Some(Display::Flex);
  ps.flex_direction = Some(FlexDirection::Column);
  ps.width = Some(CssLength::Px(CELL_SIZE * 7.0 + 16.0));
  ps.background_color = Some(CssColor::Rgba(38, 38, 38, 0.96));
  ps.color = Some(text_color.clone());
  set_border_all(&mut ps, 1.0, CssColor::Rgba(77, 77, 77, 1.0));
  set_radius_all(&mut ps, 6.0);
  ps.padding_top = Some(CssLength::Px(8.0));
  ps.padding_right = Some(CssLength::Px(8.0));
  ps.padding_bottom = Some(CssLength::Px(8.0));
  ps.padding_left = Some(CssLength::Px(8.0));
  ps.gap = Some(CssLength::Px(4.0));
  merge_popup_style(&mut ps, popup_ps);

  // ── Header row: [prev] [month year] [next] ──
  let mut header_s = lui_models::Style::default();
  header_s.display = Some(Display::Flex);
  header_s.flex_direction = Some(FlexDirection::Row);
  header_s.align_items = Some(AlignItems::Center);
  header_s.height = Some(CssLength::Px(28.0));

  let mut nav_s = lui_models::Style::default();
  nav_s.width = Some(CssLength::Px(28.0));
  nav_s.height = Some(CssLength::Px(28.0));
  nav_s.display = Some(Display::Flex);
  nav_s.justify_content = Some(JustifyContent::Center);
  nav_s.align_items = Some(AlignItems::Center);

  let prev_btn = cn(Element::Div(Div::default()), nav_s.clone(), vec![]);
  let next_btn = cn(Element::Div(Div::default()), nav_s, vec![]);

  let mut title_s = lui_models::Style::default();
  title_s.display = Some(Display::Flex);
  title_s.flex_grow = Some(1.0);
  title_s.justify_content = Some(JustifyContent::Center);
  title_s.align_items = Some(AlignItems::Center);
  let month_name = locale.month_name(dp.view_month);
  let header_text = format!("{month_name} {}", dp.view_year);
  let title_node = cn(Element::Div(Div::default()), title_s, vec![
    text_node(&header_text, &text_color, font_size),
  ]);

  let header_row = cn(Element::Div(Div::default()), header_s, vec![prev_btn, title_node, next_btn]);

  // ── Weekday header row ──
  let mut wd_row_s = lui_models::Style::default();
  wd_row_s.display = Some(Display::Flex);
  wd_row_s.flex_direction = Some(FlexDirection::Row);
  let first_dow = locale.first_day_of_week();
  let wd_children: Vec<CascadedNode> = (0..7u8).map(|i| {
    let dow = (first_dow + i) % 7;
    let label = locale.weekday_short(dow);
    let mut cell_s = lui_models::Style::default();
    cell_s.width = Some(CssLength::Px(CELL_SIZE));
    cell_s.height = Some(CssLength::Px(20.0));
    cell_s.display = Some(Display::Flex);
    cell_s.justify_content = Some(JustifyContent::Center);
    cell_s.align_items = Some(AlignItems::Center);
    cn(Element::Div(Div::default()), cell_s, vec![
      text_node(label, &dim_color, small_font),
    ])
  }).collect();
  let wd_row = cn(Element::Div(Div::default()), wd_row_s, wd_children);

  // ── Day grid (6 rows × 7 cells) ──
  let today = today_ymd();
  let first_dow_of_month = date::day_of_week(dp.view_year, dp.view_month, 1);
  let offset = ((first_dow_of_month as i32 - first_dow as i32) + 7) % 7;
  let days_this = date::days_in_month(dp.view_year, dp.view_month);
  let (pvy, pvm) = date::prev_month(dp.view_year, dp.view_month);
  let days_prev = date::days_in_month(pvy, pvm);

  let mut grid_rows: Vec<CascadedNode> = Vec::with_capacity(6);
  for row in 0..6u8 {
    let mut row_s = lui_models::Style::default();
    row_s.display = Some(Display::Flex);
    row_s.flex_direction = Some(FlexDirection::Row);

    let cells: Vec<CascadedNode> = (0..7u8).map(|col| {
      let cell_idx = row as i32 * 7 + col as i32 - offset as i32;
      let (d_year, d_month, d_day, is_current) = if cell_idx < 0 {
        (pvy, pvm, (days_prev as i32 + cell_idx + 1) as u8, false)
      } else if cell_idx >= days_this as i32 {
        let (ny, nm) = date::next_month(dp.view_year, dp.view_month);
        (ny, nm, (cell_idx - days_this as i32 + 1) as u8, false)
      } else {
        (dp.view_year, dp.view_month, cell_idx as u8 + 1, true)
      };

      let is_selected = d_year == dp.year && d_month == dp.month && d_day == dp.day;
      let is_today = d_year == today.0 && d_month == today.1 && d_day == today.2;

      let cell_color = if is_selected {
        CssColor::Rgba(255, 255, 255, 1.0)
      } else if is_current {
        text_color.clone()
      } else {
        dim_color.clone()
      };

      let mut cell_s = lui_models::Style::default();
      cell_s.width = Some(CssLength::Px(CELL_SIZE));
      cell_s.height = Some(CssLength::Px(CELL_SIZE));
      cell_s.display = Some(Display::Flex);
      cell_s.justify_content = Some(JustifyContent::Center);
      cell_s.align_items = Some(AlignItems::Center);
      set_radius_all(&mut cell_s, CELL_SIZE / 2.0);

      if is_selected {
        let sel_bg = cal_ps.and_then(|c| c.selected_bg.clone()).unwrap_or(CssColor::Rgba(59, 130, 245, 1.0));
        cell_s.background_color = Some(sel_bg);
      } else if is_today {
        let today_c = cal_ps.and_then(|c| c.today_color.clone()).unwrap_or(CssColor::Rgba(102, 153, 255, 1.0));
        set_border_all(&mut cell_s, 1.0, today_c);
      }

      let label = format!("{d_day}");
      cn(Element::Div(Div::default()), cell_s, vec![text_node(&label, &cell_color, small_font)])
    }).collect();

    grid_rows.push(cn(Element::Div(Div::default()), row_s, cells));
  }

  let mut grid_s = lui_models::Style::default();
  grid_s.display = Some(Display::Flex);
  grid_s.flex_direction = Some(FlexDirection::Column);
  let grid = cn(Element::Div(Div::default()), grid_s, grid_rows);

  // ── Time row (datetime-local only) ──
  let time_row = if dp.has_time {
    let mut tr_s = lui_models::Style::default();
    tr_s.display = Some(Display::Flex);
    tr_s.flex_direction = Some(FlexDirection::Row);
    tr_s.justify_content = Some(JustifyContent::Center);
    tr_s.align_items = Some(AlignItems::Center);
    tr_s.gap = Some(CssLength::Px(4.0));

    let make_time_field = |val: u8| -> CascadedNode {
      let mut fs = lui_models::Style::default();
      fs.width = Some(CssLength::Px(40.0));
      fs.height = Some(CssLength::Px(24.0));
      fs.display = Some(Display::Flex);
      fs.justify_content = Some(JustifyContent::Center);
      fs.align_items = Some(AlignItems::Center);
      fs.background_color = Some(CssColor::Rgba(26, 26, 26, 1.0));
      set_border_all(&mut fs, 1.0, CssColor::Rgba(89, 89, 89, 1.0));
      set_radius_all(&mut fs, 3.0);
      cn(Element::Div(Div::default()), fs, vec![
        text_node(&format!("{val:02}"), &text_color, font_size),
      ])
    };

    let hour_field = make_time_field(dp.hour);
    let colon = cn(Element::Span(Span::default()), lui_models::Style::default(), vec![
      text_node(":", &text_color, font_size),
    ]);
    let minute_field = make_time_field(dp.minute);
    Some(cn(Element::Div(Div::default()), tr_s, vec![hour_field, colon, minute_field]))
  } else {
    None
  };

  // ── Reset button ──
  let mut rb_s = lui_models::Style::default();
  rb_s.height = Some(CssLength::Px(24.0));
  rb_s.display = Some(Display::Flex);
  rb_s.justify_content = Some(JustifyContent::Center);
  rb_s.align_items = Some(AlignItems::Center);
  set_border_all(&mut rb_s, 1.0, CssColor::Rgba(77, 77, 77, 1.0));
  set_radius_all(&mut rb_s, 3.0);
  let reset_label = locale.date_picker_reset_label();
  let reset_btn = cn(Element::Div(Div::default()), rb_s, vec![
    text_node(reset_label, &dim_color, small_font),
  ]);

  // ── Assemble ──
  let mut children = vec![header_row, wd_row, grid];
  if let Some(tr) = time_row { children.push(tr); }
  children.push(reset_btn);

  let popup = cn(Element::Div(Div::default()), ps, children);
  CascadedTree { root: Some(popup) }
}

fn translate_layout_box(b: &mut LayoutBox, dx: f32, dy: f32) {
  b.margin_rect.x += dx;
  b.margin_rect.y += dy;
  b.border_rect.x += dx;
  b.border_rect.y += dy;
  b.content_rect.x += dx;
  b.content_rect.y += dy;
  b.background_rect.x += dx;
  b.background_rect.y += dy;
  for child in &mut b.children {
    translate_layout_box(child, dx, dy);
  }
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

  let [popup_x, popup_y, _, _] = dp.popup_rect;

  let picker_tree = build_date_picker_tree(dp, tree);
  let mut image_cache = lui_layout::ImageCache::default();
  let Some(mut layout) = lui_layout::layout_with_text(
    &picker_tree, text_ctx, &mut image_cache, 4096.0, 4096.0, 1.0,
  ) else { return };

  translate_layout_box(&mut layout, popup_x, popup_y);

  let saved_canvas_color = list.canvas_color;
  crate::paint::paint_layout_with_selection(&layout, list, None, lui_tree::SelectionColors::default(), 0.0);
  list.canvas_color = saved_canvas_color;

  // Overlay chevron arrows on nav buttons.
  // Layout: popup → [header_row, wd_row, grid, (time_row?), reset_btn]
  //         header_row → [prev_btn, title, next_btn]
  if let Some(header_row) = layout.children.first() {
    if header_row.children.len() >= 3 {
      let dim = resolve_dim_color(dp);
      paint_chevron(list, &header_row.children[0].border_rect, dim, true);
      paint_chevron(list, &header_row.children[2].border_rect, dim, false);
    }
  }
}

fn resolve_dim_color(dp: &DatePickerState) -> [f32; 4] {
  dp.calendar_style.as_ref()
    .and_then(|c| c.dim.as_ref())
    .and_then(|c| lui_layout::color::resolve_color(c))
    .unwrap_or([0.45, 0.45, 0.45, 1.0])
}

fn paint_chevron(list: &mut DisplayList, br: &lui_layout::Rect, color: [f32; 4], left: bool) {
  let sz = br.h.min(br.w) * 0.3;
  let t = (sz * 0.18).max(1.2);
  let cx = br.x + br.w / 2.0;
  let cy = br.y + br.h / 2.0;
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

// ── Hit testing / popup rects (unchanged) ──

pub fn compute_popup_rects(
  dp: &mut DatePickerState,
  swatch_x: f32, swatch_y: f32, swatch_h: f32,
  viewport_w: f32, viewport_h: f32,
) {
  let pw = CELL_SIZE * 7.0 + 16.0 + 2.0;
  let header_h = 28.0;
  let wd_h = 20.0;
  let gap = 4.0;
  let pad = 8.0;
  let reset_h = 24.0;
  let time_h = if dp.has_time { 24.0 + gap } else { 0.0 };
  let ph = pad + header_h + gap + wd_h + gap + CELL_SIZE * 6.0 + gap + time_h + reset_h + pad + 2.0;

  let mut px = swatch_x;
  let mut py = swatch_y + swatch_h + 4.0;
  if px + pw > viewport_w { px = (viewport_w - pw - 4.0).max(0.0); }
  if py + ph > viewport_h { py = (swatch_y - ph - 4.0).max(0.0); }

  dp.popup_rect = [px, py, pw, ph];

  let cx = px + pad + 1.0;
  let cy = py + pad + 1.0;

  let btn_w = 28.0;
  dp.prev_btn_rect = [cx, cy, btn_w, header_h];
  dp.next_btn_rect = [cx + CELL_SIZE * 7.0 - btn_w, cy, btn_w, header_h];
  dp.header_rect = [cx + btn_w, cy, CELL_SIZE * 7.0 - btn_w * 2.0, header_h];

  let gy = cy + header_h + gap + wd_h + gap;
  dp.grid_rect = [cx, gy, CELL_SIZE * 7.0, CELL_SIZE * 6.0];

  let mut bottom_y = gy + CELL_SIZE * 6.0 + gap;

  if dp.has_time {
    let fw = 40.0;
    let total = fw * 2.0 + 12.0;
    let tx = cx + (CELL_SIZE * 7.0 - total) / 2.0;
    dp.hour_rect = [tx, bottom_y, fw, 24.0];
    dp.minute_rect = [tx + fw + 12.0, bottom_y, fw, 24.0];
    bottom_y += 24.0 + gap;
  }

  dp.reset_btn_rect = [cx, bottom_y, CELL_SIZE * 7.0, reset_h];
}

pub fn hit_test(dp: &DatePickerState, pos: (f32, f32)) -> Option<DatePickerHit> {
  let (mx, my) = pos;
  let [px, py, pw, ph] = dp.popup_rect;
  if mx < px || mx > px + pw || my < py || my > py + ph { return None; }

  let [bx, by, bw, bh] = dp.prev_btn_rect;
  if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh { return Some(DatePickerHit::PrevMonth); }
  let [bx, by, bw, bh] = dp.next_btn_rect;
  if mx >= bx && mx <= bx + bw && my >= by && my <= by + bh { return Some(DatePickerHit::NextMonth); }

  let [gx, gy, gw, gh] = dp.grid_rect;
  if mx >= gx && mx <= gx + gw && my >= gy && my <= gy + gh {
    let col = ((mx - gx) / CELL_SIZE) as u8;
    let row = ((my - gy) / CELL_SIZE) as u8;
    if col < 7 && row < 6 { return Some(DatePickerHit::DayCell(row, col)); }
  }

  if dp.has_time {
    let [hx, hy, hw, hh] = dp.hour_rect;
    if mx >= hx && mx <= hx + hw && my >= hy && my <= hy + hh { return Some(DatePickerHit::HourField); }
    let [mx2, my2, mw, mh] = dp.minute_rect;
    if mx >= mx2 && mx <= mx2 + mw && my >= my2 && my <= my2 + mh { return Some(DatePickerHit::MinuteField); }
  }

  let [rbx, rby, rbw, rbh] = dp.reset_btn_rect;
  if mx >= rbx && mx <= rbx + rbw && my >= rby && my <= rby + rbh { return Some(DatePickerHit::Reset); }

  Some(DatePickerHit::Background)
}

pub fn resolve_day_cell(dp: &DatePickerState, row: u8, col: u8, first_dow: u8) -> (i32, u8, u8) {
  let first_dow_of_month = date::day_of_week(dp.view_year, dp.view_month, 1);
  let offset = ((first_dow_of_month as i32 - first_dow as i32) + 7) % 7;
  let cell_idx = row as i32 * 7 + col as i32 - offset as i32;
  let days_this = date::days_in_month(dp.view_year, dp.view_month);
  if cell_idx < 0 {
    let (py, pm) = date::prev_month(dp.view_year, dp.view_month);
    (py, pm, (date::days_in_month(py, pm) as i32 + cell_idx + 1) as u8)
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
  let secs = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0);
  civil_from_days((secs / 86400) as i32)
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
