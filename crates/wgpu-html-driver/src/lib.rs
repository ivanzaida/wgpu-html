//! Abstraction over the platform windowing/input system.
//!
//! ## Layers
//!
//! 1. **[`Driver`]** — trait with 5 mandatory methods. Implement it to connect wgpu-html to any windowing framework.
//!
//! 2. **[`Runtime`]** — struct parameterized over `D: Driver` that owns the renderer, text context, image cache,
//!    pipeline cache, and all cross-cutting state (scroll, cursor, clicks, scrollbar drag). Its event methods translate
//!    platform-agnostic data into engine operations and return whether a redraw is needed.
//!
//! ## Platform crates
//!
//! | Crate | Integration |
//! |---|---|
//! | `wgpu-html-driver-winit` | `impl Driver for winit::Window` + event dispatch + type translators |
//! | `wgpu-html-driver-egui`  | `impl Driver` for egui surfaces + `show()` helper |
//! | `wgpu-html-driver-bevy`  | `impl Driver` for bevy windows + `Plugin` |
//!
//! ## Custom driver
//!
//! ```ignore
//! struct MyDriver { window: Arc<MyWindow> }
//!
//! impl Driver for MyDriver {
//!     type Surface = MyWindow;
//!     fn surface(&self) -> &Arc<MyWindow> { &self.window }
//!     fn logical_size(&self) -> (u32, u32) { self.window.size() }
//!     fn scale_factor(&self) -> f64 { self.window.scale() }
//!     fn request_redraw(&self) { self.window.request_redraw(); }
//!     fn set_cursor(&self, cursor: Cursor) { self.window.set_cursor(cursor); }
//! }
//!
//! let driver = MyDriver { window: Arc::new(my_window) };
//! let mut rt = Runtime::new(driver, 800, 600);
//! // … in event loop:
//! rt.on_pointer_move(&mut tree, x, y);
//! rt.render_frame(&mut tree);
//! ```

use std::{sync::Arc, time::Instant};

use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use wgpu_html::{
  PipelineCache, PipelineTimings, events as ev, interactivity,
  layout::{Cursor, LayoutBox},
  renderer::{DisplayList, FrameOutcome, GLYPH_ATLAS_SIZE, Renderer},
  scroll::{
    ElementScrollbarDrag, clamp_scroll_y, paint_viewport_scrollbar, rect_contains, scroll_element_at,
    scroll_y_from_thumb_top, scrollbar_geometry, translate_display_list_y, viewport_to_document,
  },
};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{MouseButton, Tree};

// ── Driver trait ────────────────────────────────────────────────────────────

/// Bridge between wgpu-html and a platform's windowing/input system.
///
/// Implement this trait for any framework (winit, egui, bevy, SDL2,
/// glfw, etc.) and pair it with a [`Runtime`] to get full HTML
/// rendering, input handling, and scroll management.
pub trait Driver {
  /// Platform surface type. Must satisfy wgpu's raw-window-handle
  /// requirements.
  type Surface: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static;

  /// The surface that wgpu renders into.
  fn surface(&self) -> &Arc<Self::Surface>;

  /// Current logical (unscaled) dimensions of the rendering area.
  fn logical_size(&self) -> (u32, u32);

  /// DPI scale factor.
  fn scale_factor(&self) -> f64;

  /// Ask the platform event loop to issue a redraw at its next
  /// convenience.
  fn request_redraw(&self);

  /// Change the visible mouse cursor.
  fn set_cursor(&self, cursor: Cursor);
}

// ── Runtime ─────────────────────────────────────────────────────────────────

/// Engine-glue: owns the platform `Driver`, the wgpu [`Renderer`], and
/// all shared state needed for event processing and frame rendering.
///
/// `Runtime` is the primary type users interact with. Create one per
/// window, then call the `on_*` methods from your event loop followed
/// by [`Self::render_frame`] on each redraw.
pub struct Runtime<D: Driver> {
  pub driver: D,
  pub renderer: Renderer,
  pub text_ctx: TextContext,
  pub image_cache: wgpu_html::layout::ImageCache,
  pipeline_cache: PipelineCache,
  scroll_y: f32,
  cursor_pos: Option<(f32, f32)>,
  last_layout: Option<LayoutBox>,
  last_click: Option<ClickTracker>,
  scrollbar_drag: Option<ScrollbarDrag>,
}

/// Tracks multi-click (double / triple) state.
#[derive(Debug, Clone)]
struct ClickTracker {
  at: Instant,
  pos: (f32, f32),
  button: MouseButton,
  target_path: Option<Vec<usize>>,
  count: u8,
}

#[derive(Debug, Clone)]
enum ScrollbarDrag {
  Viewport { grab_offset_y: f32 },
  Element(ElementScrollbarDrag),
}

