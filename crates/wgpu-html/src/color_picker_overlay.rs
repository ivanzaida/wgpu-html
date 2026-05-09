use std::sync::Arc;

use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{ColorPickerField, ColorPickerState, EditCursor, Tree};

fn resolve_lui_color(c: &Option<wgpu_html_models::common::css_enums::CssColor>) -> Option<[f32; 4]> {
  c.as_ref().and_then(|c| wgpu_html_layout::color::resolve_color(c))
}

const POPUP_W: f32 = 260.0;
const POPUP_PAD: f32 = 10.0;
const CANVAS_SIZE: f32 = 240.0;
const BAR_H: f32 = 14.0;
const GAP: f32 = 8.0;
const FIELD_H: f32 = 20.0;

const BG: [f32; 4] = [0.15, 0.15, 0.15, 0.96];
const LABEL_COLOR: [f32; 4] = [0.85, 0.85, 0.85, 1.0];
const BORDER_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
const INDICATOR_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const INDICATOR_SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.5];
const FIELD_BG: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
const FIELD_BORDER: [f32; 4] = [0.35, 0.35, 0.35, 1.0];
const FIELD_BORDER_FOCUS: [f32; 4] = [0.4, 0.6, 1.0, 1.0];
const CARET_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const SELECTION_BG: [f32; 4] = [0.23, 0.51, 0.96, 0.45];

const SV_TEX_SIZE: u32 = 128;
const HUE_TEX_W: u32 = 256;
const HUE_TEX_H: u32 = 1;

const FONT_SIZE: f32 = 11.0;
const FIELD_PAD_X: f32 = 4.0;

fn popup_height() -> f32 {
  POPUP_PAD + CANVAS_SIZE + GAP + BAR_H + GAP + BAR_H + GAP + FIELD_H + GAP + FIELD_H + POPUP_PAD
}

fn hsv_to_srgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
  let c = v * s;
  let h6 = (h / 60.0).rem_euclid(6.0);
  let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
  let (r1, g1, b1) = match h6 as i32 {
    0 => (c, x, 0.0),
    1 => (x, c, 0.0),
    2 => (0.0, c, x),
    3 => (0.0, x, c),
    4 => (x, 0.0, c),
    _ => (c, 0.0, x),
  };
  let m = v - c;
  (r1 + m, g1 + m, b1 + m)
}

pub fn hsv_to_srgb_u8(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
  let (r, g, b) = hsv_to_srgb(h, s, v);
  (
    (r * 255.0 + 0.5) as u8,
    (g * 255.0 + 0.5) as u8,
    (b * 255.0 + 0.5) as u8,
  )
}

pub fn srgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
  let max = r.max(g).max(b);
  let min = r.min(g).min(b);
  let d = max - min;
  let h = if d < 1e-6 {
    0.0
  } else if (max - r).abs() < 1e-6 {
    60.0 * (((g - b) / d) % 6.0)
  } else if (max - g).abs() < 1e-6 {
    60.0 * ((b - r) / d + 2.0)
  } else {
    60.0 * ((r - g) / d + 4.0)
  };
  let h = if h < 0.0 { h + 360.0 } else { h };
  let s = if max < 1e-6 { 0.0 } else { d / max };
  (h, s, max)
}

fn gen_sv_texture(hue: f32) -> Vec<u8> {
  let mut data = vec![0u8; (SV_TEX_SIZE * SV_TEX_SIZE * 4) as usize];
  for y in 0..SV_TEX_SIZE {
    let v = 1.0 - y as f32 / (SV_TEX_SIZE - 1) as f32;
    for x in 0..SV_TEX_SIZE {
      let s = x as f32 / (SV_TEX_SIZE - 1) as f32;
      let (r, g, b) = hsv_to_srgb(hue, s, v);
      let idx = ((y * SV_TEX_SIZE + x) * 4) as usize;
      data[idx] = (r * 255.0 + 0.5) as u8;
      data[idx + 1] = (g * 255.0 + 0.5) as u8;
      data[idx + 2] = (b * 255.0 + 0.5) as u8;
      data[idx + 3] = 255;
    }
  }
  data
}

