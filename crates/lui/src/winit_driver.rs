use std::sync::Arc;

use winit::event::WindowEvent;
use winit::window::Window;

use crate::{Driver, RenderError};

static UA_CSS: &str = include_str!("../../../.data/ua_whatwg_html.css");

/// Winit window as a `Driver`.
pub struct WinitWindow {
    pub window: Arc<Window>,
}

impl Driver for WinitWindow {
    fn inner_size(&self) -> (u32, u32) {
        let s = self.window.inner_size();
        (s.width.max(1), s.height.max(1))
    }
    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }
    fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

/// Convenience type: `Lui` with a winit window + wgpu renderer.
pub type WinitDriver = crate::Lui<WinitWindow, crate::renderer_wgpu::Renderer>;

impl WinitDriver {
    /// Create a Lui instance bound to a winit window with a wgpu renderer.
    pub fn bind(window: Arc<Window>, html: &str) -> Self {
        let (w, h) = {
            let s = window.inner_size();
            (s.width.max(1), s.height.max(1))
        };

        let renderer = pollster::block_on(
            crate::renderer_wgpu::Renderer::new(window.clone(), w, h),
        );

        let driver = WinitWindow { window };
        let mut lui = crate::Lui::new(driver, renderer);
        let ua = lui_parse::parse_stylesheet(UA_CSS).unwrap();
        lui.set_stylesheets(&[ua]);
        lui.set_html(html);
        lui
    }

    /// Handle a winit window event. Returns true if a redraw was requested.
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                let outcome = self.render();
                if matches!(outcome, crate::display_list::FrameOutcome::Reconfigure) {
                    let (w, h) = self.driver.inner_size();
                    self.renderer.resize(w, h);
                    self.driver.request_redraw();
                }
                true
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                self.renderer.resize(size.width, size.height);
                self.driver.request_redraw();
                true
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let (w, h) = self.driver.inner_size();
                self.renderer.resize(w, h);
                self.driver.request_redraw();
                true
            }
            _ => false,
        }
    }

    pub fn request_redraw(&self) {
        self.driver.request_redraw();
    }
}