impl<D: Driver> Runtime<D> {
  // ── Construction ────────────────────────────────────────────────────────

  /// Create a new `Runtime` by building a wgpu [`Renderer`] from the
  /// driver's surface.
  ///
  /// This blocks on GPU adapter/device acquisition.
  pub fn new(driver: D, width: u32, height: u32) -> Self {
    let renderer = pollster::block_on(Renderer::new(driver.surface().clone(), width, height));
    Self::with_renderer(driver, renderer)
  }

  /// Wrap an externally-managed [`Renderer`] (e.g. from bevy's
  /// render world).
  pub fn with_renderer(driver: D, renderer: Renderer) -> Self {
    Self {
      driver,
      renderer,
      text_ctx: TextContext::new(GLYPH_ATLAS_SIZE),
      image_cache: wgpu_html::layout::ImageCache::new(),
      pipeline_cache: PipelineCache::new(),
      scroll_y: 0.0,
      cursor_pos: None,
      last_layout: None,
      last_click: None,
      scrollbar_drag: None,
    }
  }

  // ── Accessors ───────────────────────────────────────────────────────────

  pub fn layout(&self) -> Option<&LayoutBox> {
    self.last_layout.as_ref()
  }

  pub fn scroll_y(&self) -> f32 {
    self.scroll_y
  }

  // ── Frame rendering ─────────────────────────────────────────────────────

  /// Run the full cascade → layout → paint → GPU-render pipeline.
  ///
  /// Returns per-stage timings.
  pub fn render_frame(&mut self, tree: &mut Tree) -> PipelineTimings {
    self.text_ctx.sync_fonts(&tree.fonts);
    let (w, h) = self.driver.logical_size();
    let scale = tree.effective_dpi_scale(self.driver.scale_factor() as f32);

    let (mut list, layout, timings) = wgpu_html::paint_tree_cached(
      tree,
      &mut self.text_ctx,
      &mut self.image_cache,
      w as f32,
      h as f32,
      scale,
      self.scroll_y,
      &mut self.pipeline_cache,
    );

    if let Some(layout) = layout {
      self.scroll_y = clamp_scroll_y(self.scroll_y, layout, h as f32);
      translate_display_list_y(&mut list, -self.scroll_y);
      paint_viewport_scrollbar(&mut list, layout, w as f32, h as f32, self.scroll_y);
    } else {
      self.scroll_y = 0.0;
    }
    self.last_layout = self.pipeline_cache.layout().cloned();

    self
      .text_ctx
      .atlas
      .upload(&self.renderer.queue, self.renderer.glyph_atlas_texture());

    match self.renderer.render(&list) {
      FrameOutcome::Presented | FrameOutcome::Skipped => {}
      FrameOutcome::Reconfigure => {
        self.renderer.resize(w, h);
      }
    }

    timings
  }

  /// Run cascade → layout → paint and return the [`DisplayList`]
  /// *without* submitting to the GPU. Useful for frameworks that
  /// manage their own GPU submission (bevy, egui).
  ///
  /// Glyph atlas upload is also skipped — call
  /// [`Self::upload_glyphs`] separately.
  pub fn paint_frame(&mut self, tree: &mut Tree) -> (DisplayList, Option<&LayoutBox>, PipelineTimings) {
    self.text_ctx.sync_fonts(&tree.fonts);
    let (w, h) = self.driver.logical_size();
    let scale = tree.effective_dpi_scale(self.driver.scale_factor() as f32);

    let (list, layout, timings) = wgpu_html::paint_tree_cached(
      tree,
      &mut self.text_ctx,
      &mut self.image_cache,
      w as f32,
      h as f32,
      scale,
      self.scroll_y,
      &mut self.pipeline_cache,
    );

    if let Some(layout) = layout {
      self.scroll_y = clamp_scroll_y(self.scroll_y, layout, h as f32);
    } else {
      self.scroll_y = 0.0;
    }
    self.last_layout = self.pipeline_cache.layout().cloned();

    (list, self.last_layout.as_ref(), timings)
  }

  /// Upload pending glyph atlas rasters to the GPU.
  pub fn upload_glyphs(&mut self) {
    self
      .text_ctx
      .atlas
      .upload(&self.renderer.queue, self.renderer.glyph_atlas_texture());
  }

  /// Submit a pre-built [`DisplayList`] to the GPU and present.
  pub fn render_list(&mut self, list: &DisplayList) {
    match self.renderer.render(list) {
      FrameOutcome::Presented | FrameOutcome::Skipped => {}
      FrameOutcome::Reconfigure => {
        let (w, h) = self.driver.logical_size();
        self.renderer.resize(w, h);
      }
    }
  }

  /// Invalidate the pipeline cache, forcing a full cascade + layout
  /// on the next frame.
  pub fn invalidate_cache(&mut self) {
    self.pipeline_cache.invalidate();
  }

