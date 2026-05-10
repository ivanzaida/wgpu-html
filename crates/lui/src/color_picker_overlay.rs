use std::sync::Arc;

use lui_layout::LayoutBox;
use lui_renderer_wgpu::{DisplayList, Rect};
use lui_text::TextContext;
use lui_tree::{ColorPickerField, ColorPickerState, EditCursor, Element, Tree};
use lui_models::{ArcStr, Div};
use lui_models::common::css_enums::*;
use lui_style::{CascadedNode, CascadedTree};

const SV_TEX_SIZE: u32 = 128;
const HUE_TEX_W: u32 = 256;
const HUE_TEX_H: u32 = 1;

const INDICATOR_SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.5];
const SELECTION_BG: [f32; 4] = [0.23, 0.51, 0.96, 0.45];
const CARET_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

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
  ((r * 255.0 + 0.5) as u8, (g * 255.0 + 0.5) as u8, (b * 255.0 + 0.5) as u8)
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
  if a == 255 { format!("#{r:02x}{g:02x}{b:02x}") }
  else { format!("#{r:02x}{g:02x}{b:02x}{a:02x}") }
}

// ── Layout-driven painting ─────────────────────────────────────────────

fn cn(element: Element, style: lui_models::Style, children: Vec<CascadedNode>) -> CascadedNode {
  CascadedNode {
    element, style, children,
    before: None, after: None, first_line: None, first_letter: None,
    placeholder: None, selection: None, marker: None, lui_pseudo: vec![],
  }
}

fn text_node_colored(text: &str, color: &CssColor) -> CascadedNode {
  let mut s = lui_models::Style::default();
  s.color = Some(color.clone());
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
    ($side:ident, $w:ident, $s:ident, $c:ident) => {
      if ps.$w.is_some() { base.$w = ps.$w.clone(); }
      if ps.$s.is_some() { base.$s = ps.$s.clone(); }
      if ps.$c.is_some() { base.$c = ps.$c.clone(); }
    }
  }
  border!(top, border_top_width, border_top_style, border_top_color);
  border!(right, border_right_width, border_right_style, border_right_color);
  border!(bottom, border_bottom_width, border_bottom_style, border_bottom_color);
  border!(left, border_left_width, border_left_style, border_left_color);
  if let Some(r) = &ps.border_radius {
    base.border_top_left_radius = Some(r.clone());
    base.border_top_right_radius = Some(r.clone());
    base.border_bottom_left_radius = Some(r.clone());
    base.border_bottom_right_radius = Some(r.clone());
  }
}

