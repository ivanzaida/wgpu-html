//! Pointer event dispatcher.
//!
//! Hosts hand pointer events to the engine; the engine resolves the
//! hit path against a `LayoutBox`, walks `Tree` ancestry, and invokes
//! the matching callback slots on each `Node`. State (hover / active
//! paths, last pointer position) lives on `Tree::interaction`.
//!
//! First slice (matches `spec/interactivity.md` §16, M-INTER-1
//! reduced):
//! - `pointer_move` — updates `hover_path`, fires `on_mouse_leave` /
//!   `on_mouse_enter` on the symmetric difference.
//! - `mouse_down` — updates `active_path`, fires `on_mouse_down`
//!   bubbling target → root.
//! - `mouse_up` — fires `on_mouse_up`, then synthesises a click
//!   firing `on_click` if the release shares an ancestor with the
//!   press (matches browser drag-out-cancel behaviour).
//! - `pointer_leave` — clears `hover_path`, fires `on_mouse_leave`.

use wgpu_html_events as ev;
use wgpu_html_layout::LayoutBox;
use wgpu_html_tree::{Modifiers, MouseButton, MouseEvent, Tree};

// ── internal helpers ──────────────────────────────────────────────────────────

/// Map our `MouseButton` to the DOM `MouseEvent.button` value (i16).
fn button_to_i16(button: Option<MouseButton>) -> i16 {
    match button {
        None | Some(MouseButton::Primary) => 0,
        Some(MouseButton::Middle) => 1,
        Some(MouseButton::Secondary) => 2,
        Some(MouseButton::Other(n)) => n as i16,
    }
}

/// Map our `MouseButton` to its bit in the DOM `MouseEvent.buttons` bitmask.
fn button_to_bit(button: MouseButton) -> u16 {
    match button {
        MouseButton::Primary => 1,
        MouseButton::Secondary => 2,
        MouseButton::Middle => 4,
        MouseButton::Other(n) => 1u16 << (n.min(15)),
    }
}

/// Build the `wgpu_html_events` base chain for a mouse event.
fn make_mouse_html_event(
    event_type: &'static str,
    bubbles: bool,
    detail: i32,
    pos: (f32, f32),
    button: Option<MouseButton>,
    buttons_down: u16,
    modifiers: Modifiers,
    target_path: &[usize],
    current_path: Vec<usize>,
    time_stamp: f64,
) -> ev::HtmlEvent {
    let event_phase = if current_path.as_slice() == target_path {
        ev::EventPhase::AtTarget
    } else {
        ev::EventPhase::BubblingPhase
    };
    ev::HtmlEvent::Mouse(ev::events::MouseEvent {
        base: ev::events::UIEvent {
            base: ev::events::Event {
                event_type: ev::HtmlEventType::from(event_type),
                bubbles,
                cancelable: true,
                composed: true,
                target: Some(target_path.to_vec()),
                current_target: Some(current_path),
                event_phase,
                default_prevented: false,
                is_trusted: true,
                time_stamp,
            },
            detail,
        },
        screen_x: pos.0 as f64,
        screen_y: pos.1 as f64,
        client_x: pos.0 as f64,
        client_y: pos.1 as f64,
        offset_x: pos.0 as f64,
        offset_y: pos.1 as f64,
        page_x: pos.0 as f64,
        page_y: pos.1 as f64,
        movement_x: 0.0,
        movement_y: 0.0,
        button: button_to_i16(button),
        buttons: buttons_down,
        ctrl_key: modifiers.ctrl,
        shift_key: modifiers.shift,
        alt_key: modifiers.alt,
        meta_key: modifiers.meta,
        related_target: None,
    })
}

// ── public API ────────────────────────────────────────────────────────────────

