//! M2 demo: render a hand-built display list of colored rectangles.

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

use wgpu_html::renderer::{DisplayList, FrameOutcome, Rect, Renderer};

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
            .with_title("wgpu-html — M2: solid quads")
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
                let list = sample_scene(size.width as f32, size.height as f32);
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

/// Build a small nested-box display list to exercise the quad pipeline.
fn sample_scene(vw: f32, vh: f32) -> DisplayList {
    let mut list = DisplayList::new();

    // Outer page (white).
    let pad = 32.0;
    list.push_quad(
        Rect::new(pad, pad, vw - pad * 2.0, vh - pad * 2.0),
        [0.95, 0.95, 0.97, 1.0],
    );

    // Header bar.
    list.push_quad(
        Rect::new(pad, pad, vw - pad * 2.0, 64.0),
        [0.20, 0.40, 0.85, 1.0],
    );

    // Three columns under the header.
    let col_y = pad + 64.0 + 16.0;
    let col_h = vh - col_y - pad - 16.0;
    let gutter = 16.0;
    let avail_w = vw - pad * 2.0 - gutter * 2.0;
    let col_w = avail_w / 3.0;
    let colors: [[f32; 4]; 3] = [
        [0.92, 0.36, 0.36, 1.0],
        [0.36, 0.78, 0.46, 1.0],
        [0.97, 0.74, 0.30, 1.0],
    ];
    for (i, color) in colors.iter().enumerate() {
        let x = pad + (col_w + gutter) * i as f32;
        list.push_quad(Rect::new(x, col_y, col_w, col_h), *color);

        // Inner highlight rectangle.
        list.push_quad(
            Rect::new(x + 12.0, col_y + 12.0, col_w - 24.0, 40.0),
            [1.0, 1.0, 1.0, 0.35],
        );
    }

    list
}

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).expect("event loop run");
}
