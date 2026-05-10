//! egui integration for [`lui_driver`].
//!
//! Provides [`EguiRunner`] — wraps a [`Runtime`] and runs the full
//! cascade→layout→paint pipeline inside an egui region. Input is
//! read from the egui context (pointer polling, key event queue,
//! modifier state); the resulting [`DisplayList`] is returned for GPU
//! rendering via the host's wgpu backend.
//!
//! ## Quick start (eframe)
//!
//! ```ignore
//! use lui_driver_egui::EguiRunner;
//!
//! struct MyApp { html: EguiRunner<winit::window::Window> }
//!
//! impl eframe::App for MyApp {
//!     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//!         egui::CentralPanel::default().show(ctx, |ui| {
//!             self.html.show(ui, &mut self.tree, ui.available_size());
//!         });
//!     }
//! }
//! ```

use std::sync::Arc;

use egui::{Align2, Color32, Event, FontId, PointerButton, Pos2, Rect as EguiRect, Response, Sense, Ui, Vec2};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use lui::{
  PipelineTimings, interactivity,
  layout::LayoutBox,
  renderer::{DisplayList, Rect},
};
use lui_driver::{Driver, Runtime};
use lui_tree::{Modifier, MouseButton, Tree};

// ── Driver ──────────────────────────────────────────────────────────────────

/// egui-specific [`Driver`] adapter.
///
/// Generic over the host window type `W`. In eframe apps this is
/// typically `winit::window::Window`.
pub struct EguiHtml<W> {
  pub window: Arc<W>,
  size: (u32, u32),
  scale: f64,
}

impl<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static> Driver for EguiHtml<W> {
  type Surface = W;

  fn surface(&self) -> &Arc<W> {
    &self.window
  }

  fn inner_size(&self) -> (u32, u32) {
    self.size
  }

  fn scale_factor(&self) -> f64 {
    self.scale
  }

  fn request_redraw(&self) {}

  fn set_cursor(&self, _cursor: lui::layout::Cursor) {}
}

impl<W> EguiHtml<W> {
  fn set_viewport(&mut self, size: (u32, u32), scale: f64) {
    self.size = size;
    self.scale = scale;
  }
}

// ── Runner ──────────────────────────────────────────────────────────────────

/// High-level egui integration.
///
/// Wraps a [`Runtime<EguiHtml<W>>`]. Call [`Self::show`] inside an
/// egui region each frame; it synchronises modifiers, key events, and
/// pointer state, runs the pipeline, and paints a quad+text preview.
pub struct EguiRunner<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static> {
  rt: Runtime<EguiHtml<W>>,
  primary_down: bool,
  secondary_down: bool,
  middle_down: bool,
}

impl<W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static> EguiRunner<W> {
  /// Create a runner backed by the host window `w`.
  ///
  /// `width` and `height` are used only for initial [`Renderer`]
  /// creation; the viewport is updated each frame from the egui
  /// region passed to [`Self::show`].
  pub fn new(window: Arc<W>, width: u32, height: u32) -> Self {
    let driver = EguiHtml {
      window,
      size: (width, height),
      scale: 1.0,
    };
    Self {
      rt: Runtime::new(driver, width, height),
      primary_down: false,
      secondary_down: false,
      middle_down: false,
    }
  }

  /// Access the underlying [`Runtime`].
  pub fn runtime(&self) -> &Runtime<EguiHtml<W>> {
    &self.rt
  }

  pub fn runtime_mut(&mut self) -> &mut Runtime<EguiHtml<W>> {
    &mut self.rt
  }

  /// Latest layout tree (populated after at least one call to
  /// [`Self::show`]).
  pub fn layout(&self) -> Option<&LayoutBox> {
    self.rt.layout()
  }

  /// Run the full pipeline inside an egui allocation.
  ///
  /// `desired_size` is in egui points (logical pixels). Layout and
  /// paint run in physical pixels using `ui.ctx().pixels_per_point()`.
  ///
  /// Returns [`HtmlOutput`] containing the [`DisplayList`] ready for
  /// GPU submission and per-stage timings.
  pub fn show(&mut self, ui: &mut Ui, tree: &mut Tree, desired_size: Vec2) -> HtmlOutput {
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    let pp = ui.ctx().pixels_per_point();
    let scale = tree.effective_dpi_scale(pp);

    self.rt.driver.set_viewport(
      (
        (rect.width() * scale).max(1.0) as u32,
        (rect.height() * scale).max(1.0) as u32,
      ),
      scale as f64,
    );

    self.sync_modifiers(ui, tree);

    if response.clicked() {
      response.request_focus();
    }
    if response.has_focus() {
      forward_input_events(ui, tree);
    }

    self.forward_pointer(ui, tree, rect, scale);

    let (list, layout_present, timings) = self.rt.paint_frame(tree);

    // Quad + text preview.
    let layout_available = layout_present.is_some();
    paint_preview(&list, layout_present, rect, scale, ui);

    HtmlOutput {
      response,
      display_list: list,
      layout_available,
      timings,
    }
  }

