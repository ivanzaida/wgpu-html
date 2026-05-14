//! Bevy integration for [`lui`].
//!
//! [`LuiPlugin`] creates a single fullscreen transparent overlay
//! with one DOM [`Tree`]. The tree covers 100% of the viewport and
//! re-renders every frame into a Bevy [`Image`].
//!
//! Use [`HtmlOverlay`] (non-send resource) to manipulate the DOM:
//!
//! ```ignore
//! fn my_system(mut html: NonSendMut<HtmlOverlay>) {
//!     html.set_html(r#"<div id="hud">Score: 42</div>"#);
//! }
//! ```

use bevy::{
  asset::RenderAssetUsages,
  input::{ButtonState, keyboard::KeyboardInput, mouse::MouseButtonInput},
  prelude::*,
  render::render_resource::{Extent3d, TextureDimension, TextureFormat},
  window::CursorMoved,
};
use lui_renderer_wgpu::{RenderBackend, Renderer};
use lui_text::TextContext;
use lui_tree::{Modifier, MouseButton as HtmlMouseButton, Node as HtmlNode, Tree};
use lui_v1::interactivity;

// ── Plugin ─────────────────────────────────────────────────────────────────

pub struct LuiPlugin;

impl Plugin for LuiPlugin {
  fn build(&self, app: &mut App) {
    let mut renderer = pollster::block_on(Renderer::headless());
    renderer.set_clear_color([0.0, 0.0, 0.0, 0.0]);
    let text_ctx = TextContext::new(lui_renderer_wgpu::GLYPH_ATLAS_SIZE);

    let image = Image::new_fill(
      Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
      },
      TextureDimension::D2,
      &[0, 0, 0, 0],
      TextureFormat::Rgba8UnormSrgb,
      RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let image_handle = app.world_mut().resource_mut::<Assets<Image>>().add(image);

    app.world_mut().insert_non_send_resource(HtmlOverlay {
      tree: Tree::new(lui_tree::Node::new(lui_tree::Element::Body(Default::default()))),
      renderer,
      text_ctx,
      image_cache: lui_v1::layout::ImageCache::default(),
      pipeline_cache: lui_v1::PipelineCache::new(),
      image_handle: image_handle.clone(),
      captures_input: true,
      last_cursor: None,
    });

    app
      .add_systems(Startup, spawn_overlay_entity)
      .add_systems(PreUpdate, forward_input_system)
      .add_systems(Update, render_overlay_system);
  }
}

// ── Overlay marker ─────────────────────────────────────────────────────────

#[derive(Component)]
struct OverlayImage;

// ── HtmlOverlay resource ───────────────────────────────────────────────────

/// Single fullscreen HTML overlay. One DOM tree, one render target.
pub struct HtmlOverlay {
  pub tree: Tree,
  renderer: Renderer,
  text_ctx: TextContext,
  image_cache: lui_v1::layout::ImageCache,
  pipeline_cache: lui_v1::PipelineCache,
  image_handle: Handle<Image>,
  captures_input: bool,
  last_cursor: Option<(f32, f32)>,
}

impl HtmlOverlay {
  /// Append a node to the root element. Returns the child index.
  pub fn append(&mut self, node: HtmlNode) -> Option<usize> {
    self.tree.append_node(&[], node)
  }

  /// Append a node as a child of the element at `parent_path`.
  pub fn append_to(&mut self, parent_path: &[usize], node: HtmlNode) -> Option<usize> {
    self.tree.append_node(parent_path, node)
  }

  /// Remove the first element with the given `id`.
  /// Returns the removed node if found.
  pub fn remove_by_id(&mut self, id: &str) -> Option<HtmlNode> {
    fn remove_recursive(node: &mut HtmlNode, id: &str) -> Option<HtmlNode> {
      if let Some(idx) = node.children.iter().position(|c| c.element.id() == Some(id)) {
        return Some(node.children.remove(idx));
      }
      for child in &mut node.children {
        if let Some(removed) = remove_recursive(child, id) {
          return Some(removed);
        }
      }
      None
    }
    let root = self.tree.root.as_mut()?;
    let removed = remove_recursive(root, id);
    if removed.is_some() {
      self.tree.generation += 1;
    }
    removed
  }

  /// Remove all children of the root, clearing the overlay.
  pub fn clear(&mut self) {
    if let Some(root) = &mut self.tree.root {
      root.children.clear();
      self.tree.generation += 1;
    }
    self.pipeline_cache.invalidate();
  }

  /// Borrow the root node for reading.
  pub fn root(&self) -> Option<&HtmlNode> {
    self.tree.root.as_ref()
  }

  /// Mutable access to the full tree.
  pub fn tree(&self) -> &Tree {
    &self.tree
  }

  pub fn tree_mut(&mut self) -> &mut Tree {
    &mut self.tree
  }

