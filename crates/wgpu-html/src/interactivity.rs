//! Position-based interactivity wrappers.
//!
//! All real dispatch logic — hover-chain diffing, focus state,
//! keyboard delivery, click synthesis, selection updates — lives
//! in `wgpu_html_tree::dispatch` and is also exposed as inherent
//! methods on `Tree` (`tree.focus(…)`, `tree.key_down(…)`,
//! `tree.dispatch_mouse_down(…)`, etc.). New apps should drive the
//! tree directly through those.
//!
//! This module exists for the (still useful) case where the host
//! has a `wgpu_html_layout::LayoutBox` handy and would prefer to
//! pass a position rather than hit-test by hand. Each wrapper
//!
//! 1. resolves the hit path via `LayoutBox::hit_path`,
//! 2. resolves a text cursor via `LayoutBox::hit_text_cursor`,
//! 3. forwards to the matching `tree::dispatch_*` function.
//!
//! For compatibility with the previous public surface, the
//! layout-free entry points are re-exported here too:
//! [`focus`], [`blur`], [`focus_next`], [`key_down`], [`key_up`],
//! [`pointer_leave`].

use wgpu_html_layout::LayoutBox;
use wgpu_html_tree::{MouseButton, Tree};

// Re-exports of the layout-free dispatch entry points — these used
// to live here, now they live in `wgpu_html_tree::dispatch`.
pub use wgpu_html_tree::{
    blur, dispatch_pointer_leave as pointer_leave, focus, focus_next, key_down, key_up,
};

/// Update the hover path to whatever lies under `pos` and fire
/// any `on_mouse_enter` / `on_mouse_leave` callbacks the change
/// implies. Returns `true` if the hover path actually changed.
///
/// Modifier state is read from `tree.interaction.modifiers`;
/// keep it in sync with [`Tree::set_modifier`].
pub fn pointer_move(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32)) -> bool {
    let target = layout.hit_path(pos);
    let cursor = layout.hit_text_cursor(pos);
    tree.dispatch_pointer_move(target.as_deref(), pos, cursor)
}

/// Primary-button (or any-button) press at `pos`. Records the
/// active path for click synthesis on the matching release; fires
/// `on_mouse_down` bubbling target → root; on a primary press,
/// also moves keyboard focus to the deepest focusable ancestor of
/// the hit path (or clears focus if none).
pub fn mouse_down(
    tree: &mut Tree,
    layout: &LayoutBox,
    pos: (f32, f32),
    button: MouseButton,
) -> bool {
    let target = layout.hit_path(pos);
    let cursor = layout.hit_text_cursor(pos);
    tree.dispatch_mouse_down(target.as_deref(), pos, button, cursor)
}

/// Mouse-up at `pos`. Fires `on_mouse_up`; then, if `button` is
/// `Primary` and the release path shares its root with the press
/// path, synthesises a click and fires `on_click` bubbling.
pub fn mouse_up(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool {
    let target = layout.hit_path(pos);
    let cursor = layout.hit_text_cursor(pos);
    tree.dispatch_mouse_up(target.as_deref(), pos, button, cursor)
}

#[cfg(test)]
mod tests {
    //! Layout-aware integration tests. The bulk of dispatch is
    //! tested directly against `Tree` in
    //! `wgpu_html_tree::dispatch::tests`; the cases here exercise
    //! the layout → tree forwarding (hit-testing + text-cursor
    //! mapping).

    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wgpu_html_tree::{Node, SelectionColors};
    // `Modifiers` is no longer plumbed through the dispatch API —
    // these tests rely on the tree's default (all keys up) state.

    fn synthetic_text_layout() -> LayoutBox {
        let r = wgpu_html_layout::Rect::new(0.0, 0.0, 80.0, 20.0);
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
                glyphs: vec![
                    wgpu_html_text::PositionedGlyph {
                        x: 0.0,
                        y: 0.0,
                        w: 10.0,
                        h: 16.0,
                        uv_min: [0.0, 0.0],
                        uv_max: [1.0, 1.0],
                        color: [0.0, 0.0, 0.0, 1.0],
                    },
                    wgpu_html_text::PositionedGlyph {
                        x: 10.0,
                        y: 0.0,
                        w: 10.0,
                        h: 16.0,
                        uv_min: [0.0, 0.0],
                        uv_max: [1.0, 1.0],
                        color: [0.0, 0.0, 0.0, 1.0],
                    },
                    wgpu_html_text::PositionedGlyph {
                        x: 20.0,
                        y: 0.0,
                        w: 10.0,
                        h: 16.0,
                        uv_min: [0.0, 0.0],
                        uv_max: [1.0, 1.0],
                        color: [0.0, 0.0, 0.0, 1.0],
                    },
                ],
                lines: vec![wgpu_html_text::ShapedLine {
                    top: 0.0,
                    height: 16.0,
                    glyph_range: (0, 3),
                }],
                text: "abc".to_string(),
                byte_boundaries: wgpu_html_text::utf8_boundaries("abc"),
                width: 30.0,
                height: 16.0,
                ascent: 12.0,
            }),
            text_color: Some([0.0, 0.0, 0.0, 1.0]),
            text_decorations: Vec::new(),
            overflow: wgpu_html_layout::OverflowAxes::visible(),
            opacity: 1.0,
            image: None,
            background_image: None,
            children: Vec::new(),
        }
    }

    fn make_tree(counter: Arc<AtomicUsize>) -> Tree {
        let mut root = Node::new("text");
        root.on_click = Some(Arc::new(move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
        }));
        let mut tree = Tree::new(root);
        tree.interaction.selection_colors = SelectionColors::default();
        tree
    }

    #[test]
    fn drag_selection_suppresses_click() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut tree = make_tree(counter.clone());
        let lay = synthetic_text_layout();

        mouse_down(&mut tree, &lay, (1.0, 4.0), MouseButton::Primary);
        pointer_move(&mut tree, &lay, (26.0, 4.0));
        mouse_up(&mut tree, &lay, (26.0, 4.0), MouseButton::Primary);

        assert_eq!(counter.load(Ordering::Relaxed), 0);
        let sel = tree.interaction.selection.expect("selection");
        assert!(!sel.is_collapsed());
    }

    #[test]
    fn collapsed_selection_keeps_click() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut tree = make_tree(counter.clone());
        let lay = synthetic_text_layout();

        mouse_down(&mut tree, &lay, (1.0, 4.0), MouseButton::Primary);
        mouse_up(&mut tree, &lay, (1.0, 4.0), MouseButton::Primary);

        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn pointer_move_then_leave_via_layout() {
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let r = received.clone();
        let mut root = Node::new("text");
        root.on_event = Some(Arc::new(move |ev| {
            r.lock().unwrap().push(ev.event_type().to_string());
        }));
        let lay = synthetic_text_layout();
        let mut tree = Tree::new(root);

        pointer_move(&mut tree, &lay, (5.0, 5.0));
        pointer_leave(&mut tree);

        let evs = received.lock().unwrap().clone();
        assert!(evs.contains(&"mouseenter".into()), "got {evs:?}");
        assert!(evs.contains(&"mouseleave".into()), "got {evs:?}");
    }
}
