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

use lui_display_list::DisplayList;
use lui_glyph::TextContext;
use lui_layout::LayoutTree;

/// Paint a layout tree into a display list.
///
/// Walks the `LayoutBox` tree, emitting `Quad`/`GlyphQuad`/`ImageQuad`
/// primitives with proper clipping, z-ordering, and scroll handling.
/// The caller owns the `TextContext` (font/atlas system).
pub fn paint(tree: &LayoutTree<'_>, text_ctx: &mut TextContext) -> DisplayList {
    let mut dl = DisplayList::new();

    dl.canvas_color = extract_canvas_color(tree);

    let mut clip_stack = Vec::new();
    walk::paint_box(&tree.root, &mut dl, &mut clip_stack, text_ctx, 0.0, 0.0, 1.0);

    dl.finalize();
    dl
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
