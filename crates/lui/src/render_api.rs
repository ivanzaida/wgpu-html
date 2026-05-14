use std::{path::Path, sync::Arc};

use crate::display_list::{DisplayList, FrameOutcome};

#[derive(Debug)]
pub enum RenderError {
  UnsupportedFormat(String),
  MapFailed(String),
  EncodeFailed(String),
  Backend(Box<dyn std::error::Error + Send + Sync>),
}

impl std::fmt::Display for RenderError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::UnsupportedFormat(s) => write!(f, "unsupported format: {s}"),
      Self::MapFailed(s) => write!(f, "buffer map failed: {s}"),
      Self::EncodeFailed(s) => write!(f, "image encode failed: {s}"),
      Self::Backend(e) => write!(f, "backend error: {e}"),
    }
  }
}

impl std::error::Error for RenderError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Backend(e) => Some(e.as_ref()),
      _ => None,
    }
  }
}

/// A window that a renderer can create a GPU surface from.
pub trait WindowHandle:
  raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static
{
}

impl<T> WindowHandle for T where
  T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static
{
}

pub trait RenderBackend {
  /// Create the GPU surface from a window handle. Called by the driver
  /// once the window exists.
  fn init(&mut self, window: Arc<dyn WindowHandle>, width: u32, height: u32);
  fn resize(&mut self, width: u32, height: u32);
  fn set_clear_color(&mut self, color: [f32; 4]);
  fn upload_atlas_region(&mut self, x: u32, y: u32, w: u32, h: u32, data: &[u8]);
  fn render(&mut self, list: &DisplayList) -> FrameOutcome;
  fn render_to_rgba(&mut self, list: &DisplayList, width: u32, height: u32) -> Result<Vec<u8>, RenderError>;
  fn capture_to(&mut self, list: &DisplayList, width: u32, height: u32, path: &Path) -> Result<(), RenderError>;
  fn capture_next_frame_to(&mut self, path: std::path::PathBuf);
  fn glyph_atlas_size(&self) -> u32;
}