/// Update the hover path to whatever lies under `pos` and fire any
/// `on_mouse_enter` / `on_mouse_leave` callbacks that change implies.
/// Returns `true` if the hover path actually changed.
pub fn pointer_move(
    tree: &mut Tree,
    layout: &LayoutBox,
    pos: (f32, f32),
    modifiers: Modifiers,
) -> bool {
    tree.interaction.pointer_pos = Some(pos);

    if tree.interaction.selecting_text {
        if let Some(cursor) = layout.hit_text_cursor(pos) {
            if let Some(sel) = tree.interaction.selection.as_mut() {
                sel.focus = cursor;
            }
        }
    }

    let new_path = layout.hit_path(pos);
    let changed = tree.interaction.hover_path != new_path;
    if changed {
        let old = tree.interaction.hover_path.take();
        update_hover(tree, old.as_deref(), new_path.as_deref(), pos, modifiers);
        tree.interaction.hover_path = new_path;
    }
    changed
}

/// The pointer left the surface (e.g. winit `CursorLeft`). Clears the
/// hover path, fires `on_mouse_leave` on the previously-hovered chain.
pub fn pointer_leave(tree: &mut Tree, modifiers: Modifiers) {
    let pos = tree.interaction.pointer_pos.unwrap_or((-1.0, -1.0));
    let old = tree.interaction.hover_path.take();
    update_hover(tree, old.as_deref(), None, pos, modifiers);
}

/// Primary-button (or any-button) press at `pos`. Records the active
/// path for click synthesis on the matching release; fires
/// `on_mouse_down` bubbling target → root.
pub fn mouse_down(
    tree: &mut Tree,
    layout: &LayoutBox,
    pos: (f32, f32),
    button: MouseButton,
    modifiers: Modifiers,
) -> bool {
    // Update buttons bitmask.
    tree.interaction.buttons_down |= button_to_bit(button);

    let Some(target_path) = layout.hit_path(pos) else {
        if button == MouseButton::Primary {
            tree.clear_selection();
        }
        return false;
    };
    if button == MouseButton::Primary {
        tree.interaction.active_path = Some(target_path.clone());
        if let Some(cursor) = layout.hit_text_cursor(pos) {
            tree.interaction.selection = Some(wgpu_html_tree::TextSelection {
                anchor: cursor.clone(),
                focus: cursor,
            });
            tree.interaction.selecting_text = true;
        } else {
            tree.clear_selection();
        }
    }
    bubble(
        tree,
        &target_path,
        pos,
        Some(button),
        modifiers,
        Slot::MouseDown,
    );
    true
}

/// Mouse-up at `pos`. Fires `on_mouse_up`; then, if `button` is
/// `Primary` and the release path shares its root with the press path,
/// synthesises a click and fires `on_click` bubbling.
pub fn mouse_up(
    tree: &mut Tree,
    layout: &LayoutBox,
    pos: (f32, f32),
    button: MouseButton,
    modifiers: Modifiers,
) -> bool {
    // Update buttons bitmask before constructing events (button is now released).
    tree.interaction.buttons_down &= !button_to_bit(button);

    let Some(target_path) = layout.hit_path(pos) else {
        if button == MouseButton::Primary {
            tree.interaction.active_path = None;
            tree.interaction.selecting_text = false;
        }
        return false;
    };

    bubble(
        tree,
        &target_path,
        pos,
        Some(button),
        modifiers,
        Slot::MouseUp,
    );

    if button == MouseButton::Primary {
        if tree.interaction.selecting_text {
            if let Some(cursor) = layout.hit_text_cursor(pos) {
                if let Some(sel) = tree.interaction.selection.as_mut() {
                    sel.focus = cursor;
                }
            }
        }
        let press = tree.interaction.active_path.take();
        let suppress_click = tree
            .interaction
            .selection
            .as_ref()
            .is_some_and(|sel| tree.interaction.selecting_text && !sel.is_collapsed());
        tree.interaction.selecting_text = false;
        if let Some(press_path) = press {
            // Browser-style click target: deepest common ancestor of
            // the press and release paths. A drag-out-and-back release
            // still fires the click on whatever the two share.
            let click_target = common_prefix(&press_path, &target_path);
            if !suppress_click {
                bubble(
                    tree,
                    click_target,
                    pos,
                    Some(button),
                    modifiers,
                    Slot::Click,
                );
            }
        }
    }
    true
}

