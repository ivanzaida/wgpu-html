use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

use crate::{Driver, Lui};
use lui_core::SurfaceHandle;

/// Winit window wrapped as a `SurfaceHandle`.
pub(crate) struct WinitSurface {
    pub(crate) window: Arc<Window>,
}

impl SurfaceHandle for WinitSurface {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

static UA_CSS: &str = include_str!("../../../.data/ua_whatwg_html.css");

pub struct WinitDriver {
  width: u32,
  height: u32,
  title: String,
}

impl WinitDriver {
  pub fn new(width: u32, height: u32, title: &str) -> Self {
    Self {
      width,
      height,
      title: title.to_string(),
    }
  }
}

impl Driver for WinitDriver {
  fn inner_size(&self) -> (u32, u32) {
    (self.width, self.height)
  }
  fn scale_factor(&self) -> f64 {
    1.0
  }
  fn request_redraw(&self) {}

  fn run(self: Box<Self>, mut lui: Lui) {
    let ua = lui_parse::parse_stylesheet(UA_CSS).unwrap();
    lui.set_stylesheets(&[ua]);

    struct App {
      lui: Lui,
      title: String,
      initial_size: (u32, u32),
      window: Option<Arc<Window>>,
    }

    impl ApplicationHandler for App {
      fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
          return;
        }
        let attrs = WindowAttributes::default()
          .with_title(&self.title)
          .with_inner_size(winit::dpi::LogicalSize::new(self.initial_size.0, self.initial_size.1));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let (w, h) = {
          let s = window.inner_size();
          (s.width.max(1), s.height.max(1))
        };
        let surface: Arc<dyn SurfaceHandle> = Arc::new(WinitSurface { window: window.clone() });
        self.lui.renderer.init_surface(surface, w, h);
        self.window = Some(window);
      }

      fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = &self.window else { return };
        match &event {
          WindowEvent::CloseRequested => event_loop.exit(),
          WindowEvent::RedrawRequested => {
            let size = window.inner_size();
            let scale = window.scale_factor() as f32;
            let outcome = self.lui.render_frame(size.width, size.height, scale);
            if matches!(outcome, crate::display_list::FrameOutcome::Reconfigure) {
              self.lui.renderer.resize(size.width, size.height);
              window.request_redraw();
            }
          }
          WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
            self.lui.renderer.resize(size.width, size.height);
            window.request_redraw();
          }
          WindowEvent::ScaleFactorChanged { .. } => {
            let s = window.inner_size();
            self.lui.renderer.resize(s.width, s.height);
            window.request_redraw();
          }
          _ => {}
        }
      }

      fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(w) = &self.window {
          w.request_redraw();
        }
      }
    }

    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
      lui,
      title: self.title,
      initial_size: (self.width, self.height),
      window: None,
    };
    event_loop.run_app(&mut app).unwrap();
  }
}
