use std::sync::Arc;

use lui_driver_winit::WinitDriver;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{WindowAttributes, WindowId},
};

const DEFAULT_HTML: &str = include_str!("../html/test.html");

fn read_html() -> String {
    use std::io::Read;
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--html") {
        if let Some(path) = args.get(pos + 1) {
            return std::fs::read_to_string(path).expect("failed to read HTML file");
        }
    }
    if atty::is(atty::Stream::Stdin) {
        return DEFAULT_HTML.to_string();
    }
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).expect("failed to read stdin");
    buf
}

fn screenshot_arg() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == "--screenshot").and_then(|i| args.get(i + 1).cloned())
}

struct App {
    html: String,
    driver: Option<WinitDriver>,
    screenshot_path: Option<String>,
    done: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.driver.is_some() { return; }
        let attrs = WindowAttributes::default()
            .with_title("lui v2 demo")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.driver = Some(WinitDriver::bind(window, &self.html));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                return;
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == winit::event::ElementState::Pressed =>
            {
                if event.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::F12) {
                    if let Some(driver) = &mut self.driver {
                        match driver.screenshot_to("screenshot.png") {
                            Ok(()) => eprintln!("[lui-demo] saved screenshot.png"),
                            Err(e) => eprintln!("[lui-demo] screenshot failed: {e:?}"),
                        }
                    }
                    return;
                }
            }
            _ => {}
        }
        if let Some(driver) = &mut self.driver {
            driver.handle_event(&event);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(ref path) = self.screenshot_path {
            if !self.done {
                if let Some(driver) = &mut self.driver {
                    driver.handle_event(&WindowEvent::RedrawRequested);
                    match driver.screenshot_to(path) {
                        Ok(()) => eprintln!("[lui-demo] saved {path}"),
                        Err(e) => eprintln!("[lui-demo] screenshot failed: {e:?}"),
                    }
                }
                self.done = true;
                event_loop.exit();
                return;
            }
        }
        if let Some(driver) = &self.driver {
            driver.request_redraw();
        }
    }
}

fn main() {
    let html = read_html();
    let screenshot_path = screenshot_arg();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App { html, driver: None, screenshot_path, done: false };
    event_loop.run_app(&mut app).unwrap();
}
