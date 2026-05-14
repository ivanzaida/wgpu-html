use std::sync::Arc;

use winit::event::WindowEvent;
use winit::window::Window;

use crate::{Lui, RenderBackend, RenderError};

static UA_CSS: &str = include_str!("../../../.data/ua_whatwg_html.css");

/// Winit + wgpu driver. Owns `Lui` + a wgpu renderer + the window.
pub struct WinitDriver {
    pub lui: Lui,
    pub renderer: lui_renderer_wgpu::Renderer,
    pub window: Arc<Window>,
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

        let mut lui = Lui::new();
        let ua = lui_parse::parse_stylesheet(UA_CSS).unwrap();
        lui.set_stylesheets(&[ua]);
        lui.set_html(html);

        Self { lui, renderer, window }
    }

    /// Handle a winit window event. Returns true if a redraw was requested.
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                let size = self.window.inner_size();
                let scale = self.window.scale_factor() as f32;
                self.lui.set_dpi_scale(scale);
                self.lui.set_viewport(
                    size.width as f32 / scale,
                    size.height as f32 / scale,
                );
                let outcome = self.lui.render_with(&mut self.renderer);
                if matches!(outcome, lui_display_list::FrameOutcome::Reconfigure) {
                    self.renderer.resize(size.width, size.height);
                    self.window.request_redraw();
                }
                true
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                self.renderer.resize(size.width, size.height);
                self.window.request_redraw();
                true
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let s = self.window.inner_size();
                self.renderer.resize(s.width, s.height);
                self.window.request_redraw();
                true
            }
            _ => false,
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn set_html(&mut self, html: &str) {
        self.lui.set_html(html);
        self.window.request_redraw();
    }

    pub fn screenshot_to(&mut self, path: impl AsRef<std::path::Path>) -> Result<(), RenderError> {
        let size = self.window.inner_size();
        let scale = self.window.scale_factor() as f32;
        self.lui.set_dpi_scale(scale);
        self.lui.set_viewport(size.width as f32 / scale, size.height as f32 / scale);
        self.lui.screenshot_with(&mut self.renderer, size.width, size.height, path)
    }

    pub fn render_to_rgba(&mut self) -> Result<Vec<u8>, RenderError> {
        let size = self.window.inner_size();
        let scale = self.window.scale_factor() as f32;
        self.lui.set_dpi_scale(scale);
        self.lui.set_viewport(size.width as f32 / scale, size.height as f32 / scale);
        self.lui.render_to_rgba_with(&mut self.renderer, size.width, size.height)
    }
}
