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

/// Per-corner radii in physical pixels: top-left, top-right, bottom-right, bottom-left.
pub type CornerRadii = [f32; 4];

/// Per-side stroke widths in physical pixels: top, right, bottom, left.
/// All zero means the quad is filled rather than stroked.
pub type StrokeWidths = [f32; 4];

/// One quad. Two modes:
/// - **Filled**: `stroke == [0; 4]`. The whole box (with rounded corners
///   from `radii`) is filled with `color`.
/// - **Stroked ring**: at least one `stroke` component > 0. The shader
///   paints only the ring between the outer rounded box and the inner
///   one, where the inner one is inset on each side by the matching
///   stroke width. `color` is used for the entire ring.
#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub rect: Rect,
    pub color: Color,
    /// `[0; 4]` for a sharp axis-aligned rectangle.
    pub radii: CornerRadii,
    /// Per-side ring thickness. `[0; 4]` → filled mode.
    pub stroke: StrokeWidths,
}

/// Flat list of paint commands. Currently just quads; later milestones
/// will add glyph runs, images, clips.
#[derive(Debug, Default, Clone)]
pub struct DisplayList {
    pub quads: Vec<Quad>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_quad(&mut self, rect: Rect, color: Color) -> &mut Self {
        self.quads.push(Quad {
            rect,
            color,
            radii: [0.0; 4],
            stroke: [0.0; 4],
        });
        self
    }

    pub fn push_quad_rounded(
        &mut self,
        rect: Rect,
        color: Color,
        radii: CornerRadii,
    ) -> &mut Self {
        self.quads.push(Quad {
            rect,
            color,
            radii,
            stroke: [0.0; 4],
        });
        self
    }

    /// Push a stroked rounded ring. `stroke` is the per-side ring
    /// thickness in pixels (top, right, bottom, left). `radii` are the
    /// outer corner radii.
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
            radii,
            stroke,
        });
        self
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }
}
