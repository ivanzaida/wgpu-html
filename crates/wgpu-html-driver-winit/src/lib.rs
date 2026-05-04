//! winit integration for [`wgpu_html_driver`].
//!
//! ## Quick start
//!
//! ```ignore
//! use wgpu_html_driver_winit::{WgpuHtml, WinitRuntime, dispatch};
//!
//! let window = Arc::new(winit::window::Window::new(...)?);
//! let driver = WgpuHtml { window: window.clone() };
//! let mut rt = WinitRuntime::new(driver, 800, 600);
//!
//! // In your event loop:
//! event_loop.run(move |event, elwt| match event {
//!     WindowEvent::RedrawRequested => { rt.render_frame(&mut tree); }
//!     other => {
//!         if dispatch(&other, &mut rt, &mut tree) {
//!             window.request_redraw();
//!         }
//!     }
//! })?
//! ```
//!
//! ## Type translators
//!
//! Free functions at module root convert winit types into
//! engine-compatible data for use with [`Runtime::on_key`] and
//! manual modifier tracking.

use std::sync::Arc;

use wgpu_html::layout::Cursor;
use wgpu_html_driver::{Driver, Runtime};
use wgpu_html_tree::{Modifier, MouseButton, Tree};
pub use wgpu_html_tree::{SystemFontVariant, register_system_fonts, system_font_variants};
use winit::{
  dpi::PhysicalSize,
  event::{ElementState, KeyEvent, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent},
  event_loop::ActiveEventLoop,
  keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
  window::Window,
};

// ── Driver implementation ───────────────────────────────────────────────────

/// winit-specific [`Driver`] adapter.
///
/// Wrap an `Arc<winit::Window>` and use with [`Runtime`].
pub struct WgpuHtml {
  pub window: Arc<Window>,
}

impl Driver for WgpuHtml {
  type Surface = Window;

  fn surface(&self) -> &Arc<Window> {
    &self.window
  }

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
}

/// Convenience alias: [`Runtime`] parameterized on [`WgpuHtml`].
pub type WinitRuntime = Runtime<WgpuHtml>;

/// Create a self-contained winit window + driver + runtime in one call.
///
/// Useful for secondary windows (devtools, etc.) or single-window apps
/// that want the quickest possible setup.
pub fn new_window(event_loop: &ActiveEventLoop, title: &str, width: u32, height: u32) -> WinitRuntime {
  let attrs = Window::default_attributes()
    .with_title(title)
    .with_inner_size(PhysicalSize::new(width, height));
  let window = Arc::new(event_loop.create_window(attrs).expect("failed to create window"));
  let driver = WgpuHtml { window };
  WinitRuntime::new(driver, width, height)
}

// ── Event dispatch ──────────────────────────────────────────────────────────

/// Route a winit [`WindowEvent`] into the [`Runtime`] methods.
///
/// Returns `true` if the caller should request a redraw. Events that
/// don't map to any engine operation return `false` silently.
pub fn dispatch(event: &WindowEvent, rt: &mut WinitRuntime, tree: &mut Tree) -> bool {
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
      // Fire preventDefault-able wheel event first.
      let prevented = match delta {
        MouseScrollDelta::LineDelta(x, y) => rt.on_wheel_event(
          tree,
          0.0,
          0.0,
          *x as f64,
          *y as f64,
          wgpu_html::events::WheelDeltaMode::Line,
        ),
        MouseScrollDelta::PixelDelta(phys_pos) => rt.on_wheel_event(
          tree,
          0.0,
          0.0,
          phys_pos.x as f64,
          phys_pos.y as f64,
          wgpu_html::events::WheelDeltaMode::Pixel,
        ),
      };
      if prevented {
        return true;
      }
      let pixel_dy = wheel_delta_to_pixels(*delta, scale);
      let pixel_dx = match delta {
        MouseScrollDelta::LineDelta(x, _) => -*x * 48.0 * scale,
        MouseScrollDelta::PixelDelta(pos) => -pos.x as f32,
      };
      rt.on_wheel(tree, pixel_dy, pixel_dx, scale)
    }

    WindowEvent::KeyboardInput { event, .. } => {
      let pressed = event.state == ElementState::Pressed;

      // Modifier sync.
      update_modifiers(tree, event);

      // Extract key/code/text for the engine.
      let (key_str, code_str, text_opt) = extract_key_parts(event);

      rt.on_key(tree, &key_str, &code_str, pressed, event.repeat, text_opt.as_deref());

      pressed
    }

    _ => false,
  }
}

