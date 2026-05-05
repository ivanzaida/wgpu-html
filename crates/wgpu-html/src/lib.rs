//! Top-level facade for the wgpu-html stack.
//!
//! Re-exports the model types and the renderer so downstream apps only need
//! one dependency.

use std::time::Instant;

pub use wgpu_html_events as events;
pub use wgpu_html_layout as layout;
pub use wgpu_html_models as models;
pub use wgpu_html_parser as parser;
pub use wgpu_html_renderer as renderer;
pub use wgpu_html_style as style;
pub use wgpu_html_text as text;
pub use wgpu_html_tree as tree;

pub mod interactivity;
pub mod paint;
pub mod scroll;
pub use paint::{paint_tree, paint_tree_with_text};
use wgpu_html_layout::{LayoutBox, UserSelect};
use wgpu_html_renderer::{DisplayList, Renderer, ScreenshotError};
use wgpu_html_style::MediaContext;
use wgpu_html_text::TextContext;
use wgpu_html_tree::{InteractionSnapshot, TextCursor, TextSelection, Tree};

#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineTimings {
  pub cascade_ms: f64,
  pub layout_ms: f64,
  pub paint_ms: f64,
}

impl PipelineTimings {
  pub fn total_ms(self) -> f64 {
    self.cascade_ms + self.layout_ms + self.paint_ms
  }
}

/// Cascade + lay out `tree` against `text_ctx` and return the
/// resulting `LayoutBox` without painting. Hosts that need the layout
/// for hit-testing (e.g. dispatching pointer events between frames)
/// pair this with [`paint::paint_layout`] to render.
pub fn compute_layout(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
) -> Option<LayoutBox> {
  compute_layout_profiled(tree, text_ctx, image_cache, viewport_w, viewport_h, scale).0
}

pub fn compute_layout_profiled(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
) -> (Option<LayoutBox>, PipelineTimings) {
  tree.profiler.as_ref().map(|p| p.ensure_frame_begin());

  text_ctx.sync_fonts(&tree.fonts);
  if let Some(ttl) = tree.asset_cache_ttl {
    image_cache.set_ttl(ttl);
  }
  for url in &tree.preload_queue {
    image_cache.preload(url);
  }

  let cascade_t0 = Instant::now();
  let media = media_context(viewport_w, viewport_h, scale);
  let cascaded;
  {
    wgpu_html_tree::prof_scope!(&tree.profiler, "cascade");
    cascaded = wgpu_html_style::cascade_with_media(tree, &media);
  }
  let cascade_ms = cascade_t0.elapsed().as_secs_f64() * 1000.0;

  let layout_t0 = Instant::now();
  let layout;
  {
    wgpu_html_tree::prof_scope!(&tree.profiler, "layout");
    layout = wgpu_html_layout::layout_with_text(&cascaded, text_ctx, image_cache, viewport_w, viewport_h, scale);
  }
  let layout_ms = layout_t0.elapsed().as_secs_f64() * 1000.0;

  (
    layout,
    PipelineTimings {
      cascade_ms,
      layout_ms,
      paint_ms: 0.0,
    },
  )
}

/// Convenience: [`compute_layout`] + [`paint::paint_layout`] in one
/// call, returning both. The display list is finalised; the layout
/// can be retained for the next frame's hit-testing.
pub fn paint_tree_returning_layout(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  viewport_scroll_y: f32,
) -> (DisplayList, Option<LayoutBox>) {
  let (list, layout, _) = paint_tree_returning_layout_profiled(
    tree,
    text_ctx,
    image_cache,
    viewport_w,
    viewport_h,
    scale,
    viewport_scroll_y,
  );
  (list, layout)
}

