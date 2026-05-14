mod background;
mod border;
mod clip;
pub mod color;
mod convert;
mod scrollbar;
mod shadow;
pub mod style;
mod text;
mod walk;

use lui_core::display_list::DisplayList;
use lui_glyph::TextContext;
use lui_layout::LayoutTree;

/// Paint a layout tree into a display list.
///
/// Walks the `LayoutBox` tree, emitting `Quad`/`GlyphQuad`/`ImageQuad`
/// primitives with proper clipping, z-ordering, and scroll handling.
/// The caller owns the `TextContext` (font/atlas system).
pub fn paint(tree: &LayoutTree<'_>, text_ctx: &mut TextContext) -> DisplayList {
  paint_scaled(tree, text_ctx, 1.0)
}

pub fn paint_scaled(tree: &LayoutTree<'_>, text_ctx: &mut TextContext, dpi_scale: f32) -> DisplayList {
  let mut dl = DisplayList::new();

  dl.canvas_color = extract_canvas_color(tree);

  let mut clip_stack = Vec::new();
  walk::paint_box(&tree.root, &mut dl, &mut clip_stack, text_ctx, 0.0, 0.0, 1.0, dpi_scale);

  dl.finalize();
  dl
}

pub fn paint_viewport_scrollbars(
  dl: &mut DisplayList,
  tree: &LayoutTree<'_>,
  viewport_width: f32,
  viewport_height: f32,
  scroll_x: f32,
  scroll_y: f32,
) {
  scrollbar::paint_viewport_scrollbars(tree, viewport_width, viewport_height, scroll_x, scroll_y, dl);
}

fn extract_canvas_color(tree: &LayoutTree) -> Option<[f32; 4]> {
  let root = &tree.root;
  if let Some(c) = style::css_color(root.style.background_color) {
    return Some(c);
  }
  if let Some(body) = root.children.first() {
    if let Some(c) = style::css_color(body.style.background_color) {
      return Some(c);
    }
  }
  None
}