  // ── Internal ────────────────────────────────────────────────────────────

  fn sync_modifiers(&self, ui: &Ui, tree: &mut Tree) {
    ui.input(|input| {
      set_mod(tree, Modifier::Shift, input.modifiers.shift);
      set_mod(tree, Modifier::Ctrl, input.modifiers.ctrl);
      set_mod(tree, Modifier::Alt, input.modifiers.alt);
      set_mod(tree, Modifier::Meta, input.modifiers.mac_cmd);
    });
  }

  fn forward_pointer(&mut self, ui: &Ui, tree: &mut Tree, rect: EguiRect, scale: f32) {
    let sample = ui.input(|input| {
      input.pointer.hover_pos().map(|pos| {
        (
          pos,
          input.pointer.button_down(PointerButton::Primary),
          input.pointer.button_down(PointerButton::Secondary),
          input.pointer.button_down(PointerButton::Middle),
        )
      })
    });

    let Some((pos, primary, secondary, middle)) = sample else {
      tree.pointer_leave();
      self.primary_down = false;
      self.secondary_down = false;
      self.middle_down = false;
      return;
    };

    if !rect.contains(pos) {
      tree.pointer_leave();
      self.primary_down = false;
      self.secondary_down = false;
      self.middle_down = false;
      return;
    }

    let Some(layout) = self.rt.layout() else {
      return;
    };

    let local = local_pos(rect, pos, scale);
    interactivity::pointer_move(tree, layout, local);

    sync_button(
      &mut self.primary_down,
      tree,
      layout,
      local,
      MouseButton::Primary,
      primary,
    );
    sync_button(
      &mut self.secondary_down,
      tree,
      layout,
      local,
      MouseButton::Secondary,
      secondary,
    );
    sync_button(&mut self.middle_down, tree, layout, local, MouseButton::Middle, middle);
  }
}

// ── HtmlOutput ──────────────────────────────────────────────────────────────

/// Returned from [`EguiRunner::show`].
pub struct HtmlOutput {
  pub response: Response,
  pub display_list: DisplayList,
  pub layout_available: bool,
  pub timings: PipelineTimings,
}

// ── Preview painting ────────────────────────────────────────────────────────

fn paint_preview(list: &DisplayList, layout: Option<&LayoutBox>, rect: EguiRect, scale: f32, ui: &Ui) {
  let painter = ui.painter_at(rect);

  for q in &list.quads {
    let r = egui_rect(rect.min, q.rect, scale);
    if r.is_positive() {
      painter.rect_filled(r, 0.0, color32(q.color));
    }
  }

  if let Some(layout) = layout {
    paint_text_preview(layout, &painter, rect.min, scale);
  }
}

fn paint_text_preview(layout: &LayoutBox, painter: &egui::Painter, origin: Pos2, scale: f32) {
  if let Some(run) = &layout.text_run {
    let color = color32(layout.text_color.unwrap_or([0.0, 0.0, 0.0, 1.0]));
    let font_size = (run.height / scale.max(1.0)).max(8.0);
    let font = FontId::proportional(font_size);
    let inv = if scale > 0.0 { 1.0 / scale } else { 1.0 };

    if run.lines.is_empty() {
      let pos = Pos2::new(
        origin.x + layout.content_rect.x * inv,
        origin.y + layout.content_rect.y * inv,
      );
      painter.text(pos, Align2::LEFT_TOP, &run.text, font, color);
    } else {
      for line in &run.lines {
        let start = run.byte_offset_for_boundary(line.glyph_range.0);
        let end = run.byte_offset_for_boundary(line.glyph_range.1);
        if end <= start || end > run.text.len() {
          continue;
        }
        let pos = Pos2::new(
          origin.x + layout.content_rect.x * inv,
          origin.y + (layout.content_rect.y + line.top) * inv,
        );
        painter.text(pos, Align2::LEFT_TOP, &run.text[start..end], font.clone(), color);
      }
    }
  }

  for child in &layout.children {
    paint_text_preview(child, painter, origin, scale);
  }
}

