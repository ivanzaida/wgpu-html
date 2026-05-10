//! winit integration for [`lui_driver`].
//!
//! ## Quick start
//!
//! ```ignore
//! use lui_driver_winit::WinitDriver;
//!
//! // In your ApplicationHandler::resumed():
//! let window = Arc::new(event_loop.create_window(attrs).unwrap());
//! let tree = lui_parser::parse(html);
//! let mut driver = WinitDriver::bind(window, tree);
//!
//! // In window_event():
//! driver.handle_event(&event);
//!
//! // That's it — dispatch, rendering, and redraw requests
//! // are handled internally.
//! ```

use std::sync::Arc;

use lui::layout::Cursor;
use lui::PipelineTimings;
use lui_driver::{Driver, Runtime};
use lui_tree::{Modifier, MouseButton, Tree};

pub use winit::{
  event::{KeyEvent, WindowEvent},
  event_loop::ActiveEventLoop,
  window::WindowId,
};
use winit::{
  event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta},
  keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
  window::Window,
};

// ── WinitDriver ────────────────────────────────────────────────────────────

/// Binds a winit window to a [`Tree`]. Handles event dispatch and
/// rendering internally.
///
/// Users own the event loop and `ApplicationHandler`. The driver is
/// a passive helper — call [`handle_event`](Self::handle_event) from
/// `window_event()` and it does the rest.
pub struct WinitDriver {
  pub rt: Runtime<Lui, lui_renderer::Renderer>,
  pub tree: Tree,
}

impl WinitDriver {
  /// Wire a winit window to a tree.
  pub fn bind(window: Arc<Window>, tree: Tree) -> Self {
    let size = window.inner_size();
    let renderer = pollster::block_on(
      lui_renderer::Renderer::new(window.clone(), size.width, size.height),
    );
    let driver = Lui { window, clipboard: std::cell::RefCell::new(None) };
    let rt = Runtime::new(driver, renderer);
    Self { rt, tree }
  }

  /// Route a [`WindowEvent`]. Dispatches to the tree, renders on
  /// `RedrawRequested`, and requests redraws automatically.
  ///
  /// Returns render timings on `RedrawRequested`, `None` for other events.
  pub fn handle_event(&mut self, event: &WindowEvent) -> Option<PipelineTimings> {
    match event {
      WindowEvent::RedrawRequested => {
        let timings = self.rt.render_frame(&mut self.tree);
        Some(timings)
      }
      other => {
        if dispatch(other, &mut self.rt, &mut self.tree) {
          self.rt.driver.window.request_redraw();
        }
        None
      }
    }
  }

  /// Dispatch an event into an external tree (not the driver's own).
  /// Used for secondary windows (e.g. devtools) where the runtime
  /// belongs to this driver but the tree lives elsewhere.
  ///
  /// Returns `true` if the caller should request a redraw.
  pub fn dispatch_to(&mut self, event: &WindowEvent, tree: &mut Tree) -> bool {
    dispatch(event, &mut self.rt, tree)
  }

  /// Render an external tree (not the driver's own).
  /// Returns pipeline timings.
  pub fn render(&mut self, tree: &mut Tree) -> PipelineTimings {
    self.rt.render_frame(tree)
  }

  // ── Tree access ──────────────────────────────────────────────

  pub fn tree(&self) -> &Tree {
    &self.tree
  }

  pub fn tree_mut(&mut self) -> &mut Tree {
    &mut self.tree
  }

  // ── Window access ────────────────────────────────────────────

  pub fn window(&self) -> &Arc<Window> {
    &self.rt.driver.window
  }

  pub fn window_id(&self) -> WindowId {
    self.rt.driver.window.id()
  }

  pub fn request_redraw(&self) {
    self.rt.driver.window.request_redraw();
  }

}

// ── Internal Driver impl ───────────────────────────────────────────────────

pub struct Lui {
  pub(crate) window: Arc<Window>,
  clipboard: std::cell::RefCell<Option<arboard::Clipboard>>,
}

impl Driver for Lui {
  fn inner_size(&self) -> (u32, u32) {
    let s = self.window.inner_size();
    (s.width, s.height)
  }

  fn scale_factor(&self) -> f64 {
    self.window.scale_factor()
  }

  fn request_redraw(&self) {
    self.window.request_redraw();
  }

  fn set_cursor(&self, cursor: Cursor) {
    self.window.set_cursor(css_cursor_to_winit(cursor));
  }

  fn set_clipboard_text(&self, text: &str) {
    let mut cb = self.clipboard.borrow_mut();
    let cb = cb.get_or_insert_with(|| arboard::Clipboard::new().expect("clipboard init"));
    let _ = cb.set_text(text);
  }

  fn get_clipboard_text(&self) -> Option<String> {
    let mut cb = self.clipboard.borrow_mut();
    let cb = cb.get_or_insert_with(|| arboard::Clipboard::new().expect("clipboard init"));
    cb.get_text().ok()
  }
}

// ── Event dispatch (internal) ──────────────────────────────────────────────