pub fn paint_tree_returning_layout_profiled(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  viewport_scroll_y: f32,
) -> (DisplayList, Option<LayoutBox>, PipelineTimings) {
  // ensure_frame_begin is a no-op if compute_layout_profiled already started one.
  tree.profiler.as_ref().map(|p| p.ensure_frame_begin());

  let (layout, mut timings) = compute_layout_profiled(tree, text_ctx, image_cache, viewport_w, viewport_h, scale);
  let mut list = DisplayList::new();
  let paint_t0 = Instant::now();
  {
    wgpu_html_tree::prof_scope!(&tree.profiler, "paint");
    if let Some(root) = layout.as_ref() {
      // Build caret info from the interaction state.
      let edit_caret_info = tree.interaction.edit_cursor.as_ref().and_then(|ec| {
        let fp = tree.interaction.focus_path.as_deref()?;
        let elapsed_ms = tree.interaction.caret_blink_epoch.elapsed().as_millis();
        let sel = if ec.has_selection() {
          Some(ec.selection_range())
        } else {
          None
        };
        Some(paint::EditCaretInfo {
          focus_path: fp,
          cursor_byte: ec.cursor,
          selection_bytes: sel,
          caret_visible: !ec.has_selection() && (elapsed_ms % 1000) < 500,
        })
      });
      paint::paint_layout_full(
        root,
        &mut list,
        tree.interaction.selection.as_ref(),
        tree.interaction.selection_colors,
        &tree.interaction.scroll_offsets_y,
        viewport_scroll_y,
        edit_caret_info.as_ref(),
      );
      list.finalize();
    } else {
      list.finalize();
    }
  }
  timings.paint_ms = paint_t0.elapsed().as_secs_f64() * 1000.0;

  tree.profiler.as_ref().map(|p| p.frame_end());
  (list, layout, timings)
}

// ── Pipeline cache (O1 optimisation) ─────────────────────────────────────────

/// What the per-frame pipeline needs to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineAction {
  /// DOM / viewport / fonts changed — must cascade + layout + paint from scratch.
  FullPipeline,
  /// Only pseudo-class state changed (hover / active / focus).
  /// Run incremental re-cascade on affected nodes, then relayout + repaint.
  PartialCascade,
  /// The DOM changed but element selectors (tag / id / class) are
  /// unchanged — the cascaded CSS tree is still valid.  Only
  /// re-layout and repaint; skip the full cascade.
  LayoutOnly,
  /// Layout is unchanged; only repaint (scroll / selection / caret changed).
  RepaintOnly,
}

/// Cached results from the previous frame's cascade + layout. Lets
/// the harness skip the expensive stages when inputs haven't changed.
pub struct PipelineCache {
  snapshot: InteractionSnapshot,
  viewport: (f32, f32),
  scale: f32,
  font_generation: u64,
  tree_generation: u64,
  cascade_generation: u64,
  layout: Option<LayoutBox>,
  /// Cached cascade result for incremental re-cascade.
  cascaded: Option<wgpu_html_style::CascadedTree>,
  /// When `true`, a `PartialCascade` (hover / active / focus change)
  /// re-cascades the affected nodes but **skips re-layout**.  This is
  /// safe when the stylesheet's pseudo-class rules only set paint
  /// properties (`background-color`, `color`, `opacity`, …) that
  /// don't affect box dimensions.  The updated cascade is still
  /// repainted, so the visual change appears immediately.
  pub paint_only_pseudo_rules: bool,
}

impl PipelineCache {
  pub fn new() -> Self {
    Self {
      snapshot: InteractionSnapshot {
        hover_path: None,
        active_path: None,
        focus_path: None,
      },
      viewport: (0.0, 0.0),
      scale: 0.0,
      font_generation: u64::MAX, // force first frame to run
      tree_generation: u64::MAX,
      cascade_generation: u64::MAX,
      layout: None,
      cascaded: None,
      paint_only_pseudo_rules: false,
    }
  }

  /// Force a full re-cascade + relayout on the next frame.
  pub fn invalidate(&mut self) {
    self.layout = None;
    self.cascaded = None;
  }

  /// Borrow the cached layout (if any).
  pub fn layout(&self) -> Option<&LayoutBox> {
    self.layout.as_ref()
  }

  /// The viewport used for the last full pipeline render.
  pub fn viewport(&self) -> (f32, f32) {
    self.viewport
  }
}

