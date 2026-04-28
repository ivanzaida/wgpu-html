//! Backend-agnostic display list. The renderer consumes this; later
//! milestones (layout/paint stages) will produce it.

/// Linear RGBA in 0..1.
pub type Color = [f32; 4];

/// Axis-aligned rectangle in physical pixels, top-left origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

/// Per-corner radii (one component per corner) in physical pixels:
/// top-left, top-right, bottom-right, bottom-left. The renderer keeps
/// horizontal and vertical radii separately on each `Quad` so corners
/// can be elliptical.
pub type CornerRadii = [f32; 4];

/// Per-side stroke widths in physical pixels: top, right, bottom, left.
/// All zero means the quad is filled rather than stroked.
pub type StrokeWidths = [f32; 4];

/// Pattern descriptor for stroked rings: `[kind, dash, gap, _pad]`.
/// - `kind`: 0 = solid, 1 = dashed, 2 = dotted.
/// - `dash`: pixel length of each painted segment (along the path).
/// - `gap`:  pixel length of each unpainted segment.
///
/// All-zero means "solid" (the shader's default).
pub type Pattern = [f32; 4];

/// Pattern kind values written into `Pattern[0]`.
#[allow(dead_code)]
pub mod pattern_kind {
    pub const SOLID: f32 = 0.0;
    pub const DASHED: f32 = 1.0;
    pub const DOTTED: f32 = 2.0;
}

/// One quad. Two modes:
/// - **Filled**: `stroke == [0; 4]`. The whole box (with rounded corners
///   from `radii_h` / `radii_v`) is filled with `color`.
/// - **Stroked ring**: at least one `stroke` component > 0. The shader
///   paints only the ring between the outer rounded box and the inner
///   one, where the inner one is inset on each side by the matching
///   stroke width. `color` is used for the entire ring.
#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub rect: Rect,
    pub color: Color,
    /// Horizontal components of the per-corner radii (TL, TR, BR, BL).
    /// `[0; 4]` → sharp rectangle.
    pub radii_h: CornerRadii,
    /// Vertical components of the per-corner radii (TL, TR, BR, BL).
    pub radii_v: CornerRadii,
    /// Per-side ring thickness. `[0; 4]` → filled mode.
    pub stroke: StrokeWidths,
    /// Stroke pattern: `(kind, dash, gap, _)`. `kind == 0` (solid) is
    /// the default and ignores the rest. Only honoured when the quad
    /// is a one-sided rounded ring (`stroke` has exactly one positive
    /// component); other configurations render solid.
    pub pattern: Pattern,
}

/// One glyph quad. The renderer's glyph pipeline samples a single
/// `R8Unorm` atlas and multiplies coverage by `color`. UVs are in
/// `[0, 1]` across the atlas.
#[derive(Debug, Clone, Copy)]
pub struct GlyphQuad {
    pub rect: Rect,
    pub color: Color,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

/// One image quad. The renderer creates a GPU texture from the
/// decoded RGBA pixels and samples it across the full `[0,1]²` UV
/// range. Each unique `image_id` maps to one GPU texture.
#[derive(Debug, Clone)]
pub struct ImageQuad {
    pub rect: Rect,
    pub opacity: f32,
    /// Unique identifier for the image data. Images with the same
    /// `image_id` share a single GPU texture.
    pub image_id: u64,
    /// Decoded RGBA8 pixel data (width × height × 4 bytes). Only
    /// consumed on the first frame an `image_id` appears; subsequent
    /// frames reuse the cached GPU texture.
    pub data: std::sync::Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

/// One scissor-tagged run inside a `DisplayList`. The list's quads
/// and glyphs are partitioned into a sequence of `ClipRange`s in
/// render order; each range is recorded as a single `draw_indexed`
/// call after `set_scissor_rect(rect)`.
///
/// When `rect` is `None`, no scissor is active (the renderer uses
/// the full viewport). Otherwise the rectangular `rect` acts as a
/// pre-discard scissor. If any of `radii_h` / `radii_v` is non-zero,
/// the fragment shader additionally discards pixels outside the
/// rounded SDF — that's how `overflow: hidden` on a box with
/// `border-radius` clips on the rounded inner-padding edge instead
/// of the rectangular bounding box.
///
/// Corner order matches CSS `border-radius` longhands:
/// `[TL, TR, BR, BL]`.
#[derive(Debug, Clone, Copy)]
pub struct ClipRange {
    pub rect: Option<Rect>,
    pub radii_h: [f32; 4],
    pub radii_v: [f32; 4],
    pub quad_range: (u32, u32),
    pub image_range: (u32, u32),
    pub glyph_range: (u32, u32),
}

impl ClipRange {
    pub fn quad_start(&self) -> u32 {
        self.quad_range.0
    }
    pub fn quad_end(&self) -> u32 {
        self.quad_range.1
    }
    pub fn image_start(&self) -> u32 {
        self.image_range.0
    }
    pub fn image_end(&self) -> u32 {
        self.image_range.1
    }
    pub fn glyph_start(&self) -> u32 {
        self.glyph_range.0
    }
    pub fn glyph_end(&self) -> u32 {
        self.glyph_range.1
    }

