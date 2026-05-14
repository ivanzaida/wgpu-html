use std::sync::Arc;

use winit::event::WindowEvent;
use winit::window::Window;

use crate::{Driver, Lui};

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

/// Bind an existing winit window + wgpu renderer to a `Lui`.
pub fn bind(window: Arc<Window>, lui: &mut Lui) {
    let (w, h) = {
        let s = window.inner_size();
        (s.width.max(1), s.height.max(1))
    };
    let renderer = pollster::block_on(
        crate::renderer_wgpu::Renderer::new(window.clone(), w, h),
    );
    let driver = WinitWindow { window };
    lui.driver = Some(Box::new(driver));
    lui.renderer = Some(Box::new(renderer));
}

impl Lui {
    /// Open a window and run the event loop. Blocks until closed.
    #[cfg(feature = "winit")]
    pub fn run(mut self, width: u32, height: u32, title: &str) {
        use winit::application::ApplicationHandler;
        use winit::event_loop::{ActiveEventLoop, EventLoop};
        use winit::window::{WindowAttributes, WindowId};

        let ua = lui_parse::parse_stylesheet(UA_CSS).unwrap();
        self.set_stylesheets(&[ua]);

        struct App {
            lui: Lui,
            title: String,
            initial_size: (u32, u32),
            ready: bool,
        }

        impl ApplicationHandler for App {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                if self.ready { return; }
                let attrs = WindowAttributes::default()
                    .with_title(&self.title)
                    .with_inner_size(winit::dpi::LogicalSize::new(
                        self.initial_size.0, self.initial_size.1,
                    ));
                let window = Arc::new(event_loop.create_window(attrs).unwrap());
                crate::winit_driver::bind(window, &mut self.lui);
                self.ready = true;
            }

            fn window_event(
                &mut self,
                event_loop: &ActiveEventLoop,
                _id: WindowId,
                event: WindowEvent,
            ) {
                if let WindowEvent::CloseRequested = &event {
                    event_loop.exit();
                    return;
                }
                self.lui.handle_event(&event);
            }

            fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
                self.lui.request_redraw();
            }
        }

        let event_loop = EventLoop::new().unwrap();
        let mut app = App {
            lui: self,
            title: title.to_string(),
            initial_size: (width, height),
            ready: false,
        };
        event_loop.run_app(&mut app).unwrap();
    }

    /// Handle a winit window event.
    #[cfg(feature = "winit")]
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::RedrawRequested => {
                if self.renderer.is_none() { return false; }
                let outcome = self.render();
                if matches!(outcome, crate::display_list::FrameOutcome::Reconfigure) {
                    let (w, h) = self.driver.as_ref().unwrap().inner_size();
                    self.renderer.as_mut().unwrap().resize(w, h);
                    self.driver.as_ref().unwrap().request_redraw();
                }
                true
            }
            WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
                if let Some(r) = &mut self.renderer { r.resize(size.width, size.height); }
                if let Some(d) = &self.driver { d.request_redraw(); }
                true
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let (Some(d), Some(r)) = (&self.driver, &mut self.renderer) {
                    let (w, h) = d.inner_size();
                    r.resize(w, h);
                    d.request_redraw();
                }
                true
            }
            _ => false,
        }
    }

    pub fn request_redraw(&self) {
        if let Some(d) = &self.driver { d.request_redraw(); }
    }
}
