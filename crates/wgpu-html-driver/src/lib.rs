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
//!     fn inner_size(&self) -> (u32, u32) { self.window.size() }
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

use std::{sync::Arc, time::{Duration, Instant}};

use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use wgpu_html::{
  PipelineCache, PipelineTimings, events as ev, interactivity,
  layout::{Cursor, LayoutBox},
  renderer::{DisplayList, FrameOutcome, GLYPH_ATLAS_SIZE, Rect, Renderer},
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

  /// Current physical (inner) dimensions of the rendering area in
  /// device pixels.  Layout and paint use this directly as the
  /// viewport; CSS `px` values are scaled by [`scale_factor`](Self::scale_factor)
  /// to arrive at physical coordinates.
  fn inner_size(&self) -> (u32, u32);

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
  /// GPU-rendered profiling bar chart.
  pub profiling: ProfilingOverlay,
  /// When the currently-in-progress resize cooldown ends.
  /// While active, full cascade+layout is suppressed.
  resize_deadline: Option<Instant>,
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

// ── Profiling overlay ──────────────────────────────────────────────────────

/// GPU-visual profiling overlay drawn directly into the [`DisplayList`]
/// (no tree modification). Shows a bar chart of per-stage timings and
/// an FPS counter.
pub struct ProfilingOverlay {
  pub enabled: bool,
  frame_budget_ms: f32,
  bar_width: f32,
  bar_height: f32,
  bar_gap: f32,
  margin: f32,
  fps: f32,
  frame_times: [f64; 60],
  frame_time_idx: usize,
  cascade_avg: f32,
  layout_avg: f32,
  paint_avg: f32,
  render_avg: f32,
}

impl ProfilingOverlay {
  pub fn new() -> Self {
    Self {
      enabled: false,
      frame_budget_ms: 1000.0 / 60.0,
      bar_width: 140.0,
      bar_height: 5.0,
      bar_gap: 1.0,
      margin: 8.0,
      fps: 0.0,
      frame_times: [0.0; 60],
      frame_time_idx: 0,
      cascade_avg: 0.0,
      layout_avg: 0.0,
      paint_avg: 0.0,
      render_avg: 0.0,
    }
  }

  const ALPHA: f32 = 0.1;

  fn record(&mut self, timings: &PipelineTimings, frame_ms: f64) {
    self.frame_times[self.frame_time_idx % 60] = frame_ms;
    self.frame_time_idx = self.frame_time_idx.wrapping_add(1);
    let count = (self.frame_time_idx.min(60)) as usize;
    if count > 0 {
      let avg_ms = self.frame_times[..count].iter().sum::<f64>() / count as f64;
      self.fps = if avg_ms > 0.0 { (1000.0 / avg_ms) as f32 } else { 0.0 };
    }
    let a = Self::ALPHA;
    self.cascade_avg = a * timings.cascade_ms as f32 + (1.0 - a) * self.cascade_avg;
    self.layout_avg = a * timings.layout_ms as f32 + (1.0 - a) * self.layout_avg;
    self.paint_avg = a * timings.paint_ms as f32 + (1.0 - a) * self.paint_avg;
  }

  fn record_render(&mut self, render_ms: f64) {
    let a = Self::ALPHA;
    self.render_avg = a * render_ms as f32 + (1.0 - a) * self.render_avg;
  }

  /// Human-readable summary line (FPS + per-stage ms).
  pub fn summary(&self) -> String {
    format!(
      "FPS:{:.0} c:{:.2} l:{:.2} p:{:.2} r:{:.2}",
      self.fps, self.cascade_avg, self.layout_avg, self.paint_avg, self.render_avg,
    )
  }

  fn draw_bars(&self, list: &mut DisplayList, viewport_w: f32, viewport_h: f32) {
    let panel_w = self.bar_width + self.margin * 2.0;
    let panel_h = self.margin * 2.0 + (self.bar_height + self.bar_gap) * 4.0;
    let x = viewport_w - panel_w - self.margin;
    let y = viewport_h - panel_h - self.margin;

    list.push_quad_rounded(
      Rect { x, y, w: panel_w, h: panel_h },
      [0.0, 0.0, 0.0, 0.65],
      [4.0; 4],
    );

    let stages: [(&str, f32, [f32; 3]); 4] = [
      ("c", self.cascade_avg, [0.9, 0.3, 0.3]),
      ("l", self.layout_avg, [0.3, 0.5, 0.9]),
      ("p", self.paint_avg,  [0.3, 0.9, 0.3]),
      ("r", self.render_avg, [0.9, 0.8, 0.2]),
    ];

    let bar_x = x + self.margin;
    let bar_w = self.bar_width;

    for (i, (_label, ms, rgb)) in stages.iter().enumerate() {
      let bar_y = y + self.margin + i as f32 * (self.bar_height + self.bar_gap);
      let w = ((*ms / self.frame_budget_ms).min(1.0) * bar_w).max(1.0);
      list.push_quad_rounded(
        Rect { x: bar_x, y: bar_y, w, h: self.bar_height },
        [rgb[0], rgb[1], rgb[2], 0.9],
        [1.5; 4],
      );
    }
  }

  /// Inject or update the profiling overlay `<div>` in the tree.
  fn sync_overlay(&mut self, tree: &mut Tree) {
    let id = "__wgpu_profiling";
    let label = self.summary();

    // Update existing overlay text in-place.
    if let Some(div) = tree.get_element_by_id(id) {
      if let Some(span) = div.children.first_mut() {
        if let Some(text) = span.children.first_mut() {
          text.element = wgpu_html_tree::Element::Text(label);
        }
      }
      return;
    }

    // First frame: inject the overlay.
    let html = format!(
      r#"<div id="{id}" style="
        position:fixed;bottom:8px;right:8px;z-index:99999;
        pointer-events:none;font:11px monospace;
        background:rgba(0,0,0,0.65);color:#0f0;
        padding:4px 6px;border-radius:4px;
      "><span>{label}</span></div>"#
    );

    let overlay_tree = wgpu_html::parser::parse(&html);
    let Some(overlay_root) = overlay_tree.root else { return };

    let Some(root) = &mut tree.root else { return };

    let body_idx = root.children.iter()
        .position(|c| matches!(c.element, wgpu_html_tree::Element::Html(_)))
        .and_then(|hi| {
          root.children[hi].children.iter()
            .position(|c| matches!(c.element, wgpu_html_tree::Element::Body(_)))
            .map(|bi| (hi, bi))
        });

      let target = match body_idx {
        Some((hi, bi)) => &mut root.children[hi].children[bi],
        None => root,
      };

      let overlay_body = overlay_root.children.iter()
        .find(|c| matches!(c.element, wgpu_html_tree::Element::Html(_)))
        .and_then(|h| h.children.iter().find(|c| matches!(c.element, wgpu_html_tree::Element::Body(_))));
      if let Some(ob) = overlay_body {
        if let Some(div) = ob.children.first() {
      target.children.push(div.clone());
        }
      }
    }
  }

impl Default for ProfilingOverlay {
  fn default() -> Self { Self::new() }
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
      profiling: ProfilingOverlay::new(),
      resize_deadline: None,
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
  ///
  /// During an active window resize the renderer skips cascade+layout
  /// and repaints from the cached layout at the previous size.  Full
  /// pipeline work is deferred until the resize cooldown (50 ms of
  /// no resize events) expires.
  pub fn render_frame(&mut self, tree: &mut Tree) -> PipelineTimings {
    let frame_t0 = Instant::now();

    // Inject profiling overlay div into the tree.
    if self.profiling.enabled {
      self.profiling.sync_overlay(tree);
    }

    self.text_ctx.sync_fonts(&tree.fonts);
    let (w, h) = self.driver.inner_size();
    let scale = tree.effective_dpi_scale(self.driver.scale_factor() as f32);

    // Resize debounce: during interactive resize, feed the *stale*
    // viewport to paint_tree_cached so classify_frame returns
    // RepaintOnly instead of FullPipeline.  Once the cooldown
    // expires we invalidate and run the real pipeline with the
    // current size.
    //
    // We fire a redraw request while the deadline is active so the
    // frame pump doesn't stall — otherwise no event would trigger
    // the render that picks up the final size after the user
    // releases the resize handle.
    let (paint_w, paint_h) = if let Some(deadline) = self.resize_deadline {
      if Instant::now() < deadline {
        self.driver.request_redraw();
        let (cw, ch) = self.pipeline_cache.viewport();
        if cw > 0.0 && ch > 0.0 { (cw, ch) } else { (w as f32, h as f32) }
      } else {
        self.pipeline_cache.invalidate();
        self.resize_deadline = None;
        (w as f32, h as f32)
      }
    } else {
      (w as f32, h as f32)
    };

    let (mut list, layout, timings) = wgpu_html::paint_tree_cached(
      tree,
      &mut self.text_ctx,
      &mut self.image_cache,
      paint_w,
      paint_h,
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

    // Record frame timings and draw the bar chart.
    if self.profiling.enabled {
      let frame_ms = frame_t0.elapsed().as_secs_f64() * 1000.0;
      self.profiling.record(&timings, frame_ms);
      self.profiling.draw_bars(&mut list, w as f32, h as f32);
    }

    self
      .text_ctx
      .atlas
      .upload(&self.renderer.queue, self.renderer.glyph_atlas_texture());

    let render_t0 = Instant::now();
    match self.renderer.render(&list) {
      FrameOutcome::Presented | FrameOutcome::Skipped => {}
      FrameOutcome::Reconfigure => {
        self.renderer.resize(w, h);
      }
    }
    let render_ms = render_t0.elapsed().as_secs_f64() * 1000.0;

    // Update the render bar for next frame.
    if self.profiling.enabled {
      self.profiling.record_render(render_ms);
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
    let (w, h) = self.driver.inner_size();
    let scale = tree.effective_dpi_scale(self.driver.scale_factor() as f32);

    let (paint_w, paint_h) = if let Some(deadline) = self.resize_deadline {
      if Instant::now() < deadline {
        let (cw, ch) = self.pipeline_cache.viewport();
        if cw > 0.0 && ch > 0.0 { (cw, ch) } else { (w as f32, h as f32) }
      } else {
        self.pipeline_cache.invalidate();
        self.resize_deadline = None;
        (w as f32, h as f32)
      }
    } else {
      (w as f32, h as f32)
    };

    let (list, layout, timings) = wgpu_html::paint_tree_cached(
      tree,
      &mut self.text_ctx,
      &mut self.image_cache,
      paint_w,
      paint_h,
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
        let (w, h) = self.driver.inner_size();
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
        let (w, h) = self.driver.inner_size();
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

    let (_w, h) = self.driver.inner_size();
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
  ///
  /// The GPU surface is resized immediately (cheap). Full cascade +
  /// layout + paint is deferred until the resize cooldown expires
  /// (50 ms of no resize events) to avoid frame drops during
  /// interactive window dragging.
  pub fn on_resize(&mut self, tree: &mut Tree, width: u32, height: u32) {
    self.renderer.resize(width, height);
    if let Some(layout) = self.last_layout.as_ref() {
      self.scroll_y = clamp_scroll_y(self.scroll_y, layout, height as f32);
    }
    self.scrollbar_drag = None;
    // Defer the heavy cascade+layout+paint until resize stabilizes.
    // Each new resize event pushes the deadline further out.
    self.resize_deadline = Some(Instant::now() + Duration::from_millis(50));
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
    let (w, h) = self.driver.inner_size();
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