fn dispatch(event: &WindowEvent, rt: &mut Runtime<Lui, lui_renderer::Renderer>, tree: &mut Tree) -> bool {
  match event {
    WindowEvent::Resized(size) => {
      rt.on_resize(tree, size.width, size.height);
      true
    }

    WindowEvent::ScaleFactorChanged { .. } => {
      rt.on_scale_change();
      true
    }

    WindowEvent::CursorMoved { position, .. } => rt.on_pointer_move(tree, position.x as f32, position.y as f32),

    WindowEvent::CursorLeft { .. } => rt.on_pointer_leave(tree),

    WindowEvent::MouseInput {
      state: btn_state,
      button,
      ..
    } => {
      let pressed = *btn_state == ElementState::Pressed;
      let btn = mouse_button(*button);
      rt.on_mouse_button(tree, btn, pressed)
    }

    WindowEvent::MouseWheel { delta, .. } => {
      let scale = tree.effective_dpi_scale(rt.driver.scale_factor() as f32);
      let prevented = match delta {
        MouseScrollDelta::LineDelta(x, y) => rt.on_wheel_event(
          tree,
          0.0,
          0.0,
          *x as f64,
          *y as f64,
          lui::events::WheelDeltaMode::Line,
        ),
        MouseScrollDelta::PixelDelta(phys_pos) => rt.on_wheel_event(
          tree,
          0.0,
          0.0,
          phys_pos.x as f64,
          phys_pos.y as f64,
          lui::events::WheelDeltaMode::Pixel,
        ),
      };
      if prevented {
        return true;
      }
      let mut pixel_dy = wheel_delta_to_pixels(*delta, scale);
      let mut pixel_dx = match delta {
        MouseScrollDelta::LineDelta(x, _) => *x * 48.0 * scale,
        MouseScrollDelta::PixelDelta(pos) => pos.x as f32,
      };
      if tree.interaction.modifiers.shift && pixel_dx.abs() < 0.5 {
        pixel_dx = pixel_dy;
        pixel_dy = 0.0;
      }
      rt.on_wheel(tree, pixel_dy, pixel_dx, scale)
    }

    WindowEvent::ModifiersChanged(mods) => {
      let state = mods.state();
      tree.set_modifier(Modifier::Shift, state.shift_key());
      tree.set_modifier(Modifier::Ctrl, state.control_key());
      tree.set_modifier(Modifier::Alt, state.alt_key());
      tree.set_modifier(Modifier::Meta, state.super_key());
      false
    }

    WindowEvent::KeyboardInput { event, .. } => {
      let pressed = event.state == ElementState::Pressed;
      update_modifiers(tree, event);
      let (key_str, code_str, text_opt) = extract_key_parts(event);
      rt.on_key(tree, &key_str, &code_str, pressed, event.repeat, text_opt.as_deref());
      pressed
    }

    _ => false,
  }
}

// ── Type translators ───────────────────────────────────────────────────────

fn mouse_button(button: WinitMouseButton) -> MouseButton {
  match button {
    WinitMouseButton::Left => MouseButton::Primary,
    WinitMouseButton::Right => MouseButton::Secondary,
    WinitMouseButton::Middle => MouseButton::Middle,
    WinitMouseButton::Back => MouseButton::Other(3),
    WinitMouseButton::Forward => MouseButton::Other(4),
    WinitMouseButton::Other(n) => MouseButton::Other(n.min(255) as u8),
  }
}

fn keycode_to_modifier(key: KeyCode) -> Option<Modifier> {
  Some(match key {
    KeyCode::ControlLeft | KeyCode::ControlRight => Modifier::Ctrl,
    KeyCode::ShiftLeft | KeyCode::ShiftRight => Modifier::Shift,
    KeyCode::AltLeft | KeyCode::AltRight => Modifier::Alt,
    KeyCode::SuperLeft | KeyCode::SuperRight => Modifier::Meta,
    _ => return None,
  })
}

fn extract_key_parts(event: &KeyEvent) -> (String, String, Option<String>) {
  let code_str = match event.physical_key {
    PhysicalKey::Code(key) => keycode_to_dom_code(key),
    PhysicalKey::Unidentified(_) => "Unidentified",
  };
  let key_str: String = match &event.logical_key {
    Key::Named(named) => named_key_to_dom(named).to_string(),
    Key::Character(ch) => ch.to_string(),
    Key::Unidentified(_) | Key::Dead(_) => "Unidentified".to_string(),
  };
  let text = event.text.as_ref().map(|t| t.to_string());
  (key_str, code_str.to_string(), text)
}

fn update_modifiers(tree: &mut Tree, event: &KeyEvent) {
  if let PhysicalKey::Code(key) = event.physical_key {
    if let Some(modifier) = keycode_to_modifier(key) {
      let down = event.state == ElementState::Pressed;
      tree.set_modifier(modifier, down);
    }
  }
}

