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

use wgpu_html_layout::LayoutBox;
use wgpu_html_tree::{Modifiers, MouseButton, MouseEvent, Tree};

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
    let Some(target_path) = layout.hit_path(pos) else {
        return false;
    };
    if button == MouseButton::Primary {
        tree.interaction.active_path = Some(target_path.clone());
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
    let Some(target_path) = layout.hit_path(pos) else {
        if button == MouseButton::Primary {
            tree.interaction.active_path = None;
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
        let press = tree.interaction.active_path.take();
        if let Some(press_path) = press {
            // Browser-style click target: deepest common ancestor of
            // the press and release paths. A drag-out-and-back release
            // still fires the click on whatever the two share.
            let click_target = common_prefix(&press_path, &target_path);
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
    true
}

#[derive(Copy, Clone)]
enum Slot {
    MouseDown,
    MouseUp,
    Click,
}

fn bubble(
    tree: &mut Tree,
    target_path: &[usize],
    pos: (f32, f32),
    button: Option<MouseButton>,
    modifiers: Modifiers,
    slot: Slot,
) {
    let Some(root) = tree.root.as_mut() else {
        return;
    };
    let chain = root.ancestry_at_path_mut(target_path);
    // chain[0] is deepest, chain[chain.len()-1] is the document root.
    // chain[i]'s path is target_path[..target_path.len() - i].
    let depth = target_path.len();
    for (i, node) in chain.into_iter().enumerate() {
        let cb_slot = match slot {
            Slot::MouseDown => &node.on_mouse_down,
            Slot::MouseUp => &node.on_mouse_up,
            Slot::Click => &node.on_click,
        };
        let Some(cb) = cb_slot.as_ref() else { continue };
        let cb = cb.clone();
        let current_path = target_path[..depth.saturating_sub(i)].to_vec();
        let ev = MouseEvent {
            pos,
            button,
            modifiers,
            target_path: target_path.to_vec(),
            current_path,
        };
        cb(&ev);
    }
}

/// Run on each element on `old_path \ new_path` (deepest-first leave),
/// then each on `new_path \ old_path` (root-first enter). Mirrors DOM
/// `mouseenter` / `mouseleave` semantics: outer enters fire before
/// inner enters, inner leaves before outer leaves.
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
    }

    // Enters: every element on new_path strictly past `common_len`,
    // walked root-first.
    if let Some(new) = new_path {
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

#[derive(Copy, Clone)]
enum Slot2 {
    Enter,
    Leave,
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
        let cb_slot = match slot {
            Slot2::Enter => &node.on_mouse_enter,
            Slot2::Leave => &node.on_mouse_leave,
        };
        let Some(cb) = cb_slot.as_ref() else { continue };
        let cb = cb.clone();
        let current_path = path[..plen].to_vec();
        let ev = MouseEvent {
            pos,
            button: None,
            modifiers,
            target_path: target.clone(),
            current_path,
        };
        cb(&ev);
    }
}

/// Return the longest path that is a prefix of both `a` and `b`.
fn common_prefix<'a>(a: &'a [usize], b: &[usize]) -> &'a [usize] {
    let n = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();
    &a[..n]
}