/// Determine what work the pipeline needs to do this frame.
pub fn classify_frame(
  tree: &Tree,
  cache: &PipelineCache,
  image_cache: &wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
) -> PipelineAction {
  if cache.layout.is_none() || cache.cascaded.is_none() {
    return PipelineAction::FullPipeline;
  }
  // Async images still loading or animated images advancing —
  // must re-layout so newly-decoded images / next animation
  // frames appear.
  if image_cache.has_pending() || image_cache.has_animated() {
    return PipelineAction::FullPipeline;
  }
  if (cache.viewport.0 - viewport_w).abs() > 0.5
    || (cache.viewport.1 - viewport_h).abs() > 0.5
    || (cache.scale - scale).abs() > 0.001
  {
    return PipelineAction::FullPipeline;
  }
  if tree.generation != cache.tree_generation {
    // If only inline styles / text changed (same selectors)
    // we can skip the CSS cascade and just re-layout.
    if tree.cascade_generation == cache.cascade_generation {
      return PipelineAction::LayoutOnly;
    }
    return PipelineAction::FullPipeline;
  }
  if tree.fonts.generation() != cache.font_generation {
    return PipelineAction::FullPipeline;
  }
  let current = tree.interaction.cascade_snapshot();
  if current != cache.snapshot {
    return PipelineAction::PartialCascade;
  }
  PipelineAction::RepaintOnly
}

/// Run the pipeline with caching: skip cascade + layout when inputs
/// haven't changed, repaint from the cached `LayoutBox`.
///
/// Returns `(display_list, &LayoutBox, timings)`. The layout
/// reference borrows from `cache`.
pub fn paint_tree_cached<'c>(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  viewport_scroll_y: f32,
  cache: &'c mut PipelineCache,
) -> (DisplayList, Option<&'c LayoutBox>, PipelineTimings) {
  tree.profiler.as_ref().map(|p| p.ensure_frame_begin());

  let action = classify_frame(tree, cache, image_cache, viewport_w, viewport_h, scale);

  let mut timings = PipelineTimings::default();

  match action {
    PipelineAction::FullPipeline => {
      text_ctx.sync_fonts(&tree.fonts);
      if let Some(ttl) = tree.asset_cache_ttl {
        image_cache.set_ttl(ttl);
      }
      for url in &tree.preload_queue {
        image_cache.preload(url);
      }

      let cascade_t0 = Instant::now();
      let media = media_context(viewport_w, viewport_h, scale);
      let cascaded;
      {
        wgpu_html_tree::prof_scope!(&tree.profiler, "cascade");
        cascaded = wgpu_html_style::cascade_with_media(tree, &media);
      }
      timings.cascade_ms = cascade_t0.elapsed().as_secs_f64() * 1000.0;

      let layout_t0 = Instant::now();
      let layout;
      {
        wgpu_html_tree::prof_scope!(&tree.profiler, "layout");
        layout = wgpu_html_layout::layout_with_text(&cascaded, text_ctx, image_cache, viewport_w, viewport_h, scale);
      }
      timings.layout_ms = layout_t0.elapsed().as_secs_f64() * 1000.0;

      cache.layout = layout;
      cache.cascaded = Some(cascaded);
      cache.snapshot = tree.interaction.cascade_snapshot();
      cache.viewport = (viewport_w, viewport_h);
      cache.scale = scale;
      cache.font_generation = tree.fonts.generation();
      cache.tree_generation = tree.generation;
      cache.paint_only_pseudo_rules = wgpu_html_style::pseudo_rules_are_paint_only(tree);
    }
    PipelineAction::PartialCascade => {
      let cascade_t0 = Instant::now();
      let old_snapshot = cache.snapshot.clone();
      let media = media_context(viewport_w, viewport_h, scale);
      let changed;
      {
        wgpu_html_tree::prof_scope!(&tree.profiler, "cascade_incremental");
        changed = if let Some(cascaded) = &mut cache.cascaded {
          wgpu_html_style::cascade_incremental_with_media(tree, cascaded, &old_snapshot, &media)
        } else {
          false
        };
      }
      timings.cascade_ms = cascade_t0.elapsed().as_secs_f64() * 1000.0;

      // Re-layout if cascade changed any styles — unless
      // pseudo-class rules are paint-only, in which case we
      // patch colors in the existing LayoutBox tree (O(n) field
      // writes, no geometry recomputation).
      if changed {
        if cache.paint_only_pseudo_rules {
          let layout_t0 = Instant::now();
          {
            wgpu_html_tree::prof_scope!(&tree.profiler, "patch_colors");
            if let (Some(layout), Some(cascaded)) = (&mut cache.layout, &cache.cascaded) {
              wgpu_html_layout::patch_layout_colors(layout, cascaded);
            }
          }
          timings.layout_ms = layout_t0.elapsed().as_secs_f64() * 1000.0;
        } else {
          let layout_t0 = Instant::now();
          {
            wgpu_html_tree::prof_scope!(&tree.profiler, "layout");
            if let Some(cascaded) = &cache.cascaded {
              cache.layout =
                wgpu_html_layout::layout_with_text(cascaded, text_ctx, image_cache, viewport_w, viewport_h, scale);
            }
          }
          timings.layout_ms = layout_t0.elapsed().as_secs_f64() * 1000.0;
        }
      }
      cache.snapshot = tree.interaction.cascade_snapshot();
    }
    PipelineAction::LayoutOnly => {
      // Re-use the cached cascade — only inline styles or text
      // changed, so the cascaded CSS tree is still valid.
      let layout_t0 = Instant::now();
      {
        wgpu_html_tree::prof_scope!(&tree.profiler, "layout");
        if let Some(cascaded) = &cache.cascaded {
          cache.layout =
            wgpu_html_layout::layout_with_text(cascaded, text_ctx, image_cache, viewport_w, viewport_h, scale);
        }
      }
      timings.layout_ms = layout_t0.elapsed().as_secs_f64() * 1000.0;

      cache.snapshot = tree.interaction.cascade_snapshot();
      cache.viewport = (viewport_w, viewport_h);
      cache.scale = scale;
      cache.tree_generation = tree.generation;
    }
    PipelineAction::RepaintOnly => {}
  }

  // Paint (always runs — scroll / selection / caret may have changed).
  let mut list = DisplayList::new();
  let paint_t0 = Instant::now();
  {
    wgpu_html_tree::prof_scope!(&tree.profiler, "paint");
    if let Some(root) = cache.layout.as_ref() {
      let edit_caret_info = tree.interaction.edit_cursor.as_ref().and_then(|ec| {
        let fp = tree.interaction.focus_path.as_deref()?;
        let elapsed_ms = tree.interaction.caret_blink_epoch.elapsed().as_millis();
        let sel = if ec.has_selection() {
          Some(ec.selection_range())
        } else {
          None
        };
        Some(paint::EditCaretInfo {
          focus_path: fp,
          cursor_byte: ec.cursor,
          selection_bytes: sel,
          caret_visible: !ec.has_selection() && (elapsed_ms % 1000) < 500,
        })
      });
      paint::paint_layout_full(
        root,
        &mut list,
        tree.interaction.selection.as_ref(),
        tree.interaction.selection_colors,
        &tree.interaction.scroll_offsets_y,
        viewport_scroll_y,
        edit_caret_info.as_ref(),
      );
      list.finalize();
    } else {
      list.finalize();
    }
  }
  timings.paint_ms = paint_t0.elapsed().as_secs_f64() * 1000.0;

  tree.profiler.as_ref().map(|p| p.frame_end());
  (list, cache.layout.as_ref(), timings)
}