// ── private dispatch ──────────────────────────────────────────────────────────

#[derive(Copy, Clone)]
enum Slot {
    MouseDown,
    MouseUp,
    Click,
}

impl Slot {
    fn event_type_str(self) -> &'static str {
        match self {
            Slot::MouseDown => ev::HtmlEventType::MOUSEDOWN,
            Slot::MouseUp => ev::HtmlEventType::MOUSEUP,
            Slot::Click => ev::HtmlEventType::CLICK,
        }
    }

    fn bubbles(self) -> bool {
        true // mousedown, mouseup, click all bubble
    }

    fn detail(self) -> i32 {
        match self {
            Slot::Click => 1,
            _ => 0,
        }
    }
}

fn bubble(
    tree: &mut Tree,
    target_path: &[usize],
    pos: (f32, f32),
    button: Option<MouseButton>,
    modifiers: Modifiers,
    slot: Slot,
) {
    // Snapshot interaction state before borrowing root.
    let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
    let buttons_down = tree.interaction.buttons_down;

    let Some(root) = tree.root.as_mut() else {
        return;
    };
    let chain = root.ancestry_at_path_mut(target_path);
    // chain[0] is deepest, chain[chain.len()-1] is the document root.
    // chain[i]'s path is target_path[..target_path.len() - i].
    let depth = target_path.len();
    for (i, node) in chain.into_iter().enumerate() {
        let current_path = target_path[..depth.saturating_sub(i)].to_vec();

        // ── specific slot callback (legacy / ergonomic) ────────────────
        let cb_slot = match slot {
            Slot::MouseDown => &node.on_mouse_down,
            Slot::MouseUp => &node.on_mouse_up,
            Slot::Click => &node.on_click,
        };
        if let Some(cb) = cb_slot.as_ref() {
            let cb = cb.clone();
            let ev = MouseEvent {
                pos,
                button,
                modifiers,
                target_path: target_path.to_vec(),
                current_path: current_path.clone(),
            };
            cb(&ev);
        }

        // ── general on_event callback ──────────────────────────────────
        if let Some(on_ev) = node.on_event.as_ref() {
            let on_ev = on_ev.clone();
            let html_ev = make_mouse_html_event(
                slot.event_type_str(),
                slot.bubbles(),
                slot.detail(),
                pos,
                button,
                buttons_down,
                modifiers,
                target_path,
                current_path,
                time_stamp,
            );
            on_ev(&html_ev);
        }
    }
}

/// Run on each element on `old_path \ new_path` (deepest-first leave),
/// then each on `new_path \ old_path` (root-first enter). Mirrors DOM
/// `mouseenter` / `mouseleave` semantics: outer enters fire before
/// inner enters, inner leaves before outer leaves.
///
/// `fire_chain_segment` only handles nodes at depths ≥ 1.  The root node
/// (depth 0, path `[]`) is therefore fired explicitly whenever the pointer
/// transitions between hovering-nothing and hovering-something.
fn update_hover(
    tree: &mut Tree,
    old_path: Option<&[usize]>,
    new_path: Option<&[usize]>,
    pos: (f32, f32),
    modifiers: Modifiers,
) {
    let common_len = match (old_path, new_path) {
        (Some(a), Some(b)) => common_prefix(a, b).len(),
        _ => 0,
    };

    // Leaves: every element on old_path strictly past `common_len`,
    // walked deepest-first.
    if let Some(old) = old_path {
        if old.len() > common_len {
            fire_chain_segment(
                tree,
                old,
                common_len,
                old.len(),
                /*deepest_first*/ true,
                pos,
                modifiers,
                old_path,
                Slot2::Leave,
            );
        }
        // Root leave: only when the pointer is leaving the document entirely
        // (new_path is None). When just moving between children the root
        // remains continuously hovered and must not re-fire leave.
        if new_path.is_none() {
            fire_root_hover_event(tree, pos, modifiers, old, Slot2::Leave);
        }
    }

    // Enters: every element on new_path strictly past `common_len`,
    // walked root-first.
    if let Some(new) = new_path {
        // Root enter: only when the pointer was not previously over anything.
        if old_path.is_none() {
            fire_root_hover_event(tree, pos, modifiers, new, Slot2::Enter);
        }
        if new.len() > common_len {
            fire_chain_segment(
                tree,
                new,
                common_len,
                new.len(),
                /*deepest_first*/ false,
                pos,
                modifiers,
                new_path,
                Slot2::Enter,
            );
        }
    }
}

