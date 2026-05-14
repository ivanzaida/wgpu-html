//! High-level text context — wraps font management, atlas, and raster pipeline.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use cosmic_text::{Attrs, Buffer, CacheKey, Family, Metrics, Shaping, Weight};
use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};

use crate::atlas::{Atlas, AtlasRect};
use crate::font::FontContext;
use crate::font_face::{FontFace, FontHandle, FontStyleAxis};
use crate::shape::{PositionedGlyph, RunMetrics, ShapedLine, ShapedRun, TextStyle, parse_line_height_multiplier, utf8_boundaries};

const TEXT_CACHE_MAX: usize = 4096;

// ── Cache key ─────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextCacheKey {
    text_hash: u64,
    family_hash: u64,
    size_px_bits: u32,
    line_height_bits: u32,
    weight: u16,
    style: FontStyleAxis,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct LinesCacheKey {
    text_hash: u64,
    family_hash: u64,
    size_px_bits: u32,
    line_height_bits: u32,
    weight: u16,
    max_width_bits: u32,
}

fn hash_str(s: &str) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn shape_cache_key(text: &str, style: &TextStyle) -> TextCacheKey {
    TextCacheKey {
        text_hash: hash_str(text),
        family_hash: hash_str(style.font_family),
        size_px_bits: style.font_size.to_bits(),
        line_height_bits: style.line_height.to_bits(),
        weight: style.weight,
        style: FontStyleAxis::Normal,
    }
}

fn lines_cache_key(text: &str, style: &TextStyle, max_width: f32) -> LinesCacheKey {
    LinesCacheKey {
        text_hash: hash_str(text),
        family_hash: hash_str(style.font_family),
        size_px_bits: style.font_size.to_bits(),
        line_height_bits: style.line_height.to_bits(),
        weight: style.weight,
        max_width_bits: max_width.to_bits(),
    }
}

/// Cached atlas slot for a single glyph raster.
#[derive(Debug, Clone, Copy)]
struct AtlasGlyph {
    rect: AtlasRect,
    top: i32,
    left: i32,
}

/// High-level text context — font management, atlas, and raster pipeline.
pub struct TextContext {
    pub font_ctx: FontContext,
    pub atlas: Atlas,
    swash: cosmic_text::SwashCache,
    glyph_cache: HashMap<CacheKey, AtlasGlyph>,
    text_cache: HashMap<TextCacheKey, ShapedRun>,
    lines_cache: HashMap<LinesCacheKey, Vec<ShapedLine>>,
    text_cache_gen: u64,
}

impl TextContext {
    pub fn new() -> Self {
        Self {
            font_ctx: FontContext::new(),
            atlas: Atlas::new(2048, 2048),
            swash: cosmic_text::SwashCache::new(),
            glyph_cache: HashMap::new(),
            text_cache: HashMap::new(),
            lines_cache: HashMap::new(),
            text_cache_gen: 0,
        }
    }

    fn maybe_invalidate_text_cache(&mut self) {
        let current_gen = self.font_ctx.registry().generation();
        if current_gen != self.text_cache_gen {
            self.text_cache.clear();
            self.lines_cache.clear();
            self.text_cache_gen = current_gen;
        } else if self.text_cache.len() + self.lines_cache.len() >= TEXT_CACHE_MAX {
            self.text_cache.clear();
            self.lines_cache.clear();
        }
    }

    /// Cached text shaping — returns a clone from cache on hit.
    pub fn shape(&mut self, text: &str, style: &TextStyle) -> ShapedRun {
        self.maybe_invalidate_text_cache();
        let key = shape_cache_key(text, style);
        if let Some(cached) = self.text_cache.get(&key) {
            return cached.clone();
        }
        let run = self.font_ctx.shape(text, style);
        self.text_cache.insert(key, run.clone());
        run
    }

