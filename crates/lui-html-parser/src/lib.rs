use std::sync::Arc;

/// Cheaply-cloneable string — `Arc<str>`.
pub type ArcStr = Arc<str>;

/// A 2D rectangle: position + size. Copy-friendly, used for layout results.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

pub mod element;
pub mod entities;
pub mod parser;
pub mod tokenizer;

pub use element::{HtmlElement, SVG_ELEMENTS, should_auto_close};
pub use parser::{HtmlDocument, HtmlNode, parse};
pub use tokenizer::{Token, tokenize};