/// Fire hover callbacks (specific slot + `on_event`) on the *root* node only.
///
/// `fire_chain_segment` covers depths ≥ 1; this covers depth 0.
fn fire_root_hover_event(
    tree: &mut Tree,
    pos: (f32, f32),
    modifiers: Modifiers,
    target_path: &[usize],
    slot: Slot2,
) {
    let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
    let buttons_down = tree.interaction.buttons_down;
    let Some(root) = tree.root.as_mut() else {
        return;
    };
    let current_path: Vec<usize> = vec![];

    // ── specific slot callback ─────────────────────────────────────────
    let cb_slot = match slot {
        Slot2::Enter => root.on_mouse_enter.as_ref(),
        Slot2::Leave => root.on_mouse_leave.as_ref(),
    };
    if let Some(cb) = cb_slot.cloned() {
        cb(&MouseEvent {
            pos,
            button: None,
            modifiers,
            target_path: target_path.to_vec(),
            current_path: current_path.clone(),
        });
    }

    // ── general on_event callback ──────────────────────────────────────
    if let Some(on_ev) = root.on_event.as_ref().cloned() {
        let html_ev = make_mouse_html_event(
            slot.event_type_str(),
            false, // mouseenter/leave do not bubble
            0,
            pos,
            None,
            buttons_down,
            modifiers,
            target_path,
            current_path,
            time_stamp,
        );
        on_ev(&html_ev);
    }
}

#[derive(Copy, Clone)]
enum Slot2 {
    Enter,
    Leave,
}

impl Slot2 {
    fn event_type_str(self) -> &'static str {
        match self {
            Slot2::Enter => ev::HtmlEventType::MOUSEENTER,
            Slot2::Leave => ev::HtmlEventType::MOUSELEAVE,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fire_chain_segment(
    tree: &mut Tree,
    path: &[usize],
    start: usize,
    end: usize,
    deepest_first: bool,
    pos: (f32, f32),
    modifiers: Modifiers,
    target_path: Option<&[usize]>,
    slot: Slot2,
) {
    if end <= start {
        return;
    }

    // Snapshot interaction state before borrowing root.
    let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
    let buttons_down = tree.interaction.buttons_down;

    let Some(root) = tree.root.as_mut() else {
        return;
    };
    let prefix = &path[..end];
    // ancestry_at_path_mut returns deepest-first across `0..=prefix.len()`
    // prefix-lengths. The segment we want is prefix-lengths
    // `start+1 ..= end`, i.e. the first `end - start` entries.
    let mut chain = root.ancestry_at_path_mut(prefix);
    chain.truncate(end - start);
    if !deepest_first {
        // Now root-first within the segment (path-length start+1
        // first, path-length end last).
        chain.reverse();
    }
    let target = target_path.map(|p| p.to_vec()).unwrap_or_default();
    let count = chain.len();
    for (idx, node) in chain.into_iter().enumerate() {
        let plen = if deepest_first {
            end - idx
        } else {
            // After the reverse, the first entry is path-length start+1
            // and the last is path-length end. count = end - start, so
            // position `idx` is path-length start + 1 + idx.
            // Equivalent to: end - (count - 1 - idx).
            end - (count - 1 - idx)
        };
        let current_path = path[..plen].to_vec();

        // ── specific slot callback ─────────────────────────────────────
        let cb_slot = match slot {
            Slot2::Enter => &node.on_mouse_enter,
            Slot2::Leave => &node.on_mouse_leave,
        };
        if let Some(cb) = cb_slot.as_ref() {
            let cb = cb.clone();
            let ev = MouseEvent {
                pos,
                button: None,
                modifiers,
                target_path: target.clone(),
                current_path: current_path.clone(),
            };
            cb(&ev);
        }

        // ── general on_event callback ──────────────────────────────────
        if let Some(on_ev) = node.on_event.as_ref() {
            let on_ev = on_ev.clone();
            let html_ev = make_mouse_html_event(
                slot.event_type_str(),
                false, // mouseenter/leave do not bubble
                0,
                pos,
                None,
                buttons_down,
                modifiers,
                &target,
                current_path,
                time_stamp,
            );
            on_ev(&html_ev);
        }
    }
}

/// Return the longest path that is a prefix of both `a` and `b`.
fn common_prefix<'a>(a: &'a [usize], b: &[usize]) -> &'a [usize] {
    let n = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();
    &a[..n]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wgpu_html_tree::{Node, SelectionColors};

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

