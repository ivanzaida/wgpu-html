use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use lui_core::display_list::{DisplayList, Rect as DlRect};
use lui_glyph::TextContext;
use lui_layout::LayoutBox;

use super::style;

static NEXT_IMAGE_ID: AtomicU64 = AtomicU64::new(1);

pub struct DecodedImage {
  pub id: u64,
  pub data: Arc<Vec<u8>>,
  pub width: u32,
  pub height: u32,
}

pub struct ImageCache {
  cache: HashMap<String, CacheEntry>,
}

enum CacheEntry {
  Decoded(DecodedImage),
  Failed,
}

impl Default for ImageCache {
  fn default() -> Self {
    Self::new()
  }
}

impl ImageCache {
  pub fn new() -> Self {
    Self { cache: HashMap::new() }
  }

  pub fn get(&self, src: &str) -> Option<&DecodedImage> {
    match self.cache.get(src) {
      Some(CacheEntry::Decoded(img)) => Some(img),
      _ => None,
    }
  }

  pub fn contains(&self, src: &str) -> bool {
    self.cache.contains_key(src)
  }

  pub fn is_failed(&self, src: &str) -> bool {
    matches!(self.cache.get(src), Some(CacheEntry::Failed))
  }

  #[cfg(feature = "wgpu")]
  pub fn decode_and_insert(&mut self, src: &str, bytes: &[u8]) -> Option<&DecodedImage> {
    if self.cache.contains_key(src) {
      return self.get(src);
    }

    match ::image::load_from_memory(bytes) {
      Ok(img) => {
        let rgba = ::image::DynamicImage::to_rgba8(&img);
        let width = rgba.width();
        let height = rgba.height();
        let data = Arc::new(rgba.into_raw());
        let id = NEXT_IMAGE_ID.fetch_add(1, Ordering::Relaxed);
        self.cache.insert(src.to_string(), CacheEntry::Decoded(DecodedImage {
          id, data, width, height,
        }));
        self.get(src)
      }
      Err(_) => {
        self.cache.insert(src.to_string(), CacheEntry::Failed);
        None
      }
    }
  }

  pub fn insert_rgba(&mut self, src: &str, data: Vec<u8>, width: u32, height: u32) -> &DecodedImage {
    let id = NEXT_IMAGE_ID.fetch_add(1, Ordering::Relaxed);
    self.cache.insert(src.to_string(), CacheEntry::Decoded(DecodedImage {
      id, data: Arc::new(data), width, height,
    }));
    match &self.cache[src] {
      CacheEntry::Decoded(img) => img,
      _ => unreachable!(),
    }
  }

  pub fn mark_failed(&mut self, src: &str) {
    self.cache.insert(src.to_string(), CacheEntry::Failed);
  }

  pub fn remove(&mut self, src: &str) {
    self.cache.remove(src);
  }

  pub fn clear(&mut self) {
    self.cache.clear();
  }
}

pub fn paint_img_element(
  b: &LayoutBox,
  dx: f32,
  dy: f32,
  opacity: f32,
  image_cache: &ImageCache,
  text_ctx: &mut TextContext,
  dpi_scale: f32,
  dl: &mut DisplayList,
) {
  if b.node.tag_name() != "img" {
    return;
  }

  let src = match b.node.attr("src") {
    Some(s) => &**s,
    None => {
      paint_alt_text(b, dx, dy, opacity, text_ctx, dpi_scale, dl);
      return;
    }
  };

  match image_cache.get(src) {
    Some(img) => {
      let rect = DlRect::new(
        b.content.x + dx,
        b.content.y + dy,
        b.content.width,
        b.content.height,
      );
      dl.push_image_with_opacity(rect, img.id, img.data.clone(), img.width, img.height, opacity);
    }
    None => {
      if image_cache.is_failed(src) {
        paint_alt_text(b, dx, dy, opacity, text_ctx, dpi_scale, dl);
      }
    }
  }
}

fn paint_alt_text(
  b: &LayoutBox,
  dx: f32,
  dy: f32,
  opacity: f32,
  text_ctx: &mut TextContext,
  dpi_scale: f32,
  dl: &mut DisplayList,
) {
  let alt = match b.node.attr("alt") {
    Some(s) if !s.is_empty() => &**s,
    _ => return,
  };

  let font_size = style::css_f32(b.style.font_size).max(12.0);
  let mut color = style::css_color(b.style.color).unwrap_or([0.5, 0.5, 0.5, 1.0]);
  color[3] *= opacity;
  if color[3] <= 0.0 {
    return;
  }

  let line_height = font_size * 1.2;
  let weight: u16 = match b.style.font_weight {
    Some(lui_core::CssValue::Number(n)) => (*n as u16).min(1000),
    _ => 400,
  };
  let font_family = style::css_str(b.style.font_family);

  let content_x = b.content.x + dx;
  let content_y = b.content.y + dy;

  let run = text_ctx.shape_and_pack(
    alt, font_size, line_height, weight, color, font_family, dpi_scale, Some(b.content.width),
  );

  let snap_x = (content_x * dpi_scale).round() / dpi_scale;
  let snap_y = (content_y * dpi_scale).round() / dpi_scale;

  for glyph in &run.glyphs {
    if glyph.uv_min == [0.0; 2] && glyph.uv_max == [0.0; 2] {
      continue;
    }
    let rect = DlRect::new(snap_x + glyph.x, snap_y + glyph.y, glyph.w, glyph.h);
    dl.push_glyph(rect, color, glyph.uv_min, glyph.uv_max);
  }
}

pub fn paint_background_image(
  b: &LayoutBox,
  border_rect: DlRect,
  opacity: f32,
  image_cache: &ImageCache,
  dl: &mut DisplayList,
) {
  let bg_val = match b.style.background_image {
    Some(val) => val,
    None => return,
  };

  let url = match extract_url(bg_val) {
    Some(u) => u,
    None => return,
  };

  let img = match image_cache.get(url) {
    Some(img) => img,
    None => return,
  };

  dl.push_image_with_opacity(border_rect, img.id, img.data.clone(), img.width, img.height, opacity);
}

fn extract_url(val: &lui_core::CssValue) -> Option<&str> {
  match val {
    lui_core::CssValue::Url(url) => Some(&**url),
    lui_core::CssValue::String(s) | lui_core::CssValue::Unknown(s) => {
      let s = s.trim();
      if let Some(inner) = s.strip_prefix("url(") {
        let inner = inner.strip_suffix(')')?.trim();
        let inner = inner.strip_prefix(['"', '\'']).unwrap_or(inner);
        let inner = inner.strip_suffix(['"', '\'']).unwrap_or(inner);
        Some(inner)
      } else {
        None
      }
    }
    _ => None,
  }
}
