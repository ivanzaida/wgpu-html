//! Shape + raster + atlas-pack pipeline.
//!
//! `TextContext` is the long-lived, per-host text state. It owns the
//! cosmic-text shaper bridge (`FontDb`), a CPU-side R8 glyph atlas
//! (`Atlas`), and cosmic-text's own `SwashCache` for glyph rasters.
//! Layout passes a `&mut TextContext` whenever it needs to measure /
//! shape a text node.
//!
//! For each shaped glyph we:
//!   1. Look up its `(font, glyph_id, sub-pixel offset, size)` cache key in our local atlas map.
//!   2. On miss, ask cosmic-text's SwashCache for the alpha mask, pack it into `Atlas`, and remember the resulting
//!      `AtlasRect`.
//!   3. Emit a `PositionedGlyph` with the run-relative pixel rect and the atlas UVs.
//!
//! The atlas's `flush_dirty` / `upload` paths are how the renderer
//! sees newly-inserted glyphs each frame.

use std::{
  collections::HashMap,
  hash::{Hash, Hasher},
};

use cosmic_text::{Attrs, Buffer, CacheKey, Metrics, Shaping};
use wgpu_html_tree::{FontHandle, FontRegistry, FontStyleAxis};

use crate::{
  atlas::{Atlas, AtlasRect},
  font_db::FontDb,
};

// ── Text measurement cache keys ─────────────────────────────────────────────

/// Cache key for a single-span `shape_and_pack` call. Excludes
/// `color` because colour doesn't affect glyph geometry or line
/// breaking — on cache hit the caller patches glyph colours.
#[derive(Clone, PartialEq, Eq, Hash)]
struct TextCacheKey {
  text_hash: u64,
  font_handle: FontHandle,
  size_px_bits: u32,
  line_height_bits: u32,
  letter_spacing_bits: u32,
  weight: u16,
  style: FontStyleAxis,
  max_width_bits: Option<u32>,
}

/// Cache key for a multi-span `shape_paragraph` call.
#[derive(Clone, PartialEq, Eq, Hash)]
struct ParagraphCacheKey {
  /// Hash of all span texts concatenated with their attributes.
  content_hash: u64,
  max_width_bits: Option<u32>,
}

fn hash_str(s: &str) -> u64 {
  let mut h = std::collections::hash_map::DefaultHasher::new();
  s.hash(&mut h);
  h.finish()
}

/// Maximum entries per text cache before a full clear.
const TEXT_CACHE_MAX: usize = 4096;

/// One glyph after shaping + atlas packing. Positions are run-relative
/// pixel coordinates with `(0, 0)` at the top-left of the line box.
/// `color` is linear RGBA — the source span's resolved foreground;
/// every glyph carries it so a mixed-attribute paragraph (one
/// span red, one span blue) emits per-glyph colour without the
/// renderer needing to know about spans.
#[derive(Debug, Clone, Copy)]
pub struct PositionedGlyph {
  pub x: f32,
  pub y: f32,
  pub w: f32,
  pub h: f32,
  pub uv_min: [f32; 2],
  pub uv_max: [f32; 2],
  pub color: [f32; 4],
}

/// One line inside a [`ShapedRun`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ShapedLine {
  pub top: f32,
  pub height: f32,
  /// Half-open glyph slice `[start, end)` in `ShapedRun::glyphs`.
  pub glyph_range: (usize, usize),
}

/// Output of `shape_and_pack`. Holds the line metrics callers need to
/// place the run vertically inside a block (height + ascent), the
/// total advance width, and the glyph quads.
#[derive(Debug, Clone, Default)]
pub struct ShapedRun {
  pub glyphs: Vec<PositionedGlyph>,
  /// For each entry in `glyphs`, the 0-based character index of that
  /// glyph in `text`. Length equals `glyphs.len()`. Empty on
  /// synthetic (test-built) runs — callers fall back to identity
  /// mapping (`glyph_idx == char_idx`) when this is empty.
  pub glyph_chars: Vec<usize>,
  pub lines: Vec<ShapedLine>,
  /// Visible text that produced this run after whitespace collapse /
  /// text-transform / rich-text flattening.
  pub text: String,
  /// UTF-8 byte offsets for every character boundary in `text`.
  /// Length is `text.chars().count() + 1`; index 0 is the start and
  /// the last entry is `text.len()`.
  pub byte_boundaries: Vec<usize>,
  pub width: f32,
  pub height: f32,
  pub ascent: f32,
}

impl ShapedRun {
  pub fn byte_offset_for_boundary(&self, glyph_index: usize) -> usize {
    if self.byte_boundaries.is_empty() {
      return 0;
    }
    let idx = glyph_index.min(self.byte_boundaries.len().saturating_sub(1));
    self.byte_boundaries[idx]
  }

  /// Total number of characters in `text` (one more than the last
  /// valid cursor position).
  pub fn char_count(&self) -> usize {
    self.byte_boundaries.len().saturating_sub(1)
  }