fn media_context(viewport_w: f32, viewport_h: f32, scale: f32) -> MediaContext {
  let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
  MediaContext::screen((viewport_w / scale).max(0.0), (viewport_h / scale).max(0.0), scale)
}

/// Walk a child-index path through a layout tree, starting at `root`,
/// and return the matching `LayoutBox`. The empty path returns
/// `Some(root)`; any out-of-bounds index returns `None`.
///
/// Layout child indices line up 1:1 with the indices used elsewhere
/// in this crate for paths (text-cursor `path`, scroll-offset map
/// keys, etc.), so a path obtained from any of those APIs can be
/// re-used here without remapping.
pub fn layout_at_path<'a>(root: &'a LayoutBox, path: &[usize]) -> Option<&'a LayoutBox> {
  let mut cur = root;
  for &i in path {
    cur = cur.children.get(i)?;
  }
  Some(cur)
}

/// What can go wrong when capturing a screenshot of a single node
/// via [`screenshot_node_to`].
#[derive(Debug)]
pub enum NodeScreenshotError {
  /// The tree produced no layout — either it's empty or every box
  /// collapsed to zero size before paint could run.
  NoLayout,
  /// The path didn't resolve to any layout box in the tree.
  NodeNotFound(Vec<usize>),
  /// The node's `border_rect` is zero-area (e.g. `display: none`
  /// elsewhere in the cascade or a collapsed inline). There's
  /// nothing meaningful to capture.
  EmptyRect,
  /// The off-screen render itself failed during read-back or
  /// PNG encoding.
  Render(ScreenshotError),
}

