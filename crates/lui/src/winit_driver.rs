use std::sync::Arc;

use winit::{
  application::ApplicationHandler,
  event::{MouseScrollDelta, WindowEvent},
  event_loop::{ActiveEventLoop, EventLoop},
  keyboard::ModifiersState,
  window::{CursorIcon, Window, WindowAttributes, WindowId},
};

use crate::{Lui, RenderBackend};

pub struct HarnessCtx<'a> {
  pub lui: &'a mut Lui,
  pub renderer: &'a mut dyn RenderBackend,
  pub window: &'a Window,
}

pub struct WinitHarness {
  width: u32,
  height: u32,
  title: String,
}

pub fn wheel_delta_to_css(delta: &MouseScrollDelta, scale: f32, modifiers: ModifiersState) -> (f32, f32) {
  let (mut dx, mut dy) = match delta {
    MouseScrollDelta::LineDelta(x, y) => (-*x * 40.0, -*y * 40.0),
    MouseScrollDelta::PixelDelta(pos) => (-(pos.x as f32) / scale, -(pos.y as f32) / scale),
  };
  if modifiers.shift_key() && dx.abs() < 0.001 && dy.abs() > 0.0 {
    dx = dy;
    dy = 0.0;
  }
  (dx, dy)
}

impl WinitHarness {
  pub fn new(width: u32, height: u32, title: &str) -> Self {
    Self {
      width,
      height,
      title: title.to_string(),
    }
  }

  pub fn run<R: RenderBackend + 'static>(self, lui: Lui, renderer: R) {
    self.run_with(lui, renderer, |_ctx| {});
  }

  pub fn run_with<R: RenderBackend + 'static>(
    self,
    lui: Lui,
    renderer: R,
    on_frame: impl FnMut(&mut HarnessCtx) + 'static,
  ) {
    struct App<R: RenderBackend, F: FnMut(&mut HarnessCtx)> {
      lui: Lui,
      renderer: R,
      on_frame: F,
      title: String,
      initial_size: (u32, u32),
      window: Option<Arc<Window>>,
      modifiers: ModifiersState,
      clipboard: Option<arboard::Clipboard>,
    }

    impl<R: RenderBackend, F: FnMut(&mut HarnessCtx)> ApplicationHandler for App<R, F> {
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
        self.renderer.init(window.clone(), w, h);
        self.window = Some(window);
      }

      fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = &self.window else { return };
        let window = window.clone();
        match &event {
          WindowEvent::CloseRequested => event_loop.exit(),
          WindowEvent::CursorMoved { position, .. } => {
            let scale = window.scale_factor() as f32;
            self
              .lui
              .set_cursor_position(position.x as f32 / scale, position.y as f32 / scale);
            window.request_redraw();
          }
          WindowEvent::CursorLeft { .. } => {
            self.lui.clear_cursor_position();
            window.request_redraw();
          }
          WindowEvent::ModifiersChanged(modifiers) => {
            self.modifiers = modifiers.state();
          }
          WindowEvent::KeyboardInput { event, .. } => {
            if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
              let key = event
                .logical_key
                .to_text()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{:?}", event.logical_key));
              let code = format!("{keycode:?}");
              let mods = crate::KeyModifiers {
                ctrl: self.modifiers.control_key(),
                shift: self.modifiers.shift_key(),
                alt: self.modifiers.alt_key(),
                meta: self.modifiers.super_key(),
              };

              if event.state == winit::event::ElementState::Pressed && mods.ctrl {
                let size = window.inner_size();
                let scale = window.scale_factor() as f32;
                match key.as_str() {
                  "c" => {
                    if let Some(text) = self.lui.handle_copy(size.width, size.height, scale) {
                      if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(&text);
                      }
                    }
                    window.request_redraw();
                    return;
                  }
                  "x" => {
                    if let Some(text) = self.lui.handle_cut(size.width, size.height, scale) {
                      if let Some(cb) = &mut self.clipboard {
                        let _ = cb.set_text(&text);
                      }
                    }
                    window.request_redraw();
                    return;
                  }
                  "v" => {
                    if let Some(cb) = &mut self.clipboard {
                      if let Ok(text) = cb.get_text() {
                        self.lui.handle_paste(&text);
                      }
                    }
                    window.request_redraw();
                    return;
                  }
                  _ => {}
                }
              }

              match event.state {
                winit::event::ElementState::Pressed => {
                  self.lui.handle_key_down(&key, &code, event.repeat, mods);
                }
                winit::event::ElementState::Released => {
                  self.lui.handle_key_up(&key, &code, mods);
                }
              }
              window.request_redraw();
            }
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

            let Self {
              lui,
              renderer,
              on_frame,
              ..
            } = self;
            {
              let mut ctx = HarnessCtx {
                lui,
                renderer,
                window: &window,
              };
              on_frame(&mut ctx);
            }

            let outcome = lui.render_frame(renderer, size.width, size.height, scale);
            let needs_redraw = lui.take_needs_redraw();
            if matches!(outcome, crate::display_list::FrameOutcome::Reconfigure) {
              renderer.resize(size.width, size.height);
              window.request_redraw();
            } else if needs_redraw {
              window.request_redraw();
            }
            window.set_cursor(css_cursor_to_winit(lui.current_cursor()));
          }
          WindowEvent::Resized(size) if size.width > 0 && size.height > 0 => {
            self.renderer.resize(size.width, size.height);
            window.request_redraw();
          }
          WindowEvent::ScaleFactorChanged { .. } => {
            let s = window.inner_size();
            self.renderer.resize(s.width, s.height);
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
      renderer,
      on_frame,
      title: self.title,
      initial_size: (self.width, self.height),
      window: None,
      modifiers: ModifiersState::default(),
      clipboard: arboard::Clipboard::new().ok(),
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
