//! Convert a laid-out box tree into a backend-agnostic display list.
//!
//! M4: walks `wgpu_html_layout::LayoutBox` and emits one solid quad per
//! box with a resolved background color.

use std::collections::BTreeMap;

use wgpu_html_layout::{LayoutBox, Resize, UserSelect};
use wgpu_html_models::common::css_enums::Overflow;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_text::TextContext;
use wgpu_html_tree::{ScrollOffset, SelectionColors, TextCursor, TextSelection, Tree};

const OVERFLOW_VISIBLE_EXTENT: f32 = 1_000_000.0;
const SCROLLBAR_MIN_THUMB: f32 = 18.0;
fn apply_opacity(mut color: wgpu_html_renderer::Color, opacity: f32) -> wgpu_html_renderer::Color {
  color[3] *= opacity.clamp(0.0, 1.0);
  color
}

/// Convenience: cascade `tree` against any embedded `<style>` blocks,
/// lay it out at `(viewport_w × viewport_h)`, and paint the result into
/// a fresh display list. No text rendering — text leaves contribute
/// zero size. Use [`paint_tree_with_text`] when fonts are registered.
pub fn paint_tree(tree: &Tree, viewport_w: f32, viewport_h: f32) -> DisplayList {
  let mut ctx = TextContext::new(64);
  let mut image_cache = wgpu_html_layout::ImageCache::default();
  paint_tree_with_text(tree, &mut ctx, &mut image_cache, viewport_w, viewport_h, 1.0, 0.0)
}

/// Cascade + lay out + paint, threading a long-lived `TextContext`
/// through. Syncs the context's font db against `tree.fonts` first so
/// any newly-registered face is loaded before shaping.
///
/// `scale` is the CSS-px → physical-px factor (winit's `scale_factor`).
pub fn paint_tree_with_text(
  tree: &Tree,
  text_ctx: &mut TextContext,
  image_cache: &mut wgpu_html_layout::ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  viewport_scroll_y: f32,
) -> DisplayList {
  text_ctx.sync_fonts(&tree.fonts);
  if let Some(ttl) = tree.asset_cache_ttl {
    image_cache.set_ttl(ttl);
  }
  for url in &tree.preload_queue {
    image_cache.preload(url);
  }
  let scale = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
  let media =
    wgpu_html_style::MediaContext::screen((viewport_w / scale).max(0.0), (viewport_h / scale).max(0.0), scale);
  let cascaded = wgpu_html_style::cascade_with_media(tree, &media);
  let mut list = DisplayList::new();
  if let Some(root) =
    wgpu_html_layout::layout_with_text(&cascaded, text_ctx, image_cache, viewport_w, viewport_h, scale)
  {
    let mut clip_stack: Vec<ClipFrame> = Vec::new();
    let mut path = Vec::new();
    paint_box_in_clip(
      &root,
      &mut list,
      &mut clip_stack,
      &mut path,
      tree.interaction.selection.as_ref(),
      tree.interaction.selection_colors,
      &tree.interaction.scroll_offsets,
      0.0,
      0.0,
      viewport_scroll_y,
      1.0,
      None,
    );
  }
  list.finalize();
  list
}

/// One frame of the active clip stack: the rectangular scissor plus
/// optional rounded corner radii at the rect's corners (TL / TR /
/// BR / BL, matching CSS shorthand order). All-zero radii arrays
/// represent a plain rectangular clip.
#[derive(Debug, Clone, Copy)]
struct ClipFrame {
  rect: Rect,
  radii_h: [f32; 4],
  radii_v: [f32; 4],
}

/// Walk a laid-out tree, pushing one quad per styled background.
pub fn paint_layout(root: &LayoutBox, list: &mut DisplayList) {
  paint_layout_with_selection(root, list, None, SelectionColors::default(), 0.0);
  list.finalize();
}

/// Paint a precomputed layout while optionally applying an active text
/// selection highlight.
pub fn paint_layout_with_selection(
  root: &LayoutBox,
  list: &mut DisplayList,
  selection: Option<&TextSelection>,
  selection_colors: SelectionColors,
  viewport_scroll_y: f32,
) {
  let mut clip_stack: Vec<ClipFrame> = Vec::new();
  let mut path = Vec::new();
  let scroll_offsets = BTreeMap::new();
  paint_box_in_clip(
    root,
    list,
    &mut clip_stack,
    &mut path,
    selection,
    selection_colors,
    &scroll_offsets,
    0.0,
    0.0,
    viewport_scroll_y,
    1.0,
    None,
  );
}

/// Paint a precomputed layout while applying document interaction
/// state such as text selection and per-element scroll offsets.
pub fn paint_layout_with_interaction(
  root: &LayoutBox,
  list: &mut DisplayList,
  selection: Option<&TextSelection>,
  selection_colors: SelectionColors,
  scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  viewport_scroll_y: f32,
) {
  paint_layout_full(
    root,
    list,
    selection,
    selection_colors,
    scroll_offsets,
    viewport_scroll_y,
    None,
  );
}

/// Paint with full interaction state including the text editing caret.
pub fn paint_layout_full(
  root: &LayoutBox,
  list: &mut DisplayList,
  selection: Option<&TextSelection>,
  selection_colors: SelectionColors,
  scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  viewport_scroll_y: f32,
  edit_caret: Option<&EditCaretInfo<'_>>,
) {
  let mut clip_stack: Vec<ClipFrame> = Vec::new();
  let mut path = Vec::new();
  paint_box_in_clip(
    root,
    list,
    &mut clip_stack,
    &mut path,
    selection,
    selection_colors,
    scroll_offsets,
    0.0,
    0.0,
    viewport_scroll_y,
    1.0,
    edit_caret,
  );
}

/// Paint state for a text editing caret inside a focused form control.
pub struct EditCaretInfo<'a> {
  /// The layout path of the focused form control.
  pub focus_path: &'a [usize],
  /// Byte offset of the caret in the value string.
  pub cursor_byte: usize,
  /// Selection range `(start_byte, end_byte)` when a selection
  /// exists, or `None` for a collapsed caret.
  pub selection_bytes: Option<(usize, usize)>,
  /// Whether the caret should be visible this frame (blink phase).
  pub caret_visible: bool,
}