    /// Cached line breaking — returns a clone from cache on hit.
    pub fn break_into_lines(&mut self, text: &str, style: &TextStyle, max_width: f32) -> Vec<ShapedLine> {
        self.maybe_invalidate_text_cache();
        let key = lines_cache_key(text, style, max_width);
        if let Some(cached) = self.lines_cache.get(&key) {
            return cached.clone();
        }
        let lines = self.font_ctx.break_into_lines(text, style, max_width);
        self.lines_cache.insert(key, lines.clone());
        lines
    }

    /// Register a custom font face (e.g. an icon font like Lucide).
    pub fn register_font(&mut self, face: FontFace) -> FontHandle {
        self.font_ctx.register_font(face)
    }

    // ── Layout-time shaping (no rastering) ──────────────────────────

    /// Shape a text span, extracting font properties from `ComputedStyle`.
    /// Produces glyph positions without atlas UVs — use `shape_and_pack`
    /// for rendering-ready glyphs.
    pub fn shape_run(&mut self, text: &str, style: &ComputedStyle) -> ShapedRun {
        let ts = text_style_from_cascade(style);
        let families: Vec<&str> = ts.font_family.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let families = if families.is_empty() { vec!["sans-serif"] } else { families };

        self.font_ctx.shape_with_families(
            text, &families, ts.font_size, ts.line_height, ts.weight, FontStyleAxis::Normal,
        )
    }

    // ── Render-time shaping (with atlas rastering) ───────────────────

    /// Measure a text span from `ComputedStyle` — returns metrics only.
    /// Uses the text cache; shapes on miss.
    pub fn measure_run(&mut self, text: &str, style: &ComputedStyle) -> RunMetrics {
        let ts = text_style_from_cascade(style);
        let families: Vec<&str> = ts.font_family.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let family_str = if families.is_empty() { "sans-serif" } else { ts.font_family };

        self.maybe_invalidate_text_cache();
        let cache_key = TextCacheKey {
            text_hash: hash_str(text),
            family_hash: hash_str(family_str),
            size_px_bits: ts.font_size.to_bits(),
            line_height_bits: ts.line_height.to_bits(),
            weight: ts.weight,
            style: FontStyleAxis::Normal,
        };
        if let Some(cached) = self.text_cache.get(&cache_key) {
            return RunMetrics { width: cached.width, height: cached.height, ascent: cached.ascent };
        }
        let run = self.shape_run(text, style);
        let metrics = RunMetrics { width: run.width, height: run.height, ascent: run.ascent };
        self.text_cache.insert(cache_key, run);
        metrics
    }