fn build_picker_tree(cp: &ColorPickerState) -> CascadedTree {
  let popup_ps = cp.popup_style.as_deref();

  // Popup container — laid out at (0,0), translated to popup position later.
  let mut ps = lui_models::Style::default();
  ps.display = Some(Display::Flex);
  ps.flex_direction = Some(FlexDirection::Column);
  ps.width = Some(CssLength::Px(260.0));
  ps.background_color = Some(CssColor::Rgba(38, 38, 38, 0.96));
  ps.color = Some(CssColor::Rgba(217, 217, 217, 1.0));
  ps.border_top_width = Some(CssLength::Px(1.0));
  ps.border_right_width = Some(CssLength::Px(1.0));
  ps.border_bottom_width = Some(CssLength::Px(1.0));
  ps.border_left_width = Some(CssLength::Px(1.0));
  ps.border_top_style = Some(BorderStyle::Solid);
  ps.border_right_style = Some(BorderStyle::Solid);
  ps.border_bottom_style = Some(BorderStyle::Solid);
  ps.border_left_style = Some(BorderStyle::Solid);
  ps.border_top_color = Some(CssColor::Rgba(77, 77, 77, 1.0));
  ps.border_right_color = Some(CssColor::Rgba(77, 77, 77, 1.0));
  ps.border_bottom_color = Some(CssColor::Rgba(77, 77, 77, 1.0));
  ps.border_left_color = Some(CssColor::Rgba(77, 77, 77, 1.0));
  let r = CssLength::Px(6.0);
  ps.border_top_left_radius = Some(r.clone());
  ps.border_top_right_radius = Some(r.clone());
  ps.border_bottom_left_radius = Some(r.clone());
  ps.border_bottom_right_radius = Some(r);
  ps.padding_top = Some(CssLength::Px(10.0));
  ps.padding_right = Some(CssLength::Px(10.0));
  ps.padding_bottom = Some(CssLength::Px(10.0));
  ps.padding_left = Some(CssLength::Px(10.0));
  ps.gap = Some(CssLength::Px(8.0));
  merge_popup_style(&mut ps, popup_ps);

  // Resolve the effective text color for children (no cascade = no inheritance).
  let text_color = ps.color.clone().unwrap_or(CssColor::Rgba(217, 217, 217, 1.0));

  // Canvas — no explicit width, stretches via flexbox
  let mut cs = lui_models::Style::default();
  cs.height = Some(CssLength::Px(240.0));
  if let Some(pks) = cp.picker_style.as_deref() {
    if pks.canvas_height.is_some() { cs.height = pks.canvas_height.clone(); }
  }
  let canvas_node = cn(Element::Div(Div::default()), cs, vec![]);

  // Hue bar — no explicit width, stretches via flexbox
  let mut hs = lui_models::Style::default();
  hs.height = Some(CssLength::Px(14.0));
  let bar_r = CssLength::Px(7.0);
  hs.border_top_left_radius = Some(bar_r.clone());
  hs.border_top_right_radius = Some(bar_r.clone());
  hs.border_bottom_left_radius = Some(bar_r.clone());
  hs.border_bottom_right_radius = Some(bar_r);
  if let Some(pks) = cp.picker_style.as_deref() {
    if pks.range_height.is_some() { hs.height = pks.range_height.clone(); }
    if pks.range_border_radius.is_some() {
      let r = pks.range_border_radius.clone();
      hs.border_top_left_radius = r.clone();
      hs.border_top_right_radius = r.clone();
      hs.border_bottom_left_radius = r.clone();
      hs.border_bottom_right_radius = r;
    }
  }
  let hue_node = cn(Element::Div(Div::default()), hs.clone(), vec![]);
  let alpha_node = cn(Element::Div(Div::default()), hs, vec![]);

  // RGBA field
  let rgba_text = if cp.active_field == Some(ColorPickerField::Rgba) {
    cp.field_text.clone()
  } else {
    rgba_string(cp)
  };
  let rgba_node = build_input_node(&rgba_text, cp, &text_color);

  // Hex field
  let hex_text = if cp.active_field == Some(ColorPickerField::Hex) {
    cp.field_text.clone()
  } else {
    hex_string(cp)
  };
  let hex_node = build_input_node(&hex_text, cp, &text_color);

  let popup = cn(Element::Div(Div::default()), ps, vec![
    canvas_node, hue_node, alpha_node, rgba_node, hex_node,
  ]);

  CascadedTree { root: Some(popup) }
}

fn build_input_node(text: &str, cp: &ColorPickerState, text_color: &CssColor) -> CascadedNode {
  let mut is = lui_models::Style::default();
  is.height = Some(CssLength::Px(20.0));
  is.background_color = Some(CssColor::Rgba(26, 26, 26, 1.0));
  is.border_top_width = Some(CssLength::Px(1.0));
  is.border_right_width = Some(CssLength::Px(1.0));
  is.border_bottom_width = Some(CssLength::Px(1.0));
  is.border_left_width = Some(CssLength::Px(1.0));
  is.border_top_style = Some(BorderStyle::Solid);
  is.border_right_style = Some(BorderStyle::Solid);
  is.border_bottom_style = Some(BorderStyle::Solid);
  is.border_left_style = Some(BorderStyle::Solid);
  is.border_top_color = Some(CssColor::Rgba(89, 89, 89, 1.0));
  is.border_right_color = Some(CssColor::Rgba(89, 89, 89, 1.0));
  is.border_bottom_color = Some(CssColor::Rgba(89, 89, 89, 1.0));
  is.border_left_color = Some(CssColor::Rgba(89, 89, 89, 1.0));
  let r = CssLength::Px(3.0);
  is.border_top_left_radius = Some(r.clone());
  is.border_top_right_radius = Some(r.clone());
  is.border_bottom_left_radius = Some(r.clone());
  is.border_bottom_right_radius = Some(r);
  is.padding_left = Some(CssLength::Px(4.0));
  is.padding_right = Some(CssLength::Px(4.0));
  is.font_size = Some(CssLength::Px(11.0));
  is.font_family = Some(ArcStr::from("monospace"));
  is.color = Some(CssColor::Rgba(217, 217, 217, 1.0));
  is.overflow_x = Some(Overflow::Hidden);
  is.display = Some(Display::Flex);
  is.align_items = Some(AlignItems::Center);

  if let Some(pks) = cp.picker_style.as_deref() {
    if pks.input_height.is_some() { is.height = pks.input_height.clone(); }
    if pks.input_background.is_some() { is.background_color = pks.input_background.clone().map(|c| c); }
    if pks.input_border_color.is_some() {
      let c = pks.input_border_color.clone();
      is.border_top_color = c.clone();
      is.border_right_color = c.clone();
      is.border_bottom_color = c.clone();
      is.border_left_color = c;
    }
    if pks.input_border_width.is_some() {
      let w = pks.input_border_width.clone();
      is.border_top_width = w.clone();
      is.border_right_width = w.clone();
      is.border_bottom_width = w.clone();
      is.border_left_width = w;
    }
    if let Some(r) = &pks.input_border_radius {
      is.border_top_left_radius = Some(r.clone());
      is.border_top_right_radius = Some(r.clone());
      is.border_bottom_left_radius = Some(r.clone());
      is.border_bottom_right_radius = Some(r.clone());
    }
    if pks.input_font_size.is_some() { is.font_size = pks.input_font_size.clone(); }
  }

  let effective_color = is.color.clone().unwrap_or_else(|| text_color.clone());
  cn(Element::Div(Div::default()), is, vec![text_node_colored(text, &effective_color)])
}

