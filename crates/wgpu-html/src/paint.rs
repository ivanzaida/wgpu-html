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
    // Background fills the border-box (default `background-clip: border-box`).
    if let Some(color) = b.background {
        if b.border_rect.w > 0.0 && b.border_rect.h > 0.0 {
            out.push_quad(to_renderer_rect(b.border_rect), color);
        }
    }

    // Borders: 4 solid-color edge quads, painted on top of the background.
    paint_border_edges(b, out);

    for child in &b.children {
        paint_box(child, out);
    }
}

/// Emit up to four solid edge quads for the box border. Each side is
/// independently coloured: a side is skipped if its thickness is zero
/// or its colour is unset (we don't track foreground color yet for the
/// spec-default fallback).
fn paint_border_edges(b: &LayoutBox, out: &mut DisplayList) {
    let r = b.border_rect;
    let bd = b.border;
    if r.w <= 0.0 || r.h <= 0.0 || !b.border_colors.any() {
        return;
    }
    let bc = b.border_colors;

    // Top edge spans the full width; left/right edges sit between top
    // and bottom so the corners (which belong to the longer-axis edges
    // top/bottom by convention) are filled exactly once.
    if bd.top > 0.0 {
        if let Some(c) = bc.top {
            out.push_quad(Rect::new(r.x, r.y, r.w, bd.top), c);
        }
    }
    if bd.bottom > 0.0 {
        if let Some(c) = bc.bottom {
            out.push_quad(Rect::new(r.x, r.y + r.h - bd.bottom, r.w, bd.bottom), c);
        }
    }
    let inner_h = (r.h - bd.top - bd.bottom).max(0.0);
    if bd.left > 0.0 && inner_h > 0.0 {
        if let Some(c) = bc.left {
            out.push_quad(Rect::new(r.x, r.y + bd.top, bd.left, inner_h), c);
        }
    }
    if bd.right > 0.0 && inner_h > 0.0 {
        if let Some(c) = bc.right {
            out.push_quad(
                Rect::new(r.x + r.w - bd.right, r.y + bd.top, bd.right, inner_h),
                c,
            );
        }
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
    fn border_emits_four_edge_quads() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             border: 2px solid red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        // No background → 4 border edges only.
        assert_eq!(list.quads.len(), 4);
    }

    #[test]
    fn border_with_background_emits_five_quads() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: blue;
                             border: 2px solid red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 5);
    }

    #[test]
    fn border_with_no_color_is_skipped() {
        // border-width set, but no color → we don't paint edges (no
        // foreground-color fallback yet).
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: blue;
                             border-width: 2px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 1);
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