// ── Type translators ────────────────────────────────────────────────────────

/// Map an egui pointer button to the engine's [`MouseButton`].
pub fn pointer_button(button: PointerButton) -> MouseButton {
  match button {
    PointerButton::Primary => MouseButton::Primary,
    PointerButton::Secondary => MouseButton::Secondary,
    PointerButton::Middle => MouseButton::Middle,
    PointerButton::Extra1 => MouseButton::Other(3),
    PointerButton::Extra2 => MouseButton::Other(4),
  }
}

/// Map an egui key to a DOM `KeyboardEvent.key` string.
pub fn key_to_dom_key(key: egui::Key, shift: bool) -> &'static str {
  use egui::Key;
  match key {
    Key::A => {
      if shift {
        "A"
      } else {
        "a"
      }
    }
    Key::B => {
      if shift {
        "B"
      } else {
        "b"
      }
    }
    Key::C => {
      if shift {
        "C"
      } else {
        "c"
      }
    }
    Key::D => {
      if shift {
        "D"
      } else {
        "d"
      }
    }
    Key::E => {
      if shift {
        "E"
      } else {
        "e"
      }
    }
    Key::F => {
      if shift {
        "F"
      } else {
        "f"
      }
    }
    Key::G => {
      if shift {
        "G"
      } else {
        "g"
      }
    }
    Key::H => {
      if shift {
        "H"
      } else {
        "h"
      }
    }
    Key::I => {
      if shift {
        "I"
      } else {
        "i"
      }
    }
    Key::J => {
      if shift {
        "J"
      } else {
        "j"
      }
    }
    Key::K => {
      if shift {
        "K"
      } else {
        "k"
      }
    }
    Key::L => {
      if shift {
        "L"
      } else {
        "l"
      }
    }
    Key::M => {
      if shift {
        "M"
      } else {
        "m"
      }
    }
    Key::N => {
      if shift {
        "N"
      } else {
        "n"
      }
    }
    Key::O => {
      if shift {
        "O"
      } else {
        "o"
      }
    }
    Key::P => {
      if shift {
        "P"
      } else {
        "p"
      }
    }
    Key::Q => {
      if shift {
        "Q"
      } else {
        "q"
      }
    }
    Key::R => {
      if shift {
        "R"
      } else {
        "r"
      }
    }
    Key::S => {
      if shift {
        "S"
      } else {
        "s"
      }
    }
    Key::T => {
      if shift {
        "T"
      } else {
        "t"
      }
    }
    Key::U => {
      if shift {
        "U"
      } else {
        "u"
      }
    }
    Key::V => {
      if shift {
        "V"
      } else {
        "v"
      }
    }
    Key::W => {
      if shift {
        "W"
      } else {
        "w"
      }
    }
    Key::X => {
      if shift {
        "X"
      } else {
        "x"
      }
    }
    Key::Y => {
      if shift {
        "Y"
      } else {
        "y"
      }
    }
    Key::Z => {
      if shift {
        "Z"
      } else {
        "z"
      }
    }
    Key::Num0 => "0",
    Key::Num1 => "1",
    Key::Num2 => "2",
    Key::Num3 => "3",
    Key::Num4 => "4",
    Key::Num5 => "5",
    Key::Num6 => "6",
    Key::Num7 => "7",
    Key::Num8 => "8",
    Key::Num9 => "9",
    Key::Space => " ",
    Key::Enter => "Enter",
    Key::Tab => "Tab",
    Key::Backspace => "Backspace",
    Key::Delete => "Delete",
    Key::Escape => "Escape",
    Key::Home => "Home",
    Key::End => "End",
    Key::PageUp => "PageUp",
    Key::PageDown => "PageDown",
    Key::ArrowUp => "ArrowUp",
    Key::ArrowDown => "ArrowDown",
    Key::ArrowLeft => "ArrowLeft",
    Key::ArrowRight => "ArrowRight",
    _ => "Unidentified",
  }
}