// ── Type translators ────────────────────────────────────────────────────────

/// Map a winit `MouseButton` to the engine's [`MouseButton`].
pub fn mouse_button(button: WinitMouseButton) -> MouseButton {
  match button {
    WinitMouseButton::Left => MouseButton::Primary,
    WinitMouseButton::Right => MouseButton::Secondary,
    WinitMouseButton::Middle => MouseButton::Middle,
    WinitMouseButton::Back => MouseButton::Other(3),
    WinitMouseButton::Forward => MouseButton::Other(4),
    WinitMouseButton::Other(n) => MouseButton::Other(n.min(255) as u8),
  }
}

/// Map a winit `KeyCode` to a [`Modifier`] if it is one of the four
/// modifier keys; `None` otherwise.
pub fn keycode_to_modifier(key: KeyCode) -> Option<Modifier> {
  Some(match key {
    KeyCode::ControlLeft | KeyCode::ControlRight => Modifier::Ctrl,
    KeyCode::ShiftLeft | KeyCode::ShiftRight => Modifier::Shift,
    KeyCode::AltLeft | KeyCode::AltRight => Modifier::Alt,
    KeyCode::SuperLeft | KeyCode::SuperRight => Modifier::Meta,
    _ => return None,
  })
}

/// Map a winit `NamedKey` to its DOM `KeyboardEvent.key` string.
fn named_key_to_dom(key: &NamedKey) -> &'static str {
  match key {
    NamedKey::Alt => "Alt",
    NamedKey::ArrowDown => "ArrowDown",
    NamedKey::ArrowLeft => "ArrowLeft",
    NamedKey::ArrowRight => "ArrowRight",
    NamedKey::ArrowUp => "ArrowUp",
    NamedKey::Backspace => "Backspace",
    NamedKey::CapsLock => "CapsLock",
    NamedKey::Control => "Control",
    NamedKey::Delete => "Delete",
    NamedKey::End => "End",
    NamedKey::Enter => "Enter",
    NamedKey::Escape => "Escape",
    NamedKey::F1 => "F1",
    NamedKey::F2 => "F2",
    NamedKey::F3 => "F3",
    NamedKey::F4 => "F4",
    NamedKey::F5 => "F5",
    NamedKey::F6 => "F6",
    NamedKey::F7 => "F7",
    NamedKey::F8 => "F8",
    NamedKey::F9 => "F9",
    NamedKey::F10 => "F10",
    NamedKey::F11 => "F11",
    NamedKey::F12 => "F12",
    NamedKey::Home => "Home",
    NamedKey::Insert => "Insert",
    NamedKey::Meta => "Meta",
    NamedKey::NumLock => "NumLock",
    NamedKey::PageDown => "PageDown",
    NamedKey::PageUp => "PageUp",
    NamedKey::Pause => "Pause",
    NamedKey::PrintScreen => "PrintScreen",
    NamedKey::ScrollLock => "ScrollLock",
    NamedKey::Shift => "Shift",
    NamedKey::Space => " ",
    NamedKey::Tab => "Tab",
    _ => "Unidentified",
  }
}

