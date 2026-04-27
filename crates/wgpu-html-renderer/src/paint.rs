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

/// One filled rectangle.
#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub rect: Rect,
    pub color: Color,
}

/// Flat list of paint commands. For M2 this is just quads; later
/// milestones will add glyph runs, images, clips, borders.
#[derive(Debug, Default, Clone)]
pub struct DisplayList {
    pub quads: Vec<Quad>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_quad(&mut self, rect: Rect, color: Color) -> &mut Self {
        self.quads.push(Quad { rect, color });
        self
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }
}
