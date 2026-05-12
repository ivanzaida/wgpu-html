//! Abstraction over the platform windowing/input system.
//!
//! ## Layers
//!
//! 1. **[`Driver`]** — trait with 5 mandatory methods. Implement it to connect lui to any windowing framework.
//!
//! 2. **[`Runtime`]** — struct parameterized over `D: Driver` that owns the renderer, text context, image cache,
//!    pipeline cache, and all cross-cutting state (scroll, cursor, clicks, scrollbar drag). Its event methods translate
//!    platform-agnostic data into engine operations and return whether a redraw is needed.
//!
//! ## Platform crates
//!
//! | Crate | Integration |
//! |---|---|
//! | `lui-driver-winit` | `impl Driver for winit::Window` + event dispatch + type translators |
//! | `lui-driver-egui`  | `impl Driver` for egui surfaces + `show()` helper |
//! | `lui-driver-bevy`  | `impl Driver` for bevy windows + `Plugin` |
//!
//! ## Custom driver
//!
//! ```ignore
//! struct MyDriver { window: Arc<MyWindow> }
//!
//! impl Driver for MyDriver {
//!     fn inner_size(&self) -> (u32, u32) { self.window.size() }
//!     fn scale_factor(&self) -> f64 { self.window.scale() }
//!     fn request_redraw(&self) { self.window.request_redraw(); }
//!     fn set_cursor(&self, cursor: Cursor) { self.window.set_cursor(cursor); }
//! }
//!
//! let driver = MyDriver { window: Arc::new(my_window) };
//! let renderer = pollster::block_on(Renderer::new(my_window, 800, 600));
//! let mut rt = Runtime::new(driver, renderer);
//! // … in event loop:
//! rt.on_pointer_move(&mut tree, x, y);
//! rt.render_frame(&mut tree);
//! ```

use std::time::{Duration, Instant};

use lui::{
  PipelineCache, PipelineTimings, events as ev, interactivity,
  layout::{Cursor, LayoutBox},
  renderer::{DisplayList, FrameOutcome, Rect},
  scroll::{
    ElementScrollbarDrag, clamp_scroll_x, clamp_scroll_y, rect_contains, scroll_element_at, translate_display_list_x,
    translate_display_list_y, viewport_to_document,
  },
  select_all_text, selected_text,
};
use lui_render_api::RenderBackend;
use lui_text::TextContext;
use lui_tree::{MouseButton, Tree};

// ── Secondary window ───────────────────────────────────────────────────────

/// A secondary rendering surface managed alongside the host window.
///
/// Platform-agnostic: knows about the [`Tree`] but nothing about
/// winit/egui/bevy. The host integration is responsible for creating
/// the actual window, routing platform events, and triggering redraws.
pub trait SecondaryWindow {
  /// Process pending state (flags, shared data). Called every frame
  /// or event-loop iteration by the host.
  fn poll(&mut self, tree: &Tree);

  /// Whether the secondary window needs a redraw.
  fn needs_redraw(&self) -> bool;
}

// ── Driver trait ────────────────────────────────────────────────────────────

/// Bridge between lui and a platform's windowing/input system.
///
/// Implement this trait for any framework (winit, egui, bevy, SDL2,
/// glfw, etc.) and pair it with a [`Runtime`] to get full HTML
/// rendering, input handling, and scroll management.
pub trait Driver {
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

  /// Copy text to the system clipboard. Default: no-op.
  fn set_clipboard_text(&self, _text: &str) {}

  /// Read text from the system clipboard. Default: no clipboard.
  fn get_clipboard_text(&self) -> Option<String> {
    None
  }
}

// ── Runtime ─────────────────────────────────────────────────────────────────