  pub fn set_captures_input(&mut self, captures: bool) {
    self.captures_input = captures;
  }

  pub fn captures_input(&self) -> bool {
    self.captures_input
  }

  pub fn image_handle(&self) -> &Handle<Image> {
    &self.image_handle
  }
}

// ── Setup ──────────────────────────────────────────────────────────────────

fn spawn_overlay_entity(mut commands: Commands, overlay: NonSend<HtmlOverlay>) {
  commands.spawn((
    OverlayImage,
    ImageNode {
      image: overlay.image_handle.clone(),
      ..default()
    },
    Node {
      width: Val::Percent(100.0),
      height: Val::Percent(100.0),
      position_type: PositionType::Absolute,
      left: Val::Px(0.0),
      top: Val::Px(0.0),
      ..default()
    },
    GlobalZIndex(i32::MAX),
  ));
}

// ── Input ──────────────────────────────────────────────────────────────────

fn forward_input_system(
  mut overlay: NonSendMut<HtmlOverlay>,
  mut cursor_events: MessageReader<CursorMoved>,
  mut mouse_button_events: MessageReader<MouseButtonInput>,
  mut keyboard_events: MessageReader<KeyboardInput>,
  windows: Query<&Window>,
) {
  if !overlay.captures_input {
    cursor_events.clear();
    mouse_button_events.clear();
    keyboard_events.clear();
    return;
  }

  let scale = windows.iter().next().map(|w| w.scale_factor()).unwrap_or(1.0);
  let o = &mut *overlay;

  if let Some(pos) = cursor_events.read().last().map(|e| e.position) {
    let phys = (pos.x * scale, pos.y * scale);
    o.last_cursor = Some(phys);
    if let Some(layout) = o.pipeline_cache.layout() {
      interactivity::pointer_move(&mut o.tree, layout, phys);
    }
  }

  for event in mouse_button_events.read() {
    let button = match event.button {
      bevy::input::mouse::MouseButton::Left => HtmlMouseButton::Primary,
      bevy::input::mouse::MouseButton::Right => HtmlMouseButton::Secondary,
      bevy::input::mouse::MouseButton::Middle => HtmlMouseButton::Middle,
      bevy::input::mouse::MouseButton::Other(id) => HtmlMouseButton::Other(id as u8),
      _ => continue,
    };

    let Some(layout) = o.pipeline_cache.layout() else {
      continue;
    };
    let pos = o.last_cursor.unwrap_or((0.0, 0.0));

    match event.state {
      ButtonState::Pressed => {
        interactivity::mouse_down(&mut o.tree, layout, pos, button);
      }
      ButtonState::Released => {
        interactivity::mouse_up(&mut o.tree, layout, pos, button);
      }
    }
  }

  for event in keyboard_events.read() {
    let key_str = logical_key_to_dom_key(&event.logical_key);
    let code_str = key_code_to_dom_code(&event.key_code);

    match &event.logical_key {
      bevy::input::keyboard::Key::Shift => o.tree.set_modifier(Modifier::Shift, event.state.is_pressed()),
      bevy::input::keyboard::Key::Control => o.tree.set_modifier(Modifier::Ctrl, event.state.is_pressed()),
      bevy::input::keyboard::Key::Alt => o.tree.set_modifier(Modifier::Alt, event.state.is_pressed()),
      bevy::input::keyboard::Key::Super => o.tree.set_modifier(Modifier::Meta, event.state.is_pressed()),
      _ => {}
    }

    if event.state.is_pressed() {
      o.tree.key_down(key_str, code_str, event.repeat);
    } else {
      o.tree.key_up(key_str, code_str);
    }
  }
}

// ── Render ─────────────────────────────────────────────────────────────────

fn render_overlay_system(
  mut overlay: NonSendMut<HtmlOverlay>,
  mut images: ResMut<Assets<Image>>,
  windows: Query<&Window>,
) {
  let Some(window) = windows.iter().next() else {
    return;
  };
  let scale = window.scale_factor();
  let phys_w = (window.width() * scale).ceil() as u32;
  let phys_h = (window.height() * scale).ceil() as u32;

  if phys_w == 0 || phys_h == 0 {
    return;
  }

  let o = &mut *overlay;

  o.text_ctx.sync_fonts(&o.tree.fonts);

  let (mut list, _layout, _timings) = lui_v1::paint_tree_cached(
    &mut o.tree,
    &mut o.text_ctx,
    &mut o.image_cache,
    phys_w as f32,
    phys_h as f32,
    scale,
    0.0,
    &mut o.pipeline_cache,
  );

  list.finalize();

  o.text_ctx.atlas.flush_dirty(|rect, data| {
    o.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
  });

  let Ok(rgba) = o.renderer.render_to_rgba(&list, phys_w, phys_h) else {
    return;
  };

  let new_image = Image::new(
    Extent3d {
      width: phys_w,
      height: phys_h,
      depth_or_array_layers: 1,
    },
    TextureDimension::D2,
    rgba,
    TextureFormat::Rgba8UnormSrgb,
    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
  );
  let _ = images.insert(o.image_handle.id(), new_image);
}

