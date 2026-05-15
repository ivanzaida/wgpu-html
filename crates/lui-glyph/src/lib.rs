//! `lui-glyph` — text shaping and glyph atlas for the layout/render pipeline.
//!
//! `FontContext` owns the system font database and a custom font registry.
//! `TextContext` is the high-level entry point: it holds `FontContext`,
//! manages font registration, and bridges cascade `ComputedStyle` into
//! shaping calls.
//!
//! To support icon fonts (e.g. Lucide), register them via
//! `TextContext::register_font`, then use `font-family: lucide` in CSS.

pub mod atlas;
pub mod context;
pub mod font;
pub mod font_face;
pub mod font_registry;
pub mod shape;

pub use atlas::{Atlas, AtlasEntry, AtlasRect};
pub use context::{LeafSegment, ParagraphLayout, ParagraphLine, ParagraphSpan, TextContext, text_style_from_cascade};
pub use font::FontContext;
pub use font_face::{FontFace, FontHandle, FontStyleAxis};
pub use font_registry::FontRegistry;
pub use shape::{
  LineRange, PositionedGlyph, RunMetrics, ShapedLine, ShapedRun, TextStyle, parse_line_height_multiplier,
  utf8_boundaries,
};