fn keycode_to_dom_code(key: KeyCode) -> &'static str {
  use KeyCode::*;
  match key {
    KeyA => "KeyA", KeyB => "KeyB", KeyC => "KeyC", KeyD => "KeyD",
    KeyE => "KeyE", KeyF => "KeyF", KeyG => "KeyG", KeyH => "KeyH",
    KeyI => "KeyI", KeyJ => "KeyJ", KeyK => "KeyK", KeyL => "KeyL",
    KeyM => "KeyM", KeyN => "KeyN", KeyO => "KeyO", KeyP => "KeyP",
    KeyQ => "KeyQ", KeyR => "KeyR", KeyS => "KeyS", KeyT => "KeyT",
    KeyU => "KeyU", KeyV => "KeyV", KeyW => "KeyW", KeyX => "KeyX",
    KeyY => "KeyY", KeyZ => "KeyZ",
    Digit0 => "Digit0", Digit1 => "Digit1", Digit2 => "Digit2",
    Digit3 => "Digit3", Digit4 => "Digit4", Digit5 => "Digit5",
    Digit6 => "Digit6", Digit7 => "Digit7", Digit8 => "Digit8",
    Digit9 => "Digit9",
    Space => "Space", Minus => "Minus", Equal => "Equal",
    BracketLeft => "BracketLeft", BracketRight => "BracketRight",
    Backslash => "Backslash", Semicolon => "Semicolon", Quote => "Quote",
    Comma => "Comma", Period => "Period", Slash => "Slash",
    Backquote => "Backquote",
    Enter => "Enter", NumpadEnter => "NumpadEnter", Tab => "Tab",
    Backspace => "Backspace", Delete => "Delete", Escape => "Escape",
    Home => "Home", End => "End", PageUp => "PageUp", PageDown => "PageDown",
    ArrowUp => "ArrowUp", ArrowDown => "ArrowDown",
    ArrowLeft => "ArrowLeft", ArrowRight => "ArrowRight",
    Insert => "Insert",
    ShiftLeft => "ShiftLeft", ShiftRight => "ShiftRight",
    ControlLeft => "ControlLeft", ControlRight => "ControlRight",
    AltLeft => "AltLeft", AltRight => "AltRight",
    SuperLeft => "MetaLeft", SuperRight => "MetaRight",
    CapsLock => "CapsLock",
    F1 => "F1", F2 => "F2", F3 => "F3", F4 => "F4",
    F5 => "F5", F6 => "F6", F7 => "F7", F8 => "F8",
    F9 => "F9", F10 => "F10", F11 => "F11", F12 => "F12",
    _ => "Unidentified",
  }
}

fn wheel_delta_to_pixels(delta: MouseScrollDelta, scale: f32) -> f32 {
  match delta {
    MouseScrollDelta::LineDelta(_x, y) => -y * 48.0 * scale,
    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
  }
}

fn named_key_to_dom(key: &NamedKey) -> &'static str {
  match key {
    NamedKey::Alt => "Alt", NamedKey::ArrowDown => "ArrowDown",
    NamedKey::ArrowLeft => "ArrowLeft", NamedKey::ArrowRight => "ArrowRight",
    NamedKey::ArrowUp => "ArrowUp", NamedKey::Backspace => "Backspace",
    NamedKey::CapsLock => "CapsLock", NamedKey::Control => "Control",
    NamedKey::Delete => "Delete", NamedKey::End => "End",
    NamedKey::Enter => "Enter", NamedKey::Escape => "Escape",
    NamedKey::F1 => "F1", NamedKey::F2 => "F2", NamedKey::F3 => "F3",
    NamedKey::F4 => "F4", NamedKey::F5 => "F5", NamedKey::F6 => "F6",
    NamedKey::F7 => "F7", NamedKey::F8 => "F8", NamedKey::F9 => "F9",
    NamedKey::F10 => "F10", NamedKey::F11 => "F11", NamedKey::F12 => "F12",
    NamedKey::Home => "Home", NamedKey::Insert => "Insert",
    NamedKey::Meta => "Meta", NamedKey::NumLock => "NumLock",
    NamedKey::PageDown => "PageDown", NamedKey::PageUp => "PageUp",
    NamedKey::Pause => "Pause", NamedKey::PrintScreen => "PrintScreen",
    NamedKey::ScrollLock => "ScrollLock", NamedKey::Shift => "Shift",
    NamedKey::Space => " ", NamedKey::Tab => "Tab",
    _ => "Unidentified",
  }
}

fn css_cursor_to_winit(cursor: Cursor) -> winit::window::CursorIcon {
  use Cursor as C;
  use winit::window::CursorIcon as I;
  match cursor {
    C::Auto | C::Default => I::Default,
    C::Pointer => I::Pointer,
    C::Text => I::Text,
    C::Move => I::Move,
    C::NotAllowed => I::NotAllowed,
    C::Grab => I::Grab,
    C::Grabbing => I::Grabbing,
    C::Crosshair => I::Crosshair,
    C::Wait => I::Wait,
    C::Help => I::Help,
    C::Progress => I::Progress,
    C::None => I::Default,
    C::Resize => I::NwseResize,
    C::ColResize => I::ColResize,
    C::RowResize => I::RowResize,
    C::Raw(_) => I::Default,
  }
}