/// Extract (`key`, `code`, optional text) from a winit `KeyEvent`.
///
/// `key` is the DOM `KeyboardEvent.key` value (layout-dependent).
/// `code` is the DOM `KeyboardEvent.code` value (layout-independent).
/// `text` is the composed character for text insertion, if any.
pub fn extract_key_parts(event: &KeyEvent) -> (String, String, Option<String>) {
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

/// Map a winit physical `KeyCode` to a DOM `KeyboardEvent.code`
/// string. Layout-independent (always reflects the physical key).
pub fn keycode_to_dom_code(key: KeyCode) -> &'static str {
  use KeyCode::*;
  match key {
    KeyA => "KeyA",
    KeyB => "KeyB",
    KeyC => "KeyC",
    KeyD => "KeyD",
    KeyE => "KeyE",
    KeyF => "KeyF",
    KeyG => "KeyG",
    KeyH => "KeyH",
    KeyI => "KeyI",
    KeyJ => "KeyJ",
    KeyK => "KeyK",
    KeyL => "KeyL",
    KeyM => "KeyM",
    KeyN => "KeyN",
    KeyO => "KeyO",
    KeyP => "KeyP",
    KeyQ => "KeyQ",
    KeyR => "KeyR",
    KeyS => "KeyS",
    KeyT => "KeyT",
    KeyU => "KeyU",
    KeyV => "KeyV",
    KeyW => "KeyW",
    KeyX => "KeyX",
    KeyY => "KeyY",
    KeyZ => "KeyZ",
    Digit0 => "Digit0",
    Digit1 => "Digit1",
    Digit2 => "Digit2",
    Digit3 => "Digit3",
    Digit4 => "Digit4",
    Digit5 => "Digit5",
    Digit6 => "Digit6",
    Digit7 => "Digit7",
    Digit8 => "Digit8",
    Digit9 => "Digit9",
    Space => "Space",
    Minus => "Minus",
    Equal => "Equal",
    BracketLeft => "BracketLeft",
    BracketRight => "BracketRight",
    Backslash => "Backslash",
    Semicolon => "Semicolon",
    Quote => "Quote",
    Comma => "Comma",
    Period => "Period",
    Slash => "Slash",
    Backquote => "Backquote",
    Enter => "Enter",
    NumpadEnter => "NumpadEnter",
    Tab => "Tab",
    Backspace => "Backspace",
    Delete => "Delete",
    Escape => "Escape",
    Home => "Home",
    End => "End",
    PageUp => "PageUp",
    PageDown => "PageDown",
    ArrowUp => "ArrowUp",
    ArrowDown => "ArrowDown",
    ArrowLeft => "ArrowLeft",
    ArrowRight => "ArrowRight",
    Insert => "Insert",
    ShiftLeft => "ShiftLeft",
    ShiftRight => "ShiftRight",
    ControlLeft => "ControlLeft",
    ControlRight => "ControlRight",
    AltLeft => "AltLeft",
    AltRight => "AltRight",
    SuperLeft => "MetaLeft",
    SuperRight => "MetaRight",
    CapsLock => "CapsLock",
    F1 => "F1",
    F2 => "F2",
    F3 => "F3",
    F4 => "F4",
    F5 => "F5",
    F6 => "F6",
    F7 => "F7",
    F8 => "F8",
    F9 => "F9",
    F10 => "F10",
    F11 => "F11",
    F12 => "F12",
    _ => "Unidentified",
  }
}

/// If `event` involves a modifier key, update `tree.interaction.modifiers`.
pub fn update_modifiers(tree: &mut Tree, event: &KeyEvent) {
  if let PhysicalKey::Code(key) = event.physical_key {
    if let Some(modifier) = keycode_to_modifier(key) {
      let down = event.state == ElementState::Pressed;
      tree.set_modifier(modifier, down);
    }
  }
}

/// Convert a winit `MouseScrollDelta` into vertical pixel delta.
/// Positive = content moves up (user scrolled down).
pub fn wheel_delta_to_pixels(delta: MouseScrollDelta, scale: f32) -> f32 {
  match delta {
    MouseScrollDelta::LineDelta(_x, y) => -y * 48.0 * scale,
    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
  }
}

// ── Cursor mapping ─────────────────────────────────────────────────────────

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
