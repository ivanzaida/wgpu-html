//! M4 demo: parse an HTML string, lay it out via block flow, paint, render.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use wgpu_html::renderer::{FrameOutcome, Renderer};

/// Block flow: body fills the viewport, header on top, then three cards
/// stacking below it. Each card has padding and an inner highlight strip.
const DOC: &str = r#"
<body style="width: 100vw; height: 100vh; background-color: #f2f2f5; padding: 32px;">
  <div style="height: 64px; background-color: #3366d9; margin-bottom: 16px;"></div>

  <div style="background-color: #ec5c5c; padding: 12px; margin-bottom: 12px;">
    <div style="height: 40px; background-color: rgba(255,255,255,0.35);"></div>
  </div>

  <div style="background-color: #5cc775; padding: 12px; margin-bottom: 12px;">
    <div style="height: 40px; background-color: rgba(255,255,255,0.35);"></div>
  </div>

  <div style="background-color: #f7bd4d; padding: 12px;">
    <div style="height: 40px; background-color: rgba(255,255,255,0.35);"></div>
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
            .with_title("wgpu-html — M4: block layout")
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