/// Compute the padding-box rect of a layout box. CSS-2.2 §11.1.1
/// specifies that `overflow: hidden` clips at the *padding* edge —
/// children may extend over the border but not past the padding.
fn padding_box(b: &LayoutBox) -> Rect {
  let br = b.border_rect;
  Rect::new(
    br.x + b.border.left,
    br.y + b.border.top,
    (br.w - b.border.horizontal()).max(0.0),
    (br.h - b.border.vertical()).max(0.0),
  )
}

/// Compute the rounded-corner radii at the *padding-box* edge —
/// shrink each component of `b.border_radius` by the matching side
/// border thickness so the inner rounded shape stays concentric with
/// the outer one. Mirrors `inset_radii` in `wgpu-html-layout`.
fn padding_box_radii(b: &LayoutBox) -> ([f32; 4], [f32; 4]) {
  let outer_h = [
    b.border_radius.top_left.h,
    b.border_radius.top_right.h,
    b.border_radius.bottom_right.h,
    b.border_radius.bottom_left.h,
  ];
  let outer_v = [
    b.border_radius.top_left.v,
    b.border_radius.top_right.v,
    b.border_radius.bottom_right.v,
    b.border_radius.bottom_left.v,
  ];
  // Per-corner shrink: TL by left+top, TR by right+top, BR by
  // right+bottom, BL by left+bottom.
  let h = [
    (outer_h[0] - b.border.left).max(0.0),
    (outer_h[1] - b.border.right).max(0.0),
    (outer_h[2] - b.border.right).max(0.0),
    (outer_h[3] - b.border.left).max(0.0),
  ];
  let v = [
    (outer_v[0] - b.border.top).max(0.0),
    (outer_v[1] - b.border.top).max(0.0),
    (outer_v[2] - b.border.bottom).max(0.0),
    (outer_v[3] - b.border.bottom).max(0.0),
  ];
  (h, v)
}