fn gen_hue_texture() -> Vec<u8> {
  let mut data = vec![0u8; (HUE_TEX_W * HUE_TEX_H * 4) as usize];
  for x in 0..HUE_TEX_W {
    let h = x as f32 / (HUE_TEX_W - 1) as f32 * 360.0;
    let (r, g, b) = hsv_to_srgb(h, 1.0, 1.0);
    let idx = (x * 4) as usize;
    data[idx] = (r * 255.0 + 0.5) as u8;
    data[idx + 1] = (g * 255.0 + 0.5) as u8;
    data[idx + 2] = (b * 255.0 + 0.5) as u8;
    data[idx + 3] = 255;
  }
  data
}

fn gen_alpha_texture(r: u8, g: u8, b: u8) -> Vec<u8> {
  let mut data = vec![0u8; (HUE_TEX_W * 2 * 4) as usize];
  let checker_size = 8u32;
  for y in 0..2u32 {
    for x in 0..HUE_TEX_W {
      let alpha_f = x as f32 / (HUE_TEX_W - 1) as f32;
      let checker = ((x / checker_size) + (y / 1)) % 2 == 0;
      let bg = if checker { 204u8 } else { 153u8 };
      let out_r = (r as f32 * alpha_f + bg as f32 * (1.0 - alpha_f) + 0.5) as u8;
      let out_g = (g as f32 * alpha_f + bg as f32 * (1.0 - alpha_f) + 0.5) as u8;
      let out_b = (b as f32 * alpha_f + bg as f32 * (1.0 - alpha_f) + 0.5) as u8;
      let idx = ((y * HUE_TEX_W + x) * 4) as usize;
      data[idx] = out_r;
      data[idx + 1] = out_g;
      data[idx + 2] = out_b;
      data[idx + 3] = 255;
    }
  }
  data
}

fn image_id_sv(hue_quantized: u16) -> u64 {
  0xC010_0000_0000_0000 | hue_quantized as u64
}

const IMAGE_ID_HUE: u64 = 0xC010_0000_FFFF_0001;

fn image_id_alpha(r: u8, g: u8, b: u8) -> u64 {
  0xC010_0001_0000_0000 | (r as u64) << 16 | (g as u64) << 8 | b as u64
}

pub fn rgba_string(cp: &ColorPickerState) -> String {
  let (r, g, b) = hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
  let a = (cp.alpha * 255.0 + 0.5) as u8;
  format!("rgba({r}, {g}, {b}, {a})")
}

pub fn hex_string(cp: &ColorPickerState) -> String {
  let (r, g, b) = hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
  let a = (cp.alpha * 255.0 + 0.5) as u8;
  if a == 255 {
    format!("#{r:02x}{g:02x}{b:02x}")
  } else {
    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
  }
}

