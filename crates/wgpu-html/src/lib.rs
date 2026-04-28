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
use wgpu_html_tree::{TextCursor, TextSelection, Tree};

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
        paint::paint_layout_with_selection(
            root,
            &mut list,
            tree.interaction.selection.as_ref(),
            tree.interaction.selection_colors,
        );
        list.finalize();
    } else {
        list.finalize();
    }
    (list, layout)
}

/// Select every text run in document order.
pub fn select_all_text(tree: &mut Tree, layout: &LayoutBox) -> bool {
    let Some(anchor) = first_text_cursor(layout) else {
        tree.clear_selection();
        return false;
    };
    let Some(focus) = last_text_cursor(layout) else {
        tree.clear_selection();
        return false;
    };
    tree.interaction.selection = Some(TextSelection { anchor, focus });
    tree.interaction.selecting_text = false;
    true
}

/// Return the currently selected visible text, if any.
pub fn selected_text(tree: &Tree, layout: &LayoutBox) -> Option<String> {
    let selection = tree.interaction.selection.as_ref()?;
    selected_text_for_selection(layout, selection)
}

fn selected_text_for_selection(layout: &LayoutBox, selection: &TextSelection) -> Option<String> {
    if selection.is_collapsed() {
        return None;
    }
    let (start, end) = ordered_cursors(&selection.anchor, &selection.focus);
    let mut out = String::new();
    let mut prev_parent: Option<Vec<usize>> = None;
    let mut path = Vec::new();
    collect_selected_text(layout, &mut path, start, end, &mut prev_parent, &mut out);
    (!out.is_empty()).then_some(out)
}

fn first_text_cursor(layout: &LayoutBox) -> Option<TextCursor> {
    let mut path = Vec::new();
    first_text_cursor_inner(layout, &mut path)
}

fn first_text_cursor_inner(layout: &LayoutBox, path: &mut Vec<usize>) -> Option<TextCursor> {
    if let Some(run) = &layout.text_run {
        if !run.text.is_empty() || !run.glyphs.is_empty() {
            return Some(TextCursor {
                path: path.clone(),
                glyph_index: 0,
            });
        }
    }
    for (i, child) in layout.children.iter().enumerate() {
        path.push(i);
        let hit = first_text_cursor_inner(child, path);
        path.pop();
        if hit.is_some() {
            return hit;
        }
    }
    None
}

fn last_text_cursor(layout: &LayoutBox) -> Option<TextCursor> {
    let mut path = Vec::new();
    last_text_cursor_inner(layout, &mut path)
}

fn last_text_cursor_inner(layout: &LayoutBox, path: &mut Vec<usize>) -> Option<TextCursor> {
    for (i, child) in layout.children.iter().enumerate().rev() {
        path.push(i);
        let hit = last_text_cursor_inner(child, path);
        path.pop();
        if hit.is_some() {
            return hit;
        }
    }
    let run = layout.text_run.as_ref()?;
    (!run.text.is_empty() || !run.glyphs.is_empty()).then(|| TextCursor {
        path: path.clone(),
        glyph_index: run.glyphs.len(),
    })
}

fn collect_selected_text(
    layout: &LayoutBox,
    path: &mut Vec<usize>,
    start: &TextCursor,
    end: &TextCursor,
    prev_parent: &mut Option<Vec<usize>>,
    out: &mut String,
) {
    if let Some(run) = &layout.text_run {
        if !path_less(path, &start.path) && !path_less(&end.path, path) {
            let from = if path.as_slice() == start.path.as_slice() {
                run.byte_offset_for_boundary(start.glyph_index)
            } else {
                0
            };
            let to = if path.as_slice() == end.path.as_slice() {
                run.byte_offset_for_boundary(end.glyph_index)
            } else {
                run.text.len()
            };
            if to > from && to <= run.text.len() {
                let fragment = &run.text[from..to];
                if !fragment.is_empty() {
                    let parent = path[..path.len().saturating_sub(1)].to_vec();
                    if !out.is_empty() && prev_parent.as_deref() != Some(parent.as_slice()) {
                        out.push('\n');
                    }
                    out.push_str(fragment);
                    *prev_parent = Some(parent);
                }
            }
        }
    }

    for (i, child) in layout.children.iter().enumerate() {
        path.push(i);
        collect_selected_text(child, path, start, end, prev_parent, out);
        path.pop();
    }
}

fn ordered_cursors<'a>(a: &'a TextCursor, b: &'a TextCursor) -> (&'a TextCursor, &'a TextCursor) {
    if cursor_leq(a, b) { (a, b) } else { (b, a) }
}

fn cursor_leq(a: &TextCursor, b: &TextCursor) -> bool {
    if a.path == b.path {
        a.glyph_index <= b.glyph_index
    } else {
        path_less(&a.path, &b.path)
    }
}