impl std::fmt::Display for NodeScreenshotError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NoLayout => write!(f, "tree produced no layout"),
      Self::NodeNotFound(p) => write!(f, "no layout box at path {p:?}"),
      Self::EmptyRect => write!(f, "node has an empty border rect"),
      Self::Render(e) => write!(f, "{e}"),
    }
  }
}

impl std::error::Error for NodeScreenshotError {}

impl From<ScreenshotError> for NodeScreenshotError {
  fn from(e: ScreenshotError) -> Self {
    Self::Render(e)
  }
}

/// Lay out `tree`, paint it, and write the contents of the node at
/// `layout_path` to `out_path` as a PNG sized exactly to that node's
/// `border_rect`. Captures correctly even when the node is fully or
/// partially outside the visible viewport, because the off-screen
/// render target is allocated at the node's pixel size and the whole
/// display list is painted into it (translated so the node sits at
/// the origin).
///
/// `viewport_w` / `viewport_h` / `scale` mirror the pre-existing
/// layout knobs — pass the same values you use for the on-screen
/// frame to avoid re-flowing the document.
///
/// ```ignore
/// // Capture the element at DOM path [1, 0, 2] (e.g. a deeply
/// // nested footer link below the fold) at full size.
/// wgpu_html::screenshot_node_to(
///     &tree, &mut text_ctx, &mut renderer,
///     &[1, 0, 2], 1280.0, 720.0, 1.0, "footer-link.png",
/// )?;
/// ```
pub fn screenshot_node_to(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  renderer: &mut Renderer,
  layout_path: &[usize],
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  out_path: impl AsRef<std::path::Path>,
) -> Result<(), NodeScreenshotError> {
  let (list, layout) = paint_tree_returning_layout(tree, text_ctx, image_cache, viewport_w, viewport_h, scale, 0.0);
  let root_layout = layout.as_ref().ok_or(NodeScreenshotError::NoLayout)?;
  let target =
    layout_at_path(root_layout, layout_path).ok_or_else(|| NodeScreenshotError::NodeNotFound(layout_path.to_vec()))?;

  let r = target.border_rect;
  if !(r.w > 0.0 && r.h > 0.0) {
    return Err(NodeScreenshotError::EmptyRect);
  }

  let region = wgpu_html_renderer::Rect::new(r.x, r.y, r.w, r.h);
  renderer.capture_rect_to(&list, region, out_path)?;
  Ok(())
}

/// Select every text run in document order.
pub fn select_all_text(tree: &mut Tree, layout: &LayoutBox) -> bool {
  let Some(anchor) = first_text_cursor(layout) else {
    tree.clear_selection();
    return false;
  };
  let Some(focus) = last_text_cursor(layout) else {
    tree.clear_selection();
    return false;
  };
  tree.interaction.selection = Some(TextSelection { anchor, focus });
  tree.interaction.selecting_text = false;
  true
}

/// Return the currently selected visible text, if any.
pub fn selected_text(tree: &Tree, layout: &LayoutBox) -> Option<String> {
  let selection = tree.interaction.selection.as_ref()?;
  selected_text_for_selection(layout, selection)
}

/// Select the word-like token at a text cursor.
///
/// Word characters (`char::is_alphanumeric` and `_`) group together;
/// whitespace groups together; punctuation groups by exact character.
/// Returns `false` if the cursor does not point into a selectable text
/// run.
pub fn select_word_at_cursor(tree: &mut Tree, layout: &LayoutBox, cursor: &TextCursor) -> bool {
  let Some(text_box) = layout_at_path(layout, &cursor.path) else {
    return false;
  };
  if text_box.text_unselectable || text_box.user_select == UserSelect::None {
    return false;
  }
  let Some(run) = text_box.text_run.as_ref() else {
    return false;
  };
  let Some((start, end)) = word_boundaries_for_cursor(run, cursor.glyph_index) else {
    return false;
  };
  tree.interaction.selection = Some(TextSelection {
    anchor: TextCursor {
      path: cursor.path.clone(),
      glyph_index: start,
    },
    focus: TextCursor {
      path: cursor.path.clone(),
      glyph_index: end,
    },
  });
  tree.interaction.selecting_text = false;
  true
}