/// Intersect two scissor rects, clamping the result to non-negative
/// width / height. Used to nest clips correctly: a child's effective
/// clip is the intersection of its own padding-box rect with every
/// enclosing parent clip.
fn intersect_rects(a: Rect, b: Rect) -> Rect {
  let x0 = a.x.max(b.x);
  let y0 = a.y.max(b.y);
  let x1 = (a.x + a.w).min(b.x + b.w);
  let y1 = (a.y + a.h).min(b.y + b.h);
  Rect::new(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

fn paint_box_in_clip(
  b: &LayoutBox,
  out: &mut DisplayList,
  clip_stack: &mut Vec<ClipFrame>,
  path: &mut Vec<usize>,
  selection: Option<&TextSelection>,
  selection_colors: SelectionColors,
  scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>,
  paint_offset_x: f32,
  paint_offset_y: f32,
  viewport_scroll_y: f32,
  parent_opacity: f32,
  edit_caret: Option<&EditCaretInfo<'_>>,
) {
  let paint_offset_x = if b.is_fixed { 0.0 } else { paint_offset_x };
  let paint_offset_y = if b.is_fixed {
    paint_offset_y + viewport_scroll_y
  } else {
    paint_offset_y
  };
  let opacity = (parent_opacity * b.opacity).clamp(0.0, 1.0);
  let rect = to_renderer_rect_xy(b.border_rect, paint_offset_x, paint_offset_y);
  let (rh, rv) = corner_radii(b);
  let rounded = has_any_radius(&rh) || has_any_radius(&rv);

  // Background paints into the rectangle picked by `background-clip`
  // (border-box by default; padding-box / content-box also supported).
  if let Some(color) = b.background {
    let color = apply_opacity(color, opacity);
    let bg = to_renderer_rect_xy(b.background_rect, paint_offset_x, paint_offset_y);
    if bg.w > 0.0 && bg.h > 0.0 {
      let (bg_h, bg_v) = corner_radii_from(&b.background_radii);
      if has_any_radius(&bg_h) || has_any_radius(&bg_v) {
        out.push_quad_rounded_ellipse(bg, color, bg_h, bg_v);
      } else {
        out.push_quad(bg, color);
      }
    }
  }

  // CSS `background-image`. Layout pre-computed every tile rect and
  // already filtered to those overlapping `background_rect`; we just
  // dispatch them to the renderer. When the background area has
  // rounded corners, push a temporary rounded clip range so the
  // image fragment shader's SDF discard cuts the tiles to the
  // rounded shape — otherwise the rectangular tiles would paint
  // outside the rounded background.
  if let Some(ref bgi) = b.background_image {
    let bg = to_renderer_rect_xy(b.background_rect, paint_offset_x, paint_offset_y);
    if bg.w > 0.0 && bg.h > 0.0 && !bgi.tiles.is_empty() {
      let (bg_h, bg_v) = corner_radii_from(&b.background_radii);
      let needs_round_clip = has_any_radius(&bg_h) || has_any_radius(&bg_v);
      if needs_round_clip {
        out.push_clip(Some(bg), bg_h, bg_v);
      }
      for tile in &bgi.tiles {
        let r = Rect::new(tile.x + paint_offset_x, tile.y + paint_offset_y, tile.w, tile.h);
        if r.w > 0.0 && r.h > 0.0 {
          out.push_image_with_opacity(r, bgi.image_id, bgi.data.clone(), bgi.width, bgi.height, opacity);
        }
      }
      if needs_round_clip {
        let parent = clip_stack.last().copied();
        out.pop_clip(
          parent.map(|f| f.rect),
          parent.map(|f| f.radii_h).unwrap_or([0.0; 4]),
          parent.map(|f| f.radii_v).unwrap_or([0.0; 4]),
        );
      }
    }
  }

  // Borders: when all sides share the same solid colour we paint
  // the whole ring as a single stroked quad — no GPU rasterisation
  // seams between edges, works for both sharp and rounded corners.
  // Mixed colours / styles still fall back to per-side quads.
  if let Some(color) = uniform_border_color(b) {
    let color = apply_opacity(color, opacity);
    let stroke = [b.border.top, b.border.right, b.border.bottom, b.border.left];
    if stroke.iter().any(|s| *s > 0.0) {
      out.push_quad_stroke_ellipse(rect, color, rh, rv, stroke);
    }
  } else if rounded {
    paint_rounded_per_side_borders(b, rect, rh, rv, opacity, out);
  } else {
    paint_border_edges(b, out, paint_offset_x, paint_offset_y, opacity);
  }

  // Image: emit one image quad covering the content rect.
  if let Some(ref img) = b.image {
    let cr = b.content_rect;
    if cr.w > 0.0 && cr.h > 0.0 {
      out.push_image_with_opacity(
        Rect::new(cr.x + paint_offset_x, cr.y + paint_offset_y, cr.w, cr.h),
        img.image_id,
        img.data.clone(),
        img.width,
        img.height,
        opacity,
      );
    }
  }

  // Text leaves: emit one glyph quad per shaped glyph, positioned
  // relative to the text box's content origin. Glyph UVs were
  // computed at shaping time; the renderer samples its single R8
  // atlas with them.
  if let Some(run) = &b.text_run {
    let color = apply_opacity(b.text_color.unwrap_or([0.0, 0.0, 0.0, 1.0]), opacity);
    let mut origin = b.content_rect;
    origin.x += paint_offset_x;
    origin.y += paint_offset_y;
    // Form control text (placeholders + typed values) is excluded
    // from document-level drag-to-select, matching browser behavior.
    let selected_range = if b.text_unselectable || b.user_select == UserSelect::None {
      None
    } else {
      selection_range_for_path(selection, path, run)
    };

    // Decorations sit relative to the run's baseline, behind the
    // glyphs (under-line / line-through draw under the strokes;
    // over-line above). Stroke thickness scales with the font:
    // ascent / 12 keeps it ~1px at 12px text and ~2.7px at 32px.
    if !b.text_decorations.is_empty() && run.width > 0.0 && run.ascent > 0.0 {
      let baseline_y = origin.y + run.ascent;
      let thickness = (run.ascent / 12.0).max(1.0);
      for line in &b.text_decorations {
        let y = match line {
          wgpu_html_layout::TextDecorationLine::Underline => baseline_y + thickness,
          wgpu_html_layout::TextDecorationLine::LineThrough => baseline_y - run.ascent * 0.30,
          wgpu_html_layout::TextDecorationLine::Overline => origin.y,
        };
        out.push_quad(Rect::new(origin.x, y, run.width, thickness), color);
      }
    }

    if let Some((start, end)) = selected_range {
      paint_selection_background(
        run,
        origin,
        start,
        end,
        apply_opacity(selection_colors.background, opacity),
        out,
      );
    }

    // Edit-selection glyph range (form-control-internal selection).
    let edit_sel_glyph_range: Option<(usize, usize)> = edit_caret
      .and_then(|c| c.selection_bytes)
      .filter(|_| edit_caret.is_some_and(|c| path.as_slice() == c.focus_path))
      .map(|(sb, eb)| (byte_offset_to_glyph_index(run, sb), byte_offset_to_glyph_index(run, eb)));

    // Right edge of the text box — glyphs past this are clipped.
    // Without this, when a flex item shrinks below its text content
    // width, overflowing glyphs bleed into adjacent items.
    let box_left = origin.x;
    let box_right = origin.x + origin.w;

    for (idx, g) in run.glyphs.iter().enumerate() {
      let mut glyph_x = origin.x + g.x;
      let mut glyph_w = g.w;
      let mut uv_min = g.uv_min;
      let mut uv_max = g.uv_max;

      // Clip left: glyph starts before the text box origin.
      if glyph_x < box_left {
        let clip = box_left - glyph_x;
        if clip >= glyph_w {
          continue;
        }
        let frac = clip / glyph_w;
        let uv_range_x = uv_max[0] - uv_min[0];
        glyph_x = box_left;
        glyph_w -= clip;
        uv_min[0] += uv_range_x * frac;
      }

      // Clip right: glyph extends past the text box edge.
      let overflow = (glyph_x + glyph_w - box_right).max(0.0);
      if overflow >= glyph_w {
        continue;
      }
      if overflow > 0.0 {
        let keep_frac = (glyph_w - overflow) / glyph_w;
        let uv_range_x = uv_max[0] - uv_min[0];
        glyph_w -= overflow;
        uv_max[0] = uv_min[0] + uv_range_x * keep_frac;
      }

      // Per-glyph color: each glyph carries its source span's
      // resolved foreground (set at shape time). The per-leaf
      // `text_color` on the box stays around as a hint for
      // decorations / fallbacks but glyphs paint at `g.color`.
      let glyph_color = if selected_range.is_some_and(|(start, end)| idx >= start && idx < end) {
        selection_colors.foreground
      } else if edit_sel_glyph_range.is_some_and(|(s, e)| idx >= s && idx < e) {
        selection_colors.foreground
      } else {
        g.color
      };
      out.push_glyph(
        Rect::new(glyph_x, origin.y + g.y, glyph_w, g.h),
        apply_opacity(glyph_color, opacity),
        uv_min,
        uv_max,
      );
    }

    // Edit selection highlight + caret rendering for the
    // focused form control's text run.
    if let Some(caret) = edit_caret {
      // Selection highlight: paint background behind selected range.
      if let Some((sel_start, sel_end)) = caret.selection_bytes {
        if path.as_slice() == caret.focus_path {
          let start_g = byte_offset_to_glyph_index(run, sel_start);
          let end_g = byte_offset_to_glyph_index(run, sel_end);
          if start_g < end_g {
            paint_selection_background(
              run,
              origin,
              start_g,
              end_g,
              apply_opacity(selection_colors.background, opacity),
              out,
            );
          }
        }
      }
      // Caret: thin vertical bar at cursor byte offset.
      if caret.caret_visible && path.as_slice() == caret.focus_path {
        let caret_glyph_idx = byte_offset_to_glyph_index(run, caret.cursor_byte);
        let caret_x = if caret_glyph_idx == 0 {
          0.0
        } else if caret_glyph_idx <= run.glyphs.len() {
          let g = &run.glyphs[caret_glyph_idx - 1];
          g.x + g.w
        } else {
          run.width
        };
        let (caret_y, caret_h) = run
          .lines
          .iter()
          .find(|l| caret_glyph_idx >= l.glyph_range.0 && caret_glyph_idx <= l.glyph_range.1)
          .map(|l| (l.top, l.height))
          .or_else(|| run.lines.last().map(|l| (l.top, l.height)))
          .unwrap_or((0.0, run.height.max(16.0)));
        let caret_color = apply_opacity(color, opacity);
        out.push_quad(
          Rect::new(origin.x + caret_x, origin.y + caret_y, 1.5, caret_h),
          caret_color,
        );
      }
    }
  }

  // Non-visible overflow clips children to the box's padding-box
  // rect on the resolved axis. When both axes clip, rounded
  // overflow uses the rounded inner-padding edge if the box carries
  // a `border-radius`. The decoration quads
  // emitted above (background + borders) stay outside the clip —
  // they belong to the box itself, not to its children.
  //
  // Nesting rules:
  // - The rectangular scissor is the *intersection* of every ancestor's clip rect — that's the cheap pre-discard.
  // - The rounded SDF discard only honours the *innermost* clip's radii. We don't try to compose multiple rounded
  //   shapes; browsers don't either when nesting `overflow: hidden` containers with rounded corners.
  let clips_children = b.overflow.clips_any();
  let pushed = if clips_children {
    let pad = shift_rect_xy(padding_box(b), paint_offset_x, paint_offset_y);
    let effective_rect = overflow_clip_rect(b, pad, clip_stack.last().copied());
    let (inner_h, inner_v) = if b.overflow.clips_both() {
      padding_box_radii(b)
    } else {
      ([0.0; 4], [0.0; 4])
    };
    let frame = ClipFrame {
      rect: effective_rect,
      radii_h: inner_h,
      radii_v: inner_v,
    };
    clip_stack.push(frame);
    out.push_clip(Some(effective_rect), inner_h, inner_v);
    true
  } else {
    false
  };

  let scroll_x = element_scroll_x(b, path, scroll_offsets);
  let scroll_y = element_scroll_y(b, path, scroll_offsets);
  let child_offset_x = paint_offset_x - scroll_x;
  let child_offset_y = paint_offset_y - scroll_y;

  // Sort children by CSS z-index for paint order.
  let mut child_order: Vec<usize> = (0..b.children.len()).collect();
  let has_positioned = b.children.iter().any(|c| c.z_index.is_some());
  if has_positioned {
    child_order.sort_by_key(|&i| z_index_sort_key(&b.children[i]));
  }

  for &i in &child_order {
    let child = &b.children[i];
    path.push(i);
    paint_box_in_clip(
      child,
      out,
      clip_stack,
      path,
      selection,
      selection_colors,
      scroll_offsets,
      child_offset_x,
      child_offset_y,
      viewport_scroll_y,
      opacity,
      edit_caret,
    );
    path.pop();
  }

  // Resize handle (CSS `resize`). Paint after children so it sits
  // on top of all content in the bottom-right corner of the
  // padding box. The handle is only shown when overflow is non-visible
  // (the CSS spec requires `overflow` to be `scroll`, `auto`, or
  // `hidden` for `resize` to take effect).
  {
    let active = b.resize != Resize::None && (b.overflow.x != Overflow::Visible || b.overflow.y != Overflow::Visible);
    if active {
      paint_resize_handle(b, out, paint_offset_x, paint_offset_y);
    }
  }

  if pushed {
    clip_stack.pop();
    let parent = clip_stack.last().copied();
    out.pop_clip(
      parent.map(|f| f.rect),
      parent.map(|f| f.radii_h).unwrap_or([0.0; 4]),
      parent.map(|f| f.radii_v).unwrap_or([0.0; 4]),
    );
  }

  paint_scrollbars(b, out, paint_offset_x, paint_offset_y, scroll_x, scroll_y, opacity, path);
}

/// Paint the CSS resize handle (three diagonal lines) in the
/// bottom-right corner of the element's padding box.
fn paint_resize_handle(b: &LayoutBox, out: &mut DisplayList, paint_offset_x: f32, paint_offset_y: f32) {
  let pad = padding_box(b);
  let handle_size = 16.0_f32;
  let x = pad.x + pad.w - handle_size - 2.0 + paint_offset_x;
  let y = pad.y + pad.h - handle_size - 2.0 + paint_offset_y;

  if x < pad.x + paint_offset_x || y < pad.y + paint_offset_y || handle_size <= 0.0 {
    return;
  }

  let color = [0.6, 0.6, 0.6, 0.6_f32];
  let line_len = handle_size - 2.0;
  let thickness = 2.0;
  let gap = 5.0;

  for i in 0..3 {
    let offset = i as f32 * gap;
    // Diagonal lines going from bottom-left to top-right inside the handle area
    let lx = x + handle_size - line_len - offset;
    let ly = y + handle_size - thickness - offset;
    out.push_quad(
      Rect::new(lx.max(x), ly.max(y), line_len.min(handle_size), thickness),
      color,
    );
  }
}

fn selection_range_for_path(
  selection: Option<&TextSelection>,
  path: &[usize],
  run: &wgpu_html_text::ShapedRun,
) -> Option<(usize, usize)> {
  let sel = selection?;
  if sel.is_collapsed() {
    return None;
  }

  let (start, end) = ordered_cursors(&sel.anchor, &sel.focus);

  if path_less(path, &start.path) || path_less(&end.path, path) {
    return None;
  }

  let char_count = run.char_count();

  // glyph_index is a character position (cursor boundary).
  let from_char = if path == start.path.as_slice() {
    start.glyph_index.min(char_count)
  } else {
    0
  };
  let to_char = if path == end.path.as_slice() {
    end.glyph_index.min(char_count)
  } else {
    char_count
  };

  if from_char >= to_char {
    return None;
  }

  // Convert char positions to glyph indices for rendering.
  let from_glyph = run.char_to_glyph_index(from_char);
  let to_glyph = run.char_to_glyph_index(to_char);
  (from_glyph < to_glyph).then_some((from_glyph, to_glyph))
}

fn paint_selection_background(
  run: &wgpu_html_text::ShapedRun,
  origin: wgpu_html_layout::Rect,
  start: usize,
  end: usize,
  color: wgpu_html_renderer::Color,
  out: &mut DisplayList,
) {
  if run.glyphs.is_empty() || start >= end || start >= run.glyphs.len() {
    return;
  }
  let end = end.min(run.glyphs.len());

  if run.lines.is_empty() {
    // Fallback for synthetic runs that omit line metadata.
    let mut x0 = f32::INFINITY;
    let mut x1 = -f32::INFINITY;
    for g in &run.glyphs[start..end] {
      x0 = x0.min(g.x);
      x1 = x1.max(g.x + g.w);
    }
    if x1 > x0 {
      out.push_quad(Rect::new(origin.x + x0, origin.y, x1 - x0, run.height.max(1.0)), color);
    }
    return;
  }

  for line in &run.lines {
    let a = start.max(line.glyph_range.0);
    let b = end.min(line.glyph_range.1);
    if a >= b {
      continue;
    }
    let mut x0 = f32::INFINITY;
    let mut x1 = -f32::INFINITY;
    for g in &run.glyphs[a..b] {
      x0 = x0.min(g.x);
      x1 = x1.max(g.x + g.w);
    }
    if x1 <= x0 {
      continue;
    }
    out.push_quad(
      Rect::new(origin.x + x0, origin.y + line.top, x1 - x0, line.height.max(1.0)),
      color,
    );
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

fn overflow_clip_rect(b: &LayoutBox, pad: Rect, parent: Option<ClipFrame>) -> Rect {
  let local = match (b.overflow.clips_x(), b.overflow.clips_y(), parent) {
    (true, true, _) => pad,
    (true, false, Some(parent)) => Rect::new(pad.x, parent.rect.y, pad.w, parent.rect.h),
    (false, true, Some(parent)) => Rect::new(parent.rect.x, pad.y, parent.rect.w, pad.h),
    (true, false, None) => Rect::new(pad.x, -OVERFLOW_VISIBLE_EXTENT, pad.w, OVERFLOW_VISIBLE_EXTENT * 2.0),
    (false, true, None) => Rect::new(-OVERFLOW_VISIBLE_EXTENT, pad.y, OVERFLOW_VISIBLE_EXTENT * 2.0, pad.h),
    (false, false, _) => pad,
  };

  match parent {
    Some(parent) => intersect_rects(parent.rect, local),
    None => local,
  }
}

/// Sort key for CSS z-index paint order: negative z-index (-1),
/// auto / non-positioned (0), non-negative z-index (1).
/// Within each layer, sort by the z-index value.
fn z_index_sort_key(b: &LayoutBox) -> (i32, i32) {
  match b.z_index {
    Some(z) if z < 0 => (-1, z),
    Some(z) => (1, z),
    None => (0, 0),
  }
}

fn paint_scrollbars(
  b: &LayoutBox,
  out: &mut DisplayList,
  paint_offset_x: f32,
  paint_offset_y: f32,
  scroll_x: f32,
  scroll_y: f32,
  opacity: f32,
  path: &[usize],
) {
  if path.len() <= 1 {
    return;
  }
  let pad = shift_rect_xy(padding_box(b), paint_offset_x, paint_offset_y);
  if pad.w <= 0.0 || pad.h <= 0.0 {
    return;
  }
  let track_w = b.overflow.scrollbar_width.min(pad.w);
  let track_color = b.overflow.scrollbar_track.unwrap_or(crate::scroll::DEFAULT_TRACK);
  let thumb_color = b.overflow.scrollbar_thumb.unwrap_or(crate::scroll::DEFAULT_THUMB);

  let has_vbar = should_paint_vertical_scrollbar(b);
  let has_hbar = should_paint_horizontal_scrollbar(b);

  if has_vbar && track_w > 0.0 {
    let bar_h = if has_hbar { pad.h - track_w } else { pad.h };
    let track = Rect::new(pad.x + pad.w - track_w, pad.y, track_w, bar_h);
    out.push_quad(track, apply_opacity(track_color, opacity));

    let scroll_h = scrollable_content_height(b).max(pad.h);
    let max_scroll = (scroll_h - pad.h).max(0.0);
    let ratio = (bar_h / scroll_h.max(1.0)).clamp(0.0, 1.0);
    let thumb_h = (bar_h * ratio).clamp(SCROLLBAR_MIN_THUMB.min(bar_h), bar_h);
    let travel = (bar_h - thumb_h).max(0.0);
    let thumb_y = track.y + travel * (scroll_y / max_scroll.max(1.0));
    let inset = (track_w * 0.2).clamp(1.0, 3.0);
    let thumb = Rect::new(
      track.x + inset,
      thumb_y + inset,
      (track.w - inset * 2.0).max(1.0),
      (thumb_h - inset * 2.0).max(1.0),
    );
    let radius = thumb.w * 0.5;
    out.push_quad_rounded(thumb, apply_opacity(thumb_color, opacity), [radius; 4]);
  }

  if has_hbar && track_w > 0.0 {
    let bar_w = if has_vbar { pad.w - track_w } else { pad.w };
    let track = Rect::new(pad.x, pad.y + pad.h - track_w, bar_w, track_w);
    out.push_quad(track, apply_opacity(track_color, opacity));

    let scroll_w = scrollable_content_width(b).max(pad.w);
    let max_scroll = (scroll_w - pad.w).max(0.0);
    let ratio = (bar_w / scroll_w.max(1.0)).clamp(0.0, 1.0);
    let thumb_w = (bar_w * ratio).clamp(SCROLLBAR_MIN_THUMB.min(bar_w), bar_w);
    let travel = (bar_w - thumb_w).max(0.0);
    let thumb_x = track.x + travel * (scroll_x / max_scroll.max(1.0));
    let inset = (track_w * 0.2).clamp(1.0, 3.0);
    let thumb = Rect::new(
      thumb_x + inset,
      track.y + inset,
      (thumb_w - inset * 2.0).max(1.0),
      (track.h - inset * 2.0).max(1.0),
    );
    let radius = thumb.h * 0.5;
    out.push_quad_rounded(thumb, apply_opacity(thumb_color, opacity), [radius; 4]);
  }
}

fn element_scroll_y(b: &LayoutBox, path: &[usize], scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>) -> f32 {
  let max_scroll = (scrollable_content_height(b) - padding_box(b).h).max(0.0);
  scroll_offsets
    .get(path)
    .map(|s| s.y)
    .unwrap_or(0.0)
    .clamp(0.0, max_scroll)
}

fn element_scroll_x(b: &LayoutBox, path: &[usize], scroll_offsets: &BTreeMap<Vec<usize>, ScrollOffset>) -> f32 {
  let max_scroll = (scrollable_content_width(b) - padding_box(b).w).max(0.0);
  scroll_offsets
    .get(path)
    .map(|s| s.x)
    .unwrap_or(0.0)
    .clamp(0.0, max_scroll)
}

fn should_paint_vertical_scrollbar(b: &LayoutBox) -> bool {
  if b.overflow.scrollbar_width <= 0.0 {
    return false;
  }
  match b.overflow.y {
    Overflow::Scroll => true,
    Overflow::Auto => scrollable_content_height(b) > padding_box(b).h + 0.5,
    _ => false,
  }
}

fn should_paint_horizontal_scrollbar(b: &LayoutBox) -> bool {
  if b.overflow.scrollbar_width <= 0.0 {
    return false;
  }
  match b.overflow.x {
    Overflow::Scroll => true,
    Overflow::Auto => scrollable_content_width(b) > padding_box(b).w + 0.5,
    _ => false,
  }
}

fn scrollable_content_height(b: &LayoutBox) -> f32 {
  let pad = padding_box(b);
  let mut bottom = pad.y + pad.h;
  for child in &b.children {
    bottom = bottom.max(subtree_bottom(child));
  }
  (bottom - pad.y).max(0.0)
}

fn subtree_bottom(b: &LayoutBox) -> f32 {
  let mut bottom = b.margin_rect.y + b.margin_rect.h;
  for child in &b.children {
    bottom = bottom.max(subtree_bottom(child));
  }
  bottom
}

fn scrollable_content_width(b: &LayoutBox) -> f32 {
  let pad = padding_box(b);
  let mut right = pad.x + pad.w;
  for child in &b.children {
    right = right.max(subtree_right(child));
  }
  (right - pad.x).max(0.0)
}

fn subtree_right(b: &LayoutBox) -> f32 {
  let mut right = b.margin_rect.x + b.margin_rect.w;
  for child in &b.children {
    right = right.max(subtree_right(child));
  }
  right
}

/// If every set border side shares the same colour AND a renderable
/// `solid` style (or has no style set, which we treat as solid when the
/// width and colour are present), return that colour. Non-solid styles
/// like dashed/dotted force a fall-back to per-side edge segments
/// because the ring shader can only render solid strokes.
fn uniform_border_color(b: &LayoutBox) -> Option<wgpu_html_renderer::Color> {
  use wgpu_html_models::common::css_enums::BorderStyle;

  let bd = b.border;
  let bc = b.border_colors;
  let bs = &b.border_styles;
  let mut chosen: Option<wgpu_html_renderer::Color> = None;
  let pairs = [
    (bd.top, bc.top, &bs.top),
    (bd.right, bc.right, &bs.right),
    (bd.bottom, bc.bottom, &bs.bottom),
    (bd.left, bc.left, &bs.left),
  ];
  for (w, c, s) in pairs {
    if w <= 0.0 {
      continue;
    }
    match s {
      None | Some(BorderStyle::Solid) => {}
      // `none` / `hidden` skip painting entirely; treat as
      // "non-uniform" so the per-side path runs and skips them
      // individually instead of drawing a single ring of width 0.
      Some(BorderStyle::None) | Some(BorderStyle::Hidden) => return None,
      // Dashed / dotted / double / groove / ridge / inset / outset
      // can't be expressed by the SDF ring; fall back to per-side.
      Some(_) => return None,
    }
    let c = c?;
    match chosen {
      None => chosen = Some(c),
      Some(existing) if existing == c => {}
      Some(_) => return None,
    }
  }
  chosen
}

fn corner_radii(b: &LayoutBox) -> ([f32; 4], [f32; 4]) {
  corner_radii_from(&b.border_radius)
}

/// Mixed-style / mixed-colour borders on a rounded box. Each solid
/// side gets its own ring quad with stroke widths zero on the other
/// three sides — the SDF then naturally restricts the painted band to
/// just that side, with corners following the rounded path. `none` /
/// `hidden` sides are skipped. Dashed / dotted on rounded boxes still
/// emit sharp segments along that side (visible but the corner curves
/// aren't stylised).
fn paint_rounded_per_side_borders(
  b: &LayoutBox,
  rect: Rect,
  rh: [f32; 4],
  rv: [f32; 4],
  opacity: f32,
  out: &mut DisplayList,
) {
  use wgpu_html_models::common::css_enums::BorderStyle;

  let r = b.border_rect;
  let bd = b.border;
  let bc = b.border_colors;
  let bs = &b.border_styles;

  let sides: [(Side, f32, Option<wgpu_html_renderer::Color>, &Option<BorderStyle>); 4] = [
    (Side::Top, bd.top, bc.top, &bs.top),
    (Side::Right, bd.right, bc.right, &bs.right),
    (Side::Bottom, bd.bottom, bc.bottom, &bs.bottom),
    (Side::Left, bd.left, bc.left, &bs.left),
  ];

  for (side, w, color, style) in sides {
    if w <= 0.0 {
      continue;
    }
    let Some(color) = color else { continue };
    let color = apply_opacity(color, opacity);
    let kind = match style {
      None | Some(BorderStyle::Solid) => EdgeKind::Solid,
      Some(BorderStyle::None) | Some(BorderStyle::Hidden) => EdgeKind::Skip,
      Some(BorderStyle::Dashed) => EdgeKind::Dashed,
      Some(BorderStyle::Dotted) => EdgeKind::Dotted,
      // Double / Groove / Ridge / Inset / Outset → solid for now.
      Some(_) => EdgeKind::Solid,
    };
    match kind {
      EdgeKind::Skip => {}
      EdgeKind::Solid => {
        let stroke = side.one_sided_stroke(w);
        out.push_quad_stroke_ellipse(rect, color, rh, rv, stroke);
      }
      EdgeKind::Dashed | EdgeKind::Dotted => {
        // If every corner is uniform-circular, the shader can
        // dash along the rounded path itself. Otherwise fall
        // back to straight dashed segments along the side's
        // straight portion (corner curves stay bare — better
        // than nothing while elliptical arc-length isn't yet
        // implemented).
        let radii = &b.border_radius;
        if uniform_circular_radius(radii).is_some() {
          let stroke = side.one_sided_stroke(w);
          let (dash, gap) = match kind {
            EdgeKind::Dashed => ((w * 3.0).max(2.0), w.max(1.0)),
            EdgeKind::Dotted => (w.max(1.0), w.max(1.0)),
            _ => (0.0, 0.0),
          };
          let pattern = [
            match kind {
              EdgeKind::Dashed => 1.0,
              EdgeKind::Dotted => 2.0,
              _ => 0.0,
            },
            dash,
            gap,
            0.0,
          ];
          out.push_quad_stroke_patterned(rect, color, rh, rv, stroke, pattern);
        } else {
          let edge_rect = shift_rect_xy(side.edge_rect_rounded(r, bd, radii), rect.x - r.x, rect.y - r.y);
          let axis = side.axis();
          paint_edge(edge_rect, axis, w, kind, color, out);
        }
      }
    }
  }
}

#[derive(Copy, Clone)]
enum Side {
  Top,
  Right,
  Bottom,
  Left,
}

impl Side {
  fn one_sided_stroke(self, w: f32) -> [f32; 4] {
    // Order matches the shader: top, right, bottom, left.
    match self {
      Side::Top => [w, 0.0, 0.0, 0.0],
      Side::Right => [0.0, w, 0.0, 0.0],
      Side::Bottom => [0.0, 0.0, w, 0.0],
      Side::Left => [0.0, 0.0, 0.0, w],
    }
  }

  fn axis(self) -> Axis {
    match self {
      Side::Top | Side::Bottom => Axis::Horizontal,
      Side::Left | Side::Right => Axis::Vertical,
    }
  }

  #[allow(dead_code)]
  fn edge_rect(self, r: wgpu_html_layout::Rect, bd: wgpu_html_layout::Insets) -> Rect {
    let inner_h = (r.h - bd.top - bd.bottom).max(0.0);
    match self {
      Side::Top => Rect::new(r.x, r.y, r.w, bd.top),
      Side::Bottom => Rect::new(r.x, r.y + r.h - bd.bottom, r.w, bd.bottom),
      Side::Left => Rect::new(r.x, r.y + bd.top, bd.left, inner_h),
      Side::Right => Rect::new(r.x + r.w - bd.right, r.y + bd.top, bd.right, inner_h),
    }
  }

  /// Same as [`Self::edge_rect`] but on a rounded box: the strip is
  /// clamped to the *straight* portion of the side, between the two
  /// adjacent corner radii. With zero radii this collapses to the
  /// regular corner-owning rectangle, so it's safe for the rounded
  /// path even when only some corners are rounded.
  fn edge_rect_rounded(
    self,
    r: wgpu_html_layout::Rect,
    bd: wgpu_html_layout::Insets,
    radii: &wgpu_html_layout::CornerRadii,
  ) -> Rect {
    match self {
      Side::Top => {
        let x_start = radii.top_left.h;
        let x_end = (r.w - radii.top_right.h).max(x_start);
        Rect::new(r.x + x_start, r.y, x_end - x_start, bd.top)
      }
      Side::Bottom => {
        let x_start = radii.bottom_left.h;
        let x_end = (r.w - radii.bottom_right.h).max(x_start);
        Rect::new(r.x + x_start, r.y + r.h - bd.bottom, x_end - x_start, bd.bottom)
      }
      Side::Left => {
        let y_start = radii.top_left.v.max(bd.top);
        let y_end = (r.h - radii.bottom_left.v).max(y_start);
        Rect::new(r.x, r.y + y_start, bd.left, y_end - y_start)
      }
      Side::Right => {
        let y_start = radii.top_right.v.max(bd.top);
        let y_end = (r.h - radii.bottom_right.v).max(y_start);
        Rect::new(r.x + r.w - bd.right, r.y + y_start, bd.right, y_end - y_start)
      }
    }
  }
}

/// Returns the shared radius if every corner has the same circular
/// (h == v) radius; `None` otherwise. The dashed-along-curve shader
/// path only handles the uniform-circular case for now.
fn uniform_circular_radius(r: &wgpu_html_layout::CornerRadii) -> Option<f32> {
  let corners = [r.top_left, r.top_right, r.bottom_right, r.bottom_left];
  let target = corners[0].h;
  for c in corners {
    if (c.h - target).abs() > 1e-3 || (c.v - target).abs() > 1e-3 {
      return None;
    }
  }
  Some(target)
}

fn corner_radii_from(r: &wgpu_html_layout::CornerRadii) -> ([f32; 4], [f32; 4]) {
  (
    [r.top_left.h, r.top_right.h, r.bottom_right.h, r.bottom_left.h],
    [r.top_left.v, r.top_right.v, r.bottom_right.v, r.bottom_left.v],
  )
}

fn has_any_radius(r: &[f32; 4]) -> bool {
  r.iter().any(|v| *v > 0.0)
}

/// Emit per-side border edges for a sharp (non-rounded) box. Every side
/// is independently coloured and styled. `solid` is one full-edge quad;
/// `dashed` and `dotted` are emitted as a row of short segment quads;
/// `none` and `hidden` are skipped. Other values render as solid.
fn paint_border_edges(b: &LayoutBox, out: &mut DisplayList, paint_offset_x: f32, paint_offset_y: f32, opacity: f32) {
  use wgpu_html_models::common::css_enums::BorderStyle;

  let r = b.border_rect;
  let bd = b.border;
  if r.w <= 0.0 || r.h <= 0.0 || !b.border_colors.any() {
    return;
  }
  let bc = b.border_colors;
  let bs = &b.border_styles;

  let inner_h = (r.h - bd.top - bd.bottom).max(0.0);

  // Top edge — horizontal strip at the very top; covers the corner
  // pixels for left/right edges so corners draw exactly once.
  if bd.top > 0.0 {
    if let Some(c) = bc.top {
      let c = apply_opacity(c, opacity);
      paint_edge(
        Rect::new(r.x + paint_offset_x, r.y + paint_offset_y, r.w, bd.top),
        Axis::Horizontal,
        bd.top,
        resolve_style(&bs.top),
        c,
        out,
      );
    }
  }
  // Bottom edge.
  if bd.bottom > 0.0 {
    if let Some(c) = bc.bottom {
      let c = apply_opacity(c, opacity);
      paint_edge(
        Rect::new(
          r.x + paint_offset_x,
          r.y + paint_offset_y + r.h - bd.bottom,
          r.w,
          bd.bottom,
        ),
        Axis::Horizontal,
        bd.bottom,
        resolve_style(&bs.bottom),
        c,
        out,
      );
    }
  }
  // Left edge — sits between the top and bottom strips.
  if bd.left > 0.0 && inner_h > 0.0 {
    if let Some(c) = bc.left {
      let c = apply_opacity(c, opacity);
      paint_edge(
        Rect::new(r.x + paint_offset_x, r.y + paint_offset_y + bd.top, bd.left, inner_h),
        Axis::Vertical,
        bd.left,
        resolve_style(&bs.left),
        c,
        out,
      );
    }
  }
  // Right edge.
  if bd.right > 0.0 && inner_h > 0.0 {
    if let Some(c) = bc.right {
      let c = apply_opacity(c, opacity);
      paint_edge(
        Rect::new(
          r.x + paint_offset_x + r.w - bd.right,
          r.y + paint_offset_y + bd.top,
          bd.right,
          inner_h,
        ),
        Axis::Vertical,
        bd.right,
        resolve_style(&bs.right),
        c,
        out,
      );
    }
  }

  fn resolve_style(s: &Option<BorderStyle>) -> EdgeKind {
    match s {
      None | Some(BorderStyle::Solid) => EdgeKind::Solid,
      Some(BorderStyle::Dashed) => EdgeKind::Dashed,
      Some(BorderStyle::Dotted) => EdgeKind::Dotted,
      Some(BorderStyle::None) | Some(BorderStyle::Hidden) => EdgeKind::Skip,
      // Double / Groove / Ridge / Inset / Outset: render as solid for now.
      Some(_) => EdgeKind::Solid,
    }
  }
}

#[derive(Copy, Clone)]
enum Axis {
  Horizontal,
  Vertical,
}

#[derive(Copy, Clone)]
enum EdgeKind {
  Solid,
  Dashed,
  Dotted,
  Skip,
}

fn paint_edge(
  rect: Rect,
  axis: Axis,
  thickness: f32,
  kind: EdgeKind,
  color: wgpu_html_renderer::Color,
  out: &mut DisplayList,
) {
  match kind {
    EdgeKind::Skip => {}
    EdgeKind::Solid => {
      out.push_quad(rect, color);
    }
    EdgeKind::Dashed => {
      // CSS-style approximation: dashes are ~3 thicknesses long
      // with a 1-thickness gap, with sane minimums for very thin
      // borders.
      let dash = (thickness * 3.0).max(2.0);
      let gap = thickness.max(1.0);
      paint_segments(rect, axis, dash, gap, color, out);
    }
    EdgeKind::Dotted => {
      // Square dots with one-thickness gaps.
      let dot = thickness.max(1.0);
      let gap = thickness.max(1.0);
      paint_segments(rect, axis, dot, gap, color, out);
    }
  }
}

/// Convert a byte offset in a value string to a glyph index in the
/// shaped run. Uses the run's `byte_boundaries` to map byte positions
/// to glyph positions.
fn byte_offset_to_glyph_index(run: &wgpu_html_text::ShapedRun, byte_offset: usize) -> usize {
  if run.byte_boundaries.is_empty() {
    return 0;
  }
  // byte_boundaries[i] is the byte offset of glyph i. Find the
  // first boundary >= byte_offset.
  run
    .byte_boundaries
    .iter()
    .position(|&b| b >= byte_offset)
    .unwrap_or(run.glyphs.len())
}

/// Emit a sequence of `on`-length segments with `off`-length gaps along
/// `axis` inside `rect`. Final segment is clipped if it would overflow.
fn paint_segments(rect: Rect, axis: Axis, on: f32, off: f32, color: wgpu_html_renderer::Color, out: &mut DisplayList) {
  let stride = on + off;
  if stride <= 0.0 {
    return;
  }
  let total = match axis {
    Axis::Horizontal => rect.w,
    Axis::Vertical => rect.h,
  };
  let mut t = 0.0_f32;
  while t < total {
    let len = on.min(total - t);
    if len > 0.0 {
      let seg = match axis {
        Axis::Horizontal => Rect::new(rect.x + t, rect.y, len, rect.h),
        Axis::Vertical => Rect::new(rect.x, rect.y + t, rect.w, len),
      };
      out.push_quad(seg, color);
    }
    t += stride;
  }
}

fn to_renderer_rect_xy(r: wgpu_html_layout::Rect, dx: f32, dy: f32) -> Rect {
  Rect::new(r.x + dx, r.y + dy, r.w, r.h)
}

fn shift_rect_xy(r: Rect, dx: f32, dy: f32) -> Rect {
  Rect::new(r.x + dx, r.y + dy, r.w, r.h)
}