        mouse_down(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );
        pointer_move(&mut tree, &lay, (26.0, 4.0), Modifiers::default());
        mouse_up(
            &mut tree,
            &lay,
            (26.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );

        assert_eq!(counter.load(Ordering::Relaxed), 0);
        let sel = tree.interaction.selection.expect("selection");
        assert!(!sel.is_collapsed());
    }

    #[test]
    fn collapsed_selection_keeps_click() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut tree = make_tree(counter.clone());
        let lay = synthetic_text_layout();

        mouse_down(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );
        mouse_up(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );

        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn on_event_receives_html_click_event() {
        use std::sync::Mutex;
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let received2 = received.clone();

        let mut root = Node::new("text");
        root.on_event = Some(Arc::new(move |ev| {
            received2
                .lock()
                .unwrap()
                .push(ev.event_type().to_string());
        }));
        let lay = synthetic_text_layout();
        let mut tree = Tree::new(root);

        mouse_down(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );
        mouse_up(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );

        let events = received.lock().unwrap().clone();
        assert!(
            events.contains(&"mousedown".to_string()),
            "expected mousedown in {events:?}"
        );
        assert!(
            events.contains(&"mouseup".to_string()),
            "expected mouseup in {events:?}"
        );
        assert!(
            events.contains(&"click".to_string()),
            "expected click in {events:?}"
        );
    }

    #[test]
    fn on_event_receives_mouseenter_and_mouseleave() {
        use std::sync::Mutex;
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let received2 = received.clone();

        let mut root = Node::new("text");
        root.on_mouse_enter = None; // not wired
        root.on_event = Some(Arc::new(move |ev| {
            received2
                .lock()
                .unwrap()
                .push(ev.event_type().to_string());
        }));
        let lay = synthetic_text_layout();
        let mut tree = Tree::new(root);

        // Move into the element.
        pointer_move(&mut tree, &lay, (5.0, 5.0), Modifiers::default());
        // Move out.
        pointer_leave(&mut tree, Modifiers::default());

        let events = received.lock().unwrap().clone();
        assert!(
            events.contains(&"mouseenter".to_string()),
            "expected mouseenter in {events:?}"
        );
        assert!(
            events.contains(&"mouseleave".to_string()),
            "expected mouseleave in {events:?}"
        );
    }

    #[test]
    fn buttons_down_bitmask_tracks_press_and_release() {
        let mut root = Node::new("text");
        root.on_event = Some(Arc::new(|_| {}));
        let lay = synthetic_text_layout();
        let mut tree = Tree::new(root);

        assert_eq!(tree.interaction.buttons_down, 0);
        mouse_down(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );
        assert_eq!(tree.interaction.buttons_down, 1);
        mouse_up(
            &mut tree,
            &lay,
            (1.0, 4.0),
            MouseButton::Primary,
            Modifiers::default(),
        );
        assert_eq!(tree.interaction.buttons_down, 0);
    }
}
