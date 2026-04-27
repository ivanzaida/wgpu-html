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

/// Flat list of paint commands. The renderer draws `quads` first, then
/// `glyphs` on top, in source order within each list.
#[derive(Debug, Default, Clone)]
pub struct DisplayList {
    pub quads: Vec<Quad>,
    pub glyphs: Vec<GlyphQuad>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_quad(&mut self, rect: Rect, color: Color) -> &mut Self {
        self.quads.push(Quad {
            rect,
            color,
            radii_h: [0.0; 4],
            radii_v: [0.0; 4],
            stroke: [0.0; 4],
            pattern: [0.0; 4],
        });
        self
    }

    /// Push a filled box with circular rounded corners (`radii.h == radii.v`).
    pub fn push_quad_rounded(
        &mut self,
        rect: Rect,
        color: Color,
        radii: CornerRadii,
    ) -> &mut Self {
        self.quads.push(Quad {
            rect,
            color,
            radii_h: radii,
            radii_v: radii,
            stroke: [0.0; 4],
            pattern: [0.0; 4],
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
        self.quads.push(Quad {
            rect,
            color,
            radii_h,
            radii_v,
            stroke: [0.0; 4],
            pattern: [0.0; 4],
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
        self.quads.push(Quad {
            rect,
            color,
            radii_h: radii,
            radii_v: radii,
            stroke,
            pattern: [0.0; 4],
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
        self.quads.push(Quad {
            rect,
            color,
            radii_h,
            radii_v,
            stroke,
            pattern: [0.0; 4],
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
        self.quads.push(Quad {
            rect,
            color,
            radii_h,
            radii_v,
            stroke,
            pattern,
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
        self.glyphs.push(GlyphQuad {
            rect,
            color,
            uv_min,
            uv_max,
        });
        self
    }

    pub fn clear(&mut self) {
        self.quads.clear();
        self.glyphs.clear();
    }
}