// ── Key translation ────────────────────────────────────────────────────────

pub fn logical_key_to_dom_key(key: &bevy::input::keyboard::Key) -> &'static str {
  use bevy::input::keyboard::Key;
  match key {
    Key::Character(c) => match c.as_str() {
      "a" => "a",
      "b" => "b",
      "c" => "c",
      "d" => "d",
      "e" => "e",
      "f" => "f",
      "g" => "g",
      "h" => "h",
      "i" => "i",
      "j" => "j",
      "k" => "k",
      "l" => "l",
      "m" => "m",
      "n" => "n",
      "o" => "o",
      "p" => "p",
      "q" => "q",
      "r" => "r",
      "s" => "s",
      "t" => "t",
      "u" => "u",
      "v" => "v",
      "w" => "w",
      "x" => "x",
      "y" => "y",
      "z" => "z",
      "A" => "A",
      "B" => "B",
      "C" => "C",
      "D" => "D",
      "E" => "E",
      "F" => "F",
      "G" => "G",
      "H" => "H",
      "I" => "I",
      "J" => "J",
      "K" => "K",
      "L" => "L",
      "M" => "M",
      "N" => "N",
      "O" => "O",
      "P" => "P",
      "Q" => "Q",
      "R" => "R",
      "S" => "S",
      "T" => "T",
      "U" => "U",
      "V" => "V",
      "W" => "W",
      "X" => "X",
      "Y" => "Y",
      "Z" => "Z",
      "0" => "0",
      "1" => "1",
      "2" => "2",
      "3" => "3",
      "4" => "4",
      "5" => "5",
      "6" => "6",
      "7" => "7",
      "8" => "8",
      "9" => "9",
      " " => " ",
      _ => "Unidentified",
    },
    Key::Enter => "Enter",
    Key::Tab => "Tab",
    Key::Space => " ",
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
    Key::Shift => "Shift",
    Key::Control => "Control",
    Key::Alt => "Alt",
    Key::Super => "Meta",
    _ => "Unidentified",
  }
}

pub fn key_code_to_dom_code(code: &KeyCode) -> &'static str {
  match code {
    KeyCode::KeyA => "KeyA",
    KeyCode::KeyB => "KeyB",
    KeyCode::KeyC => "KeyC",
    KeyCode::KeyD => "KeyD",
    KeyCode::KeyE => "KeyE",
    KeyCode::KeyF => "KeyF",
    KeyCode::KeyG => "KeyG",
    KeyCode::KeyH => "KeyH",
    KeyCode::KeyI => "KeyI",
    KeyCode::KeyJ => "KeyJ",
    KeyCode::KeyK => "KeyK",
    KeyCode::KeyL => "KeyL",
    KeyCode::KeyM => "KeyM",
    KeyCode::KeyN => "KeyN",
    KeyCode::KeyO => "KeyO",
    KeyCode::KeyP => "KeyP",
    KeyCode::KeyQ => "KeyQ",
    KeyCode::KeyR => "KeyR",
    KeyCode::KeyS => "KeyS",
    KeyCode::KeyT => "KeyT",
    KeyCode::KeyU => "KeyU",
    KeyCode::KeyV => "KeyV",
    KeyCode::KeyW => "KeyW",
    KeyCode::KeyX => "KeyX",
    KeyCode::KeyY => "KeyY",
    KeyCode::KeyZ => "KeyZ",
    KeyCode::Digit0 => "Digit0",
    KeyCode::Digit1 => "Digit1",
    KeyCode::Digit2 => "Digit2",
    KeyCode::Digit3 => "Digit3",
    KeyCode::Digit4 => "Digit4",
    KeyCode::Digit5 => "Digit5",
    KeyCode::Digit6 => "Digit6",
    KeyCode::Digit7 => "Digit7",
    KeyCode::Digit8 => "Digit8",
    KeyCode::Digit9 => "Digit9",
    KeyCode::Space => "Space",
    KeyCode::Enter => "Enter",
    KeyCode::Tab => "Tab",
    KeyCode::Backspace => "Backspace",
    KeyCode::Delete => "Delete",
    KeyCode::Escape => "Escape",
    KeyCode::Home => "Home",
    KeyCode::End => "End",
    KeyCode::PageUp => "PageUp",
    KeyCode::PageDown => "PageDown",
    KeyCode::ArrowUp => "ArrowUp",
    KeyCode::ArrowDown => "ArrowDown",
    KeyCode::ArrowLeft => "ArrowLeft",
    KeyCode::ArrowRight => "ArrowRight",
    _ => "Unidentified",
  }
}
