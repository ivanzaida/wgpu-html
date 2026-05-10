//! Backend-agnostic render API.
//!
//! Defines the `RenderBackend` trait that any GPU renderer must implement.
//! The wgpu renderer (`lui-renderer`) is the reference implementation;
//! future crates (`lui-renderer-dx12`, `-vk`, `-gl`, …) provide
//! native backends.

use std::path::Path;

use lui_display_list::{DisplayList, FrameOutcome};

/// Error type returned by render operations.
#[derive(Debug)]
pub enum RenderError {
  /// The surface texture format is not supported for readback.
  UnsupportedFormat(String),
  /// GPU buffer mapping failed.
  MapFailed(String),
  /// Image encoding (PNG) failed.
  EncodeFailed(String),
  /// Backend-specific error.
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

/// Trait that every render backend must implement.
///
/// The display list (produced by layout + paint) is backend-agnostic.
/// This trait maps it onto a concrete GPU API.
pub trait RenderBackend {
  /// Resize the render surface.
  fn resize(&mut self, width: u32, height: u32);

  /// Set the clear color used at the start of each frame.
  fn set_clear_color(&mut self, color: [f32; 4]);

  /// Upload a rectangular region of the glyph atlas to the GPU.
  /// Called once per dirty rect returned by `Atlas::flush_dirty`.
  fn upload_atlas_region(&mut self, x: u32, y: u32, w: u32, h: u32, data: &[u8]);

  /// Submit a display list to the GPU and present the frame.
  fn render(&mut self, list: &DisplayList) -> FrameOutcome;

  /// Render a display list into an offscreen buffer and return RGBA8 pixels.
  fn render_to_rgba(
    &mut self,
    list: &DisplayList,
    width: u32,
    height: u32,
  ) -> Result<Vec<u8>, RenderError>;

  /// Render a display list into an offscreen buffer and save as PNG.
  fn capture_to(
    &mut self,
    list: &DisplayList,
    width: u32,
    height: u32,
    path: &Path,
  ) -> Result<(), RenderError>;

  /// Schedule the next on-screen frame to be saved to `path` as PNG.
  fn capture_next_frame_to(&mut self, path: std::path::PathBuf);

  /// The glyph atlas size this backend was created with.
  fn glyph_atlas_size(&self) -> u32;
}
