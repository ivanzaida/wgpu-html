//! Block layout.
//!
//! Walks a `CascadedTree` (one Style per node, already cascaded) and
//! produces a `LayoutBox` tree positioned in physical pixels. The renderer
//! consumes the result directly — it never re-resolves CSS.
//!
//! Scope (M4):
//! - Block formatting context only: every element stacks vertically inside its parent's content box.
//! - Margin and padding (per-side or shorthand) are honoured.
//! - Width auto-fills the parent's content width; height auto-fits content.
//! - Borders are not drawn yet (treated as zero); inline / flex / floats come in later milestones.
//! - Text nodes contribute zero height; M5 brings real text layout.

pub use lui_assets::{current_frame, AssetIo, Fetcher, ImageData, ImageFrame};
pub type ImageCache = AssetIo<lui_assets::blocking::BlockingFetcher>;

pub use color::{resolve_color, resolve_with_current, Color};
pub use lui_models::common::css_enums::{Cursor, PointerEvents, Resize, TextOverflow, UserSelect, VerticalAlign, WordBreak};
pub use lui_models::Style;
pub use lui_models::common::css_enums::WhiteSpace;
pub use lui_text::{PositionedGlyph, ShapedRun};

#[cfg(test)]
use lui_style::CascadedTree;

pub mod color;
pub mod types;
pub use types::*;

pub(crate) mod layout_profile;
pub(crate) mod background;
pub(crate) mod box_model;
pub(crate) use box_model::*;
pub(crate) mod positioned;
pub(crate) use positioned::*;
pub(crate) mod text_shaping;
pub(crate) use text_shaping::*;
pub(crate) mod inline;
pub(crate) use inline::*;
pub(crate) mod form_controls;
pub(crate) use form_controls::*;
pub(crate) mod entry;
pub(crate) mod block;

mod flex;
mod gradient;
mod grid;
mod length;
mod svg;
mod table;
mod hit_test;

mod incremental;
pub use incremental::{
  layout_incremental, padding_box_rect, patch_form_controls,
  patch_layout_colors, resolve_lui_properties, lui_popup_from_pseudo,
  lui_color_from_pseudo, lui_calendar_from_pseudo, file_button_from_pseudo,
};

// Crate-root re-exports — needed by flex, grid, table, positioned, etc.
pub(crate) use block::{layout_block, layout_block_at_with, BlockOverrides};
pub(crate) use types::Ctx;
pub(crate) use types::TextCtx;

// Public entry points.
pub use entry::*;

#[cfg(test)]
mod tests;