/// Map an egui key to a DOM `KeyboardEvent.code` string.
pub fn key_to_dom_code(key: egui::Key) -> &'static str {
  use egui::Key;
  match key {
    Key::A => "KeyA",
    Key::B => "KeyB",
    Key::C => "KeyC",
    Key::D => "KeyD",
    Key::E => "KeyE",
    Key::F => "KeyF",
    Key::G => "KeyG",
    Key::H => "KeyH",
    Key::I => "KeyI",
    Key::J => "KeyJ",
    Key::K => "KeyK",
    Key::L => "KeyL",
    Key::M => "KeyM",
    Key::N => "KeyN",
    Key::O => "KeyO",
    Key::P => "KeyP",
    Key::Q => "KeyQ",
    Key::R => "KeyR",
    Key::S => "KeyS",
    Key::T => "KeyT",
    Key::U => "KeyU",
    Key::V => "KeyV",
    Key::W => "KeyW",
    Key::X => "KeyX",
    Key::Y => "KeyY",
    Key::Z => "KeyZ",
    Key::Num0 => "Digit0",
    Key::Num1 => "Digit1",
    Key::Num2 => "Digit2",
    Key::Num3 => "Digit3",
    Key::Num4 => "Digit4",
    Key::Num5 => "Digit5",
    Key::Num6 => "Digit6",
    Key::Num7 => "Digit7",
    Key::Num8 => "Digit8",
    Key::Num9 => "Digit9",
    Key::Space => "Space",
    Key::Enter => "Enter",
    Key::Tab => "Tab",
    Key::Backspace => "Backspace",
    Key::Delete => "Delete",
    Key::Escape => "Escape",
    Key::Home => "Home",
    Key::End => "End",
    Key::PageUp => "PageUp",
    Key::PageDown => "PageDown",
    Key::ArrowUp => "ArrowUp",
    Key::ArrowDown => "ArrowDown",
    Key::ArrowLeft => "ArrowLeft",
    Key::ArrowRight => "ArrowRight",
    _ => "Unidentified",
  }
}

/// Forward a single egui key event into the tree.
pub fn forward_key(tree: &mut Tree, key: egui::Key, pressed: bool, repeat: bool) -> bool {
  let shift = tree.modifiers().shift;
  let key_str = key_to_dom_key(key, shift);
  let code_str = key_to_dom_code(key);
  if key_str == "Unidentified" && code_str == "Unidentified" {
    return false;
  }
  if pressed {
    tree.key_down(key_str, code_str, repeat)
  } else {
    tree.key_up(key_str, code_str)
  }
}

/// Forward all queued egui key events into the tree.
pub fn forward_input_events(ui: &Ui, tree: &mut Tree) {
  ui.input(|input| {
    for event in &input.events {
      if let Event::Key {
        key,
        pressed,
        repeat,
        modifiers,
        ..
      } = event
      {
        set_mod(tree, Modifier::Shift, modifiers.shift);
        set_mod(tree, Modifier::Ctrl, modifiers.ctrl);
        set_mod(tree, Modifier::Alt, modifiers.alt);
        set_mod(tree, Modifier::Meta, modifiers.mac_cmd);
        forward_key(tree, *key, *pressed, *repeat);
      }
    }
  });
}

// ── Internal helpers ────────────────────────────────────────────────────────

fn set_mod(tree: &mut Tree, modifier: Modifier, down: bool) {
  tree.set_modifier(modifier, down);
}

fn sync_button(
  was_down: &mut bool,
  tree: &mut Tree,
  layout: &LayoutBox,
  pos: (f32, f32),
  button: MouseButton,
  down: bool,
) {
  match (*was_down, down) {
    (false, true) => {
      interactivity::mouse_down(tree, layout, pos, button);
    }
    (true, false) => {
      interactivity::mouse_up(tree, layout, pos, button);
    }
    _ => {}
  }
  *was_down = down;
}

fn local_pos(rect: EguiRect, pos: Pos2, scale: f32) -> (f32, f32) {
  ((pos.x - rect.min.x) * scale, (pos.y - rect.min.y) * scale)
}

fn egui_rect(origin: Pos2, rect: Rect, scale: f32) -> EguiRect {
  let inv = if scale > 0.0 { 1.0 / scale } else { 1.0 };
  EguiRect::from_min_size(
    Pos2::new(origin.x + rect.x * inv, origin.y + rect.y * inv),
    Vec2::new(rect.w * inv, rect.h * inv),
  )
}

fn color32(c: [f32; 4]) -> Color32 {
  let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
  Color32::from_rgba_unmultiplied(to_u8(c[0]), to_u8(c[1]), to_u8(c[2]), to_u8(c[3]))
}
