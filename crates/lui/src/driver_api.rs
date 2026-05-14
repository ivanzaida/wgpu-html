use crate::Lui;

/// A window surface handle that renderers can use to create GPU surfaces.
pub trait WindowSurface:
    raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static
{
}

impl<T: raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle + Send + Sync + 'static>
    WindowSurface for T
{
}

/// Platform driver — provides window info and runs the event loop.
pub trait Driver {
    fn inner_size(&self) -> (u32, u32);
    fn scale_factor(&self) -> f64;
    fn request_redraw(&self);
    fn run(self: Box<Self>, lui: Lui);
}

pub(crate) struct NullDriver;

impl Driver for NullDriver {
    fn inner_size(&self) -> (u32, u32) { (0, 0) }
    fn scale_factor(&self) -> f64 { 1.0 }
    fn request_redraw(&self) {}
    fn run(self: Box<Self>, _lui: Lui) {}
}