pub fn paint_color_picker_overlay(
  list: &mut DisplayList,
  _root: &LayoutBox,
  tree: &Tree,
  text_ctx: &mut TextContext,
  _scroll_y: f32,
  _scale: f32,
  _viewport_w: f32,
  _viewport_h: f32,
) {
  let cp = match &tree.interaction.color_picker {
    Some(cp) => cp,
    None => return,
  };

  let [popup_x, popup_y, pw, ph] = cp.popup_rect;
  let [cx, cy, canvas_w, canvas_h] = cp.canvas_rect;
  let [hx, hy, hue_w, hue_h] = cp.hue_rect;
  let [ax, ay, alpha_w, alpha_h] = cp.alpha_rect;

  let corner_r = 6.0;
  let ind_r = 5.0;
  let slider_knob_w = 6.0;
  let slider_knob_r = 2.0;

  let bg = cp.popup_style.as_ref().and_then(|s| resolve_lui_color(&s.background_color)).unwrap_or(BG);
  let border = cp.popup_style.as_ref().and_then(|s| resolve_lui_color(&s.border_top_color)).unwrap_or(BORDER_COLOR);
  let indicator = cp.picker_style.as_ref().and_then(|s| resolve_lui_color(&s.thumb_color)).unwrap_or(INDICATOR_BORDER);
  let label_color = cp.popup_style.as_ref().and_then(|s| resolve_lui_color(&s.color)).unwrap_or(LABEL_COLOR);

  // Background
  list.push_quad_rounded(Rect::new(popup_x, popup_y, pw, ph), bg, [corner_r; 4]);
  list.push_quad_stroke(Rect::new(popup_x, popup_y, pw, ph), border, [corner_r; 4], [1.0; 4]);

  // SV canvas
  let hue_q = (cp.hue * 2.0) as u16;
  let sv_data = gen_sv_texture(cp.hue);
  list.push_image(Rect::new(cx, cy, canvas_w, canvas_h), image_id_sv(hue_q), Arc::new(sv_data), SV_TEX_SIZE, SV_TEX_SIZE);

  // SV crosshair
  let ind_x = cx + cp.saturation * canvas_w;
  let ind_y = cy + (1.0 - cp.value) * canvas_h;
  list.push_quad_stroke(Rect::new(ind_x - ind_r, ind_y - ind_r, ind_r * 2.0, ind_r * 2.0), INDICATOR_SHADOW, [ind_r + 1.0; 4], [2.0; 4]);
  list.push_quad_stroke(Rect::new(ind_x - ind_r, ind_y - ind_r, ind_r * 2.0, ind_r * 2.0), indicator, [ind_r; 4], [1.5; 4]);

  // Hue bar
  let hue_data = gen_hue_texture();
  let bar_r = hue_h / 2.0;
  list.push_clip(Some(Rect::new(hx, hy, hue_w, hue_h)), [bar_r; 4], [bar_r; 4]);
  list.push_image(Rect::new(hx, hy, hue_w, hue_h), IMAGE_ID_HUE, Arc::new(hue_data), HUE_TEX_W, HUE_TEX_H);
  list.pop_clip(None, [0.0; 4], [0.0; 4]);

  let hue_frac = cp.hue / 360.0;
  let hi_x = hx + hue_frac * hue_w;
  list.push_quad_rounded(Rect::new(hi_x - slider_knob_w * 0.5, hy - 1.0, slider_knob_w, hue_h + 2.0), indicator, [slider_knob_r; 4]);

  // Alpha bar
  let (ar, ag, ab) = hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
  let alpha_data = gen_alpha_texture(ar, ag, ab);
  let alpha_bar_r = alpha_h / 2.0;
  list.push_clip(Some(Rect::new(ax, ay, alpha_w, alpha_h)), [alpha_bar_r; 4], [alpha_bar_r; 4]);
  list.push_image(Rect::new(ax, ay, alpha_w, alpha_h), image_id_alpha(ar, ag, ab), Arc::new(alpha_data), HUE_TEX_W, 2);
  list.pop_clip(None, [0.0; 4], [0.0; 4]);

  let ai_x = ax + cp.alpha * alpha_w;
  list.push_quad_rounded(Rect::new(ai_x - slider_knob_w * 0.5, ay - 1.0, slider_knob_w, alpha_h + 2.0), indicator, [slider_knob_r; 4]);

  // RGBA field
  let rgba_text = if cp.active_field == Some(ColorPickerField::Rgba) {
    cp.field_text.clone()
  } else {
    rgba_string(cp)
  };
  paint_field(list, text_ctx, &rgba_text, cp.rgba_field_rect, label_color,
    cp.active_field == Some(ColorPickerField::Rgba), cp);

  // Hex field
  let hex_text = if cp.active_field == Some(ColorPickerField::Hex) {
    cp.field_text.clone()
  } else {
    hex_string(cp)
  };
  paint_field(list, text_ctx, &hex_text, cp.hex_field_rect, label_color,
    cp.active_field == Some(ColorPickerField::Hex), cp);
}