/// Engine-glue: owns the platform `Driver`, the wgpu [`Renderer`], and
/// all shared state needed for event processing and frame rendering.
///
/// `Runtime` is the primary type users interact with. Create one per
/// window, then call the `on_*` methods from your event loop followed
/// by [`Self::render_frame`] on each redraw.
pub struct Runtime<D: Driver, B: RenderBackend> {
  pub driver: D,
  pub renderer: B,
  pub text_ctx: TextContext,
  pub image_cache: lui::layout::ImageCache,
  pipeline_cache: PipelineCache,
  scroll_x: f32,
  scroll_y: f32,
  cursor_pos: Option<(f32, f32)>,
  last_layout: Option<LayoutBox>,
  last_click: Option<ClickTracker>,
  scrollbar_drag: Option<ScrollbarDrag>,
  pub profiling: ProfilingOverlay,
  resize_deadline: Option<Instant>,
  inspect_overlay_path: Option<Vec<usize>>,
  needs_viewport_scrollbar_y: bool,
  needs_viewport_scrollbar_x: bool,
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
  Viewport {
    grab_offset: f32,
    axis: lui::scroll::ScrollbarAxis,
  },
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
  frame_count: u64,
  last_fps_time: Instant,
  last_fps_count: u64,
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
      bar_width: 210.0,
      bar_height: 7.5,
      bar_gap: 1.5,
      margin: 12.0,
      fps: 0.0,
      frame_count: 0,
      last_fps_time: Instant::now(),
      last_fps_count: 0,
      cascade_avg: 0.0,
      layout_avg: 0.0,
      paint_avg: 0.0,
      render_avg: 0.0,
    }
  }

  const ALPHA: f32 = 0.1;

  fn record(&mut self, timings: &PipelineTimings, _frame_ms: f64) {
    self.frame_count += 1;
    let elapsed = self.last_fps_time.elapsed();
    if elapsed.as_millis() >= 500 {
      let frames = self.frame_count - self.last_fps_count;
      self.fps = frames as f32 / elapsed.as_secs_f32();
      self.last_fps_time = Instant::now();
      self.last_fps_count = self.frame_count;
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

  fn draw_bars(&self, list: &mut DisplayList, viewport_w: f32, _viewport_h: f32) {
    let panel_w = self.bar_width + self.margin * 2.0;
    let bars_h = self.margin * 2.0 + (self.bar_height + self.bar_gap) * 4.0;
    let text_h = 12.0 + 4.0 * 12.0 + 6.0;
    let panel_h = bars_h + text_h;
    let x = viewport_w - panel_w - self.margin;
    let y = self.margin;

    list.push_quad_rounded(
      Rect {
        x,
        y,
        w: panel_w,
        h: panel_h,
      },
      [0.0, 0.0, 0.0, 0.65],
      [4.0; 4],
    );

    let stages: [(&str, f32, [f32; 3]); 4] = [
      ("c", self.cascade_avg, [0.9, 0.3, 0.3]),
      ("l", self.layout_avg, [0.3, 0.5, 0.9]),
      ("p", self.paint_avg, [0.3, 0.9, 0.3]),
      ("r", self.render_avg, [0.9, 0.8, 0.2]),
    ];

    let bar_x = x + self.margin;
    let bar_w = self.bar_width;

    for (i, (_label, ms, rgb)) in stages.iter().enumerate() {
      let bar_y = y + self.margin + i as f32 * (self.bar_height + self.bar_gap);
      let w = ((*ms / self.frame_budget_ms).min(1.0) * bar_w).max(1.0);
      list.push_quad_rounded(
        Rect {
          x: bar_x,
          y: bar_y,
          w,
          h: self.bar_height,
        },
        [rgb[0], rgb[1], rgb[2], 0.9],
        [1.5; 4],
      );
    }

    // FPS number below bars using tiny quads as pixel digits.
    let fps_y = y + bars_h + 2.0;
    let fps_text = format!("FPS {:.0}", self.fps);
    draw_pixel_string(list, x + self.margin, fps_y, &fps_text, [0.0, 1.0, 0.0, 0.9]);

    // Stage labels + ms values
    let labels = [
      ("C", self.cascade_avg, [0.9, 0.3, 0.3, 0.9]),
      ("L", self.layout_avg, [0.3, 0.5, 0.9, 0.9]),
      ("P", self.paint_avg, [0.3, 0.9, 0.3, 0.9]),
      ("R", self.render_avg, [0.9, 0.8, 0.2, 0.9]),
    ];
    let label_y = fps_y + 12.0;
    for (i, (label, ms, color)) in labels.iter().enumerate() {
      let ly = label_y + i as f32 * 12.0;
      draw_pixel_string(list, x + self.margin, ly, label, *color);
      let ms_text = format!("{:.1}", ms);
      draw_pixel_string(list, x + self.margin + 15.0, ly, &ms_text, [0.8, 0.8, 0.8, 0.8]);
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
          text.element = lui_tree::Element::Text(label.into());
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

    let overlay_tree = lui::parser::parse(&html);
    let Some(overlay_root) = overlay_tree.root else { return };

    let Some(root) = &mut tree.root else { return };

    let body_idx = root
      .children
      .iter()
      .position(|c| matches!(c.element, lui_tree::Element::Html(_)))
      .and_then(|hi| {
        root.children[hi]
          .children
          .iter()
          .position(|c| matches!(c.element, lui_tree::Element::Body(_)))
          .map(|bi| (hi, bi))
      });

    let target = match body_idx {
      Some((hi, bi)) => &mut root.children[hi].children[bi],
      None => root,
    };

    let overlay_body = overlay_root
      .children
      .iter()
      .find(|c| matches!(c.element, lui_tree::Element::Html(_)))
      .and_then(|h| {
        h.children
          .iter()
          .find(|c| matches!(c.element, lui_tree::Element::Body(_)))
      });
    if let Some(ob) = overlay_body {
      if let Some(div) = ob.children.first() {
        target.children.push(div.clone());
      }
    }
  }
}

impl Default for ProfilingOverlay {
  fn default() -> Self {
    Self::new()
  }
}

impl<D: Driver, B: RenderBackend> Runtime<D, B> {
  // ── Construction ────────────────────────────────────────────────────────

  /// Create a runtime with the given driver and render backend.
  ///
  /// The caller is responsible for creating the backend (e.g. wgpu
  /// `Renderer`, a D3D12 backend, etc.) and passing it in.
  pub fn new(driver: D, renderer: B) -> Self {
    let atlas_size = renderer.glyph_atlas_size();
    Self {
      driver,
      renderer,
      text_ctx: TextContext::new(atlas_size),
      image_cache: lui::layout::ImageCache::default(),
      pipeline_cache: PipelineCache::new(),
      scroll_x: 0.0,
      scroll_y: 0.0,
      cursor_pos: None,
      last_layout: None,
      last_click: None,
      scrollbar_drag: None,
      profiling: ProfilingOverlay::new(),
      resize_deadline: None,
      inspect_overlay_path: None,
      needs_viewport_scrollbar_y: false,
      needs_viewport_scrollbar_x: false,
    }
  }

  // ── Accessors ───────────────────────────────────────────────────────────

  pub fn layout(&self) -> Option<&LayoutBox> {
    self.last_layout.as_ref()
  }

  pub fn scroll_x(&self) -> f32 {
    self.scroll_x
  }

  pub fn scroll_y(&self) -> f32 {
    self.scroll_y
  }

  pub fn set_inspect_overlay(&mut self, path: Option<Vec<usize>>) {
    self.inspect_overlay_path = path;
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

    // Resize debounce: keep requesting redraws while the deadline is
    // active so the frame pump doesn't stall after the user releases
    // the resize handle.  Always paint at the actual window size so
    // the display list coordinates match the GPU surface — using
    // stale cached dimensions produces a blank canvas.
    if let Some(deadline) = self.resize_deadline {
      if Instant::now() < deadline {
        self.driver.request_redraw();
      } else {
        self.resize_deadline = None;
      }
    }
    let (paint_w, paint_h) = (w as f32, h as f32);

    // Reserve space for the viewport scrollbar so content doesn't
    // overlap. Uses previous frame's state; if it changes we
    // invalidate the cache and re-layout next frame.
    let scrollbar_w = lui::scroll::VIEWPORT_SCROLLBAR_WIDTH + 4.0;
    let content_w = if self.needs_viewport_scrollbar_y {
      paint_w - scrollbar_w
    } else {
      paint_w
    };

    // Pre-shape overlay text so the atlas includes all glyphs before the main paint.
    lui::color_picker_overlay::update_cached_layout(tree, &mut self.text_ctx);

    let (mut list, layout, timings) = lui::paint_tree_cached(
      tree,
      &mut self.text_ctx,
      &mut self.image_cache,
      content_w,
      paint_h,
      scale,
      self.scroll_y,
      &mut self.pipeline_cache,
    );
    tree.dirty_paths.clear();

    if let Some(layout) = layout {
      lui::update_edit_scroll(tree, layout);
      self.scroll_y = clamp_scroll_y(self.scroll_y, layout, h as f32);
      if let Some(bg) = layout.background {
        self.renderer.set_clear_color(bg);
      }
    } else {
      self.scroll_y = 0.0;
    }

    self.last_layout = self.pipeline_cache.layout().cloned();

    let body_scroll = tree
      .interaction
      .scroll_offsets
      .values()
      .next()
      .map(|s| s.y)
      .unwrap_or(0.0);
    let effective_scroll_y = if self.scroll_y.abs() > 0.5 {
      self.scroll_y
    } else {
      body_scroll
    };

    let max_scroll_y = self
      .last_layout
      .as_ref()
      .map(|l| {
        let driver_max = lui::scroll::max_scroll_y(l, h as f32);
        if driver_max > 0.5 {
          driver_max
        } else {
          lui::scroll::body_max_scroll(l, h as f32)
        }
      })
      .unwrap_or(0.0);
    let max_scroll_x = self
      .last_layout
      .as_ref()
      .map(|l| lui::scroll::max_scroll_x(l, w as f32))
      .unwrap_or(0.0);

    let needs_y = max_scroll_y > 0.5;
    let needs_x = max_scroll_x > 0.5;
    if needs_y != self.needs_viewport_scrollbar_y {
      self.needs_viewport_scrollbar_y = needs_y;
      self.pipeline_cache.invalidate();
    }
    if needs_x != self.needs_viewport_scrollbar_x {
      self.needs_viewport_scrollbar_x = needs_x;
    }

    if let Some(path) = &self.inspect_overlay_path {
      if let Some(ref layout) = self.last_layout {
        lui::inspect_overlay::paint_inspect_overlay(
          &mut list,
          layout,
          tree,
          &mut self.text_ctx,
          path,
          0.0,
          scale,
          content_w,
          paint_h,
        );
      }
    }

    if let Some(ref layout) = self.last_layout {
      lui::color_picker_overlay::paint_color_picker_overlay(&mut list, layout, tree, 0.0, scale, content_w, paint_h);
      lui::date_picker_overlay::paint_date_picker_overlay(&mut list, layout, tree, &mut self.text_ctx);
    }

    translate_display_list_x(&mut list, -self.scroll_x);
    translate_display_list_y(&mut list, -self.scroll_y);

    // Paint viewport scrollbars.
    let thumb_color = self
      .last_layout
      .as_ref()
      .and_then(|l| l.overflow.scrollbar_thumb)
      .unwrap_or(lui::scroll::DEFAULT_THUMB);
    let vw = w as f32;
    let vh = h as f32;
    let track_w = lui::scroll::VIEWPORT_SCROLLBAR_WIDTH;
    let margin = 2.0;
    let inset = 1.0;

    if self.needs_viewport_scrollbar_y || self.needs_viewport_scrollbar_x {
      list.push_clip(None, [0.0; 4], [0.0; 4]);
    }
    if self.needs_viewport_scrollbar_y {
      let bar_h = if self.needs_viewport_scrollbar_x {
        vh - track_w
      } else {
        vh
      };
      let doc_h = max_scroll_y + vh;
      let track_h = bar_h - margin * 2.0;
      let thumb_h = (track_h * vh / doc_h).clamp(24.0, track_h);
      let travel = (track_h - thumb_h).max(0.0);
      let thumb_y = margin + travel * (effective_scroll_y / max_scroll_y.max(1.0));
      let thumb_x = vw - track_w - margin + inset;
      let thumb_w = track_w - inset * 2.0;
      let radius = thumb_w * 0.5;
      list.push_quad_rounded(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), thumb_color, [radius; 4]);
    }
    if self.needs_viewport_scrollbar_x {
      let bar_w = if self.needs_viewport_scrollbar_y {
        vw - track_w
      } else {
        vw
      };
      let doc_w = max_scroll_x + vw;
      let track_w_h = bar_w - margin * 2.0;
      let thumb_w = (track_w_h * vw / doc_w).clamp(24.0, track_w_h);
      let travel = (track_w_h - thumb_w).max(0.0);
      let thumb_x = margin + travel * (self.scroll_x / max_scroll_x.max(1.0));
      let thumb_y = vh - track_w - margin + inset;
      let thumb_h = track_w - inset * 2.0;
      let radius = thumb_h * 0.5;
      list.push_quad_rounded(Rect::new(thumb_x, thumb_y, thumb_w, thumb_h), thumb_color, [radius; 4]);
    }
    if self.needs_viewport_scrollbar_y || self.needs_viewport_scrollbar_x {
      list.finalize();
    }

    // Record frame timings and draw the bar chart.
    if self.profiling.enabled {
      let frame_ms = frame_t0.elapsed().as_secs_f64() * 1000.0;
      self.profiling.record(&timings, frame_ms);
      self.profiling.draw_bars(&mut list, w as f32, h as f32);
    }

    self.text_ctx.atlas.flush_dirty(|rect, data| {
      self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
    });

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
        if cw > 0.0 && ch > 0.0 {
          (cw, ch)
        } else {
          (w as f32, h as f32)
        }
      } else {
        self.pipeline_cache.invalidate();
        self.resize_deadline = None;
        (w as f32, h as f32)
      }
    } else {
      (w as f32, h as f32)
    };

    let (list, layout, timings) = lui::paint_tree_cached(
      tree,
      &mut self.text_ctx,
      &mut self.image_cache,
      paint_w,
      paint_h,
      scale,
      self.scroll_y,
      &mut self.pipeline_cache,
    );
    tree.dirty_paths.clear();

    if let Some(layout) = layout {
      self.scroll_y = clamp_scroll_y(self.scroll_y, layout, h as f32);
      if let Some(bg) = layout.background {
        self.renderer.set_clear_color(bg);
      }
    } else {
      self.scroll_y = 0.0;
    }
    self.last_layout = self.pipeline_cache.layout().cloned();

    (list, self.last_layout.as_ref(), timings)
  }

  /// Upload pending glyph atlas rasters to the GPU.
  pub fn upload_glyphs(&mut self) {
    self.text_ctx.atlas.flush_dirty(|rect, data| {
      self.renderer.upload_atlas_region(rect.x, rect.y, rect.w, rect.h, data);
    });
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
          ScrollbarDrag::Viewport { grab_offset, axis } => {
            use lui::scroll::ScrollbarAxis;
            let vw = w as f32;
            let vh = h as f32;
            let track_w = lui::scroll::VIEWPORT_SCROLLBAR_WIDTH;
            let margin = 2.0;
            match axis {
              ScrollbarAxis::Vertical => {
                let max_sy = lui::scroll::max_scroll_y(layout, vh);
                let max_sx = lui::scroll::max_scroll_x(layout, vw);
                let bar_h = if max_sx > 0.5 { vh - track_w } else { vh };
                let track_h = bar_h - margin * 2.0;
                let thumb_h = (track_h * vh / (max_sy + vh)).clamp(24.0, track_h);
                let travel = (track_h - thumb_h).max(0.0);
                let t = ((y - grab_offset - margin) / travel.max(1.0)).clamp(0.0, 1.0);
                self.scroll_y = t * max_sy;
              }
              ScrollbarAxis::Horizontal => {
                let max_sx = lui::scroll::max_scroll_x(layout, vw);
                let max_sy = lui::scroll::max_scroll_y(layout, vh);
                let bar_w = if max_sy > 0.5 { vw - track_w } else { vw };
                let track_w_px = bar_w - margin * 2.0;
                let thumb_w = (track_w_px * vw / (max_sx + vw)).clamp(24.0, track_w_px);
                let travel = (track_w_px - thumb_w).max(0.0);
                let t = ((x - grab_offset - margin) / travel.max(1.0)).clamp(0.0, 1.0);
                self.scroll_x = t * max_sx;
              }
            }
          }
          ScrollbarDrag::Element(el_drag) => {
            let doc_pos = viewport_to_document((x, y), self.scroll_x, self.scroll_y);
            el_drag.update(layout, tree, doc_pos.0, doc_pos.1);
          }
        }
        self.driver.request_redraw();
        return true;
      }
    }

    let Some(layout) = self.last_layout.as_ref() else {
      return false;
    };

    let doc_pos = viewport_to_document((x, y), self.scroll_x, self.scroll_y);
    let (changed, css_cursor) = interactivity::pointer_move_with_cursor(tree, layout, doc_pos);
    self.driver.set_cursor(css_cursor);

    changed || tree.interaction.selecting_text
  }

  /// Process a pointer leave (cursor exited the window).
  /// Returns `true` if the caller should redraw.
  pub fn on_pointer_leave(&mut self, tree: &mut Tree) -> bool {
    self.cursor_pos = None;
    self.scrollbar_drag = None;
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

    let doc_pos = viewport_to_document(pos, self.scroll_x, self.scroll_y);

    if pressed {
      let target_path = {
        let Some(layout) = self.last_layout.as_ref() else {
          return false;
        };
        layout.hit_path_scrolled(doc_pos, &tree.interaction.scroll_offsets)
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
  pub fn on_wheel(&mut self, tree: &mut Tree, pixel_dy: f32, pixel_dx: f32, _effective_scale: f32) -> bool {
    let Some(layout) = self.last_layout.as_ref() else {
      return false;
    };
    let Some(pos) = self.cursor_pos else {
      return false;
    };
    let doc_pos = viewport_to_document(pos, self.scroll_x, self.scroll_y);

    if scroll_element_at(tree, layout, doc_pos, pixel_dx, pixel_dy) {
      interactivity::pointer_move(tree, layout, doc_pos);
      return true;
    }

    let (w, h) = self.driver.inner_size();
    self.scroll_y = clamp_scroll_y(self.scroll_y + pixel_dy, layout, h as f32);
    self.scroll_x = clamp_scroll_x(self.scroll_x + pixel_dx, layout, w as f32);
    let new_doc_pos = viewport_to_document(pos, self.scroll_x, self.scroll_y);
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
    let doc_pos = viewport_to_document(pos, self.scroll_x, self.scroll_y);
    tree.wheel_event(doc_pos, dx, dy, mode)
  }

  /// Process a keyboard event.
  ///
  /// `key` is the DOM `KeyboardEvent.key` value,
  /// `code` is `KeyboardEvent.code` (physical key identifier),
  /// `text` is the composed character for text insertion (if any).
  pub fn on_key(&mut self, tree: &mut Tree, key: &str, code: &str, pressed: bool, repeat: bool, text: Option<&str>) {
    if pressed {
      let ctrl = tree.modifiers().ctrl;
      let shift = tree.modifiers().shift;

      // Color picker field intercept
      if tree
        .interaction
        .color_picker
        .as_ref()
        .is_some_and(|cp| cp.active_field.is_some())
      {
        if ctrl && !repeat {
          match code {
            "KeyA" | "KeyC" | "KeyX" | "KeyV" => {
              if let Some(cp) = &mut tree.interaction.color_picker {
                match code {
                  "KeyA" => {
                    lui::color_picker_overlay::field_key_down(cp, key, code, ctrl, shift);
                  }
                  "KeyC" => {
                    if let Some(sel) = lui::color_picker_overlay::field_selected_text(cp) {
                      self.driver.set_clipboard_text(&sel);
                    }
                  }
                  "KeyX" => {
                    if let Some(sel) = lui::color_picker_overlay::field_selected_text(cp) {
                      self.driver.set_clipboard_text(&sel);
                      let (v, c) = lui_tree::text_edit::delete_selection(&cp.field_text, &cp.field_cursor);
                      cp.field_text = v;
                      cp.field_cursor = c;
                    }
                  }
                  "KeyV" => {
                    if let Some(clip) = self.driver.get_clipboard_text() {
                      if !clip.is_empty() {
                        lui::color_picker_overlay::field_text_input(cp, &clip);
                      }
                    }
                  }
                  _ => {}
                }
              }
              self.driver.request_redraw();
              return;
            }
            _ => {}
          }
        }
        if let Some(cp) = &mut tree.interaction.color_picker {
          if lui::color_picker_overlay::field_key_down(cp, key, code, ctrl, shift) {
            if cp.active_field.is_none() {
              let path = cp.path.clone();
              let (r, g, b) = lui::color_picker_overlay::hsv_to_srgb_u8(cp.hue, cp.saturation, cp.value);
              let a = cp.alpha;
              lui_tree::set_color_value(tree, &path, r, g, b, a);
            }
            self.driver.request_redraw();
            return;
          }
        }
        if !ctrl && !tree.modifiers().meta {
          if let Some(s) = text {
            if !s.is_empty() && s.chars().all(|c| !c.is_control()) {
              if let Some(cp) = &mut tree.interaction.color_picker {
                lui::color_picker_overlay::field_text_input(cp, s);
              }
              self.driver.request_redraw();
              return;
            }
          }
        }
        return;
      }

      // ── Clipboard / selection shortcuts ──────────────────────────
      if ctrl && !repeat {
        match code {
          "KeyA" => {
            if tree.interaction.edit_cursor.is_none() {
              if let Some(layout) = self.last_layout.as_ref() {
                select_all_text(tree, layout);
              }
            }
            tree.key_down(key, code, repeat);
            return;
          }
          "KeyC" => {
            tree.clipboard_event("copy");
            if let Some(layout) = self.last_layout.as_ref() {
              if let Some(text) = selected_text(tree, layout) {
                self.driver.set_clipboard_text(&text);
              }
            }
            tree.key_down(key, code, repeat);
            return;
          }
          "KeyX" => {
            tree.clipboard_event("cut");
            if let Some(cut_text) = lui_tree::cut_selection(tree) {
              self.driver.set_clipboard_text(&cut_text);
            } else if let Some(layout) = self.last_layout.as_ref() {
              if let Some(text) = selected_text(tree, layout) {
                self.driver.set_clipboard_text(&text);
              }
            }
            tree.key_down(key, code, repeat);
            return;
          }
          "KeyV" => {
            tree.clipboard_event("paste");
            if let Some(text) = self.driver.get_clipboard_text() {
              if !text.is_empty() {
                lui_tree::text_input(tree, &text);
              }
            }
            tree.key_down(key, code, repeat);
            return;
          }
          _ => {}
        }
      }

      tree.key_down(key, code, repeat);
      // Feed typed text into the focused form control.
      // Skip when Ctrl or Meta is held — those are shortcuts.
      if !ctrl && !tree.modifiers().meta {
        if let Some(s) = text {
          if !s.is_empty() && s.chars().all(|c| !c.is_control()) {
            lui_tree::text_input(tree, s);
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
  pub fn set_modifier(&self, tree: &mut Tree, modifier: lui_tree::Modifier, down: bool) {
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
      self.scroll_x = clamp_scroll_x(self.scroll_x, layout, width as f32);
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
    use lui::scroll::ScrollbarAxis;
    let Some(layout) = self.last_layout.as_ref() else {
      return false;
    };
    let (w, h) = self.driver.inner_size();
    let doc_pos = viewport_to_document(pos, self.scroll_x, self.scroll_y);

    if let Some(el_drag) = ElementScrollbarDrag::try_start(layout, doc_pos, tree) {
      self.scrollbar_drag = Some(ScrollbarDrag::Element(el_drag));
      return true;
    }

    let vw = w as f32;
    let vh = h as f32;
    let track_w = lui::scroll::VIEWPORT_SCROLLBAR_WIDTH;
    let margin = 2.0;
    let inset = 1.0;
    let max_sy = lui::scroll::max_scroll_y(layout, vh);
    let max_sx = lui::scroll::max_scroll_x(layout, vw);

    if max_sy > 0.5 {
      let bar_h = if max_sx > 0.5 { vh - track_w } else { vh };
      let track_h = bar_h - margin * 2.0;
      let thumb_h = (track_h * vh / (max_sy + vh)).clamp(24.0, track_h);
      let travel = (track_h - thumb_h).max(0.0);
      let thumb_y = margin + travel * (self.scroll_y / max_sy.max(1.0));
      let thumb_x = vw - track_w - margin + inset;
      let thumb_w = track_w - inset * 2.0;
      let track_rect = Rect::new(vw - track_w - margin, margin, track_w, bar_h - margin * 2.0);
      let thumb_rect = Rect::new(thumb_x, thumb_y, thumb_w, thumb_h);

      if rect_contains(thumb_rect, pos) {
        self.scrollbar_drag = Some(ScrollbarDrag::Viewport {
          grab_offset: pos.1 - thumb_y,
          axis: ScrollbarAxis::Vertical,
        });
        return true;
      }
      if rect_contains(track_rect, pos) {
        let t = ((pos.1 - margin - thumb_h * 0.5) / travel.max(1.0)).clamp(0.0, 1.0);
        self.scroll_y = t * max_sy;
        self.scrollbar_drag = Some(ScrollbarDrag::Viewport {
          grab_offset: thumb_h * 0.5,
          axis: ScrollbarAxis::Vertical,
        });
        return true;
      }
    }

    if max_sx > 0.5 {
      let bar_w = if max_sy > 0.5 { vw - track_w } else { vw };
      let track_w_px = bar_w - margin * 2.0;
      let thumb_w = (track_w_px * vw / (max_sx + vw)).clamp(24.0, track_w_px);
      let travel = (track_w_px - thumb_w).max(0.0);
      let thumb_x = margin + travel * (self.scroll_x / max_sx.max(1.0));
      let thumb_y = vh - track_w - margin + inset;
      let thumb_h = track_w - inset * 2.0;
      let track_rect = Rect::new(margin, vh - track_w - margin, bar_w - margin * 2.0, track_w);
      let thumb_rect = Rect::new(thumb_x, thumb_y, thumb_w, thumb_h);

      if rect_contains(thumb_rect, pos) {
        self.scrollbar_drag = Some(ScrollbarDrag::Viewport {
          grab_offset: pos.0 - thumb_x,
          axis: ScrollbarAxis::Horizontal,
        });
        return true;
      }
      if rect_contains(track_rect, pos) {
        let t = ((pos.0 - margin - thumb_w * 0.5) / travel.max(1.0)).clamp(0.0, 1.0);
        self.scroll_x = t * max_sx;
        self.scrollbar_drag = Some(ScrollbarDrag::Viewport {
          grab_offset: thumb_w * 0.5,
          axis: ScrollbarAxis::Horizontal,
        });
        return true;
      }
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

// ── Pixel-font overlay helpers ──────────────────────────────────────────────

const PIXEL_SIZE: f32 = 2.5;
const CHAR_W: f32 = 4.0 * PIXEL_SIZE;
const CHAR_GAP: f32 = 1.0 * PIXEL_SIZE;

fn draw_pixel_string(list: &mut DisplayList, x: f32, y: f32, s: &str, color: [f32; 4]) {
  let mut cx = x;
  for ch in s.chars() {
    draw_pixel_char(list, cx, y, ch, color);
    cx += CHAR_W + CHAR_GAP;
  }
}

fn draw_pixel_char(list: &mut DisplayList, x: f32, y: f32, ch: char, color: [f32; 4]) {
  let bitmap: [u8; 5] = match ch {
    '0' => [0b1111, 0b1001, 0b1001, 0b1001, 0b1111],
    '1' => [0b0010, 0b0110, 0b0010, 0b0010, 0b0111],
    '2' => [0b1111, 0b0001, 0b1111, 0b1000, 0b1111],
    '3' => [0b1111, 0b0001, 0b0111, 0b0001, 0b1111],
    '4' => [0b1001, 0b1001, 0b1111, 0b0001, 0b0001],
    '5' => [0b1111, 0b1000, 0b1111, 0b0001, 0b1111],
    '6' => [0b1111, 0b1000, 0b1111, 0b1001, 0b1111],
    '7' => [0b1111, 0b0001, 0b0010, 0b0100, 0b0100],
    '8' => [0b1111, 0b1001, 0b1111, 0b1001, 0b1111],
    '9' => [0b1111, 0b1001, 0b1111, 0b0001, 0b1111],
    '.' => [0b0000, 0b0000, 0b0000, 0b0000, 0b0100],
    'C' => [0b1111, 0b1000, 0b1000, 0b1000, 0b1111],
    'L' => [0b1000, 0b1000, 0b1000, 0b1000, 0b1111],
    'P' => [0b1111, 0b1001, 0b1111, 0b1000, 0b1000],
    'R' => [0b1111, 0b1001, 0b1111, 0b1010, 0b1001],
    'F' => [0b1111, 0b1000, 0b1110, 0b1000, 0b1000],
    'S' => [0b1111, 0b1000, 0b1111, 0b0001, 0b1111],
    _ => [0b0000, 0b0000, 0b0000, 0b0000, 0b0000],
  };
  for (row, &bits) in bitmap.iter().enumerate() {
    for col in 0..4 {
      if bits & (1 << (3 - col)) != 0 {
        list.push_quad(
          Rect::new(
            x + col as f32 * PIXEL_SIZE,
            y + row as f32 * PIXEL_SIZE,
            PIXEL_SIZE,
            PIXEL_SIZE,
          ),
          color,
        );
      }
    }
  }
}
