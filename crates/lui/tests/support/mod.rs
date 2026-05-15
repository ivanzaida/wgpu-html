use std::{
  path::Path,
  sync::{Arc, Mutex},
};

use lui::{
  Lui, RenderBackend, RenderError, WindowHandle,
  display_list::{DisplayList, FrameOutcome},
};

pub const TEST_WIDTH: u32 = 200;
pub const TEST_HEIGHT: u32 = 200;

#[derive(Clone, Default)]
pub struct RenderSpy {
  last_list: Arc<Mutex<Option<DisplayList>>>,
}

impl RenderSpy {
  pub fn take_last_list(&self) -> DisplayList {
    self
      .last_list
      .lock()
      .unwrap()
      .clone()
      .expect("expected a rendered display list")
  }
}

impl RenderBackend for RenderSpy {
  fn init(&mut self, _window: Arc<dyn WindowHandle>, _width: u32, _height: u32) {}
  fn resize(&mut self, _width: u32, _height: u32) {}
  fn set_clear_color(&mut self, _color: [f32; 4]) {}
  fn upload_atlas_region(&mut self, _x: u32, _y: u32, _w: u32, _h: u32, _data: &[u8]) {}

  fn render(&mut self, list: &DisplayList) -> FrameOutcome {
    *self.last_list.lock().unwrap() = Some(list.clone());
    FrameOutcome::Presented
  }

  fn render_to_rgba(&mut self, _list: &DisplayList, _width: u32, _height: u32) -> Result<Vec<u8>, RenderError> {
    Ok(Vec::new())
  }

  fn capture_to(&mut self, _list: &DisplayList, _width: u32, _height: u32, _path: &Path) -> Result<(), RenderError> {
    Ok(())
  }

  fn capture_next_frame_to(&mut self, _path: std::path::PathBuf) {}

  fn glyph_atlas_size(&self) -> u32 {
    0
  }
}

pub fn test_lui(html: &str) -> (Lui, RenderSpy) {
  let spy = RenderSpy::default();
  let mut lui = Lui::new();
  lui.set_stylesheets(&[lui_parse::parse_stylesheet("* { margin: 0; padding: 0; border-width: 0; }").unwrap()]);
  lui.set_html(html);
  (lui, spy)
}

#[cfg(feature = "ua_whatwg")]
pub fn ua_lui(html: &str) -> (Lui, RenderSpy) {
  let spy = RenderSpy::default();
  let mut lui = Lui::new();
  lui.set_html(html);
  (lui, spy)
}

pub fn find_node_by_id_mut<'a>(node: &'a mut lui_core::HtmlNode, id: &str) -> Option<&'a mut lui_core::HtmlNode> {
  if node.id.as_deref() == Some(id) {
    return Some(node);
  }
  for child in &mut node.children {
    if let Some(found) = find_node_by_id_mut(child, id) {
      return Some(found);
    }
  }
  None
}

pub fn red_quad_y(list: &DisplayList) -> f32 {
  list
    .quads
    .iter()
    .find(|q| q.color[0] > 0.9 && q.color[1] < 0.2 && q.color[2] < 0.2)
    .map(|q| q.rect.y)
    .expect("expected red quad")
}
