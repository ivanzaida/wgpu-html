//! bevy integration for [`wgpu_html_driver`].
//!
//! WIP — will provide a bevy [`Plugin`] that creates a
//! [`Runtime`] resource and systems for input forwarding and
//! GPU rendering.

// Stub — implementation pending.
// The bevy driver will:
// 1. Implement `Driver` for bevy's `Window` wrapper
// 2. Create a `Plugin` that initializes the `Runtime` as a bevy resource
// 3. Add systems: `sync_input`, `paint_frame`, `render_html`
// 4. Bevy already owns the wgpu device/queue — use `Runtime::with_renderer`
pub struct BevyHtmlPlugin;