fn path_less(a: &[usize], b: &[usize]) -> bool {
    a.cmp(b).is_lt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_box(text: &str, x: f32) -> LayoutBox {
        let r = wgpu_html_layout::Rect::new(x, 0.0, 100.0, 20.0);
        let glyphs = text
            .chars()
            .enumerate()
            .map(|(i, _)| wgpu_html_text::PositionedGlyph {
                x: i as f32 * 8.0,
                y: 0.0,
                w: 8.0,
                h: 16.0,
                uv_min: [0.0, 0.0],
                uv_max: [1.0, 1.0],
                color: [0.0, 0.0, 0.0, 1.0],
            })
            .collect();
        LayoutBox {
            margin_rect: r,
            border_rect: r,
            content_rect: r,
            background: None,
            background_rect: r,
            background_radii: wgpu_html_layout::CornerRadii::zero(),
            border: wgpu_html_layout::Insets::zero(),
            border_colors: wgpu_html_layout::BorderColors::default(),
            border_styles: wgpu_html_layout::BorderStyles::default(),
            border_radius: wgpu_html_layout::CornerRadii::zero(),
            kind: wgpu_html_layout::BoxKind::Text,
            text_run: Some(wgpu_html_text::ShapedRun {
                glyphs,
                lines: vec![wgpu_html_text::ShapedLine {
                    top: 0.0,
                    height: 20.0,
                    glyph_range: (0, text.chars().count()),
                }],
                text: text.to_owned(),
                byte_boundaries: wgpu_html_text::utf8_boundaries(text),
                width: text.chars().count() as f32 * 8.0,
                height: 20.0,
                ascent: 14.0,
            }),
            text_color: Some([0.0, 0.0, 0.0, 1.0]),
            text_decorations: Vec::new(),
            overflow: wgpu_html_layout::OverflowAxes::visible(),
            image: None,
            background_image: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn select_all_spans_first_to_last_text_box() {
        let root = LayoutBox {
            margin_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 40.0),
            border_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 40.0),
            content_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 40.0),
            background: None,
            background_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 40.0),
            background_radii: wgpu_html_layout::CornerRadii::zero(),
            border: wgpu_html_layout::Insets::zero(),
            border_colors: wgpu_html_layout::BorderColors::default(),
            border_styles: wgpu_html_layout::BorderStyles::default(),
            border_radius: wgpu_html_layout::CornerRadii::zero(),
            kind: wgpu_html_layout::BoxKind::Block,
            text_run: None,
            text_color: None,
            text_decorations: Vec::new(),
            overflow: wgpu_html_layout::OverflowAxes::visible(),
            image: None,
            background_image: None,
            children: vec![text_box("Hello", 0.0), text_box("World", 0.0)],
        };
        let mut tree = Tree::new(wgpu_html_tree::Node::new("root"));
        assert!(select_all_text(&mut tree, &root));
        let sel = tree.interaction.selection.expect("selection");
        assert_eq!(sel.anchor.path, vec![0]);
        assert_eq!(sel.anchor.glyph_index, 0);
        assert_eq!(sel.focus.path, vec![1]);
        assert_eq!(sel.focus.glyph_index, 5);
    }

    #[test]
    fn selected_text_uses_newlines_between_different_parents_and_not_within_same_parent() {
        let inline_parent = LayoutBox {
            margin_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 20.0),
            border_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 20.0),
            content_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 20.0),
            background: None,
            background_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 200.0, 20.0),
            background_radii: wgpu_html_layout::CornerRadii::zero(),
            border: wgpu_html_layout::Insets::zero(),
            border_colors: wgpu_html_layout::BorderColors::default(),
            border_styles: wgpu_html_layout::BorderStyles::default(),
            border_radius: wgpu_html_layout::CornerRadii::zero(),
            kind: wgpu_html_layout::BoxKind::Block,
            text_run: None,
            text_color: None,
            text_decorations: Vec::new(),
            overflow: wgpu_html_layout::OverflowAxes::visible(),
            image: None,
            background_image: None,
            children: vec![text_box("Hello ", 0.0), text_box("world", 48.0)],
        };
        let root = LayoutBox {
            margin_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 300.0, 60.0),
            border_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 300.0, 60.0),
            content_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 300.0, 60.0),
            background: None,
            background_rect: wgpu_html_layout::Rect::new(0.0, 0.0, 300.0, 60.0),
            background_radii: wgpu_html_layout::CornerRadii::zero(),
            border: wgpu_html_layout::Insets::zero(),
            border_colors: wgpu_html_layout::BorderColors::default(),
            border_styles: wgpu_html_layout::BorderStyles::default(),
            border_radius: wgpu_html_layout::CornerRadii::zero(),
            kind: wgpu_html_layout::BoxKind::Block,
            text_run: None,
            text_color: None,
            text_decorations: Vec::new(),
            overflow: wgpu_html_layout::OverflowAxes::visible(),
            image: None,
            background_image: None,
            children: vec![inline_parent, text_box("Second", 0.0)],
        };
        let mut tree = Tree::new(wgpu_html_tree::Node::new("root"));
        tree.interaction.selection = Some(TextSelection {
            anchor: TextCursor {
                path: vec![0, 0],
                glyph_index: 0,
            },
            focus: TextCursor {
                path: vec![1],
                glyph_index: 6,
            },
        });

        assert_eq!(selected_text(&tree, &root).as_deref(), Some("Hello world\nSecond"));
    }
}