    /// True when at least one corner of the clip rect has a
    /// non-zero radius — i.e. the fragment shader needs to do an
    /// SDF discard, not just rely on the rectangular scissor.
    pub fn is_rounded(&self) -> bool {
        self.radii_h.iter().any(|r| *r > 0.0) || self.radii_v.iter().any(|r| *r > 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayCommandKind {
    Quad,
    Image,
    Glyph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayCommand {
    pub kind: DisplayCommandKind,
    pub index: u32,
    pub clip_index: u32,
}

/// Flat list of paint commands. Typed instance vectors are kept for GPU
/// upload efficiency; `commands` preserves cross-type paint order so a
/// later background can correctly cover earlier text.
///
/// `clips` partitions all instance vectors into render-order runs;
/// for a list with no `overflow: hidden` boxes the partition has a
/// single entry covering everything.
#[derive(Debug, Clone)]
pub struct DisplayList {
    pub quads: Vec<Quad>,
    pub images: Vec<ImageQuad>,
    pub glyphs: Vec<GlyphQuad>,
    pub clips: Vec<ClipRange>,
    pub commands: Vec<DisplayCommand>,
}

impl Default for DisplayList {
    fn default() -> Self {
        // Start with a single all-encompassing clip range. The paint
        // pass can split / nest it; producers that only push quads /
        // glyphs without ever touching `clips` keep one range that
        // grows to cover every instance.
        Self {
            quads: Vec::new(),
            images: Vec::new(),
            glyphs: Vec::new(),
            clips: vec![ClipRange {
                rect: None,
                radii_h: [0.0; 4],
                radii_v: [0.0; 4],
                quad_range: (0, 0),
                image_range: (0, 0),
                glyph_range: (0, 0),
            }],
            commands: Vec::new(),
        }
    }
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the trailing clip range to cover any quads / glyphs / images
    /// pushed since the last range opened.
    fn extend_open_range(&mut self) {
        if let Some(last) = self.clips.last_mut() {
            last.quad_range.1 = self.quads.len() as u32;
            last.image_range.1 = self.images.len() as u32;
            last.glyph_range.1 = self.glyphs.len() as u32;
        }
    }

    fn current_clip_index(&self) -> u32 {
        self.clips.len().saturating_sub(1) as u32
    }

    /// Open a new clip range with the given scissor rect and rounded
    /// corner radii. `radii_h` / `radii_v` are zeros for a plain
    /// rectangular clip; non-zero values trigger SDF discard at the
    /// fragment level so a rounded `overflow: hidden` cuts children
    /// off at the rounded inner-padding edge.
    pub fn push_clip(&mut self, rect: Option<Rect>, radii_h: [f32; 4], radii_v: [f32; 4]) {
        self.extend_open_range();
        let qs = self.quads.len() as u32;
        let is = self.images.len() as u32;
        let gs = self.glyphs.len() as u32;
        self.clips.push(ClipRange {
            rect,
            radii_h,
            radii_v,
            quad_range: (qs, qs),
            image_range: (is, is),
            glyph_range: (gs, gs),
        });
    }

    /// Close the most recently pushed clip range and return to
    /// whatever scissor was active before. Pushes a fresh range
    /// using the *enclosing* clip (so subsequent paint commands
    /// aren't clipped by the popped scope).
    pub fn pop_clip(
        &mut self,
        parent_rect: Option<Rect>,
        parent_radii_h: [f32; 4],
        parent_radii_v: [f32; 4],
    ) {
        self.extend_open_range();
        let qs = self.quads.len() as u32;
        let is = self.images.len() as u32;
        let gs = self.glyphs.len() as u32;
        self.clips.push(ClipRange {
            rect: parent_rect,
            radii_h: parent_radii_h,
            radii_v: parent_radii_v,
            quad_range: (qs, qs),
            image_range: (is, is),
            glyph_range: (gs, gs),
        });
    }

    /// Final fix-up before consumption — make sure the trailing
    /// range covers every instance.
    pub fn finalize(&mut self) {
        self.extend_open_range();
        // Drop any leading / trailing empty ranges that didn't get any
        // instances. We keep at least one range so the renderer can
        // always iterate.
        self.clips.retain(|r| {
            r.quad_range.0 != r.quad_range.1
                || r.image_range.0 != r.image_range.1
                || r.glyph_range.0 != r.glyph_range.1
        });
        if self.clips.is_empty() {
            self.clips.push(ClipRange {
                rect: None,
                radii_h: [0.0; 4],
                radii_v: [0.0; 4],
                quad_range: (0, 0),
                image_range: (0, 0),
                glyph_range: (0, 0),
            });
        }
    }

    pub fn push_quad(&mut self, rect: Rect, color: Color) -> &mut Self {
        let index = self.quads.len() as u32;
        self.quads.push(Quad {
            rect,
            color,
            radii_h: [0.0; 4],
            radii_v: [0.0; 4],
            stroke: [0.0; 4],
            pattern: [0.0; 4],
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push a filled box with circular rounded corners (`radii.h == radii.v`).
    pub fn push_quad_rounded(&mut self, rect: Rect, color: Color, radii: CornerRadii) -> &mut Self {
        let index = self.quads.len() as u32;
        self.quads.push(Quad {
            rect,
            color,
            radii_h: radii,
            radii_v: radii,
            stroke: [0.0; 4],
            pattern: [0.0; 4],
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push a filled box with arbitrary elliptical corners.
    pub fn push_quad_rounded_ellipse(
        &mut self,
        rect: Rect,
        color: Color,
        radii_h: CornerRadii,
        radii_v: CornerRadii,
    ) -> &mut Self {
        let index = self.quads.len() as u32;
        self.quads.push(Quad {
            rect,
            color,
            radii_h,
            radii_v,
            stroke: [0.0; 4],
            pattern: [0.0; 4],
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push a stroked rounded ring with circular corners.
    pub fn push_quad_stroke(
        &mut self,
        rect: Rect,
        color: Color,
        radii: CornerRadii,
        stroke: StrokeWidths,
    ) -> &mut Self {
        let index = self.quads.len() as u32;
        self.quads.push(Quad {
            rect,
            color,
            radii_h: radii,
            radii_v: radii,
            stroke,
            pattern: [0.0; 4],
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push a stroked rounded ring with arbitrary elliptical corners.
    pub fn push_quad_stroke_ellipse(
        &mut self,
        rect: Rect,
        color: Color,
        radii_h: CornerRadii,
        radii_v: CornerRadii,
        stroke: StrokeWidths,
    ) -> &mut Self {
        let index = self.quads.len() as u32;
        self.quads.push(Quad {
            rect,
            color,
            radii_h,
            radii_v,
            stroke,
            pattern: [0.0; 4],
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push a stroked rounded ring with a dash/dot pattern. The shader
    /// only honours the pattern on one-sided rings (i.e. exactly one
    /// stroke component > 0); for any other configuration the pattern
    /// is ignored and the ring renders solid.
    pub fn push_quad_stroke_patterned(
        &mut self,
        rect: Rect,
        color: Color,
        radii_h: CornerRadii,
        radii_v: CornerRadii,
        stroke: StrokeWidths,
        pattern: Pattern,
    ) -> &mut Self {
        let index = self.quads.len() as u32;
        self.quads.push(Quad {
            rect,
            color,
            radii_h,
            radii_v,
            stroke,
            pattern,
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Quad,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push one glyph quad. The renderer's glyph pipeline samples the
    /// shared atlas at `[uv_min, uv_max]` and multiplies coverage by
    /// `color`.
    pub fn push_glyph(
        &mut self,
        rect: Rect,
        color: Color,
        uv_min: [f32; 2],
        uv_max: [f32; 2],
    ) -> &mut Self {
        let index = self.glyphs.len() as u32;
        self.glyphs.push(GlyphQuad {
            rect,
            color,
            uv_min,
            uv_max,
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Glyph,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    /// Push one image quad. The renderer will create a GPU texture
    /// from `data` on first use and cache it by `image_id`.
    pub fn push_image(
        &mut self,
        rect: Rect,
        image_id: u64,
        data: std::sync::Arc<Vec<u8>>,
        width: u32,
        height: u32,
    ) -> &mut Self {
        self.push_image_with_opacity(rect, image_id, data, width, height, 1.0)
    }

    /// Push one image quad with a subtree opacity multiplier.
    pub fn push_image_with_opacity(
        &mut self,
        rect: Rect,
        image_id: u64,
        data: std::sync::Arc<Vec<u8>>,
        width: u32,
        height: u32,
        opacity: f32,
    ) -> &mut Self {
        let index = self.images.len() as u32;
        self.images.push(ImageQuad {
            rect,
            opacity: opacity.clamp(0.0, 1.0),
            image_id,
            data,
            width,
            height,
        });
        self.commands.push(DisplayCommand {
            kind: DisplayCommandKind::Image,
            index,
            clip_index: self.current_clip_index(),
        });
        self
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.images.clear();
        self.glyphs.clear();
        self.clips.clear();
        self.commands.clear();
        self.clips.push(ClipRange {
            rect: None,
            radii_h: [0.0; 4],
            radii_v: [0.0; 4],
            quad_range: (0, 0),
            image_range: (0, 0),
            glyph_range: (0, 0),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_commands_preserve_cross_type_push_order() {
        let mut list = DisplayList::new();
        list.push_glyph(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
        );
        list.push_quad(Rect::new(0.0, 0.0, 20.0, 20.0), [1.0, 0.0, 0.0, 1.0]);
        list.push_glyph(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            [1.0, 1.0, 1.0, 1.0],
            [0.0, 0.0],
            [1.0, 1.0],
        );

        assert_eq!(
            list.commands,
            vec![
                DisplayCommand {
                    kind: DisplayCommandKind::Glyph,
                    index: 0,
                    clip_index: 0,
                },
                DisplayCommand {
                    kind: DisplayCommandKind::Quad,
                    index: 0,
                    clip_index: 0,
                },
                DisplayCommand {
                    kind: DisplayCommandKind::Glyph,
                    index: 1,
                    clip_index: 0,
                },
            ]
        );
    }
}