/// Select the shaped line at a text cursor.
///
/// This is the document-level equivalent of browser triple-click line
/// selection. If the run has no line metadata, the whole run is
/// selected.
pub fn select_line_at_cursor(tree: &mut Tree, layout: &LayoutBox, cursor: &TextCursor) -> bool {
  let Some(text_box) = layout_at_path(layout, &cursor.path) else {
    return false;
  };
  if text_box.text_unselectable || text_box.user_select == UserSelect::None {
    return false;
  }
  let Some(run) = text_box.text_run.as_ref() else {
    return false;
  };
  if run.text.is_empty() && run.glyphs.is_empty() {
    return false;
  }
  let (start, end) = line_boundaries_for_cursor(run, cursor.glyph_index);
  tree.interaction.selection = Some(TextSelection {
    anchor: TextCursor {
      path: cursor.path.clone(),
      glyph_index: start,
    },
    focus: TextCursor {
      path: cursor.path.clone(),
      glyph_index: end,
    },
  });
  tree.interaction.selecting_text = false;
  true
}

fn selected_text_for_selection(layout: &LayoutBox, selection: &TextSelection) -> Option<String> {
  if selection.is_collapsed() {
    return None;
  }
  let (start, end) = ordered_cursors(&selection.anchor, &selection.focus);
  let mut out = String::new();
  let mut prev_parent: Option<Vec<usize>> = None;
  let mut path = Vec::new();
  collect_selected_text(layout, &mut path, start, end, &mut prev_parent, &mut out);
  (!out.is_empty()).then_some(out)
}

fn first_text_cursor(layout: &LayoutBox) -> Option<TextCursor> {
  let mut path = Vec::new();
  first_text_cursor_inner(layout, &mut path)
}

fn first_text_cursor_inner(layout: &LayoutBox, path: &mut Vec<usize>) -> Option<TextCursor> {
  if !layout.text_unselectable && layout.user_select != UserSelect::None {
    if let Some(run) = &layout.text_run {
      if !run.text.is_empty() || !run.glyphs.is_empty() {
        return Some(TextCursor {
          path: path.clone(),
          glyph_index: 0,
        });
      }
    }
  }
  for (i, child) in layout.children.iter().enumerate() {
    path.push(i);
    let hit = first_text_cursor_inner(child, path);
    path.pop();
    if hit.is_some() {
      return hit;
    }
  }
  None
}

fn last_text_cursor(layout: &LayoutBox) -> Option<TextCursor> {
  let mut path = Vec::new();
  last_text_cursor_inner(layout, &mut path)
}

fn last_text_cursor_inner(layout: &LayoutBox, path: &mut Vec<usize>) -> Option<TextCursor> {
  for (i, child) in layout.children.iter().enumerate().rev() {
    path.push(i);
    let hit = last_text_cursor_inner(child, path);
    path.pop();
    if hit.is_some() {
      return hit;
    }
  }
  if layout.text_unselectable || layout.user_select == UserSelect::None {
    return None;
  }
  let run = layout.text_run.as_ref()?;
  (!run.text.is_empty() || !run.glyphs.is_empty()).then(|| TextCursor {
    path: path.clone(),
    glyph_index: run.char_count(),
  })
}

fn collect_selected_text(
  layout: &LayoutBox,
  path: &mut Vec<usize>,
  start: &TextCursor,
  end: &TextCursor,
  prev_parent: &mut Option<Vec<usize>>,
  out: &mut String,
) {
  if let Some(run) = &layout.text_run {
    if !layout.text_unselectable
      && layout.user_select != UserSelect::None
      && !path_less(path, &start.path)
      && !path_less(&end.path, path)
    {
      let from = if path.as_slice() == start.path.as_slice() {
        run.byte_offset_for_boundary(start.glyph_index)
      } else {
        0
      };
      let to = if path.as_slice() == end.path.as_slice() {
        run.byte_offset_for_boundary(end.glyph_index)
      } else {
        run.text.len()
      };
      if to > from && to <= run.text.len() {
        let fragment = &run.text[from..to];
        if !fragment.is_empty() {
          let parent = path[..path.len().saturating_sub(1)].to_vec();
          if !out.is_empty() && prev_parent.as_deref() != Some(parent.as_slice()) {
            out.push('\n');
          }
          out.push_str(fragment);
          *prev_parent = Some(parent);
        }
      }
    }
  }

  for (i, child) in layout.children.iter().enumerate() {
    path.push(i);
    collect_selected_text(child, path, start, end, prev_parent, out);
    path.pop();
  }
}

