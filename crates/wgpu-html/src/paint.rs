//! Convert a laid-out box tree into a backend-agnostic display list.
//!
//! M4: walks `wgpu_html_layout::LayoutBox` and emits one solid quad per
//! box with a resolved background color.

use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_tree::Tree;

/// Convenience: cascade `tree` against any embedded `<style>` blocks,
/// lay it out at `(viewport_w × viewport_h)`, and paint the result into
/// a fresh display list.
pub fn paint_tree(tree: &Tree, viewport_w: f32, viewport_h: f32) -> DisplayList {
    let cascaded = wgpu_html_style::cascade(tree);
    let mut list = DisplayList::new();
    if let Some(root) = wgpu_html_layout::layout(&cascaded, viewport_w, viewport_h) {
        paint_box(&root, &mut list);
    }
    list
}

/// Walk a laid-out tree, pushing one quad per styled background.
pub fn paint_layout(root: &LayoutBox, list: &mut DisplayList) {
    paint_box(root, list);
}

fn paint_box(b: &LayoutBox, out: &mut DisplayList) {
    if let Some(color) = b.background {
        if b.border_rect.w > 0.0 && b.border_rect.h > 0.0 {
            out.push_quad(to_renderer_rect(b.border_rect), color);
        }
    }
    for child in &b.children {
        paint_box(child, out);
    }
}

fn to_renderer_rect(r: wgpu_html_layout::Rect) -> Rect {
    Rect::new(r.x, r.y, r.w, r.h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paints_single_styled_box() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px; background-color: red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 1);
        let q = list.quads[0];
        assert_eq!(q.rect.w, 100.0);
        assert_eq!(q.rect.h, 50.0);
    }

    #[test]
    fn skips_boxes_without_background() {
        let tree = wgpu_html_parser::parse("<div><p>hi</p></div>");
        let list = paint_tree(&tree, 800.0, 600.0);
        assert!(list.quads.is_empty());
    }

    #[test]
    fn child_uses_block_flow_position() {
        // No absolute positioning: header is at (0,0), card stacks below.
        let tree = wgpu_html_parser::parse(
            r#"<body>
                <div style="height: 64px; background-color: blue;"></div>
                <div style="height: 30px; background-color: red;"></div>
            </body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 2);
        // First (blue header) at y=0
        assert_eq!(list.quads[0].rect.y, 0.0);
        // Second (red) stacks immediately under it (no margin)
        assert_eq!(list.quads[1].rect.y, 64.0);
    }
}