fn paint_field(
  list: &mut DisplayList,
  text_ctx: &mut TextContext,
  text: &str,
  rect: [f32; 4],
  text_color: [f32; 4],
  focused: bool,
  cp: &ColorPickerState,
) {
  let [x, y, w, h] = rect;
  let r = 3.0;

  list.push_quad_rounded(Rect::new(x, y, w, h), FIELD_BG, [r; 4]);
  let bdr = if focused { FIELD_BORDER_FOCUS } else { FIELD_BORDER };
  list.push_quad_stroke(Rect::new(x, y, w, h), bdr, [r; 4], [1.0; 4]);

  let font_size = FONT_SIZE;
  let families = ["monospace", "sans-serif"];
  let font = text_ctx.pick_font(&families, 400, wgpu_html_tree::FontStyleAxis::Normal);
  let Some(font_handle) = font else { return };

  let text_x = x + FIELD_PAD_X;
  let text_max_w = w - FIELD_PAD_X * 2.0;

  let shaped = text_ctx.shape_and_pack(
    text, font_handle, font_size, font_size * 1.3, 0.0, 400,
    wgpu_html_tree::FontStyleAxis::Normal, None, text_color,
  );
  let Some(run) = shaped else { return };

  list.push_clip(Some(Rect::new(x + 1.0, y, w - 2.0, h)), [0.0; 4], [0.0; 4]);

  // Selection highlight
  if focused && cp.field_cursor.has_selection() {
    let (sel_start, sel_end) = cp.field_cursor.selection_range();
    let start_x = byte_to_x(&run, sel_start);
    let end_x = byte_to_x(&run, sel_end);
    let sel_y = y + (h - font_size) / 2.0;
    list.push_quad(Rect::new(text_x + start_x, sel_y, end_x - start_x, font_size), SELECTION_BG);
  }

  // Text glyphs
  for g in &run.glyphs {
    let gx = (text_x + g.x).round();
    let gy = (y + (h - run.height) / 2.0 + g.y).round();
    if gx + g.w > text_x + text_max_w { break; }
    list.push_glyph(Rect::new(gx, gy, g.w, g.h), g.color, g.uv_min, g.uv_max);
  }

  // Caret
  if focused && !cp.field_cursor.has_selection() {
    let elapsed = cp.field_blink_epoch.elapsed().as_millis();
    if (elapsed % 1000) < 500 {
      let caret_x = text_x + byte_to_x(&run, cp.field_cursor.cursor);
      let caret_y = y + (h - font_size) / 2.0;
      list.push_quad(Rect::new(caret_x, caret_y, 1.5, font_size), CARET_COLOR);
    }
  }

  list.pop_clip(None, [0.0; 4], [0.0; 4]);
}

fn byte_to_x(run: &wgpu_html_text::ShapedRun, byte_offset: usize) -> f32 {
  if byte_offset == 0 || run.glyphs.is_empty() {
    return 0.0;
  }
  for (i, &boundary) in run.byte_boundaries.iter().enumerate() {
    if boundary >= byte_offset {
      if i < run.glyphs.len() {
        return run.glyphs[i].x;
      }
      return run.width;
    }
  }
  run.width
}

pub fn compute_popup_rects(cp: &mut ColorPickerState, swatch_x: f32, swatch_y: f32, swatch_h: f32, scale: f32, viewport_w: f32, viewport_h: f32) {
  let s = scale.max(0.5);
  let pw = POPUP_W * s;
  let ph = popup_height() * s;
  let pad = POPUP_PAD * s;
  let canvas_sz = CANVAS_SIZE * s;
  let bar_h = BAR_H * s;
  let gap = GAP * s;
  let field_h = FIELD_H * s;

  let mut px = swatch_x;
  let mut py = swatch_y + swatch_h + 4.0 * s;

  if px + pw > viewport_w {
    px = (viewport_w - pw - 4.0 * s).max(0.0);
  }
  if py + ph > viewport_h {
    py = (swatch_y - ph - 4.0 * s).max(0.0);
  }

  cp.popup_rect = [px, py, pw, ph];

  let cx = px + pad;
  let cy = py + pad;
  cp.canvas_rect = [cx, cy, canvas_sz, canvas_sz];

  let hy = cy + canvas_sz + gap;
  cp.hue_rect = [cx, hy, canvas_sz, bar_h];

  let ay = hy + bar_h + gap;
  cp.alpha_rect = [cx, ay, canvas_sz, bar_h];

  let fy = ay + bar_h + gap;
  cp.rgba_field_rect = [cx, fy, canvas_sz, field_h];

  let fy2 = fy + field_h + gap;
  cp.hex_field_rect = [cx, fy2, canvas_sz, field_h];
}