    /// Shape text, raster glyphs into the atlas, and emit
    /// `PositionedGlyph`s with UV coords and color.
    ///
    /// `dpi_scale` controls rasterization resolution: glyphs are shaped at
    /// `font_size * dpi_scale` for sharp physical pixels, then positions and
    /// sizes are divided back to logical pixels for the display list.
    pub fn shape_and_pack(
        &mut self,
        text: &str,
        font_size: f32,
        line_height: f32,
        weight: u16,
        color: [f32; 4],
        font_family: &str,
        dpi_scale: f32,
    ) -> ShapedRun {
        self.maybe_invalidate_text_cache();

        let family = if font_family.is_empty() { "sans-serif" } else { font_family };
        let phys_size = font_size * dpi_scale;
        let phys_lh = line_height * dpi_scale;
        let cache_key = TextCacheKey {
            text_hash: hash_str(text),
            family_hash: hash_str(family),
            size_px_bits: phys_size.to_bits(),
            line_height_bits: phys_lh.to_bits(),
            weight,
            style: FontStyleAxis::Normal,
        };
        if let Some(cached) = self.text_cache.get(&cache_key) {
            let has_uvs = cached.glyphs.first().is_some_and(|g| g.uv_min != [0.0; 2] || g.uv_max != [0.0; 2]);
            if has_uvs {
                let mut run = cached.clone();
                for g in &mut run.glyphs { g.color = color; }
                return run;
            }
            self.text_cache.remove(&cache_key);
        }

        let ts = TextStyle { font_family: family, font_size: phys_size, line_height: phys_lh, weight, ..Default::default() };
        let attrs = crate::shape::build_attrs(&ts);

        let metrics = Metrics::new(phys_size, phys_lh);
        let mut buffer = Buffer::new(self.font_ctx.font_system_mut(), metrics);
        buffer.set_size(None, None);
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(self.font_ctx.font_system_mut(), false);

        let inv = 1.0 / dpi_scale;
        let bb = utf8_boundaries(text);
        let mut glyphs = Vec::new();
        let mut glyph_chars = Vec::new();
        let mut line_count = 0;
        let mut ascent = 0.0;
        let (atlas_w, atlas_h) = self.atlas.dimensions();

        for run in buffer.layout_runs() {
            if line_count == 0 { ascent = run.line_y * inv; }
            line_count += 1;
            for g in run.glyphs {
                let physical = g.physical((0.0, 0.0), 1.0);
                let key = physical.cache_key;
                let entry = match self.glyph_cache.get(&key).copied() {
                    Some(e) => e,
                    None => {
                        let Some(image) = self.swash.get_image_uncached(self.font_ctx.font_system_mut(), key) else {
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
                        let entry = AtlasGlyph { rect, top: image.placement.top, left: image.placement.left };
                        self.glyph_cache.insert(key, entry);
                        entry
                    }
                };

                let uv_min = [entry.rect.x as f32 / atlas_w as f32, entry.rect.y as f32 / atlas_h as f32];
                let uv_max = [
                    (entry.rect.x + entry.rect.w) as f32 / atlas_w as f32,
                    (entry.rect.y + entry.rect.h) as f32 / atlas_h as f32,
                ];
                let gx = (g.x + entry.left as f32) * inv;
                let gy = (run.line_y - entry.top as f32).round() * inv;
                glyphs.push(PositionedGlyph {
                    glyph_id: g.glyph_id, x: gx, y: gy,
                    w: entry.rect.w as f32 * inv, h: entry.rect.h as f32 * inv,
                    uv_min, uv_max, color,
                });
                let char_idx = bb.partition_point(|&b| b < g.start)
                    .min(bb.len().saturating_sub(1));
                glyph_chars.push(char_idx);
            }
        }
        if let Some(min_x) = glyphs.iter().map(|g| g.x).reduce(f32::min) {
            if min_x != 0.0 {
                for g in &mut glyphs { g.x -= min_x; }
            }
        }
        let width = glyphs.iter().map(|g| g.x + g.w).fold(0.0f32, f32::max);
        let shaped = ShapedRun {
            glyphs, glyph_chars,
            text: text.to_owned(), byte_boundaries: bb,
            width, ascent,
            height: line_count as f32 * line_height,
            line_height, font_size, line_count,
        };
        self.text_cache.insert(cache_key, shaped.clone());
        shaped
    }

    /// Drain dirty atlas rects for GPU upload.
    pub fn flush_dirty<F: FnMut(AtlasRect, &[u8])>(&mut self, sink: F) {
        self.atlas.flush_dirty(sink);
    }

    /// CSS `line-height: normal` multiplier for a registered font.
    pub fn normal_line_height_multiplier(&self, handle: FontHandle) -> Option<f32> {
        let face = self.font_ctx.registry().get(handle)?;
        parse_line_height_multiplier(&face.data)
    }

    /// Resolve a CSS family list to a `FontHandle`.
    pub fn pick_font(&self, families: &[&str], weight: u16, style: FontStyleAxis) -> Option<FontHandle> {
        self.font_ctx.pick_font(families, weight, style)
    }

    // ── Paragraph / rich-text shaping ─────────────────────────────────

    /// Shape a multi-span paragraph using cosmic-text's `set_rich_text`.
    /// Each span carries a `leaf_id` the caller uses to map glyphs back
    /// to its source tree (e.g. per-element backgrounds/decorations).
    pub fn shape_paragraph(
        &mut self,
        spans: &[ParagraphSpan<'_>],
        max_width_px: Option<f32>,
    ) -> Option<ParagraphLayout> {
        if spans.is_empty() { return None; }

        let default_size = spans.iter().map(|s| s.size_px).fold(0.0_f32, f32::max).max(1.0);
        let default_lh = spans.iter().map(|s| s.line_height_px).fold(0.0_f32, f32::max).max(default_size);
        let metrics = Metrics::new(default_size, default_lh);

        let attrs_per_span: Vec<Attrs<'_>> = spans
            .iter()
            .map(|s| {
                let cosmic_style = crate::shape::convert_style(s.style);
                let r = (s.color[0].clamp(0.0, 1.0) * 255.0).round() as u8;
                let g = (s.color[1].clamp(0.0, 1.0) * 255.0).round() as u8;
                let b = (s.color[2].clamp(0.0, 1.0) * 255.0).round() as u8;
                let a = (s.color[3].clamp(0.0, 1.0) * 255.0).round() as u8;
                Attrs::new()
                    .family(Family::Name(s.family))
                    .weight(Weight(s.weight))
                    .style(cosmic_style)
                    .metrics(Metrics::new(s.size_px.max(1.0), s.line_height_px.max(s.size_px).max(1.0)))
                    .color(cosmic_text::Color::rgba(r, g, b, a))
                    .metadata(s.leaf_id as usize)
            })
            .collect();

        let default_attrs = Attrs::new();
        let mut buffer = Buffer::new(self.font_ctx.font_system_mut(), metrics);
        buffer.set_size(max_width_px, None);
        buffer.set_rich_text(
            spans.iter().zip(attrs_per_span.iter()).map(|(s, a)| (s.text, a.clone())),
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(self.font_ctx.font_system_mut(), false);

        let (atlas_w, atlas_h) = self.atlas.dimensions();
        let mut all_glyphs: Vec<PositionedGlyph> = Vec::new();
        let mut lines_meta: Vec<ParagraphLine> = Vec::new();
        let mut leaf_segments: HashMap<u32, Vec<LeafSegment>> = HashMap::new();
        let mut max_line_width: f32 = 0.0;
        let mut total_height: f32 = 0.0;

        for (line_idx, run) in buffer.layout_runs().enumerate() {
            let mut line_max_x: f32 = 0.0;
            let mut leaf_x: HashMap<u32, (f32, f32)> = HashMap::new();
            let glyph_start = all_glyphs.len();

            for g in run.glyphs {
                let leaf_id = g.metadata as u32;
                let physical = g.physical((0.0, 0.0), 1.0);
                let key = physical.cache_key;

                let entry = match self.glyph_cache.get(&key).copied() {
                    Some(e) => e,
                    None => {
                        let Some(image) = self.swash.get_image_uncached(self.font_ctx.font_system_mut(), key) else {
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
                        let entry = AtlasGlyph { rect, top: image.placement.top, left: image.placement.left };
                        self.glyph_cache.insert(key, entry);
                        entry
                    }
                };

                let color = g.color_opt
                    .map(|c| [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, c.a() as f32 / 255.0])
                    .unwrap_or([0.0, 0.0, 0.0, 1.0]);

                let pos_x = physical.x as f32 + entry.left as f32;
                let pos_y = (run.line_y - entry.top as f32).round();

                if entry.rect.w > 0 && entry.rect.h > 0 {
                    let uv_min = [entry.rect.x as f32 / atlas_w as f32, entry.rect.y as f32 / atlas_h as f32];
                    let uv_max = [
                        (entry.rect.x + entry.rect.w) as f32 / atlas_w as f32,
                        (entry.rect.y + entry.rect.h) as f32 / atlas_h as f32,
                    ];
                    all_glyphs.push(PositionedGlyph {
                        glyph_id: g.glyph_id, x: pos_x, y: pos_y,
                        w: entry.rect.w as f32, h: entry.rect.h as f32,
                        uv_min, uv_max, color,
                    });
                }

                let advance_left = g.x;
                let advance_right = g.x + g.w;
                let entry_xs = leaf_x.entry(leaf_id).or_insert((advance_left, advance_right));
                if advance_left < entry_xs.0 { entry_xs.0 = advance_left; }
                if advance_right > entry_xs.1 { entry_xs.1 = advance_right; }
                if advance_right > line_max_x { line_max_x = advance_right; }
            }

            for (leaf_id, (x_start, x_end)) in &leaf_x {
                leaf_segments.entry(*leaf_id).or_default().push(LeafSegment {
                    line_index: line_idx,
                    x_start: *x_start,
                    x_end: *x_end,
                });
            }

            max_line_width = max_line_width.max(line_max_x);
            let glyph_end = all_glyphs.len();
            lines_meta.push(ParagraphLine {
                top: run.line_top,
                baseline: run.line_y,
                height: run.line_height,
                line_width: line_max_x,
                glyph_range: (glyph_start, glyph_end),
            });
            total_height = (run.line_top + run.line_height).max(total_height);
        }

        if lines_meta.is_empty() { return None; }

        let first_line_ascent = lines_meta[0].baseline - lines_meta[0].top;
        Some(ParagraphLayout {
            glyphs: all_glyphs,
            lines: lines_meta,
            width: max_line_width,
            height: total_height,
            first_line_ascent,
            leaf_segments,
        })
    }

    pub fn font_system_mut(&mut self) -> &mut cosmic_text::FontSystem {
        self.font_ctx.font_system_mut()
    }
}

impl Default for TextContext {
    fn default() -> Self { Self::new() }
}

/// Extract text-relevant values from `ComputedStyle`.
pub fn text_style_from_cascade<'a>(style: &'a ComputedStyle<'a>) -> TextStyle<'a> {
    let font_size = match style.font_size {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(CssValue::Number(n)) => *n as f32,
        _ => 16.0,
    };
    let line_height = match style.line_height {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(CssValue::Number(n)) => (*n * font_size as f64) as f32,
        _ => font_size * 1.2,
    };
    let weight = match style.font_weight {
        Some(CssValue::Number(n)) => (*n as u16).min(1000),
        _ => 400,
    };
    let family = match style.font_family {
        Some(CssValue::String(s)) | Some(CssValue::Unknown(s)) => s.as_ref(),
        _ => "sans-serif",
    };
    let letter_spacing = match style.letter_spacing {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(CssValue::Number(n)) => *n as f32,
        _ => 0.0,
    };
    let word_spacing = match style.word_spacing {
        Some(CssValue::Dimension { value, unit: CssUnit::Px }) => *value as f32,
        Some(CssValue::Number(n)) => *n as f32,
        _ => 0.0,
    };
    TextStyle { font_size, line_height, font_family: family, weight, letter_spacing, word_spacing }
}

// ── Paragraph types ───────────────────────────────────────────────────

/// One span in a rich-text paragraph with uniform attributes.
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

/// Per-line metrics for a shaped paragraph.
#[derive(Debug, Clone, Copy)]
pub struct ParagraphLine {
    pub top: f32,
    pub baseline: f32,
    pub height: f32,
    pub line_width: f32,
    pub glyph_range: (usize, usize),
}

/// One contiguous advance range a source span occupies on a given line.
#[derive(Debug, Clone, Copy)]
pub struct LeafSegment {
    pub line_index: usize,
    pub x_start: f32,
    pub x_end: f32,
}

/// Shaped + atlas-packed multi-span paragraph.
#[derive(Debug, Clone, Default)]
pub struct ParagraphLayout {
    pub glyphs: Vec<PositionedGlyph>,
    pub lines: Vec<ParagraphLine>,
    pub width: f32,
    pub height: f32,
    pub first_line_ascent: f32,
    pub leaf_segments: HashMap<u32, Vec<LeafSegment>>,
}
