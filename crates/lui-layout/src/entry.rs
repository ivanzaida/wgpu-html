//! Top-level entry points that drive the full layout pipeline from a
//! [`CascadedTree`] to a [`LayoutBox`] tree.

use lui_style::CascadedTree;
use lui_text::TextContext;

use crate::{
    block::{layout_block, BlockOverrides},
    layout_profile::LayoutProfiler,
    types::{Ctx, LayoutBox, Rect, TextCtx},
    ImageCache,
};

/// Lay the cascaded tree out into a viewport of `viewport_w × viewport_h`
/// physical pixels, using `text_ctx` to shape any text leaves. The
/// returned root box's `margin_rect` covers the viewport.
pub fn layout_with_text(
  tree: &CascadedTree,
  text_ctx: &mut TextContext,
  image_cache: &mut ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
) -> Option<LayoutBox> {
  let default_locale = lui_tree::DefaultLocale;
  layout_with_text_locale(tree, text_ctx, image_cache, viewport_w, viewport_h, scale, &default_locale)
}

pub fn layout_with_text_locale(
  tree: &CascadedTree,
  text_ctx: &mut TextContext,
  image_cache: &mut ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  locale: &dyn lui_tree::Locale,
) -> Option<LayoutBox> {
  layout_with_text_profiled(tree, text_ctx, image_cache, viewport_w, viewport_h, scale, false, locale, None, None)
}

pub fn layout_with_text_locale_date(
  tree: &CascadedTree,
  text_ctx: &mut TextContext,
  image_cache: &mut ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  locale: &dyn lui_tree::Locale,
  date_display: Option<String>,
  date_focus_iso: Option<String>,
) -> Option<LayoutBox> {
  layout_with_text_profiled(tree, text_ctx, image_cache, viewport_w, viewport_h, scale, false, locale, date_display, date_focus_iso)
}

/// Like [`layout_with_text`] but optionally enables the layout
/// sub-profiler. When `profile` is true, prints a `[layout-profile]`
/// summary line to stderr after layout completes.
pub fn layout_with_text_profiled(
  tree: &CascadedTree,
  text_ctx: &mut TextContext,
  image_cache: &mut ImageCache,
  viewport_w: f32,
  viewport_h: f32,
  scale: f32,
  profile: bool,
  locale: &dyn lui_tree::Locale,
  date_display: Option<String>,
  date_focus_iso: Option<String>,
) -> Option<LayoutBox> {
  let root = tree.root.as_ref()?;
  let mut ctx = Ctx {
    viewport_w,
    viewport_h,
    scale,
    text: TextCtx { ctx: text_ctx },
    images: image_cache,
    locale,
    date_display_value: date_display,
    date_focus_iso,
    profiler: if profile {
      Some(LayoutProfiler::new())
    } else {
      None
    },
  };
  let result = layout_block(
    root,
    0.0,
    0.0,
    viewport_w,
    viewport_h,
    Rect::new(0.0, 0.0, viewport_w, viewport_h),
    BlockOverrides::default(),
    &mut ctx,
  );
  if let Some(p) = &ctx.profiler {
    p.dump();
  }
  Some(result)
}

/// Compatibility wrapper for callers that don't render text. Builds a
/// throw-away `TextContext` (no fonts registered → text leaves shape
/// to zero size) at scale 1.0.
pub fn layout(tree: &CascadedTree, viewport_w: f32, viewport_h: f32) -> Option<LayoutBox> {
  let mut text_ctx = TextContext::new(64);
  let mut image_cache = ImageCache::default();
  layout_with_text(tree, &mut text_ctx, &mut image_cache, viewport_w, viewport_h, 1.0)
}