  /// Convert a glyph index (index into `self.glyphs`) to its
  /// corresponding character position in `self.text`.
  /// Falls back to identity when `glyph_chars` is empty (synthetic runs).
  pub fn glyph_to_char_index(&self, glyph_idx: usize) -> usize {
    if self.glyph_chars.is_empty() {
      return glyph_idx;
    }
    self.glyph_chars.get(glyph_idx).copied().unwrap_or_else(|| {
      // Past the last glyph: char after the last mapped char.
      self.glyph_chars.last().copied().map(|c| c + 1).unwrap_or(glyph_idx)
    })
  }

  /// Convert a character position (index into `byte_boundaries`) to
  /// the nearest glyph index in `self.glyphs`.
  /// Returns the index of the first rendered glyph whose char index
  /// is >= `char_idx`, or `glyphs.len()` if none qualifies.
  /// Falls back to identity when `glyph_chars` is empty.
  pub fn char_to_glyph_index(&self, char_idx: usize) -> usize {
    if self.glyph_chars.is_empty() {
      return char_idx.min(self.glyphs.len());
    }
    self
      .glyph_chars
      .iter()
      .position(|&c| c >= char_idx)
      .unwrap_or(self.glyphs.len())
  }
}

pub fn utf8_boundaries(text: &str) -> Vec<usize> {
  let mut out = Vec::with_capacity(text.chars().count() + 1);
  out.push(0);
  for (idx, ch) in text.char_indices() {
    out.push(idx + ch.len_utf8());
  }
  out
}

/// One span in a rich-text paragraph: a run of source text with
/// uniform attributes. The `leaf_id` is opaque to the shaper; the
/// caller uses it to map shaped glyphs back onto its source tree
/// (e.g. for emitting per-element backgrounds and decorations).
#[derive(Debug, Clone, Copy)]
pub struct ParagraphSpan<'a> {
  pub text: &'a str,
  pub family: &'a str,
  pub weight: u16,
  pub style: FontStyleAxis,
  pub size_px: f32,
  pub line_height_px: f32,
  pub color: [f32; 4],
  pub leaf_id: u32,
}

/// Per-line metric block for a shaped paragraph. Glyph y coordinates
/// in `ParagraphLayout::glyphs` already include line offsets — these
/// are informational, used by the caller to emit per-line
/// backgrounds, decorations, and text-align shifts.
///
/// `glyph_range` is a half-open `[start, end)` slice into
/// `ParagraphLayout::glyphs` covering this line's contribution.
/// Iterating `&para.glyphs[line.glyph_range.0..line.glyph_range.1]`
/// is how the caller applies a per-line text-align dx without
/// re-deriving line membership from glyph y coordinates.
#[derive(Debug, Clone, Copy)]
pub struct ParagraphLine {
  pub top: f32,
  pub baseline: f32,
  pub height: f32,
  pub line_width: f32,
  pub glyph_range: (usize, usize),
}

/// One contiguous run of advance space a single source span occupies
/// on a given line. A span that wraps across a line break produces
/// multiple segments — one per line.
#[derive(Debug, Clone, Copy)]
pub struct LeafSegment {
  pub line_index: usize,
  pub x_start: f32,
  pub x_end: f32,
}

/// Shaped + atlas-packed multi-span paragraph. Glyph positions are
/// absolute within the paragraph rectangle (line offsets baked in).
/// Backgrounds and decorations for inline elements that wrap across
/// line breaks are emitted by mapping a `leaf_id` range through
/// `leaf_segments`.
#[derive(Debug, Clone, Default)]
pub struct ParagraphLayout {
  pub glyphs: Vec<PositionedGlyph>,
  pub lines: Vec<ParagraphLine>,
  pub width: f32,
  pub height: f32,
  pub first_line_ascent: f32,
  pub leaf_segments: HashMap<u32, Vec<LeafSegment>>,
}

/// Cached atlas slot for a single glyph raster.
#[derive(Debug, Clone, Copy)]
struct AtlasGlyph {
  rect: AtlasRect,
  /// Bearing in pixels — distance from the layout origin to the top
  /// of the glyph bitmap. Positive = down.
  top: i32,
  /// Bearing in pixels — distance from the layout origin to the
  /// left edge of the bitmap.
  left: i32,
  /// Bitmap dimensions.
  w: u32,
  h: u32,
}

/// Per-host text shaping + glyph atlas state. Long-lived; cheap to
/// keep across frames.
pub struct TextContext {
  pub font_db: FontDb,
  /// Mirror of the document's `FontRegistry`. Populated by
  /// `sync_fonts`; consulted by `pick_font` for CSS-aware font
  /// matching (family list + weight + style). Cheap to clone — the
  /// underlying byte arcs are shared.
  pub fonts: FontRegistry,
  pub atlas: Atlas,
  swash: cosmic_text::SwashCache,
  glyph_cache: HashMap<CacheKey, AtlasGlyph>,
  /// Generation of the `FontRegistry` at the last successful
  /// `sync_fonts` call. Used to short-circuit when fonts haven't
  /// changed between frames.
  last_font_generation: u64,
  /// Cached shaped runs keyed by text + font + size parameters
  /// (excludes colour). Cleared when fonts change.
  text_cache: HashMap<TextCacheKey, ShapedRun>,
  /// Cached paragraph layouts keyed by span content hash.
  paragraph_cache: HashMap<ParagraphCacheKey, ParagraphLayout>,
  /// Font generation at the time the text caches were last
  /// validated. When `last_font_generation` advances past this
  /// value, both caches are cleared.
  cache_font_generation: u64,
}

