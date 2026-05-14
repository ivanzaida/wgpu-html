pub use lui_core;
pub use lui_core::display_list;
pub use lui_parse;
pub use lui_cascade;
pub use lui_layout;
pub use lui_glyph;

pub mod paint;

mod lui;
pub use lui::Lui;

mod render_api;
pub use render_api::{RenderBackend, RenderError};

/// Platform driver — provides window info and runs the event loop.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
    fn run(self: Box<Self>, lui: Lui);
}

#[cfg(feature = "wgpu")]
pub mod renderer_wgpu;

#[cfg(feature = "winit")]
mod winit_driver;
#[cfg(feature = "winit")]
pub use winit_driver::WinitDriver;

#[cfg(feature = "winit")]
pub use winit;
