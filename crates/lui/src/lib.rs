pub use lui_core;
pub use lui_core::display_list;
pub use lui_parse;
pub use lui_cascade;
pub use lui_layout;
pub use lui_glyph;
pub use lui_paint;

mod lui;
pub use lui::Lui;

mod render_api;
pub use render_api::{RenderBackend, RenderError};

/// Minimal trait for platform windows.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
}

#[cfg(feature = "wgpu")]
pub mod renderer_wgpu;

#[cfg(feature = "winit")]
mod winit_driver;
#[cfg(feature = "winit")]
pub use winit_driver::WinitDriver;

#[cfg(feature = "winit")]
pub use winit;
