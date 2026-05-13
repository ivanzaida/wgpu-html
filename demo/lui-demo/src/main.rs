use std::sync::Arc;

use lui_driver_winit::WinitDriver;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const HTML: &str = r#"
<html>
<body style="margin: 0; font-family: sans-serif; background: #1a1a2e">
  <div style="display: flex; flex-direction: column; align-items: center; padding: 40px">
    <h1 style="color: #e94560; font-size: 32px; margin-bottom: 16px">lui v2</h1>
    <p style="color: #eee; font-size: 16px; margin-bottom: 32px">HTML → parse → cascade → layout → paint → wgpu</p>

    <div style="display: flex; gap: 16px; margin-bottom: 32px">
      <div style="width: 120px; height: 120px; background: #e94560; border-radius: 12px; display: flex; align-items: center; justify-content: center">
        <span style="color: white; font-size: 14px">Block</span>
      </div>
      <div style="width: 120px; height: 120px; background: #0f3460; border-radius: 12px; display: flex; align-items: center; justify-content: center">
        <span style="color: white; font-size: 14px">Flex</span>
      </div>
      <div style="width: 120px; height: 120px; background: #533483; border-radius: 12px; display: flex; align-items: center; justify-content: center">
        <span style="color: white; font-size: 14px">Grid</span>
      </div>
    </div>

    <table style="width: 400px; border-spacing: 2px">
      <thead>
        <tr>
          <th style="background: #16213e; color: #eee; padding: 8px; height: 20px">Feature</th>
          <th style="background: #16213e; color: #eee; padding: 8px; height: 20px">Status</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td style="background: #1a1a2e; color: #aaa; padding: 8px; height: 18px">Layout engine</td>
          <td style="background: #1a1a2e; color: #4ecca3; padding: 8px; height: 18px">Done</td>
        </tr>
        <tr>
          <td style="background: #1a1a2e; color: #aaa; padding: 8px; height: 18px">Paint crate</td>
          <td style="background: #1a1a2e; color: #4ecca3; padding: 8px; height: 18px">Done</td>
        </tr>
        <tr>
          <td style="background: #1a1a2e; color: #aaa; padding: 8px; height: 18px">Winit driver</td>
          <td style="background: #1a1a2e; color: #4ecca3; padding: 8px; height: 18px">Done</td>
        </tr>
      </tbody>
    </table>
  </div>
</body>
</html>
"#;

struct App {
    driver: Option<WinitDriver>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.driver.is_some() { return; }
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
