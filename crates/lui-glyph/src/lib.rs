//! `lui-glyph` — text shaping and glyph atlas for the layout/render pipeline.
//!
//! `FontContext` owns the system font database. `shape()` produces
//! `ShapedRun` with positioned glyphs. `break_into_lines()` handles
//! line breaking.
//!
//! The layout engine extracts font properties from `ComputedStyle` and
//! passes them as `TextStyle` to avoid coupling cascade ↔ glyph.

pub mod font;
pub mod shape;

pub use font::FontContext;
pub use shape::{PositionedGlyph, ShapedLine, ShapedRun, TextStyle};
