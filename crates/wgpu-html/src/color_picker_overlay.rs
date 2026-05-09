use std::sync::Arc;

use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{ColorPickerState, Tree};

const POPUP_W: f32 = 260.0;
const POPUP_PAD: f32 = 10.0;
const CANVAS_SIZE: f32 = 240.0;
const BAR_H: f32 = 14.0;
const GAP: f32 = 8.0;
const LABEL_H: f32 = 16.0;

const BG: [f32; 4] = [0.15, 0.15, 0.15, 0.96];
const LABEL_COLOR: [f32; 4] = [0.85, 0.85, 0.85, 1.0];
const BORDER_COLOR: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
const INDICATOR_BORDER: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const INDICATOR_SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

const SV_TEX_SIZE: u32 = 128;
const HUE_TEX_W: u32 = 256;
const HUE_TEX_H: u32 = 1;

const FONT_SIZE: f32 = 11.0;

fn popup_height() -> f32 {
  POPUP_PAD + CANVAS_SIZE + GAP + BAR_H + GAP + BAR_H + GAP + LABEL_H + GAP + LABEL_H + POPUP_PAD
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

  // Background
  list.push_quad_rounded(
    Rect::new(popup_x, popup_y, pw, ph),
    BG,
    [corner_r; 4],
  );
  list.push_quad_stroke(
    Rect::new(popup_x, popup_y, pw, ph),
    BORDER_COLOR,
    [corner_r; 4],
    [1.0; 4],
  );

  // SV canvas
  let hue_q = (cp.hue * 2.0) as u16;
  let sv_data = gen_sv_texture(cp.hue);
  list.push_image(
    Rect::new(cx, cy, canvas_w, canvas_h),
    image_id_sv(hue_q),
    Arc::new(sv_data),
    SV_TEX_SIZE,
    SV_TEX_SIZE,
  );

  // SV crosshair indicator
  let ind_x = cx + cp.saturation * canvas_w;
  let ind_y = cy + (1.0 - cp.value) * canvas_h;
  list.push_quad_stroke(
    Rect::new(ind_x - ind_r, ind_y - ind_r, ind_r * 2.0, ind_r * 2.0),
    INDICATOR_SHADOW,
    [ind_r + 1.0; 4],
    [2.0; 4],
  );
  list.push_quad_stroke(
    Rect::new(ind_x - ind_r, ind_y - ind_r, ind_r * 2.0, ind_r * 2.0),
    INDICATOR_BORDER,
    [ind_r; 4],
    [1.5; 4],
  );

  // Hue bar
  let hue_data = gen_hue_texture();
  let bar_r = hue_h / 2.0;
  list.push_clip(
    Some(Rect::new(hx, hy, hue_w, hue_h)),
    [bar_r; 4],
    [bar_r; 4],
  );
  list.push_image(
    Rect::new(hx, hy, hue_w, hue_h),
    IMAGE_ID_HUE,
    Arc::new(hue_data),
    HUE_TEX_W,
    HUE_TEX_H,
  );
  list.pop_clip(None, [0.0; 4], [0.0; 4]);

  // Hue indicator
  let hue_frac = cp.hue / 360.0;
  let hi_x = hx + hue_frac * hue_w;
  list.push_quad_rounded(
    Rect::new(hi_x - slider_knob_w * 0.5, hy - 1.0, slider_knob_w, hue_h + 2.0),
    INDICATOR_BORDER,
    [slider_knob_r; 4],
  );

  // Alpha bar
  let (ar, ag, ab) = hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
  let alpha_data = gen_alpha_texture(ar, ag, ab);
  let alpha_bar_r = alpha_h / 2.0;
  list.push_clip(
    Some(Rect::new(ax, ay, alpha_w, alpha_h)),
    [alpha_bar_r; 4],
    [alpha_bar_r; 4],
  );
  list.push_image(
    Rect::new(ax, ay, alpha_w, alpha_h),
    image_id_alpha(ar, ag, ab),
    Arc::new(alpha_data),
    HUE_TEX_W,
    2,
  );
  list.pop_clip(None, [0.0; 4], [0.0; 4]);

  // Alpha indicator
  let ai_x = ax + cp.alpha * alpha_w;
  list.push_quad_rounded(
    Rect::new(ai_x - slider_knob_w * 0.5, ay - 1.0, slider_knob_w, alpha_h + 2.0),
    INDICATOR_BORDER,
    [slider_knob_r; 4],
  );

  // Labels
  let label_y = ay + alpha_h + GAP;
  let (sr, sg, sb) = hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
  let a_byte = (cp.alpha * 255.0 + 0.5) as u8;
  let rgba_str = format!("rgba({sr}, {sg}, {sb}, {a_byte})");
  let hex_str = if a_byte == 255 {
    format!("#{sr:02x}{sg:02x}{sb:02x}")
  } else {
    format!("#{sr:02x}{sg:02x}{sb:02x}{a_byte:02x}")
  };

  let font_size = FONT_SIZE;
  paint_label(list, text_ctx, &rgba_str, cx, label_y, canvas_w, font_size);
  paint_label(list, text_ctx, &hex_str, cx, label_y + LABEL_H + GAP, canvas_w, font_size);
}

fn paint_label(
  list: &mut DisplayList,
  text_ctx: &mut TextContext,
  text: &str,
  x: f32,
  y: f32,
  max_w: f32,
  font_size: f32,
) {
  let families = ["monospace", "sans-serif"];
  let font = text_ctx.pick_font(&families, 400, wgpu_html_tree::FontStyleAxis::Normal);
  let Some(font_handle) = font else { return };
  let shaped = text_ctx.shape_and_pack(
    text,
    font_handle,
    font_size,
    font_size * 1.3,
    0.0,
    400,
    wgpu_html_tree::FontStyleAxis::Normal,
    None,
    LABEL_COLOR,
  );
  let Some(run) = shaped else { return };
  for g in &run.glyphs {
    let gx = (x + g.x).round();
    let gy = (y + g.y).round();
    if gx + g.w > x + max_w {
      break;
    }
    list.push_glyph(
      Rect::new(gx, gy, g.w, g.h),
      g.color,
      g.uv_min,
      g.uv_max,
    );
  }
}

pub fn compute_popup_rects(cp: &mut ColorPickerState, swatch_x: f32, swatch_y: f32, swatch_h: f32, scale: f32, viewport_w: f32, viewport_h: f32) {
  let s = scale.max(0.5);
  let pw = POPUP_W * s;
  let ph = popup_height() * s;
  let pad = POPUP_PAD * s;
  let canvas_sz = CANVAS_SIZE * s;
  let bar_h = BAR_H * s;
  let gap = GAP * s;

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

  Some(ColorPickerHit::Background)
}

#[derive(Debug, Clone, Copy)]
pub enum ColorPickerHit {
  Canvas(f32, f32),
  HueBar(f32),
  AlphaBar(f32),
  Background,
}
