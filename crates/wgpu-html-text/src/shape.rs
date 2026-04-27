//! Shape + raster + atlas-pack pipeline.
//!
//! `TextContext` is the long-lived, per-host text state. It owns the
//! cosmic-text shaper bridge (`FontDb`), a CPU-side R8 glyph atlas
//! (`Atlas`), and cosmic-text's own `SwashCache` for glyph rasters.
//! Layout passes a `&mut TextContext` whenever it needs to measure /
//! shape a text node.
//!
//! For each shaped glyph we:
//!   1. Look up its `(font, glyph_id, sub-pixel offset, size)`
//!      cache key in our local atlas map.
//!   2. On miss, ask cosmic-text's SwashCache for the alpha mask,
//!      pack it into `Atlas`, and remember the resulting
//!      `AtlasRect`.
//!   3. Emit a `PositionedGlyph` with the run-relative pixel rect
//!      and the atlas UVs.
//!
//! The atlas's `flush_dirty` / `upload` paths are how the renderer
//! sees newly-inserted glyphs each frame.

use std::collections::HashMap;

use cosmic_text::{Attrs, Buffer, CacheKey, Metrics, Shaping};

use wgpu_html_tree::{FontHandle, FontRegistry, FontStyleAxis};

use crate::atlas::{Atlas, AtlasRect};
use crate::font_db::FontDb;

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

/// Output of `shape_and_pack`. Holds the line metrics callers need to
/// place the run vertically inside a block (height + ascent), the
/// total advance width, and the glyph quads.
#[derive(Debug, Clone, Default)]
pub struct ShapedRun {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
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
        }
    }

    /// Reconcile the cosmic-text font system against `registry`.
    /// Stores a clone of the registry so `pick_font` can do
    /// CSS-aware family / weight / style matching without a fresh
    /// borrow from the host.
    pub fn sync_fonts(&mut self, registry: &FontRegistry) {
        self.fonts = registry.clone();
        self.font_db.sync(registry);
    }

    /// Pick a `FontHandle` for a CSS `font-family` list, weight, and
    /// style. Walks the comma-separated family list left-to-right,
    /// returning the first match per CSS-Fonts-3-style scoring (see
    /// `wgpu_html_tree::FontRegistry::find`). Falls back to the
    /// first registered face if no listed family matches; returns
    /// `None` only when the registry is empty.
    pub fn pick_font(
        &self,
        families: &[&str],
        weight: u16,
        style: FontStyleAxis,
    ) -> Option<FontHandle> {
        self.fonts
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
    pub fn resolve_family(
        &mut self,
        families: &[&str],
        weight: u16,
        style: FontStyleAxis,
    ) -> Option<String> {
        let handle = self.pick_font(families, weight, style)?;
        let fontdb_id = self.font_db.fontdb_id(handle)?;
        let db = self.font_db.font_system_mut().db_mut();
        let face = db.face(fontdb_id)?;
        face.families.first().map(|(name, _)| name.clone())
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
        let fontdb_id = self.font_db.fontdb_id(font)?;

        // cosmic-text needs the family name to find the face. We have
        // the fontdb ID from earlier — fish the family back out.
        let family_name = {
            let db = self.font_db.font_system_mut().db_mut();
            let face = db.face(fontdb_id)?;
            // `face.families` is `Vec<(String, Language)>`; take the
            // first family name.
            face.families
                .first()
                .map(|(name, _)| name.clone())?
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
        buffer.set_text(
            self.font_db.font_system_mut(),
            text,
            attrs,
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(self.font_db.font_system_mut(), false);

        // Snapshot every layout run (one per line). We keep the
        // (physical, x, w, line_y, line_h) tuples up-front so the
        // glyph loop below can borrow `&mut self.font_db` /
        // `&mut self.swash` without colliding with the `Buffer`'s
        // outstanding `&mut FontSystem`.
        let layout_lines: Vec<(Vec<(cosmic_text::PhysicalGlyph, f32, f32)>, f32, f32)> = buffer
            .layout_runs()
            .map(|run| {
                let line_y = run.line_y;
                let line_h = run.line_height;
                let glyphs: Vec<_> = run
                    .glyphs
                    .iter()
                    .map(|g| (g.physical((0.0, 0.0), 1.0), g.x, g.w))
                    .collect();
                (glyphs, line_y, line_h)
            })
            .collect();

        let first_line = layout_lines.first()?;
        let ascent_px = first_line.1;
        let total_height: f32 = layout_lines.iter().map(|(_, _, h)| *h).sum();

        let glyph_capacity: usize = layout_lines.iter().map(|(g, _, _)| g.len()).sum();
        let mut glyphs: Vec<PositionedGlyph> = Vec::with_capacity(glyph_capacity);
        let mut max_x: f32 = 0.0;
        let (atlas_w, atlas_h) = self.atlas.dimensions();

        for (line_glyphs, line_y, _line_h) in &layout_lines {
            let baseline_y = *line_y;
            for (glyph_index, (physical, layout_x, layout_w)) in line_glyphs.iter().enumerate() {
            // Cumulative `letter-spacing` offset for this glyph (zero
            // for the first one, then `letter_spacing_px` per logical
            // glyph step).
            let spacing_dx = (glyph_index as f32) * letter_spacing_px;
            let key = physical.cache_key;
            let entry = match self.glyph_cache.get(&key).copied() {
                Some(e) => e,
                None => {
                    let Some(image) =
                        self.swash.get_image_uncached(
                            self.font_db.font_system_mut(),
                            key,
                        )
                    else {
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
            let pos_x = (physical.x as f32 + entry.left as f32 + spacing_dx).round();
            let pos_y = (baseline_y - entry.top as f32).round();

            let quad_w = entry.w as f32;
            let quad_h = entry.h as f32;

            if entry.w > 0 && entry.h > 0 {
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
            }

            // The line's used width is the right edge of the last
            // glyph's advance (with letter-spacing). `max_x` tracks
            // the widest line.
            let advance_right = layout_x + layout_w + spacing_dx;
            if advance_right > max_x {
                max_x = advance_right;
            }
            }
        }

        Some(ShapedRun {
            glyphs,
            width: max_x,
            height: total_height,
            ascent: ascent_px,
        })
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
    pub fn shape_paragraph(
        &mut self,
        spans: &[ParagraphSpan<'_>],
        max_width_px: Option<f32>,
    ) -> Option<ParagraphLayout> {
        if spans.is_empty() {
            return None;
        }

        // Buffer-default Metrics: pick the largest span size so a
        // mixed paragraph with `<small>` doesn't constrain the line
        // height of the regular text. Per-span Attrs.metrics_opt
        // overrides where each glyph actually sits.
        let default_size = spans
            .iter()
            .map(|s| s.size_px)
            .fold(0.0_f32, f32::max)
            .max(1.0);
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
                        let Some(image) = self
                            .swash
                            .get_image_uncached(self.font_db.font_system_mut(), key)
                        else {
                            continue;
                        };
                        let w = image.placement.width;
                        let h = image.placement.height;
                        let rect = if w == 0 || h == 0 {
                            AtlasRect {
                                x: 0,
                                y: 0,
                                w: 0,
                                h: 0,
                            }
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
                let pos_y = (line.baseline - entry.top as f32).round();
                let quad_w = entry.w as f32;
                let quad_h = entry.h as f32;

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
}
