//! M3 demo: parse an HTML string, paint it to a `DisplayList`, render.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use wgpu_html::renderer::{FrameOutcome, Renderer};

const DOC: &str = r#"
<body style="width: 100vw; height: 100vh; background-color: #f2f2f5;">
  <div style="left: 32px; top: 32px; width: 960px; height: 64px; background-color: #3366d9;"></div>
  <div style="left: 32px; top: 112px; width: 308px; height: 600px; background-color: #ec5c5c;">
    <div style="left: 12px; top: 12px; width: 284px; height: 40px; background-color: rgba(255,255,255,0.35);"></div>
  </div>
  <div style="left: 358px; top: 112px; width: 308px; height: 600px; background-color: #5cc775;">
    <div style="left: 12px; top: 12px; width: 284px; height: 40px; background-color: rgba(255,255,255,0.35);"></div>
  </div>
  <div style="left: 684px; top: 112px; width: 308px; height: 600px; background-color: #f7bd4d;">
    <div style="left: 12px; top: 12px; width: 284px; height: 40px; background-color: rgba(255,255,255,0.35);"></div>
  </div>
</body>
"#;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = Window::default_attributes()
            .with_title("wgpu-html — M3: paint a parsed HTML tree")
            .with_inner_size(PhysicalSize::new(1024u32, 768u32));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );

        let size = window.inner_size();
        let renderer = pollster::block_on(Renderer::new(window.clone(), size.width, size.height));

        self.window = Some(window);
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        let (Some(window), Some(renderer)) = (self.window.as_ref(), self.renderer.as_mut()) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                renderer.resize(size.width, size.height);
                window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                let tree = wgpu_html::parser::parse(DOC);
                let list = wgpu_html::paint_tree(&tree, size.width as f32, size.height as f32);
                match renderer.render(&list) {
                    FrameOutcome::Presented | FrameOutcome::Skipped => {}
                    FrameOutcome::Reconfigure => {
                        renderer.resize(size.width, size.height);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("event loop run");
}