impl TextContext {
  /// Empty context with the given square atlas size in pixels (e.g.
  /// 2048). Atlas grows are out of scope for T3 — pick something
  /// generous up front.
  pub fn new(atlas_size: u32) -> Self {
    Self {
      font_db: FontDb::new(),
      fonts: FontRegistry::default(),
      atlas: Atlas::new(atlas_size, atlas_size),
      swash: cosmic_text::SwashCache::new(),
      glyph_cache: HashMap::new(),
      last_font_generation: 0,
      text_cache: HashMap::new(),
      paragraph_cache: HashMap::new(),
      cache_font_generation: 0,
    }
  }

  /// Invalidate text measurement caches if fonts have changed.
  fn maybe_invalidate_text_caches(&mut self) {
    if self.last_font_generation != self.cache_font_generation {
      self.text_cache.clear();
      self.paragraph_cache.clear();
      self.cache_font_generation = self.last_font_generation;
    }
    // Simple size bound — prevent unbounded growth.
    if self.text_cache.len() > TEXT_CACHE_MAX {
      self.text_cache.clear();
    }
    if self.paragraph_cache.len() > TEXT_CACHE_MAX {
      self.paragraph_cache.clear();
    }
  }

  /// Reconcile the cosmic-text font system against `registry`.
  /// Stores a clone of the registry so `pick_font` can do
  /// CSS-aware family / weight / style matching without a fresh
  /// borrow from the host.
  ///
  /// Short-circuits when the registry's generation counter hasn't
  /// changed since the last sync — avoids a full clone + bridge
  /// reconciliation on every frame.
  pub fn sync_fonts(&mut self, registry: &FontRegistry) {
    if registry.generation() == self.last_font_generation && registry.len() == self.fonts.len() {
      return;
    }
    self.fonts = registry.clone();
    self.font_db.sync(registry);
    self.last_font_generation = registry.generation();
  }

  /// Pick a `FontHandle` for a CSS `font-family` list, weight, and
  /// style. Walks the comma-separated family list left-to-right,
  /// returning the first match per CSS-Fonts-3-style scoring (see
  /// `wgpu_html_tree::FontRegistry::find`). Falls back to the
  /// first registered face if no listed family matches; returns
  /// `None` only when the registry is empty.
  pub fn pick_font(&self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
    self
      .fonts
      .find_first(families, weight, style)
      .or_else(|| self.font_db.first_handle())
  }