  // ── Input events ────────────────────────────────────────────────────────

  /// Process a pointer move. Handles scrollbar drag continuation,
  /// hit-testing, hover updates, and cursor changes.
  ///
  /// `pos` is in viewport (physical) coordinates. Returns `true` if
  /// the caller should request a redraw.
  pub fn on_pointer_move(&mut self, tree: &mut Tree, x: f32, y: f32) -> bool {
    self.cursor_pos = Some((x, y));

    if let Some(drag) = self.scrollbar_drag.clone() {
      if let Some(layout) = self.last_layout.as_ref() {
        let (w, h) = self.driver.logical_size();
        match &drag {
          ScrollbarDrag::Viewport { grab_offset_y } => {
            self.scroll_y = scroll_y_from_thumb_top(y - grab_offset_y, layout, w as f32, h as f32);
          }
          ScrollbarDrag::Element(el_drag) => {
            let doc_pos = viewport_to_document((x, y), self.scroll_y);
            el_drag.update(layout, tree, doc_pos.1);
          }
        }
        self.driver.request_redraw();
        return true;
      }
    }

    let Some(layout) = self.last_layout.as_ref() else {
      return false;
    };

    let doc_pos = viewport_to_document((x, y), self.scroll_y);
    let (changed, css_cursor) = interactivity::pointer_move_with_cursor(tree, layout, doc_pos);
    self.driver.set_cursor(css_cursor);

    changed || tree.interaction.selecting_text
  }

  /// Process a pointer leave (cursor exited the window).
  /// Returns `true` if the caller should redraw.
  pub fn on_pointer_leave(&mut self, tree: &mut Tree) -> bool {
    self.cursor_pos = None;
    tree.pointer_leave();
    true
  }

  /// Process a mouse button press or release. Handles scrollbar drag
  /// start/end, click counting, and dispatch to the tree.
  ///
  /// Uses the position from the most recent
  /// [`Self::on_pointer_move`] call. Returns `true` if the caller
  /// should redraw.
  pub fn on_mouse_button(&mut self, tree: &mut Tree, button: MouseButton, pressed: bool) -> bool {
    let Some(pos) = self.cursor_pos else {
      return false;
    };

    // Scrollbar drag book-keeping on primary button.
    if button == MouseButton::Primary {
      if pressed {
        if self.start_scrollbar_drag(tree, pos) {
          return true;
        }
      } else if self.scrollbar_drag.take().is_some() {
        return true;
      }
    }

    let doc_pos = viewport_to_document(pos, self.scroll_y);

    if pressed {
      let target_path = {
        let Some(layout) = self.last_layout.as_ref() else {
          return false;
        };
        layout.hit_path_scrolled(doc_pos, &tree.interaction.scroll_offsets_y)
      };
      let click_count = self.next_click_count(button, doc_pos, target_path);
      let Some(layout) = self.last_layout.as_ref() else {
        return false;
      };
      interactivity::mouse_down_with_click_count(tree, layout, doc_pos, button, click_count);
    } else {
      let Some(layout) = self.last_layout.as_ref() else {
        return false;
      };
      interactivity::mouse_up(tree, layout, doc_pos, button);
    }

    true
  }

  /// Process a mouse wheel / scroll event.
  ///
  /// `pixel_dy` is the vertical scroll delta in physical pixels
  /// (positive = content moves up, i.e. user scrolled down).
  /// `effective_scale` is the current DPI scale factor.
  ///
  /// Uses the position from the most recent
  /// [`Self::on_pointer_move`]. Returns `true` if the caller should
  /// redraw.
  pub fn on_wheel(&mut self, tree: &mut Tree, pixel_dy: f32, _pixel_dx: f32, _effective_scale: f32) -> bool {
    // Fire wheel event first so listeners can preventDefault.
    // Callers should call on_wheel_event separately for this.
    // Here we just handle scroll.

    let Some(layout) = self.last_layout.as_ref() else {
      return false;
    };
    let Some(pos) = self.cursor_pos else {
      return false;
    };
    let doc_pos = viewport_to_document(pos, self.scroll_y);

    if scroll_element_at(tree, layout, doc_pos, pixel_dy) {
      interactivity::pointer_move(tree, layout, doc_pos);
      return true;
    }

    let (_w, h) = self.driver.logical_size();
    self.scroll_y = clamp_scroll_y(self.scroll_y + pixel_dy, layout, h as f32);
    let new_doc_pos = viewport_to_document(pos, self.scroll_y);
    if let Some(layout) = self.last_layout.as_ref() {
      interactivity::pointer_move(tree, layout, new_doc_pos);
    }

    true
  }