pub fn hit_test_color_picker(cp: &ColorPickerState, pos: (f32, f32)) -> Option<ColorPickerHit> {
  let (mx, my) = (pos.0, pos.1);
  let [popup_x, popup_y, pw, ph] = cp.popup_rect;

  if mx < popup_x || mx > popup_x + pw || my < popup_y || my > popup_y + ph {
    return None;
  }

  let [canvas_x, canvas_y, canvas_w, canvas_h] = cp.canvas_rect;
  if mx >= canvas_x && mx <= canvas_x + canvas_w && my >= canvas_y && my <= canvas_y + canvas_h {
    let s = ((mx - canvas_x) / canvas_w).clamp(0.0, 1.0);
    let v = (1.0 - (my - canvas_y) / canvas_h).clamp(0.0, 1.0);
    return Some(ColorPickerHit::Canvas(s, v));
  }

  let [hue_x, hue_y, hue_w, hue_h] = cp.hue_rect;
  if mx >= hue_x && mx <= hue_x + hue_w && my >= hue_y && my <= hue_y + hue_h {
    let frac = ((mx - hue_x) / hue_w).clamp(0.0, 1.0);
    return Some(ColorPickerHit::HueBar(frac));
  }

  let [alpha_x, alpha_y, alpha_w, alpha_h] = cp.alpha_rect;
  if mx >= alpha_x && mx <= alpha_x + alpha_w && my >= alpha_y && my <= alpha_y + alpha_h {
    let frac = ((mx - alpha_x) / alpha_w).clamp(0.0, 1.0);
    return Some(ColorPickerHit::AlphaBar(frac));
  }

  let [rx, ry, rw, rh] = cp.rgba_field_rect;
  if mx >= rx && mx <= rx + rw && my >= ry && my <= ry + rh {
    return Some(ColorPickerHit::Field(ColorPickerField::Rgba));
  }

  let [hfx, hfy, hfw, hfh] = cp.hex_field_rect;
  if mx >= hfx && mx <= hfx + hfw && my >= hfy && my <= hfy + hfh {
    return Some(ColorPickerHit::Field(ColorPickerField::Hex));
  }

  Some(ColorPickerHit::Background)
}

#[derive(Debug, Clone, Copy)]
pub enum ColorPickerHit {
  Canvas(f32, f32),
  HueBar(f32),
  AlphaBar(f32),
  Field(ColorPickerField),
  Background,
}

pub fn activate_field(cp: &mut ColorPickerState, field: ColorPickerField) {
  let text = match field {
    ColorPickerField::Rgba => rgba_string(cp),
    ColorPickerField::Hex => hex_string(cp),
  };
  cp.field_cursor = wgpu_html_tree::text_edit::select_all(&text);
  cp.field_text = text;
  cp.active_field = Some(field);
  cp.field_blink_epoch = std::time::Instant::now();
}

pub fn deactivate_field(cp: &mut ColorPickerState) {
  if cp.active_field.is_some() {
    commit_field(cp);
    cp.active_field = None;
    cp.field_text.clear();
  }
}

fn commit_field(cp: &mut ColorPickerState) {
  let text = cp.field_text.trim();
  if text.is_empty() { return; }

  match cp.active_field {
    Some(ColorPickerField::Hex) => {
      if let Some(color) = wgpu_html_layout::color::parse_color_str(text) {
        let r = wgpu_html_layout::color::linear_to_srgb(color[0]);
        let g = wgpu_html_layout::color::linear_to_srgb(color[1]);
        let b = wgpu_html_layout::color::linear_to_srgb(color[2]);
        let (h, s, v) = srgb_to_hsv(r, g, b);
        cp.hue = h;
        cp.saturation = s;
        cp.value = v;
        cp.alpha = color[3];
      }
    }
    Some(ColorPickerField::Rgba) => {
      if let Some(color) = wgpu_html_layout::color::parse_color_str(text) {
        let r = wgpu_html_layout::color::linear_to_srgb(color[0]);
        let g = wgpu_html_layout::color::linear_to_srgb(color[1]);
        let b = wgpu_html_layout::color::linear_to_srgb(color[2]);
        let (h, s, v) = srgb_to_hsv(r, g, b);
        cp.hue = h;
        cp.saturation = s;
        cp.value = v;
        cp.alpha = color[3];
      }
    }
    None => {}
  }
}

