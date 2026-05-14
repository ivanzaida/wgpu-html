use std::any::Any;

/// A window surface that a GPU renderer can create a swapchain from.
///
/// Wraps a platform window object. Renderers call `as_any()` and downcast
/// to the concrete type they need (e.g. `winit::window::Window`).
///
/// This avoids coupling lui-core to any windowing or GPU crate while still
/// allowing type-safe access in the renderer.
pub trait SurfaceHandle: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
}
