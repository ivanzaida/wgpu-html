//! Convert a laid-out box tree into a backend-agnostic display list.
//!
//! M4: walks `wgpu_html_layout::LayoutBox` and emits one solid quad per
//! box with a resolved background color.

use wgpu_html_layout::LayoutBox;
use wgpu_html_renderer::{DisplayList, Rect};
use wgpu_html_text::TextContext;
use wgpu_html_tree::Tree;

/// Convenience: cascade `tree` against any embedded `<style>` blocks,
/// lay it out at `(viewport_w × viewport_h)`, and paint the result into
/// a fresh display list. No text rendering — text leaves contribute
/// zero size. Use [`paint_tree_with_text`] when fonts are registered.
pub fn paint_tree(tree: &Tree, viewport_w: f32, viewport_h: f32) -> DisplayList {
    let mut ctx = TextContext::new(64);
    paint_tree_with_text(tree, &mut ctx, viewport_w, viewport_h, 1.0)
}

/// Cascade + lay out + paint, threading a long-lived `TextContext`
/// through. Syncs the context's font db against `tree.fonts` first so
/// any newly-registered face is loaded before shaping.
///
/// `scale` is the CSS-px → physical-px factor (winit's `scale_factor`).
pub fn paint_tree_with_text(
    tree: &Tree,
    text_ctx: &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale: f32,
) -> DisplayList {
    text_ctx.sync_fonts(&tree.fonts);
    let cascaded = wgpu_html_style::cascade(tree);
    let mut list = DisplayList::new();
    if let Some(root) = wgpu_html_layout::layout_with_text(
        &cascaded,
        text_ctx,
        viewport_w,
        viewport_h,
        scale,
    ) {
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
    let (rh, rv) = corner_radii(b);
    let rounded = has_any_radius(&rh) || has_any_radius(&rv);

    // Background paints into the rectangle picked by `background-clip`
    // (border-box by default; padding-box / content-box also supported).
    if let Some(color) = b.background {
        let bg = to_renderer_rect(b.background_rect);
        if bg.w > 0.0 && bg.h > 0.0 {
            let (bg_h, bg_v) = corner_radii_from(&b.background_radii);
            if has_any_radius(&bg_h) || has_any_radius(&bg_v) {
                out.push_quad_rounded_ellipse(bg, color, bg_h, bg_v);
            } else {
                out.push_quad(bg, color);
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
                out.push_quad_stroke_ellipse(rect, color, rh, rv, stroke);
            }
        } else {
            // Mixed colours / styles on a rounded box: emit each solid
            // side as its own one-sided ring quad so the corners follow
            // the rounded path. Sides set to none / hidden are skipped;
            // dashed / dotted on rounded boxes are still rendered as
            // sharp segments — they overlap the rounded path slightly
            // at the corners (acknowledged limitation).
            paint_rounded_per_side_borders(b, rect, rh, rv, out);
        }
    } else {
        paint_border_edges(b, out);
    }

    // Text leaves: emit one glyph quad per shaped glyph, positioned
    // relative to the text box's content origin. Glyph UVs were
    // computed at shaping time; the renderer samples its single R8
    // atlas with them.
    if let Some(run) = &b.text_run {
        let color = b.text_color.unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let origin = b.content_rect;

        // Decorations sit relative to the run's baseline, behind the
        // glyphs (under-line / line-through draw under the strokes;
        // over-line above). Stroke thickness scales with the font:
        // ascent / 12 keeps it ~1px at 12px text and ~2.7px at 32px.
        if !b.text_decorations.is_empty() && run.width > 0.0 && run.ascent > 0.0 {
            let baseline_y = origin.y + run.ascent;
            let thickness = (run.ascent / 12.0).max(1.0);
            for line in &b.text_decorations {
                let y = match line {
                    wgpu_html_layout::TextDecorationLine::Underline => baseline_y + thickness,
                    wgpu_html_layout::TextDecorationLine::LineThrough => {
                        baseline_y - run.ascent * 0.30
                    }
                    wgpu_html_layout::TextDecorationLine::Overline => origin.y,
                };
                out.push_quad(
                    Rect::new(origin.x, y, run.width, thickness),
                    color,
                );
            }
        }

        for g in &run.glyphs {
            out.push_glyph(
                Rect::new(origin.x + g.x, origin.y + g.y, g.w, g.h),
                color,
                g.uv_min,
                g.uv_max,
            );
        }
    }

    for child in &b.children {
        paint_box(child, out);
    }
}

/// If every set border side shares the same colour AND a renderable
/// `solid` style (or has no style set, which we treat as solid when the
/// width and colour are present), return that colour. Non-solid styles
/// like dashed/dotted force a fall-back to per-side edge segments
/// because the ring shader can only render solid strokes.
fn uniform_border_color(b: &LayoutBox) -> Option<wgpu_html_renderer::Color> {
    use wgpu_html_models::common::css_enums::BorderStyle;

    let bd = b.border;
    let bc = b.border_colors;
    let bs = &b.border_styles;
    let mut chosen: Option<wgpu_html_renderer::Color> = None;
    let pairs = [
        (bd.top, bc.top, &bs.top),
        (bd.right, bc.right, &bs.right),
        (bd.bottom, bc.bottom, &bs.bottom),
        (bd.left, bc.left, &bs.left),
    ];
    for (w, c, s) in pairs {
        if w <= 0.0 {
            continue;
        }
        match s {
            None | Some(BorderStyle::Solid) => {}
            // `none` / `hidden` skip painting entirely; treat as
            // "non-uniform" so the per-side path runs and skips them
            // individually instead of drawing a single ring of width 0.
            Some(BorderStyle::None) | Some(BorderStyle::Hidden) => return None,
            // Dashed / dotted / double / groove / ridge / inset / outset
            // can't be expressed by the SDF ring; fall back to per-side.
            Some(_) => return None,
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

fn corner_radii(b: &LayoutBox) -> ([f32; 4], [f32; 4]) {
    corner_radii_from(&b.border_radius)
}

/// Mixed-style / mixed-colour borders on a rounded box. Each solid
/// side gets its own ring quad with stroke widths zero on the other
/// three sides — the SDF then naturally restricts the painted band to
/// just that side, with corners following the rounded path. `none` /
/// `hidden` sides are skipped. Dashed / dotted on rounded boxes still
/// emit sharp segments along that side (visible but the corner curves
/// aren't stylised).
fn paint_rounded_per_side_borders(
    b: &LayoutBox,
    rect: Rect,
    rh: [f32; 4],
    rv: [f32; 4],
    out: &mut DisplayList,
) {
    use wgpu_html_models::common::css_enums::BorderStyle;

    let r = b.border_rect;
    let bd = b.border;
    let bc = b.border_colors;
    let bs = &b.border_styles;

    let sides: [(Side, f32, Option<wgpu_html_renderer::Color>, &Option<BorderStyle>); 4] = [
        (Side::Top, bd.top, bc.top, &bs.top),
        (Side::Right, bd.right, bc.right, &bs.right),
        (Side::Bottom, bd.bottom, bc.bottom, &bs.bottom),
        (Side::Left, bd.left, bc.left, &bs.left),
    ];

    for (side, w, color, style) in sides {
        if w <= 0.0 {
            continue;
        }
        let Some(color) = color else { continue };
        let kind = match style {
            None | Some(BorderStyle::Solid) => EdgeKind::Solid,
            Some(BorderStyle::None) | Some(BorderStyle::Hidden) => EdgeKind::Skip,
            Some(BorderStyle::Dashed) => EdgeKind::Dashed,
            Some(BorderStyle::Dotted) => EdgeKind::Dotted,
            // Double / Groove / Ridge / Inset / Outset → solid for now.
            Some(_) => EdgeKind::Solid,
        };
        match kind {
            EdgeKind::Skip => {}
            EdgeKind::Solid => {
                let stroke = side.one_sided_stroke(w);
                out.push_quad_stroke_ellipse(rect, color, rh, rv, stroke);
            }
            EdgeKind::Dashed | EdgeKind::Dotted => {
                // If every corner is uniform-circular, the shader can
                // dash along the rounded path itself. Otherwise fall
                // back to straight dashed segments along the side's
                // straight portion (corner curves stay bare — better
                // than nothing while elliptical arc-length isn't yet
                // implemented).
                let radii = &b.border_radius;
                if uniform_circular_radius(radii).is_some() {
                    let stroke = side.one_sided_stroke(w);
                    let (dash, gap) = match kind {
                        EdgeKind::Dashed => ((w * 3.0).max(2.0), w.max(1.0)),
                        EdgeKind::Dotted => (w.max(1.0), w.max(1.0)),
                        _ => (0.0, 0.0),
                    };
                    let pattern = [
                        match kind {
                            EdgeKind::Dashed => 1.0,
                            EdgeKind::Dotted => 2.0,
                            _ => 0.0,
                        },
                        dash,
                        gap,
                        0.0,
                    ];
                    out.push_quad_stroke_patterned(rect, color, rh, rv, stroke, pattern);
                } else {
                    let edge_rect = side.edge_rect_rounded(r, bd, radii);
                    let axis = side.axis();
                    paint_edge(edge_rect, axis, w, kind, color, out);
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    fn one_sided_stroke(self, w: f32) -> [f32; 4] {
        // Order matches the shader: top, right, bottom, left.
        match self {
            Side::Top => [w, 0.0, 0.0, 0.0],
            Side::Right => [0.0, w, 0.0, 0.0],
            Side::Bottom => [0.0, 0.0, w, 0.0],
            Side::Left => [0.0, 0.0, 0.0, w],
        }
    }

    fn axis(self) -> Axis {
        match self {
            Side::Top | Side::Bottom => Axis::Horizontal,
            Side::Left | Side::Right => Axis::Vertical,
        }
    }

    #[allow(dead_code)]
    fn edge_rect(self, r: wgpu_html_layout::Rect, bd: wgpu_html_layout::Insets) -> Rect {
        let inner_h = (r.h - bd.top - bd.bottom).max(0.0);
        match self {
            Side::Top => Rect::new(r.x, r.y, r.w, bd.top),
            Side::Bottom => Rect::new(r.x, r.y + r.h - bd.bottom, r.w, bd.bottom),
            Side::Left => Rect::new(r.x, r.y + bd.top, bd.left, inner_h),
            Side::Right => Rect::new(r.x + r.w - bd.right, r.y + bd.top, bd.right, inner_h),
        }
    }

    /// Same as [`Self::edge_rect`] but on a rounded box: the strip is
    /// clamped to the *straight* portion of the side, between the two
    /// adjacent corner radii. With zero radii this collapses to the
    /// regular corner-owning rectangle, so it's safe for the rounded
    /// path even when only some corners are rounded.
    fn edge_rect_rounded(
        self,
        r: wgpu_html_layout::Rect,
        bd: wgpu_html_layout::Insets,
        radii: &wgpu_html_layout::CornerRadii,
    ) -> Rect {
        match self {
            Side::Top => {
                let x_start = radii.top_left.h;
                let x_end = (r.w - radii.top_right.h).max(x_start);
                Rect::new(r.x + x_start, r.y, x_end - x_start, bd.top)
            }
            Side::Bottom => {
                let x_start = radii.bottom_left.h;
                let x_end = (r.w - radii.bottom_right.h).max(x_start);
                Rect::new(
                    r.x + x_start,
                    r.y + r.h - bd.bottom,
                    x_end - x_start,
                    bd.bottom,
                )
            }
            Side::Left => {
                let y_start = radii.top_left.v.max(bd.top);
                let y_end = (r.h - radii.bottom_left.v).max(y_start);
                Rect::new(r.x, r.y + y_start, bd.left, y_end - y_start)
            }
            Side::Right => {
                let y_start = radii.top_right.v.max(bd.top);
                let y_end = (r.h - radii.bottom_right.v).max(y_start);
                Rect::new(
                    r.x + r.w - bd.right,
                    r.y + y_start,
                    bd.right,
                    y_end - y_start,
                )
            }
        }
    }
}

/// Returns the shared radius if every corner has the same circular
/// (h == v) radius; `None` otherwise. The dashed-along-curve shader
/// path only handles the uniform-circular case for now.
fn uniform_circular_radius(r: &wgpu_html_layout::CornerRadii) -> Option<f32> {
    let corners = [
        r.top_left,
        r.top_right,
        r.bottom_right,
        r.bottom_left,
    ];
    let target = corners[0].h;
    for c in corners {
        if (c.h - target).abs() > 1e-3 || (c.v - target).abs() > 1e-3 {
            return None;
        }
    }
    Some(target)
}

fn corner_radii_from(r: &wgpu_html_layout::CornerRadii) -> ([f32; 4], [f32; 4]) {
    (
        [
            r.top_left.h,
            r.top_right.h,
            r.bottom_right.h,
            r.bottom_left.h,
        ],
        [
            r.top_left.v,
            r.top_right.v,
            r.bottom_right.v,
            r.bottom_left.v,
        ],
    )
}

fn has_any_radius(r: &[f32; 4]) -> bool {
    r.iter().any(|v| *v > 0.0)
}

/// Emit per-side border edges for a sharp (non-rounded) box. Every side
/// is independently coloured and styled. `solid` is one full-edge quad;
/// `dashed` and `dotted` are emitted as a row of short segment quads;
/// `none` and `hidden` are skipped. Other values render as solid.
fn paint_border_edges(b: &LayoutBox, out: &mut DisplayList) {
    use wgpu_html_models::common::css_enums::BorderStyle;

    let r = b.border_rect;
    let bd = b.border;
    if r.w <= 0.0 || r.h <= 0.0 || !b.border_colors.any() {
        return;
    }
    let bc = b.border_colors;
    let bs = &b.border_styles;

    let inner_h = (r.h - bd.top - bd.bottom).max(0.0);

    // Top edge — horizontal strip at the very top; covers the corner
    // pixels for left/right edges so corners draw exactly once.
    if bd.top > 0.0 {
        if let Some(c) = bc.top {
            paint_edge(
                Rect::new(r.x, r.y, r.w, bd.top),
                Axis::Horizontal,
                bd.top,
                resolve_style(&bs.top),
                c,
                out,
            );
        }
    }
    // Bottom edge.
    if bd.bottom > 0.0 {
        if let Some(c) = bc.bottom {
            paint_edge(
                Rect::new(r.x, r.y + r.h - bd.bottom, r.w, bd.bottom),
                Axis::Horizontal,
                bd.bottom,
                resolve_style(&bs.bottom),
                c,
                out,
            );
        }
    }
    // Left edge — sits between the top and bottom strips.
    if bd.left > 0.0 && inner_h > 0.0 {
        if let Some(c) = bc.left {
            paint_edge(
                Rect::new(r.x, r.y + bd.top, bd.left, inner_h),
                Axis::Vertical,
                bd.left,
                resolve_style(&bs.left),
                c,
                out,
            );
        }
    }
    // Right edge.
    if bd.right > 0.0 && inner_h > 0.0 {
        if let Some(c) = bc.right {
            paint_edge(
                Rect::new(r.x + r.w - bd.right, r.y + bd.top, bd.right, inner_h),
                Axis::Vertical,
                bd.right,
                resolve_style(&bs.right),
                c,
                out,
            );
        }
    }

    fn resolve_style(s: &Option<BorderStyle>) -> EdgeKind {
        match s {
            None | Some(BorderStyle::Solid) => EdgeKind::Solid,
            Some(BorderStyle::Dashed) => EdgeKind::Dashed,
            Some(BorderStyle::Dotted) => EdgeKind::Dotted,
            Some(BorderStyle::None) | Some(BorderStyle::Hidden) => EdgeKind::Skip,
            // Double / Groove / Ridge / Inset / Outset: render as solid for now.
            Some(_) => EdgeKind::Solid,
        }
    }
}

#[derive(Copy, Clone)]
enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone)]
enum EdgeKind {
    Solid,
    Dashed,
    Dotted,
    Skip,
}

fn paint_edge(
    rect: Rect,
    axis: Axis,
    thickness: f32,
    kind: EdgeKind,
    color: wgpu_html_renderer::Color,
    out: &mut DisplayList,
) {
    match kind {
        EdgeKind::Skip => {}
        EdgeKind::Solid => {
            out.push_quad(rect, color);
        }
        EdgeKind::Dashed => {
            // CSS-style approximation: dashes are ~3 thicknesses long
            // with a 1-thickness gap, with sane minimums for very thin
            // borders.
            let dash = (thickness * 3.0).max(2.0);
            let gap = thickness.max(1.0);
            paint_segments(rect, axis, dash, gap, color, out);
        }
        EdgeKind::Dotted => {
            // Square dots with one-thickness gaps.
            let dot = thickness.max(1.0);
            let gap = thickness.max(1.0);
            paint_segments(rect, axis, dot, gap, color, out);
        }
    }
}

/// Emit a sequence of `on`-length segments with `off`-length gaps along
/// `axis` inside `rect`. Final segment is clipped if it would overflow.
fn paint_segments(
    rect: Rect,
    axis: Axis,
    on: f32,
    off: f32,
    color: wgpu_html_renderer::Color,
    out: &mut DisplayList,
) {
    let stride = on + off;
    if stride <= 0.0 {
        return;
    }
    let total = match axis {
        Axis::Horizontal => rect.w,
        Axis::Vertical => rect.h,
    };
    let mut t = 0.0_f32;
    while t < total {
        let len = on.min(total - t);
        if len > 0.0 {
            let seg = match axis {
                Axis::Horizontal => Rect::new(rect.x + t, rect.y, len, rect.h),
                Axis::Vertical => Rect::new(rect.x, rect.y + t, rect.w, len),
            };
            out.push_quad(seg, color);
        }
        t += stride;
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
        assert_eq!(q.radii_h, [1.0, 2.0, 3.0, 4.0]);
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
        assert_eq!(q.radii_h, [16.0; 4]);
        assert_eq!(q.radii_v, [16.0; 4]);
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
    fn rounded_with_per_side_colors_emits_per_side_ring_quads() {
        // Each solid side gets its own one-sided ring quad so the
        // corners follow the rounded path — 1 rounded background + 4
        // ring quads = 5 total. (Same total count as the old sharp-
        // fallback path; the difference is each border quad now has
        // a non-zero stroke and curves at the corner.)
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-color: red green blue orange;
                             border-radius: 8px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 5);
        // Each border quad has stroke set on exactly one side.
        for q in &list.quads[1..] {
            let nonzero_sides = q.stroke.iter().filter(|s| **s > 0.0).count();
            assert_eq!(nonzero_sides, 1);
        }
    }

    #[test]
    fn rounded_with_mixed_styles_skips_none_sides() {
        // border-style: solid solid none solid → bottom side is omitted,
        // remaining 3 sides emit ring quads.
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             background-color: red;
                             border-width: 2px;
                             border-style: solid solid none solid;
                             border-color: grey;
                             border-radius: 8px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        // 1 background + 3 ring quads (top / right / left).
        assert_eq!(list.quads.len(), 4);
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
        assert_eq!(list.quads[0].radii_h, [0.0; 4]);
        assert_eq!(list.quads[0].radii_v, [0.0; 4]);
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

    #[test]
    fn dashed_border_emits_multiple_segments_per_side() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 200px; height: 100px;
                             border: 2px dashed red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        // Solid would emit 4 quads. Dashed should produce many more.
        assert!(
            list.quads.len() > 8,
            "expected dashed border to emit many segments, got {}",
            list.quads.len()
        );
    }

    #[test]
    fn dotted_border_emits_segments_too() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 200px; height: 100px;
                             border: 2px dotted blue;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert!(list.quads.len() > 8);
    }

    #[test]
    fn border_style_none_skips_that_side() {
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 100px; height: 50px;
                             border-width: 2px;
                             border-style: solid solid none solid;
                             border-color: red;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        // Per-side fallback path; bottom side skipped → 3 solid edges.
        assert_eq!(list.quads.len(), 3);
    }

    #[test]
    fn dashed_with_rounded_emits_per_side_patterned_rings() {
        // Uniform-circular corners → dashed pattern goes through the
        // shader as one one-sided ring quad per side; the dash pattern
        // wraps around the corner curve in the fragment shader.
        // 1 rounded background + 4 ring quads (top / right / bottom /
        // left, each with pattern set).
        let tree = wgpu_html_parser::parse(
            r#"<body style="width: 200px; height: 100px;
                             background-color: white;
                             border: 2px dashed red;
                             border-radius: 12px;"></body>"#,
        );
        let list = paint_tree(&tree, 800.0, 600.0);
        assert_eq!(list.quads.len(), 5);
        // Background first, no stroke / pattern.
        assert_eq!(list.quads[0].stroke, [0.0; 4]);
        assert_eq!(list.quads[0].pattern, [0.0; 4]);
        // Each border ring carries the dashed pattern (kind=1.0).
        for q in &list.quads[1..] {
            assert_eq!(q.pattern[0], 1.0);
            let nonzero = q.stroke.iter().filter(|s| **s > 0.0).count();
            assert_eq!(nonzero, 1);
        }
    }
}