/// Build/rebuild the picker layout and cache it on the state.
/// Call this BEFORE the main paint pass so the atlas has all needed glyphs.
pub fn update_cached_layout(tree: &mut Tree, text_ctx: &mut TextContext) {
  let cp = match &mut tree.interaction.color_picker {
    Some(cp) => cp,
    None => return,
  };
  let prev_gen = cp.layout_generation;
  let picker_tree = build_picker_tree(cp);
  let mut image_cache = lui_layout::ImageCache::default();
  let Some(mut layout) = lui_layout::layout_with_text(
    &picker_tree, text_ctx, &mut image_cache, 4096.0, 4096.0, 1.0,
  ) else { return };
  let [popup_x, popup_y, _, _] = cp.popup_rect;
  translate_layout_box(&mut layout, popup_x, popup_y);
  cp.cached_layout = lui_tree::CachedLayout(Some(Box::new(layout)));
  cp.layout_generation = prev_gen + 1;
}

pub fn paint_color_picker_overlay(
  list: &mut DisplayList,
  _root: &LayoutBox,
  tree: &Tree,
  _scroll_y: f32,
  _scale: f32,
  _viewport_w: f32,
  _viewport_h: f32,
) {
  let cp = match &tree.interaction.color_picker {
    Some(cp) => cp,
    None => return,
  };

  let popup_box = match cp.cached_layout.0.as_ref().and_then(|l| l.downcast_ref::<LayoutBox>()) {
    Some(l) => l,
    None => return,
  };
  if popup_box.children.len() < 5 { return; }

  let saved_canvas_color = list.canvas_color;
  crate::paint::paint_layout_with_selection(popup_box, list, None, lui_tree::SelectionColors::default(), 0.0);
  list.canvas_color = saved_canvas_color;

  // Canvas gradient
  let cb = &popup_box.children[0];
  let cr = cb.content_rect;
  if cr.w > 0.0 && cr.h > 0.0 {
    let hue_q = (cp.hue * 2.0) as u16;
    let sv_data = gen_sv_texture(cp.hue);
    list.push_image(Rect::new(cr.x, cr.y, cr.w, cr.h), image_id_sv(hue_q), Arc::new(sv_data), SV_TEX_SIZE, SV_TEX_SIZE);

    // Crosshair
    let ind_x = cr.x + cp.saturation * cr.w;
    let ind_y = cr.y + (1.0 - cp.value) * cr.h;
    let ind_r = 5.0;
    let indicator = thumb_color(cp);
    list.push_quad_stroke(Rect::new(ind_x - ind_r, ind_y - ind_r, ind_r * 2.0, ind_r * 2.0), INDICATOR_SHADOW, [ind_r + 1.0; 4], [2.0; 4]);
    list.push_quad_stroke(Rect::new(ind_x - ind_r, ind_y - ind_r, ind_r * 2.0, ind_r * 2.0), indicator, [ind_r; 4], [1.5; 4]);
  }

  // Hue bar gradient
  let hb = &popup_box.children[1];
  let hr = hb.border_rect;
  if hr.w > 0.0 && hr.h > 0.0 {
    let hue_data = gen_hue_texture();
    let bar_r = hr.h / 2.0;
    list.push_clip(Some(Rect::new(hr.x, hr.y, hr.w, hr.h)), [bar_r; 4], [bar_r; 4]);
    list.push_image(Rect::new(hr.x, hr.y, hr.w, hr.h), IMAGE_ID_HUE, Arc::new(hue_data), HUE_TEX_W, HUE_TEX_H);
    list.pop_clip(None, [0.0; 4], [0.0; 4]);

    let indicator = thumb_color(cp);
    let hue_frac = cp.hue / 360.0;
    let hi_x = hr.x + hue_frac * hr.w;
    let tw = thumb_w(cp);
    list.push_quad_rounded(Rect::new(hi_x - tw * 0.5, hr.y - 1.0, tw, hr.h + 2.0), indicator, [2.0; 4]);
  }

  // Alpha bar gradient
  let ab = &popup_box.children[2];
  let ar = ab.border_rect;
  if ar.w > 0.0 && ar.h > 0.0 {
    let (sr, sg, sb) = hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
    let alpha_data = gen_alpha_texture(sr, sg, sb);
    let bar_r = ar.h / 2.0;
    list.push_clip(Some(Rect::new(ar.x, ar.y, ar.w, ar.h)), [bar_r; 4], [bar_r; 4]);
    list.push_image(Rect::new(ar.x, ar.y, ar.w, ar.h), image_id_alpha(sr, sg, sb), Arc::new(alpha_data), HUE_TEX_W, 2);
    list.pop_clip(None, [0.0; 4], [0.0; 4]);

    let indicator = thumb_color(cp);
    let ai_x = ar.x + cp.alpha * ar.w;
    let tw = thumb_w(cp);
    list.push_quad_rounded(Rect::new(ai_x - tw * 0.5, ar.y - 1.0, tw, ar.h + 2.0), indicator, [2.0; 4]);
  }

  // Caret / selection overlays for focused fields
  if let Some(field) = cp.active_field {
    let field_idx = match field { ColorPickerField::Rgba => 3, ColorPickerField::Hex => 4 };
    let fb = &popup_box.children[field_idx];
    let fr = fb.content_rect;
    if let Some(run) = fb.children.first().and_then(|c| c.text_run.as_ref()) {
      let text_x = fr.x;
      let font_size = run.height;
      let text_y = fr.y + (fr.h - font_size) / 2.0;

      if cp.field_cursor.has_selection() {
        let (sel_start, sel_end) = cp.field_cursor.selection_range();
        let start_x = byte_to_x(run, sel_start);
        let end_x = byte_to_x(run, sel_end);
        list.push_quad(Rect::new(text_x + start_x, text_y, end_x - start_x, font_size), SELECTION_BG);
      } else {
        let elapsed = cp.field_blink_epoch.elapsed().as_millis();
        if (elapsed % 1000) < 500 {
          let caret_x = text_x + byte_to_x(run, cp.field_cursor.cursor);
          list.push_quad(Rect::new(caret_x, text_y, 1.5, font_size), CARET_COLOR);
        }
      }
    }
  }
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

fn thumb_color(cp: &ColorPickerState) -> [f32; 4] {
  cp.picker_style.as_ref()
    .and_then(|s| s.thumb_color.as_ref())
    .and_then(|c| lui_layout::color::resolve_color(c))
    .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

fn thumb_w(cp: &ColorPickerState) -> f32 {
  cp.picker_style.as_ref()
    .and_then(|s| s.thumb_width.as_ref())
    .and_then(|l| match l { CssLength::Px(v) => Some(*v), _ => None })
    .unwrap_or(6.0)
}

fn byte_to_x(run: &lui_text::ShapedRun, byte_offset: usize) -> f32 {
  if byte_offset == 0 || run.glyphs.is_empty() { return 0.0; }
  for (i, &boundary) in run.byte_boundaries.iter().enumerate() {
    if boundary >= byte_offset {
      if i < run.glyphs.len() { return run.glyphs[i].x; }
      return run.width;
    }
  }
  run.width
}

// ── Hit testing (uses stored rects from compute_popup_rects) ──

pub fn compute_popup_rects(cp: &mut ColorPickerState, swatch_x: f32, swatch_y: f32, swatch_h: f32, _scale: f32, viewport_w: f32, viewport_h: f32) {
  let pw = 260.0;
  let ph = 370.0;
  let mut px = swatch_x;
  let mut py = swatch_y + swatch_h + 4.0;
  if px + pw > viewport_w { px = (viewport_w - pw - 4.0).max(0.0); }
  if py + ph > viewport_h { py = (swatch_y - ph - 4.0).max(0.0); }
  cp.popup_rect = [px, py, pw, ph];
  let pad = 10.0;
  let gap = 8.0;
  let cx = px + pad + 1.0;
  let cy = py + pad + 1.0;
  let canvas_sz = 240.0;
  let bar_h = 14.0;
  let field_h = 20.0;
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
    return Some(ColorPickerHit::HueBar(((mx - hue_x) / hue_w).clamp(0.0, 1.0)));
  }
  let [alpha_x, alpha_y, alpha_w, alpha_h] = cp.alpha_rect;
  if mx >= alpha_x && mx <= alpha_x + alpha_w && my >= alpha_y && my <= alpha_y + alpha_h {
    return Some(ColorPickerHit::AlphaBar(((mx - alpha_x) / alpha_w).clamp(0.0, 1.0)));
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
  cp.field_cursor = lui_tree::text_edit::select_all(&text);
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
  if let Some(color) = lui_layout::color::parse_color_str(text) {
    let r = lui_layout::color::linear_to_srgb(color[0]);
    let g = lui_layout::color::linear_to_srgb(color[1]);
    let b = lui_layout::color::linear_to_srgb(color[2]);
    let (h, s, v) = srgb_to_hsv(r, g, b);
    cp.hue = h;
    cp.saturation = s;
    cp.value = v;
    cp.alpha = color[3];
  }
}

pub fn field_text_input(cp: &mut ColorPickerState, text: &str) -> bool {
  if cp.active_field.is_none() { return false; }
  let (new_val, new_cursor) = lui_tree::text_edit::insert_text(&cp.field_text, &cp.field_cursor, text);
  cp.field_text = new_val;
  cp.field_cursor = new_cursor;
  cp.field_blink_epoch = std::time::Instant::now();
  true
}

pub fn field_key_down(cp: &mut ColorPickerState, key: &str, code: &str, ctrl: bool, shift: bool) -> bool {
  if cp.active_field.is_none() { return false; }
  use lui_tree::text_edit;
  match key {
    "Backspace" => { let (v, c) = text_edit::delete_backward(&cp.field_text, &cp.field_cursor); cp.field_text = v; cp.field_cursor = c; cp.field_blink_epoch = std::time::Instant::now(); }
    "Delete" => { let (v, c) = text_edit::delete_forward(&cp.field_text, &cp.field_cursor); cp.field_text = v; cp.field_cursor = c; cp.field_blink_epoch = std::time::Instant::now(); }
    "ArrowLeft" => { cp.field_cursor = if ctrl { text_edit::move_word_left(&cp.field_text, &cp.field_cursor, shift) } else { text_edit::move_left(&cp.field_text, &cp.field_cursor, shift) }; cp.field_blink_epoch = std::time::Instant::now(); }
    "ArrowRight" => { cp.field_cursor = if ctrl { text_edit::move_word_right(&cp.field_text, &cp.field_cursor, shift) } else { text_edit::move_right(&cp.field_text, &cp.field_cursor, shift) }; cp.field_blink_epoch = std::time::Instant::now(); }
    "Home" => { cp.field_cursor = text_edit::move_home(&cp.field_text, &cp.field_cursor, shift); cp.field_blink_epoch = std::time::Instant::now(); }
    "End" => { cp.field_cursor = text_edit::move_end(&cp.field_text, &cp.field_cursor, shift); cp.field_blink_epoch = std::time::Instant::now(); }
    "Enter" => { commit_field(cp); cp.active_field = None; cp.field_text.clear(); }
    "Escape" => { cp.active_field = None; cp.field_text.clear(); }
    "Tab" => {
      commit_field(cp);
      let next = match cp.active_field { Some(ColorPickerField::Rgba) => ColorPickerField::Hex, _ => ColorPickerField::Rgba };
      let text = match next { ColorPickerField::Rgba => rgba_string(cp), ColorPickerField::Hex => hex_string(cp) };
      cp.field_cursor = text_edit::select_all(&text);
      cp.field_text = text;
      cp.active_field = Some(next);
      cp.field_blink_epoch = std::time::Instant::now();
    }
    _ => {
      if ctrl && code == "KeyA" { cp.field_cursor = text_edit::select_all(&cp.field_text); cp.field_blink_epoch = std::time::Instant::now(); }
      else { return false; }
    }
  }
  true
}

pub fn field_selected_text(cp: &ColorPickerState) -> Option<String> {
  if cp.active_field.is_none() || !cp.field_cursor.has_selection() { return None; }
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