pub fn field_text_input(cp: &mut ColorPickerState, text: &str) -> bool {
  if cp.active_field.is_none() { return false; }
  let (new_val, new_cursor) = wgpu_html_tree::text_edit::insert_text(&cp.field_text, &cp.field_cursor, text);
  cp.field_text = new_val;
  cp.field_cursor = new_cursor;
  cp.field_blink_epoch = std::time::Instant::now();
  true
}

pub fn field_key_down(cp: &mut ColorPickerState, key: &str, code: &str, ctrl: bool, shift: bool) -> bool {
  if cp.active_field.is_none() { return false; }
  use wgpu_html_tree::text_edit;

  match key {
    "Backspace" => {
      let (v, c) = text_edit::delete_backward(&cp.field_text, &cp.field_cursor);
      cp.field_text = v;
      cp.field_cursor = c;
      cp.field_blink_epoch = std::time::Instant::now();
    }
    "Delete" => {
      let (v, c) = text_edit::delete_forward(&cp.field_text, &cp.field_cursor);
      cp.field_text = v;
      cp.field_cursor = c;
      cp.field_blink_epoch = std::time::Instant::now();
    }
    "ArrowLeft" => {
      cp.field_cursor = if ctrl {
        text_edit::move_word_left(&cp.field_text, &cp.field_cursor, shift)
      } else {
        text_edit::move_left(&cp.field_text, &cp.field_cursor, shift)
      };
      cp.field_blink_epoch = std::time::Instant::now();
    }
    "ArrowRight" => {
      cp.field_cursor = if ctrl {
        text_edit::move_word_right(&cp.field_text, &cp.field_cursor, shift)
      } else {
        text_edit::move_right(&cp.field_text, &cp.field_cursor, shift)
      };
      cp.field_blink_epoch = std::time::Instant::now();
    }
    "Home" => {
      cp.field_cursor = text_edit::move_home(&cp.field_text, &cp.field_cursor, shift);
      cp.field_blink_epoch = std::time::Instant::now();
    }
    "End" => {
      cp.field_cursor = text_edit::move_end(&cp.field_text, &cp.field_cursor, shift);
      cp.field_blink_epoch = std::time::Instant::now();
    }
    "Enter" => {
      commit_field(cp);
      cp.active_field = None;
      cp.field_text.clear();
    }
    "Escape" => {
      cp.active_field = None;
      cp.field_text.clear();
    }
    "Tab" => {
      commit_field(cp);
      let next = match cp.active_field {
        Some(ColorPickerField::Rgba) => ColorPickerField::Hex,
        Some(ColorPickerField::Hex) => ColorPickerField::Rgba,
        None => return false,
      };
      let text = match next {
        ColorPickerField::Rgba => rgba_string(cp),
        ColorPickerField::Hex => hex_string(cp),
      };
      cp.field_cursor = text_edit::select_all(&text);
      cp.field_text = text;
      cp.active_field = Some(next);
      cp.field_blink_epoch = std::time::Instant::now();
    }
    _ => {
      if ctrl && code == "KeyA" {
        cp.field_cursor = text_edit::select_all(&cp.field_text);
        cp.field_blink_epoch = std::time::Instant::now();
      } else {
        return false;
      }
    }
  }
  true
}

pub fn field_selected_text(cp: &ColorPickerState) -> Option<String> {
  if cp.active_field.is_none() || !cp.field_cursor.has_selection() {
    return None;
  }
  let (start, end) = cp.field_cursor.selection_range();
  let start = start.min(cp.field_text.len());
  let end = end.min(cp.field_text.len());
  if start >= end { return None; }
  Some(cp.field_text[start..end].to_string())
}

pub fn sync_field_text_from_color(cp: &mut ColorPickerState) {
  if cp.active_field.is_none() { return; }
  let text = match cp.active_field {
    Some(ColorPickerField::Rgba) => rgba_string(cp),
    Some(ColorPickerField::Hex) => hex_string(cp),
    None => return,
  };
  cp.field_cursor = EditCursor::collapsed(text.len());
  cp.field_text = text;
}