fn ordered_cursors<'a>(a: &'a TextCursor, b: &'a TextCursor) -> (&'a TextCursor, &'a TextCursor) {
  if cursor_leq(a, b) { (a, b) } else { (b, a) }
}

fn cursor_leq(a: &TextCursor, b: &TextCursor) -> bool {
  if a.path == b.path {
    a.glyph_index <= b.glyph_index
  } else {
    path_less(&a.path, &b.path)
  }
}

fn path_less(a: &[usize], b: &[usize]) -> bool {
  a.cmp(b).is_lt()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextTokenKind {
  Word,
  Whitespace,
  Punctuation(char),
}

fn word_boundaries_for_cursor(run: &wgpu_html_text::ShapedRun, glyph_index: usize) -> Option<(usize, usize)> {
  let chars: Vec<(usize, usize, char)> = run
    .text
    .char_indices()
    .map(|(start, ch)| (start, start + ch.len_utf8(), ch))
    .collect();
  if chars.is_empty() {
    return None;
  }

  let boundary = glyph_index.min(chars.len());
  let mut char_idx = if boundary == 0 { 0 } else { boundary - 1 };
  if char_idx >= chars.len() {
    char_idx = chars.len() - 1;
  }
  let kind = token_kind(chars[char_idx].2);

  let mut start = char_idx;
  while start > 0 && token_kind(chars[start - 1].2) == kind {
    start -= 1;
  }
  let mut end = char_idx + 1;
  while end < chars.len() && token_kind(chars[end].2) == kind {
    end += 1;
  }

  Some((
    boundary_index_for_byte(run, chars[start].0),
    boundary_index_for_byte(run, chars[end - 1].1),
  ))
}

fn token_kind(ch: char) -> TextTokenKind {
  if ch.is_alphanumeric() || ch == '_' {
    TextTokenKind::Word
  } else if ch.is_whitespace() {
    TextTokenKind::Whitespace
  } else {
    TextTokenKind::Punctuation(ch)
  }
}

fn line_boundaries_for_cursor(run: &wgpu_html_text::ShapedRun, glyph_index: usize) -> (usize, usize) {
  let char_count = run.char_count();
  if run.lines.is_empty() {
    return (0, char_count);
  }
  // glyph_index is a char position; convert to glyph index to find
  // which line the cursor is on.
  let char_idx = glyph_index.min(char_count);
  let glyph_idx = run.char_to_glyph_index(char_idx).min(run.glyphs.len());
  for (line_idx, line) in run.lines.iter().enumerate() {
    let is_last = line_idx + 1 == run.lines.len();
    if glyph_idx >= line.glyph_range.0 && (glyph_idx < line.glyph_range.1 || is_last) {
      // Convert glyph range back to char positions.
      let start_char = run.glyph_to_char_index(line.glyph_range.0);
      let end_char = if line.glyph_range.1 >= run.glyphs.len() {
        char_count
      } else {
        run.glyph_to_char_index(line.glyph_range.1)
      };
      return (start_char, end_char);
    }
  }
  run
    .lines
    .last()
    .map(|line| {
      let start_char = run.glyph_to_char_index(line.glyph_range.0);
      let end_char = if line.glyph_range.1 >= run.glyphs.len() {
        char_count
      } else {
        run.glyph_to_char_index(line.glyph_range.1)
      };
      (start_char, end_char)
    })
    .unwrap_or((0, char_count))
}

fn boundary_index_for_byte(run: &wgpu_html_text::ShapedRun, byte: usize) -> usize {
  run
    .byte_boundaries
    .iter()
    .position(|b| *b >= byte)
    .unwrap_or_else(|| run.byte_boundaries.len().saturating_sub(1))
}