  /// Resolve a CSS family list + weight + style to the concrete
  /// family name cosmic-text needs to see in `Attrs.family`. Used
  /// by callers that don't go through `shape_and_pack` (which
  /// already does this internally) — most importantly the
  /// rich-text paragraph path, where each span's family has to
  /// arrive pre-resolved so cosmic-text's `set_rich_text` can pick
  /// the right face per span.
  pub fn resolve_family(&mut self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<String> {
    let handle = self.pick_font(families, weight, style)?;
    let fontdb_id = self.font_db.fontdb_id(handle)?;
    let db = self.font_db.font_system_mut().db_mut();
    let face = db.face(fontdb_id)?;
    face.families.first().map(|(name, _)| name.clone())
  }

  /// CSS `line-height: normal` multiplier for the given font face.
  ///
  /// Computed as `(hhea.ascender - hhea.descender + hhea.lineGap) /
  /// units_per_em`, matching the browser formula for the `normal`
  /// keyword. Returns `None` when the handle has no loaded data.
  pub fn normal_line_height_multiplier(&self, handle: FontHandle) -> Option<f32> {
    let face_data = self.fonts.get(handle)?;
    parse_line_height_multiplier(&face_data.data)
  }

  /// Shape `text` using the registered face `font`, at `size_px`
  /// font size, with `line_height_px` total line box height,
  /// `letter_spacing_px` extra advance after each glyph, and
  /// `weight` / `axis` to bias cosmic-text's within-family face
  /// matching (so `<strong>` picks the bold face if one is
  /// registered alongside the regular under the same family).
  /// Returns `None` if the font isn't loaded into the bridge.
  ///
  /// `max_width_px` enables soft-wrap line breaking: when `Some`,
  /// cosmic-text wraps lines at the given pixel width using its
  /// UAX 14 break opportunities. When `None`, the run stays on a
  /// single line of unbounded length.
  ///
  /// The returned `ShapedRun` may carry glyphs from multiple lines
  /// (each glyph's `y` is in run-relative coords with line offsets
  /// already applied); `height` is the stacked total, `ascent` is
  /// the first line's baseline, `width` is the widest line.
  /// Like [`shape_and_pack`] but only returns the metrics (width,
  /// height, ascent) without cloning the full glyph data. Used by
  /// flex intrinsic sizing which only needs dimensions. On cache
  /// miss, shapes the text and caches it (so the subsequent full
  /// `shape_and_pack` will hit).
  pub fn measure_only(
    &mut self,
    text: &str,
    font: FontHandle,
    size_px: f32,
    line_height_px: f32,
    letter_spacing_px: f32,
    weight: u16,
    axis: FontStyleAxis,
    max_width_px: Option<f32>,
  ) -> Option<(f32, f32, f32)> {
    self.maybe_invalidate_text_caches();

    let cache_key = TextCacheKey {
      text_hash: hash_str(text),
      font_handle: font,
      size_px_bits: size_px.to_bits(),
      line_height_bits: line_height_px.to_bits(),
      letter_spacing_bits: letter_spacing_px.to_bits(),
      weight,
      style: axis,
      max_width_bits: max_width_px.map(|w| w.to_bits()),
    };
    if let Some(cached) = self.text_cache.get(&cache_key) {
      return Some((cached.width, cached.height, cached.ascent));
    }
    // Cache miss — fall through to full shaping, which will populate
    // the cache for subsequent calls.
    let run = self.shape_and_pack(
      text,
      font,
      size_px,
      line_height_px,
      letter_spacing_px,
      weight,
      axis,
      max_width_px,
      [0.0, 0.0, 0.0, 1.0],
    )?;
    Some((run.width, run.height, run.ascent))
  }

  pub fn shape_and_pack(
    &mut self,
    text: &str,
    font: FontHandle,
    size_px: f32,
    line_height_px: f32,
    letter_spacing_px: f32,
    weight: u16,
    axis: FontStyleAxis,
    max_width_px: Option<f32>,
    color: [f32; 4],
  ) -> Option<ShapedRun> {
    self.maybe_invalidate_text_caches();

    let cache_key = TextCacheKey {
      text_hash: hash_str(text),
      font_handle: font,
      size_px_bits: size_px.to_bits(),
      line_height_bits: line_height_px.to_bits(),
      letter_spacing_bits: letter_spacing_px.to_bits(),
      weight,
      style: axis,
      max_width_bits: max_width_px.map(|w| w.to_bits()),
    };
    if let Some(cached) = self.text_cache.get(&cache_key) {
      let mut run = cached.clone();
      for g in &mut run.glyphs {
        g.color = color;
      }
      return Some(run);
    }

    let fontdb_id = self.font_db.fontdb_id(font)?;

    // cosmic-text needs the family name to find the face. We have
    // the fontdb ID from earlier — fish the family back out.
    let family_name = {
      let db = self.font_db.font_system_mut().db_mut();
      let face = db.face(fontdb_id)?;
      // `face.families` is `Vec<(String, Language)>`; take the
      // first family name.
      face.families.first().map(|(name, _)| name.clone())?
    };

    let metrics = Metrics::new(size_px, line_height_px.max(size_px));
    // Set family + weight + style on the Attrs so cosmic-text's
    // within-family face matching can pick e.g. Inter-Bold for a
    // 700 weight when both regular and bold faces are loaded.
    let cosmic_style = match axis {
      FontStyleAxis::Normal => cosmic_text::Style::Normal,
      FontStyleAxis::Italic => cosmic_text::Style::Italic,
      FontStyleAxis::Oblique => cosmic_text::Style::Oblique,
    };
    let attrs = Attrs::new()
      .family(cosmic_text::Family::Name(&family_name))
      .weight(cosmic_text::Weight(weight))
      .style(cosmic_style);
    // cosmic-text 0.12's `Attrs` doesn't expose letter-spacing,
    // so we apply it post-shape: each glyph past the first gains
    // a cumulative `letter_spacing_px` shift, and the run's
    // reported width grows by `(n - 1) * letter_spacing_px`.

    // Setting a finite width opts cosmic-text into UAX 14 line
    // breaking; `None` keeps everything on one infinite line, the
    // T3 behaviour that single-line / nowrap callers still want.
    let mut buffer = Buffer::new(self.font_db.font_system_mut(), metrics);
    buffer.set_size(self.font_db.font_system_mut(), max_width_px, None);
    buffer.set_text(self.font_db.font_system_mut(), text, attrs, Shaping::Advanced);
    buffer.shape_until_scroll(self.font_db.font_system_mut(), false);

    // Pre-compute byte→char boundary table for glyph_chars mapping.
    let bb = utf8_boundaries(text);

    // Snapshot every layout run (one per line). We keep the
    // (physical, x, w, byte_start, line_y, line_h) tuples up-front
    // so the glyph loop below can borrow `&mut self.font_db` /
    // `&mut self.swash` without colliding with the `Buffer`'s
    // outstanding `&mut FontSystem`.
    let layout_lines: Vec<(Vec<(cosmic_text::PhysicalGlyph, f32, f32, usize)>, f32, f32, f32)> = buffer
      .layout_runs()
      .map(|run| {
        let line_top = run.line_top;
        let line_y = run.line_y;
        let line_h = run.line_height;
        let glyphs: Vec<_> = run
          .glyphs
          .iter()
          .map(|g| (g.physical((0.0, 0.0), 1.0), g.x, g.w, g.start))
          .collect();
        (glyphs, line_top, line_y, line_h)
      })
      .collect();

    let first_line = layout_lines.first()?;
    let ascent_px = first_line.2;
    let total_height: f32 = layout_lines.iter().map(|(_, top, _, h)| top + h).fold(0.0, f32::max);

    let glyph_capacity: usize = layout_lines.iter().map(|(g, ..)| g.len()).sum();
    let mut glyphs: Vec<PositionedGlyph> = Vec::with_capacity(glyph_capacity);
    let mut glyph_chars: Vec<usize> = Vec::with_capacity(glyph_capacity);
    let mut lines: Vec<ShapedLine> = Vec::with_capacity(layout_lines.len());
    let mut max_x: f32 = 0.0;
    let mut min_x: f32 = 0.0;
    let (atlas_w, atlas_h) = self.atlas.dimensions();

    for (line_glyphs, line_top, line_y, line_h) in &layout_lines {
      let line_glyph_start = glyphs.len();
      let baseline_y = *line_y;
      for (glyph_index, (physical, layout_x, layout_w, g_start)) in line_glyphs.iter().enumerate() {
        // Cumulative `letter-spacing` offset for this glyph (zero
        // for the first one, then `letter_spacing_px` per logical
        // glyph step).
        let spacing_dx = (glyph_index as f32) * letter_spacing_px;
        let key = physical.cache_key;
        let entry = match self.glyph_cache.get(&key).copied() {
          Some(e) => e,
          None => {
            let Some(image) = self.swash.get_image_uncached(self.font_db.font_system_mut(), key) else {
              // Glyph has no rasterisable outline (e.g. control
              // char). Skip it; it still contributes its
              // advance.
              continue;
            };

            let w = image.placement.width;
            let h = image.placement.height;
            let rect = if w == 0 || h == 0 {
              AtlasRect { x: 0, y: 0, w: 0, h: 0 }
            } else {
              // SwashImage data: row-major, top-down, R8 alpha.
              match self.atlas.insert(w, h, &image.data) {
                Some(e) => e.rect,
                None => continue, // atlas full — skip glyph
              }
            };
            let entry = AtlasGlyph {
              rect,
              top: image.placement.top,
              left: image.placement.left,
              w,
              h,
            };
            self.glyph_cache.insert(key, entry);
            entry
          }
        };

        // Glyph position. `baseline_y` is this line's baseline
        // in run-relative pixel coords (cosmic-text reports
        // `run.line_y` per layout run). Subtract the bitmap top
        // bearing for the quad's top-left. Both coords are
        // rounded so the linear sampler doesn't blend the mask
        // with zeroed padding rows.
        // BUG: zeno's Placement.top carries a baseline-relative
        // offset that lands glyphs ~10% too low.  The 0.10-baseline
        // upward shift empirically matches browser rendering.
        let pos_x = (physical.x as f32 + entry.left as f32 + spacing_dx).round();
        let pos_y = (baseline_y - entry.top as f32 - baseline_y * 0.10).round();

        let quad_w = entry.w as f32;
        // Extra 1px on the quad height prevents bottom-row clipping
        // from GPU rasterisation boundary conditions.
        let quad_h = entry.h as f32 + 1.0;

        // Track left/right ink extents. After the loop we shift all
        // glyphs so min_x = 0 and include the overshoot in run.width.
        if pos_x < min_x {
          min_x = pos_x;
        }

        if entry.w > 0 && entry.h > 0 {
          // UVs span the exact pixel boundaries of the atlas entry.
          // The atlas packer inserts a 1px gutter between entries to
          // prevent bilinear bleed; we intentionally do NOT inset the
          // UVs here so that the bottom-most pixel row is not lost.
          let uv_min = [
            entry.rect.x as f32 / atlas_w as f32,
            entry.rect.y as f32 / atlas_h as f32,
          ];
          let uv_max = [
            (entry.rect.x + entry.rect.w) as f32 / atlas_w as f32,
            (entry.rect.y + entry.rect.h) as f32 / atlas_h as f32,
          ];
          glyphs.push(PositionedGlyph {
            x: pos_x,
            y: pos_y,
            w: quad_w,
            h: quad_h,
            uv_min,
            uv_max,
            color,
          });
          // Map this glyph's byte start → char index.
          let char_idx = bb.partition_point(|&b| b < *g_start).min(bb.len().saturating_sub(1));
          glyph_chars.push(char_idx);
        }

        // The line's used width must cover both the advance cursor
        // AND the ink extent of the last glyph (its bitmap may
        // extend past the advance via right-side bearing).
        let advance_right = layout_x + layout_w + spacing_dx;
        let ink_right = pos_x + quad_w;
        let right = advance_right.max(ink_right);
        if right > max_x {
          max_x = right;
        }
      }
      lines.push(ShapedLine {
        top: *line_top,
        height: *line_h,
        glyph_range: (line_glyph_start, glyphs.len()),
      });
    }

    // The box height is determined by the line-height, not by glyph
    // extents. In CSS the line box is never expanded for descenders
    // or ascender overshoot — glyphs are allowed to paint outside
    // the line box. Expanding the box based on actual glyph bounds
    // makes the height content-dependent: a span with descenders
    // would get a taller box than one without, causing flex
    // `align-items: center` to place them at different offsets.

    // Shift all glyph x-positions so the leftmost ink edge is at
    // x=0 (eliminates negative left-side bearing overshoot). This
    // ensures adjacent flex items don't visually overlap.
    if min_x < 0.0 {
      let shift = -min_x;
      for g in &mut glyphs {
        g.x += shift;
      }
      max_x += shift;
    }

    let run = ShapedRun {
      glyphs,
      glyph_chars,
      lines,
      text: text.to_owned(),
      byte_boundaries: bb,
      width: max_x,
      height: total_height,
      ascent: ascent_px,
    };
    self.text_cache.insert(cache_key, run.clone());
    Some(run)
  }

  /// Shape a multi-span paragraph in one go. cosmic-text's
  /// `set_rich_text` lets break opportunities cross span boundaries
  /// — so `<p>aaa <strong>bbb</strong> ccc</p>` can wrap inside
  /// either piece without losing per-span attributes (font / weight
  /// / colour / size).
  ///
  /// `max_width_px = Some(w)` enables UAX-14 soft-wrap at width
  /// `w`. With `None` the paragraph stays on one infinite line.
  ///
  /// Each span carries a `leaf_id` the caller picks; the returned
  /// `ParagraphLayout::leaf_segments` maps that id to the
  /// per-line advance ranges the span occupies. The IFC uses this
  /// to expand inline-element backgrounds (`<mark>`) and
  /// decorations across line breaks.
  pub fn shape_paragraph(&mut self, spans: &[ParagraphSpan<'_>], max_width_px: Option<f32>) -> Option<ParagraphLayout> {
    if spans.is_empty() {
      return None;
    }

    self.maybe_invalidate_text_caches();

    // Build a cache key that hashes all span content + attributes.
    let para_cache_key = {
      let mut h = std::collections::hash_map::DefaultHasher::new();
      for s in spans {
        s.text.hash(&mut h);
        s.family.hash(&mut h);
        s.weight.hash(&mut h);
        s.style.hash(&mut h);
        s.size_px.to_bits().hash(&mut h);
        s.line_height_px.to_bits().hash(&mut h);
        // Exclude color — doesn't affect geometry.
        s.leaf_id.hash(&mut h);
      }
      ParagraphCacheKey {
        content_hash: h.finish(),
        max_width_bits: max_width_px.map(|w| w.to_bits()),
      }
    };
    if let Some(cached) = self.paragraph_cache.get(&para_cache_key) {
      return Some(cached.clone());
    }

    // Buffer-default Metrics: pick the largest span size so a
    // mixed paragraph with `<small>` doesn't constrain the line
    // height of the regular text. Per-span Attrs.metrics_opt
    // overrides where each glyph actually sits.
    let default_size = spans.iter().map(|s| s.size_px).fold(0.0_f32, f32::max).max(1.0);
    let default_lh = spans
      .iter()
      .map(|s| s.line_height_px)
      .fold(0.0_f32, f32::max)
      .max(default_size);
    let metrics = Metrics::new(default_size, default_lh);

    // Build per-span Attrs. Family / text strings borrow from
    // `spans`, which the caller keeps alive for the duration of
    // this call.
    let attrs_per_span: Vec<Attrs<'_>> = spans
      .iter()
      .map(|s| {
        let cosmic_style = match s.style {
          FontStyleAxis::Normal => cosmic_text::Style::Normal,
          FontStyleAxis::Italic => cosmic_text::Style::Italic,
          FontStyleAxis::Oblique => cosmic_text::Style::Oblique,
        };
        // Linear → 8-bit colour for cosmic-text's tagged
        // `Color`. The bit depth lossy-rounds at the source
        // colour but text colours don't notice.
        let r = (s.color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (s.color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (s.color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (s.color[3].clamp(0.0, 1.0) * 255.0).round() as u8;
        Attrs::new()
          .family(cosmic_text::Family::Name(s.family))
          .weight(cosmic_text::Weight(s.weight))
          .style(cosmic_style)
          .metrics(Metrics::new(
            s.size_px.max(1.0),
            s.line_height_px.max(s.size_px).max(1.0),
          ))
          .color(cosmic_text::Color::rgba(r, g, b, a))
          .metadata(s.leaf_id as usize)
      })
      .collect();

    let default_attrs = Attrs::new();
    let mut buffer = Buffer::new(self.font_db.font_system_mut(), metrics);
    buffer.set_size(self.font_db.font_system_mut(), max_width_px, None);
    buffer.set_rich_text(
      self.font_db.font_system_mut(),
      spans
        .iter()
        .zip(attrs_per_span.iter())
        .map(|(s, a)| (s.text, a.clone())),
      default_attrs,
      Shaping::Advanced,
    );
    buffer.shape_until_scroll(self.font_db.font_system_mut(), false);

    // Snapshot the runs first so we can borrow `&mut self.swash`
    // and `&mut self.font_db` independently below.
    struct GlyphSnap {
      physical: cosmic_text::PhysicalGlyph,
      layout_x: f32,
      layout_w: f32,
      metadata: usize,
      color: [f32; 4],
    }
    struct LineSnap {
      top: f32,
      baseline: f32,
      height: f32,
      glyphs: Vec<GlyphSnap>,
    }
    let lines_snap: Vec<LineSnap> = buffer
      .layout_runs()
      .map(|run| {
        let glyphs: Vec<GlyphSnap> = run
          .glyphs
          .iter()
          .map(|g| {
            let physical = g.physical((0.0, 0.0), 1.0);
            let color = g
              .color_opt
              .map(|c| {
                [
                  c.r() as f32 / 255.0,
                  c.g() as f32 / 255.0,
                  c.b() as f32 / 255.0,
                  c.a() as f32 / 255.0,
                ]
              })
              .unwrap_or([0.0, 0.0, 0.0, 1.0]);
            GlyphSnap {
              physical,
              layout_x: g.x,
              layout_w: g.w,
              metadata: g.metadata,
              color,
            }
          })
          .collect();
        LineSnap {
          top: run.line_top,
          baseline: run.line_y,
          height: run.line_height,
          glyphs,
        }
      })
      .collect();

    if lines_snap.is_empty() {
      return None;
    }

    let (atlas_w, atlas_h) = self.atlas.dimensions();
    let mut all_glyphs: Vec<PositionedGlyph> = Vec::new();
    let mut lines_meta: Vec<ParagraphLine> = Vec::with_capacity(lines_snap.len());
    let mut leaf_segments: HashMap<u32, Vec<LeafSegment>> = HashMap::new();
    let mut max_line_width: f32 = 0.0;
    let mut total_height: f32 = 0.0;

    for (line_idx, line) in lines_snap.iter().enumerate() {
      let mut line_max_x: f32 = 0.0;
      // Per-span x range on this line.
      let mut leaf_x: HashMap<u32, (f32, f32)> = HashMap::new();
      let glyph_start = all_glyphs.len();

      for snap in &line.glyphs {
        let leaf_id = snap.metadata as u32;
        let key = snap.physical.cache_key;
        let entry = match self.glyph_cache.get(&key).copied() {
          Some(e) => e,
          None => {
            let Some(image) = self.swash.get_image_uncached(self.font_db.font_system_mut(), key) else {
              continue;
            };
            let w = image.placement.width;
            let h = image.placement.height;
            let rect = if w == 0 || h == 0 {
              AtlasRect { x: 0, y: 0, w: 0, h: 0 }
            } else {
              match self.atlas.insert(w, h, &image.data) {
                Some(e) => e.rect,
                None => continue,
              }
            };
            let entry = AtlasGlyph {
              rect,
              top: image.placement.top,
              left: image.placement.left,
              w,
              h,
            };
            self.glyph_cache.insert(key, entry);
            entry
          }
        };

        let pos_x = (snap.physical.x as f32 + entry.left as f32).round();
        let pos_y = (line.baseline - entry.top as f32 - line.baseline * 0.10).round();
        let quad_w = entry.w as f32;
        let quad_h = entry.h as f32 + 1.0;

        if entry.w > 0 && entry.h > 0 {
          let uv_min = [
            entry.rect.x as f32 / atlas_w as f32,
            entry.rect.y as f32 / atlas_h as f32,
          ];
          let uv_max = [
            (entry.rect.x + entry.rect.w) as f32 / atlas_w as f32,
            (entry.rect.y + entry.rect.h) as f32 / atlas_h as f32,
          ];
          all_glyphs.push(PositionedGlyph {
            x: pos_x,
            y: pos_y,
            w: quad_w,
            h: quad_h,
            uv_min,
            uv_max,
            color: snap.color,
          });
        }

        let advance_left = snap.layout_x;
        let advance_right = snap.layout_x + snap.layout_w;
        let entry_xs = leaf_x.entry(leaf_id).or_insert((advance_left, advance_right));
        if advance_left < entry_xs.0 {
          entry_xs.0 = advance_left;
        }
        if advance_right > entry_xs.1 {
          entry_xs.1 = advance_right;
        }
        if advance_right > line_max_x {
          line_max_x = advance_right;
        }
      }

      for (leaf_id, (x_start, x_end)) in leaf_x {
        leaf_segments.entry(leaf_id).or_default().push(LeafSegment {
          line_index: line_idx,
          x_start,
          x_end,
        });
      }

      if line_max_x > max_line_width {
        max_line_width = line_max_x;
      }
      let glyph_end = all_glyphs.len();
      lines_meta.push(ParagraphLine {
        top: line.top,
        baseline: line.baseline,
        height: line.height,
        line_width: line_max_x,
        glyph_range: (glyph_start, glyph_end),
      });
      total_height = (line.top + line.height).max(total_height);
    }

    // Height is determined by line-heights, not glyph extents.
    // Glyphs may overflow above/below — CSS allows this.

    let first_line_ascent = lines_meta[0].baseline - lines_meta[0].top;

    let layout = ParagraphLayout {
      glyphs: all_glyphs,
      lines: lines_meta,
      width: max_line_width,
      height: total_height,
      first_line_ascent,
      leaf_segments,
    };
    self.paragraph_cache.insert(para_cache_key, layout.clone());
    Some(layout)
  }
}

/// Parse `line-height: normal` multiplier from raw font bytes.
///
/// Uses the same algorithm as browsers (Chrome / Safari):
///
/// 1. If the OS/2 table has the `USE_TYPO_METRICS` flag (bit 7 of `fsSelection`), use `(sTypoAscender − sTypoDescender
///    + sTypoLineGap) / unitsPerEm`.
/// 2. Otherwise use `(usWinAscent + usWinDescent) / unitsPerEm`.
/// 3. If there is no OS/2 table, fall back to the hhea table: `(ascender − descender + lineGap) / unitsPerEm`.
///
/// Returns `None` if the required tables can't be located.
pub fn parse_line_height_multiplier(data: &[u8]) -> Option<f32> {
  // For TTC (font collections), use the first offset table.
  let offset_table_start = if data.len() >= 12 && &data[0..4] == b"ttcf" {
    let n = u32::from_be_bytes(data[8..12].try_into().ok()?) as usize;
    if n == 0 || data.len() < 16 {
      return None;
    }
    u32::from_be_bytes(data[12..16].try_into().ok()?) as usize
  } else {
    0
  };

  // Read number of tables from the offset table.
  let d = &data[offset_table_start..];
  if d.len() < 12 {
    return None;
  }
  let num_tables = u16::from_be_bytes(d[4..6].try_into().ok()?) as usize;

  // Scan the table directory for `head`, `hhea`, and `OS/2`.
  let mut head_off = None;
  let mut hhea_off = None;
  let mut os2_off = None;
  for i in 0..num_tables {
    let rec = 12 + i * 16;
    if rec + 16 > d.len() {
      break;
    }
    let tag = &d[rec..rec + 4];
    let off = u32::from_be_bytes(d[rec + 8..rec + 12].try_into().ok()?) as usize;
    match tag {
      b"head" => head_off = Some(off),
      b"hhea" => hhea_off = Some(off),
      b"OS/2" => os2_off = Some(off),
      _ => {}
    }
  }

  let head = head_off?;

  // head: unitsPerEm is at offset 18 (uint16).
  if head + 20 > data.len() {
    return None;
  }
  let upem = u16::from_be_bytes(data[head + 18..head + 20].try_into().ok()?) as f32;
  if upem == 0.0 {
    return None;
  }

  // Try OS/2 table first (browser-preferred path).
  if let Some(os2) = os2_off {
    // fsSelection is at offset 62 (uint16).
    // sTypoAscender @ 68, sTypoDescender @ 70, sTypoLineGap @ 72 (int16).
    // usWinAscent @ 74, usWinDescent @ 76 (uint16).
    if os2 + 78 <= data.len() {
      let fs_selection = u16::from_be_bytes(data[os2 + 62..os2 + 64].try_into().ok()?);
      let use_typo_metrics = fs_selection & (1 << 7) != 0;

      if use_typo_metrics {
        let typo_asc = i16::from_be_bytes(data[os2 + 68..os2 + 70].try_into().ok()?) as f32;
        let typo_desc = i16::from_be_bytes(data[os2 + 70..os2 + 72].try_into().ok()?) as f32;
        let typo_gap = i16::from_be_bytes(data[os2 + 72..os2 + 74].try_into().ok()?) as f32;
        return Some((typo_asc - typo_desc + typo_gap) / upem);
      } else {
        let win_asc = u16::from_be_bytes(data[os2 + 74..os2 + 76].try_into().ok()?) as f32;
        let win_desc = u16::from_be_bytes(data[os2 + 76..os2 + 78].try_into().ok()?) as f32;
        return Some((win_asc + win_desc) / upem);
      }
    }
  }

  // Fallback: hhea table.
  let hhea = hhea_off?;
  if hhea + 10 > data.len() {
    return None;
  }
  let ascender = i16::from_be_bytes(data[hhea + 4..hhea + 6].try_into().ok()?) as f32;
  let descender = i16::from_be_bytes(data[hhea + 6..hhea + 8].try_into().ok()?) as f32;
  let line_gap = i16::from_be_bytes(data[hhea + 8..hhea + 10].try_into().ok()?) as f32;

  Some((ascender - descender + line_gap) / upem)
}

#[cfg(test)]
#[path = "shape_tests.rs"]
mod tests_shape;
