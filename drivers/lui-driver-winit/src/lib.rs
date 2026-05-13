//! Winit integration for the v2 driver.
//!
//! ```rust,no_run
//! use lui_driver_winit::WinitDriver;
//! use lui_parse::parse;
//!
//! let event_loop = winit::event_loop::EventLoop::new().unwrap();
//! // ... create window, parse HTML, bind driver, run event loop
//! ```

use std::sync::Arc;

use lui_driver::{Driver, Runtime};
use lui_parse::HtmlDocument;
use winit::event::WindowEvent;
use winit::window::Window;

/// Winit-backed driver. Owns the runtime and the parsed document.
pub struct WinitDriver {
    pub rt: Runtime<Winit, lui_renderer_wgpu::Renderer>,
    pub doc: HtmlDocument,
}

/// Winit platform bridge.
pub struct Winit {
    window: Arc<Window>,
}

impl Driver for Winit {
    fn inner_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width.max(1), size.height.max(1))
    }

    fn scale_factor(&self) -> f64 {
        self.window.scale_factor()
    }

    fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

impl WinitDriver {
    /// Create a driver bound to a winit window.
    pub fn bind(window: Arc<Window>, html: &str) -> Self {
        let (w, h) = {
            let s = window.inner_size();
            (s.width.max(1), s.height.max(1))
        };

        let renderer = pollster::block_on(
            lui_renderer_wgpu::Renderer::new(window.clone(), w, h),
        );

        let winit = Winit { window };
        let mut rt = Runtime::new(winit, renderer);

        let doc = lui_parse::parse(html);
        let ua = lui_parse::parse_stylesheet(include_str!("../../../.data/ua_whatwg_html.css")).unwrap();
        rt.set_stylesheets(&[ua]);

        Self { rt, doc }
    }

    /// Handle a winit window event. Returns true if a redraw was requested.
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                let outcome = self.rt.render_frame(&self.doc);
                match outcome {
                    lui_display_list::FrameOutcome::Reconfigure => {
                        let (w, h) = self.rt.driver.inner_size();
                        self.rt.renderer.resize(w, h);
                        self.rt.driver.request_redraw();
                    }
                    _ => {}
                }
                true
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.rt.renderer.resize(size.width, size.height);
                    self.rt.driver.request_redraw();
                }
                true
            }
            _ => false,
        }
    }

    /// Update the HTML content and request a redraw.
    pub fn set_html(&mut self, html: &str) {
        self.doc = lui_parse::parse(html);
        self.rt.driver.request_redraw();
    }
}
