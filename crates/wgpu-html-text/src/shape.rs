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

use wgpu_html_tree::{FontHandle, FontRegistry};

use crate::atlas::{Atlas, AtlasRect};
use crate::font_db::FontDb;

/// One glyph after shaping + atlas packing. Positions are run-relative
/// pixel coordinates with `(0, 0)` at the top-left of the line box.
#[derive(Debug, Clone, Copy)]
pub struct PositionedGlyph {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
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
            atlas: Atlas::new(atlas_size, atlas_size),
            swash: cosmic_text::SwashCache::new(),
            glyph_cache: HashMap::new(),
        }
    }

    /// Reconcile the cosmic-text font system against `registry`.
    /// Convenience that delegates to `font_db.sync`.
    pub fn sync_fonts(&mut self, registry: &FontRegistry) {
        self.font_db.sync(registry);
    }

    /// Shape `text` using the registered face `font`, at `size_px`
    /// font size, with `line_height_px` total line box height. Returns
    /// `None` if the font isn't loaded into the bridge.
    ///
    /// For T3 the buffer is laid out at unbounded width (no breaks)
    /// and only the first layout run is read. Multi-line breaking is
    /// T4's job.
    pub fn shape_and_pack(
        &mut self,
        text: &str,
        font: FontHandle,
        size_px: f32,
        line_height_px: f32,
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
        let attrs = Attrs::new()
            .family(cosmic_text::Family::Name(&family_name));

        // Build a single-line BufferLine directly so we get full control
        // over the shaping and don't pay for cosmic-text's
        // wrap-friendly Buffer machinery on a known single-line run.
        // We still use `Buffer` because that's what shapes against the
        // FontSystem in 0.12.
        let mut buffer = Buffer::new(self.font_db.font_system_mut(), metrics);
        // Unbounded width → single line for the whole run. Height is
        // the line height; cosmic-text produces only the first run.
        buffer.set_size(self.font_db.font_system_mut(), None, None);
        buffer.set_text(
            self.font_db.font_system_mut(),
            text,
            attrs,
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(self.font_db.font_system_mut(), false);

        // First (and only) layout run.
        let run = buffer.layout_runs().next()?;
        let ascent_px = run.line_y; // baseline measured from line top
        let line_h = run.line_height;

        let mut glyphs: Vec<PositionedGlyph> = Vec::with_capacity(run.glyphs.len());
        let mut max_x: f32 = 0.0;
        let (atlas_w, atlas_h) = self.atlas.dimensions();

        // Collect glyph data without holding both `&mut self.swash` and
        // `&mut self.font_db` borrows simultaneously across the closure.
        let layout_glyphs: Vec<_> = run
            .glyphs
            .iter()
            .map(|g| (g.physical((0.0, 0.0), 1.0), g.x, g.w))
            .collect();

        for (physical, layout_x, layout_w) in layout_glyphs {
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

            // Position relative to the run origin (line top-left).
            // `physical.x/y` are integer-snapped advances from the run
            // origin; add the bitmap bearing to get the quad top-left
            // corner. The vertical position depends on `ascent_px`
            // which is `run.line_y` from cosmic-text and routinely
            // fractional — round both coords so the quad lands on the
            // pixel grid. Without this, the linear sampler blends the
            // mask with the zeroed padding rows above/below it, and
            // the entire glyph reads as low-coverage grey.
            let pos_x = (physical.x as f32 + entry.left as f32).round();
            let pos_y = (ascent_px - entry.top as f32).round();

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
                });
            }

            // Run width tracks the right edge of the *advance*, not
            // the rasterised glyph. `layout_x + layout_w` matches
            // cosmic-text's pen position after this glyph.
            let advance_right = layout_x + layout_w;
            if advance_right > max_x {
                max_x = advance_right;
            }
        }

        Some(ShapedRun {
            glyphs,
            width: max_x,
            height: line_h,
            ascent: ascent_px,
        })
    }
}
