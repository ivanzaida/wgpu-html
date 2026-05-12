//! Geometry primitives.
//!
//! `Rect` is defined in `lui_html_parser` and re-exported here so
//! `HtmlNode.layout_rect` and `LayoutBox.content` share the same type.

pub use lui_html_parser::Rect;

/// A 2D position or offset.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }
}

/// A 2D size.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// Insets for the four sides (margin, border, padding).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RectEdges<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T> RectEdges<T> {
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self { top, right, bottom, left }
    }
}

impl RectEdges<f32> {
    pub fn horizontal(&self) -> f32 { self.left + self.right }
    pub fn vertical(&self) -> f32 { self.top + self.bottom }
}

/// Extension methods for `Rect` from `lui_html_parser`.
pub trait RectExt {
    fn max_x(&self) -> f32;
    fn max_y(&self) -> f32;
    fn contains(&self, px: f32, py: f32) -> bool;
}

impl RectExt for Rect {
    fn max_x(&self) -> f32 { self.x + self.width }
    fn max_y(&self) -> f32 { self.y + self.height }
    fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.max_x() && py >= self.y && py <= self.max_y()
    }
}
