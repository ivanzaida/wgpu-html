//! Text shaping + glyph atlas, fed by the host-registered fonts on a
//! `Tree`. See `docs/text.md` for the broader plan.
//!
//! Scope (T2):
//! - `Atlas` — CPU-side R8 glyph atlas with a shelf packer and a
//!   pending dirty-rect list. `flush_dirty` drains those into a
//!   caller-supplied sink; `upload` is a thin wgpu wrapper.
//! - `FontDb` — registry-keyed cache placeholder. Built from a
//!   `&FontRegistry`; remembers which `Arc<[u8]>` payloads it has
//!   already ingested so a re-cascade against the same registry
//!   doesn't reload anything. The actual cosmic-text bridge lands in
//!   T3 — the cache shape is set up here so adding it doesn't shift
//!   the public API.
//!
//! No GPU pipeline yet: the renderer crate is untouched in T2.

mod atlas;
mod font_db;
mod shape;

pub use atlas::{Atlas, AtlasEntry, AtlasRect};
pub use font_db::FontDb;
pub use shape::{
    LeafSegment, ParagraphLayout, ParagraphLine, ParagraphSpan, PositionedGlyph, ShapedLine,
    ShapedRun, TextContext, parse_line_height_multiplier, utf8_boundaries,
};

// Re-export the host-facing font types so callers don't need to depend
// on `wgpu-html-tree` just to talk to the text crate.
pub use wgpu_html_tree::{FontFace, FontHandle, FontRegistry, FontStyleAxis};
