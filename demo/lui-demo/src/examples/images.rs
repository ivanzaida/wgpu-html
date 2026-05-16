use lui::{Lui, StylesheetHandle};
use lui_parse::html::parser::parse_nodes;

use crate::examples::Example;

const IMAGES_HTML: &str = include_str!("../../html/images.html");

#[derive(Default)]
pub struct ImagesExample {
  handles: Vec<StylesheetHandle>,
}

impl Example for ImagesExample {
  fn get_name(&self) -> &'static str {
    "Images"
  }

  fn render(&mut self, lui: &mut Lui) -> super::ExampleOutput {
    insert_test_images(lui);

    let (nodes, sheets) = parse_nodes(IMAGES_HTML);
    self.handles = lui.add_stylesheets(sheets);
    super::ExampleOutput::Nodes(nodes)
  }

  fn cleanup(&mut self, lui: &mut Lui) {
    let handles = std::mem::take(&mut self.handles);
    lui.remove_stylesheets(handles);
  }
}

fn insert_test_images(lui: &mut Lui) {
  if !lui.image_cache.contains("checkerboard") {
    let (data, w, h) = generate_checkerboard(128, 16);
    lui.image_cache.insert_rgba("checkerboard", data, w, h);
  }
  if !lui.image_cache.contains("gradient") {
    let (data, w, h) = generate_gradient(200, 100);
    lui.image_cache.insert_rgba("gradient", data, w, h);
  }
  if !lui.image_cache.is_failed("missing.png") {
    lui.image_cache.mark_failed("missing.png");
  }
  if !lui.image_cache.is_failed("broken") {
    lui.image_cache.mark_failed("broken");
  }
}

fn generate_checkerboard(size: u32, cell: u32) -> (Vec<u8>, u32, u32) {
  let mut pixels = vec![0u8; (size * size * 4) as usize];
  for y in 0..size {
    for x in 0..size {
      let is_dark = ((x / cell) + (y / cell)) % 2 == 0;
      let c = if is_dark { 40u8 } else { 200u8 };
      let idx = ((y * size + x) * 4) as usize;
      pixels[idx] = c;
      pixels[idx + 1] = c;
      pixels[idx + 2] = c;
      pixels[idx + 3] = 255;
    }
  }
  (pixels, size, size)
}

fn generate_gradient(w: u32, h: u32) -> (Vec<u8>, u32, u32) {
  let mut pixels = vec![0u8; (w * h * 4) as usize];
  for y in 0..h {
    for x in 0..w {
      let r = ((x as f32 / w as f32) * 255.0) as u8;
      let g = ((y as f32 / h as f32) * 255.0) as u8;
      let b = 180u8;
      let idx = ((y * w + x) * 4) as usize;
      pixels[idx] = r;
      pixels[idx + 1] = g;
      pixels[idx + 2] = b;
      pixels[idx + 3] = 255;
    }
  }
  (pixels, w, h)
}
