use std::sync::Arc;

use winit::{
  application::ApplicationHandler,
  event::{MouseScrollDelta, WindowEvent},
  event_loop::{ActiveEventLoop, EventLoop},
  keyboard::ModifiersState,
  window::{CursorIcon, Window, WindowAttributes, WindowId},
};

use crate::{Driver, Lui};

pub struct WinitDriver {
  width: u32,
  height: u32,
  title: String,
}

pub fn wheel_delta_to_css(delta: &MouseScrollDelta, scale: f32, modifiers: ModifiersState) -> (f32, f32) {
  let (mut dx, mut dy) = match delta {
    // Winit uses positive Y for wheel-up; CSS scroll deltas are positive when content
    // should move down, so invert here at the platform boundary.
    MouseScrollDelta::LineDelta(x, y) => (-*x * 40.0, -*y * 40.0),
    MouseScrollDelta::PixelDelta(pos) => (-(pos.x as f32) / scale, -(pos.y as f32) / scale),
  };
  if modifiers.shift_key() && dx.abs() < 0.001 && dy.abs() > 0.0 {
    dx = dy;
    dy = 0.0;
  }
  (dx, dy)
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

  fn run(self: Box<Self>, lui: Lui) {
    struct App {
      lui: Lui,
      title: String,
      initial_size: (u32, u32),
      window: Option<Arc<Window>>,
      modifiers: ModifiersState,
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
        self.lui.init_renderer(window.clone(), w, h);
        self.window = Some(window);
      }

      fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = &self.window else { return };
        match &event {
          WindowEvent::CloseRequested => event_loop.exit(),
          WindowEvent::CursorMoved { position, .. } => {
            let scale = window.scale_factor() as f32;
            self
              .lui
              .set_cursor_position(position.x as f32 / scale, position.y as f32 / scale);
          }
          WindowEvent::CursorLeft { .. } => {
            self.lui.clear_cursor_position();
          }
          WindowEvent::ModifiersChanged(modifiers) => {
            self.modifiers = modifiers.state();
          }
          WindowEvent::MouseInput { state, button, .. } => {
            let scale = window.scale_factor() as f32;
            let size = window.inner_size();
            let btn = match button {
              winit::event::MouseButton::Left => 0,
              winit::event::MouseButton::Middle => 1,
              winit::event::MouseButton::Right => 2,
              _ => 0,
            };
            match state {
              winit::event::ElementState::Pressed => {
                self.lui.handle_mouse_down(size.width, size.height, scale, btn);
              }
              winit::event::ElementState::Released => {
                self.lui.handle_mouse_release(size.width, size.height, scale, btn);
              }
            }
            window.request_redraw();
          }
          WindowEvent::MouseWheel { delta, .. } => {
            let scale = window.scale_factor() as f32;
            let (dx, dy) = wheel_delta_to_css(delta, scale, self.modifiers);
            let size = window.inner_size();
            if self.lui.handle_wheel(size.width, size.height, scale, dx, dy) {
              window.request_redraw();
            }
          }
          WindowEvent::RedrawRequested => {
            let size = window.inner_size();
            let scale = window.scale_factor() as f32;
            let outcome = self.lui.render_frame(size.width, size.height, scale);
            if matches!(outcome, crate::display_list::FrameOutcome::Reconfigure) {
              self.lui.renderer.resize(size.width, size.height);
              window.request_redraw();
            }
            window.set_cursor(css_cursor_to_winit(self.lui.current_cursor()));
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
      modifiers: ModifiersState::default(),
    };
    event_loop.run_app(&mut app).unwrap();
  }
}

fn css_cursor_to_winit(css: &str) -> CursorIcon {
  match css {
    "pointer" => CursorIcon::Pointer,
    "text" => CursorIcon::Text,
    "move" => CursorIcon::Move,
    "not-allowed" => CursorIcon::NotAllowed,
    "no-drop" => CursorIcon::NoDrop,
    "crosshair" => CursorIcon::Crosshair,
    "grab" => CursorIcon::Grab,
    "grabbing" => CursorIcon::Grabbing,
    "help" => CursorIcon::Help,
    "wait" => CursorIcon::Wait,
    "progress" => CursorIcon::Progress,
    "cell" => CursorIcon::Cell,
    "vertical-text" => CursorIcon::VerticalText,
    "alias" => CursorIcon::Alias,
    "copy" => CursorIcon::Copy,
    "col-resize" => CursorIcon::ColResize,
    "row-resize" => CursorIcon::RowResize,
    "e-resize" => CursorIcon::EResize,
    "n-resize" => CursorIcon::NResize,
    "ne-resize" => CursorIcon::NeResize,
    "nw-resize" => CursorIcon::NwResize,
    "s-resize" => CursorIcon::SResize,
    "se-resize" => CursorIcon::SeResize,
    "sw-resize" => CursorIcon::SwResize,
    "w-resize" => CursorIcon::WResize,
    "ew-resize" => CursorIcon::EwResize,
    "ns-resize" => CursorIcon::NsResize,
    "nesw-resize" => CursorIcon::NeswResize,
    "nwse-resize" => CursorIcon::NwseResize,
    "all-scroll" => CursorIcon::AllScroll,
    "zoom-in" => CursorIcon::ZoomIn,
    "zoom-out" => CursorIcon::ZoomOut,
    "none" => CursorIcon::Default,
    _ => CursorIcon::Default,
  }
}