  /// Fire a DOM `wheel` event (preventable by listeners).
  ///
  /// Call this **before** [`Self::on_wheel`]. If this returns
  /// `true`, skip `on_wheel` — the event was consumed.
  pub fn on_wheel_event(&self, tree: &mut Tree, _x: f32, _y: f32, dx: f64, dy: f64, mode: ev::WheelDeltaMode) -> bool {
    let Some(pos) = self.cursor_pos else {
      return false;
    };
    let doc_pos = viewport_to_document(pos, self.scroll_y);
    tree.wheel_event(doc_pos, dx, dy, mode)
  }

  /// Process a keyboard event.
  ///
  /// `key` is the DOM `KeyboardEvent.key` value,
  /// `code` is `KeyboardEvent.code` (physical key identifier),
  /// `text` is the composed character for text insertion (if any).
  pub fn on_key(&mut self, tree: &mut Tree, key: &str, code: &str, pressed: bool, repeat: bool, text: Option<&str>) {
    if pressed {
      tree.key_down(key, code, repeat);
      // Feed typed text into the focused form control.
      // Skip when Ctrl or Meta is held — those are shortcuts.
      if !tree.modifiers().ctrl && !tree.modifiers().meta {
        if let Some(s) = text {
          if !s.is_empty() && s.chars().all(|c| !c.is_control()) {
            wgpu_html_tree::text_input(tree, s);
          }
        }
      }
    } else {
      tree.key_up(key, code);
    }
  }

  /// Notify the runtime that the modifier state has changed.
  /// Platform crates call this when a modifier key is pressed or
  /// released.
  pub fn set_modifier(&self, tree: &mut Tree, modifier: wgpu_html_tree::Modifier, down: bool) {
    tree.set_modifier(modifier, down);
  }

  /// Handle a window resize.
  pub fn on_resize(&mut self, tree: &mut Tree, width: u32, height: u32) {
    self.renderer.resize(width, height);
    if let Some(layout) = self.last_layout.as_ref() {
      self.scroll_y = clamp_scroll_y(self.scroll_y, layout, height as f32);
    }
    self.scrollbar_drag = None;
    self.pipeline_cache.invalidate();
    tree.resize_event();
  }

  /// Handle a DPI scale factor change.
  pub fn on_scale_change(&mut self) {
    self.pipeline_cache.invalidate();
    self.scrollbar_drag = None;
  }

  // ── Scrollbar helpers ───────────────────────────────────────────────────

  fn start_scrollbar_drag(&mut self, tree: &mut Tree, pos: (f32, f32)) -> bool {
    let Some(layout) = self.last_layout.as_ref() else {
      return false;
    };
    let (w, h) = self.driver.logical_size();
    let doc_pos = viewport_to_document(pos, self.scroll_y);

    // Element-level scrollbars first.
    if let Some(el_drag) = ElementScrollbarDrag::try_start(layout, doc_pos, tree) {
      self.scrollbar_drag = Some(ScrollbarDrag::Element(el_drag));
      return true;
    }

    // Viewport scrollbar.
    let Some(geom) = scrollbar_geometry(layout, w as f32, h as f32, self.scroll_y) else {
      return false;
    };
    if rect_contains(geom.thumb, pos) {
      self.scrollbar_drag = Some(ScrollbarDrag::Viewport {
        grab_offset_y: pos.1 - geom.thumb.y,
      });
      return true;
    }
    if rect_contains(geom.track, pos) {
      let thumb_top = pos.1 - geom.thumb.h * 0.5;
      self.scroll_y = scroll_y_from_thumb_top(thumb_top, layout, w as f32, h as f32);
      if let Some(updated) = scrollbar_geometry(layout, w as f32, h as f32, self.scroll_y) {
        self.scrollbar_drag = Some(ScrollbarDrag::Viewport {
          grab_offset_y: pos.1 - updated.thumb.y,
        });
      }
      return true;
    }
    false
  }

  // ── Click counting ──────────────────────────────────────────────────────

  fn next_click_count(&mut self, button: MouseButton, pos: (f32, f32), target_path: Option<Vec<usize>>) -> u8 {
    const MULTI_CLICK_MAX_MS: u128 = 500;
    const MULTI_CLICK_MAX_DIST: f32 = 5.0;

    let now = Instant::now();
    let count = self
      .last_click
      .as_ref()
      .filter(|last| last.button == button)
      .filter(|last| last.target_path == target_path)
      .filter(|last| now.duration_since(last.at).as_millis() <= MULTI_CLICK_MAX_MS)
      .filter(|last| {
        let dx = last.pos.0 - pos.0;
        let dy = last.pos.1 - pos.1;
        dx * dx + dy * dy <= MULTI_CLICK_MAX_DIST * MULTI_CLICK_MAX_DIST
      })
      .map(|last| last.count.saturating_add(1).min(3))
      .unwrap_or(1);

    self.last_click = Some(ClickTracker {
      at: now,
      pos,
      button,
      target_path,
      count,
    });
    count
  }
}
