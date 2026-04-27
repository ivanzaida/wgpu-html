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
    let rect = to_renderer_rect(b.border_rect);
    let radii = corner_radii(b);
    let rounded = has_any_radius(&radii);

    // Background fills the border-box (default `background-clip: border-box`).
    if let Some(color) = b.background {
        if b.border_rect.w > 0.0 && b.border_rect.h > 0.0 {
            if rounded {
                out.push_quad_rounded(rect, color, radii);
            } else {
                out.push_quad(rect, color);
            }
        }
    }

    // Borders: when the box has any rounded corner AND a uniform border
    // colour we can paint the whole stroked ring as a single rounded
    // quad. Otherwise (sharp corners, or per-side colours) fall back to
    // emitting up to four sharp edge quads.
    if rounded {
        if let Some(color) = uniform_border_color(b) {
            let stroke = [b.border.top, b.border.right, b.border.bottom, b.border.left];
            if stroke.iter().any(|s| *s > 0.0) {
                out.push_quad_stroke(rect, color, radii, stroke);
            }
        } else {
            // Per-side colours with rounded corners are not yet
            // supported — best we can do for now is paint the 4 edges
            // sharply on top of the rounded background.
            paint_border_edges(b, out);
        }
    } else {
        paint_border_edges(b, out);
    }

    for child in &b.children {
        paint_box(child, out);
    }
}

/// If every set border side shares the same colour (and any unset side
/// has zero thickness so it contributes nothing), return that colour.
/// Otherwise return `None`, telling the caller to paint per-side edges.
fn uniform_border_color(b: &LayoutBox) -> Option<wgpu_html_renderer::Color> {
    let bd = b.border;
    let bc = b.border_colors;
    let mut chosen: Option<wgpu_html_renderer::Color> = None;
    let pairs = [
        (bd.top, bc.top),
        (bd.right, bc.right),
        (bd.bottom, bc.bottom),
        (bd.left, bc.left),
    ];
    for (w, c) in pairs {
        if w <= 0.0 {
            continue;
        }
        let c = c?;
        match chosen {
            None => chosen = Some(c),
            Some(existing) if existing == c => {}
            Some(_) => return None,
        }
    }
    chosen
}

fn corner_radii(b: &LayoutBox) -> [f32; 4] {
    [
        b.border_radius.top_left,
        b.border_radius.top_right,
        b.border_radius.bottom_right,
        b.border_radius.bottom_left,
    ]
}

fn has_any_radius(r: &[f32; 4]) -> bool {
    r.iter().any(|v| *v > 0.0)
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
    fn radii_carry_through_to_display_list() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-radius: 1px 2px 3px 4px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 1);
        let q = list.quads[0];
        // Order: TL, TR, BR, BL.
        assert_eq!(q.radii, [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn rounded_uniform_border_emits_single_ring_quad() {
        // border-radius + uniform `border:` → one stroked rounded ring,
        // not four sharp edge quads.
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             border: 1px solid grey;
                             border-radius: 16px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 1);
        let q = list.quads[0];
        assert_eq!(q.radii, [16.0; 4]);
        assert_eq!(q.stroke, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn rounded_with_background_and_border_emits_two_quads() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border: 2px solid blue;
                             border-radius: 8px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        // 1 rounded background + 1 ring border = 2 quads.
        assert_eq!(list.quads.len(), 2);
        // Background is the first push and has no stroke.
        assert_eq!(list.quads[0].stroke, [0.0; 4]);
        assert_eq!(list.quads[1].stroke, [2.0, 2.0, 2.0, 2.0]);
    }

    #[test]
    fn rounded_with_per_side_colors_falls_back_to_sharp_edges() {
        // We don't support per-side colours with rounded corners yet,
        // so the four sharp edge quads are emitted on top of the
        // rounded background instead of one ring.
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-color: red green blue orange;
                             border-radius: 8px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        // 1 rounded background + 4 sharp edges
        assert_eq!(list.quads.len(), 5);
    }

    #[test]
    fn sharp_box_border_still_uses_four_edges() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             border: 2px solid red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 4);
        for q in &list.quads {
            assert_eq!(q.stroke, [0.0; 4]);
        }
    }

    #[test]
    fn no_radius_keeps_sharp_quad() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 1);
        assert_eq!(list.quads[0].radii, [0.0; 4]);
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
