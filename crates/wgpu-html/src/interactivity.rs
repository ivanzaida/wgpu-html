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
    let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets_y);
    let cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets_y);
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
    mouse_down_with_click_count(tree, layout, pos, button, 1)
}

/// Like [`mouse_down`], but lets hosts pass an already-detected click
/// count. `2` selects the word/token under the pointer, `3+` selects
/// the shaped line.
pub fn mouse_down_with_click_count(
    tree: &mut Tree,
    layout: &LayoutBox,
    pos: (f32, f32),
    button: MouseButton,
    click_count: u8,
) -> bool {
    let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets_y);
    let cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets_y);
    let result = tree.dispatch_mouse_down(target.as_deref(), pos, button, cursor.clone());

    // After focus is set on a form control, position the edit caret
    // at the clicked glyph. Walk the layout tree to find the form
    // control's text run and convert glyph_index → byte_offset.
    if button == MouseButton::Primary {
        if tree.interaction.edit_cursor.is_some() {
            if let Some(focus_path) = tree.interaction.focus_path.clone() {
                // Read the actual value length to distinguish
                // placeholder (empty value) from typed content.
                let value = field_value(tree, &focus_path).unwrap_or_default();
                let value_len = value.len();

                let byte_offset = if value_len == 0 {
                    // Field is empty (showing placeholder) — caret
                    // goes to position 0, not inside the placeholder.
                    0
                } else if let Some(text_box) = crate::layout_at_path(layout, &focus_path) {
                    if let Some(run) = &text_box.text_run {
                        let click_x = pos.0 - text_box.content_rect.x;
                        let glyph_idx = run
                            .glyphs
                            .iter()
                            .position(|g| g.x + g.w * 0.5 > click_x)
                            .unwrap_or(run.glyphs.len());
                        if glyph_idx < run.byte_boundaries.len() {
                            run.byte_boundaries[glyph_idx]
                        } else {
                            value_len
                        }
                    } else {
                        0
                    }
                } else {
                    0
                };
                tree.interaction.edit_cursor = Some(edit_cursor_for_click_count(
                    &value,
                    byte_offset,
                    click_count,
                ));
                tree.interaction.caret_blink_epoch = std::time::Instant::now();
            }
        } else if let Some(cursor) = cursor.as_ref() {
            if click_count >= 3 {
                crate::select_line_at_cursor(tree, layout, cursor);
            } else if click_count == 2 {
                crate::select_word_at_cursor(tree, layout, cursor);
            }
        }
    }

    result
}

fn field_value(tree: &Tree, focus_path: &[usize]) -> Option<String> {
    tree.root
        .as_ref()
        .and_then(|r| r.at_path(focus_path))
        .and_then(|node| match &node.element {
            wgpu_html_tree::Element::Input(inp) => Some(inp.value.clone().unwrap_or_default()),
            wgpu_html_tree::Element::Textarea(ta) => Some(ta.value.clone().unwrap_or_default()),
            _ => None,
        })
}

fn edit_cursor_for_click_count(
    value: &str,
    byte_offset: usize,
    click_count: u8,
) -> wgpu_html_tree::EditCursor {
    if click_count >= 3 {
        let (start, end) = line_byte_range(value, byte_offset);
        wgpu_html_tree::EditCursor {
            cursor: end,
            selection_anchor: Some(start),
        }
    } else if click_count == 2 {
        let (start, end) = word_byte_range(value, byte_offset);
        wgpu_html_tree::EditCursor {
            cursor: end,
            selection_anchor: Some(start),
        }
    } else {
        wgpu_html_tree::EditCursor::collapsed(byte_offset)
    }
}

fn word_byte_range(value: &str, byte_offset: usize) -> (usize, usize) {
    let chars: Vec<(usize, usize, char)> = value
        .char_indices()
        .map(|(start, ch)| (start, start + ch.len_utf8(), ch))
        .collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let mut idx = chars
        .iter()
        .position(|(_, end, _)| *end >= byte_offset)
        .unwrap_or(chars.len() - 1);
    if idx > 0 && chars[idx].0 >= byte_offset {
        idx -= 1;
    }
    let kind = edit_token_kind(chars[idx].2);
    let mut start = idx;
    while start > 0 && edit_token_kind(chars[start - 1].2) == kind {
        start -= 1;
    }
    let mut end = idx + 1;
    while end < chars.len() && edit_token_kind(chars[end].2) == kind {
        end += 1;
    }
    (chars[start].0, chars[end - 1].1)
}

fn line_byte_range(value: &str, byte_offset: usize) -> (usize, usize) {
    let pos = byte_offset.min(value.len());
    let start = value[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = value[pos..]
        .find('\n')
        .map(|i| pos + i)
        .unwrap_or(value.len());
    (start, end)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditTokenKind {
    Word,
    Whitespace,
    Punctuation(char),
}

fn edit_token_kind(ch: char) -> EditTokenKind {
    if ch.is_alphanumeric() || ch == '_' {
        EditTokenKind::Word
    } else if ch.is_whitespace() {
        EditTokenKind::Whitespace
    } else {
        EditTokenKind::Punctuation(ch)
    }
}

/// Mouse-up at `pos`. Fires `on_mouse_up`; then, if `button` is
/// `Primary` and the release path shares its root with the press
/// path, synthesises a click and fires `on_click` bubbling.
pub fn mouse_up(tree: &mut Tree, layout: &LayoutBox, pos: (f32, f32), button: MouseButton) -> bool {
    let target = layout.hit_path_scrolled(pos, &tree.interaction.scroll_offsets_y);
    let cursor = layout.hit_text_cursor_scrolled(pos, &tree.interaction.scroll_offsets_y);
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
                glyph_chars: vec![],
                text: "abc".to_string(),
                byte_boundaries: wgpu_html_text::utf8_boundaries("abc"),
                width: 30.0,
                height: 16.0,
                ascent: 12.0,
            }),
            text_color: Some([0.0, 0.0, 0.0, 1.0]),
            text_unselectable: false,
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
    fn double_click_selects_word() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut tree = make_tree(counter);
        let lay = synthetic_text_layout();

        mouse_down_with_click_count(&mut tree, &lay, (11.0, 4.0), MouseButton::Primary, 2);

        let sel = tree.interaction.selection.expect("selection");
        assert_eq!(sel.anchor.glyph_index, 0);
        assert_eq!(sel.focus.glyph_index, 3);
        assert!(!tree.interaction.selecting_text);
    }

    #[test]
    fn edit_double_click_selects_word_and_triple_click_selects_line() {
        let cursor = edit_cursor_for_click_count("one two\nthree", 5, 2);
        assert_eq!(cursor.selection_range(), (4, 7));

        let cursor = edit_cursor_for_click_count("one two\nthree", 5, 3);
        assert_eq!(cursor.selection_range(), (0, 7));
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
