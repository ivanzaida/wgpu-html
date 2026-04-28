//! Top-level facade for the wgpu-html stack.
//!
//! Re-exports the model types and the renderer so downstream apps only need
//! one dependency.

pub use wgpu_html_layout as layout;
pub use wgpu_html_models as models;
pub use wgpu_html_parser as parser;
pub use wgpu_html_renderer as renderer;
pub use wgpu_html_style as style;
pub use wgpu_html_tree as tree;

pub use wgpu_html_text as text;

pub mod interactivity;
pub mod paint;
pub use paint::{paint_tree, paint_tree_with_text};

use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::DisplayList;
use wgpu_html_text::TextContext;
use wgpu_html_tree::Tree;

/// Cascade + lay out `tree` against `text_ctx` and return the
/// resulting `LayoutBox` without painting. Hosts that need the layout
/// for hit-testing (e.g. dispatching pointer events between frames)
/// pair this with [`paint::paint_layout`] to render.
pub fn compute_layout(
    tree: &Tree,
    text_ctx: &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale: f32,
) -> Option<LayoutBox> {
    text_ctx.sync_fonts(&tree.fonts);
    if let Some(ttl) = tree.asset_cache_ttl {
        wgpu_html_layout::set_image_cache_ttl(ttl);
    }
    for url in &tree.preload_queue {
        wgpu_html_layout::preload_image(url);
    }
    let cascaded = wgpu_html_style::cascade(tree);
    wgpu_html_layout::layout_with_text(&cascaded, text_ctx, viewport_w, viewport_h, scale)
}

/// Convenience: [`compute_layout`] + [`paint::paint_layout`] in one
/// call, returning both. The display list is finalised; the layout
/// can be retained for the next frame's hit-testing.
pub fn paint_tree_returning_layout(
    tree: &Tree,
    text_ctx: &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale: f32,
) -> (DisplayList, Option<LayoutBox>) {
    let layout = compute_layout(tree, text_ctx, viewport_w, viewport_h, scale);
    let mut list = DisplayList::new();
    if let Some(root) = layout.as_ref() {
        paint::paint_layout(root, &mut list);
    } else {
        list.finalize();
    }
    (list, layout)
}
