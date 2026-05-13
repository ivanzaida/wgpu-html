use std::sync::Arc;

use lui_driver_winit::WinitDriver;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{WindowAttributes, WindowId},
};

const HTML: &str = include_str!("../html/test.html");

struct App {
  driver: Option<WinitDriver>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.driver.is_some() {
      return;
    }
    let attrs = WindowAttributes::default()
      .with_title("lui v2 demo")
      .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
    let window = Arc::new(event_loop.create_window(attrs).unwrap());
    self.driver = Some(WinitDriver::bind(window, HTML));
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    match &event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
        return;
      }
      _ => {}
    }
    if let Some(driver) = &mut self.driver {
      driver.handle_event(&event);
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(driver) = &self.driver {
      driver.request_redraw();
    }
  }
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
  let mut app = App { driver: None };
  event_loop.run_app(&mut app).unwrap();
}
