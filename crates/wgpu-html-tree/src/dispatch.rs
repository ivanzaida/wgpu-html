//! Layout-independent event dispatch.
//!
//! All the tree-touching parts of interactivity live here:
//! hover-chain diffing, mouse / click bubbling, focus state and
//! `:focus` cascade plumbing, keyboard delivery, and Tab navigation.
//! Hosts that already have a hit-tested target path can drive the
//! engine end-to-end through this module without taking a
//! `wgpu-html-layout` dependency.
//!
//! Layout-aware sugar (position-based `mouse_down`, `mouse_up`,
//! `pointer_move`) lives in `wgpu-html` / `wgpu-html-layout`; those
//! wrappers do hit-testing then forward into the dispatchers here.
//!
//! Public surface:
//!
//! - **Pointer:** [`dispatch_pointer_move`], [`dispatch_pointer_leave`], [`dispatch_mouse_down`],
//!   [`dispatch_mouse_up`].
//! - **Focus:** [`focus`], [`blur`], [`focus_next`].
//! - **Keyboard:** [`key_down`], [`key_up`].
//!
//! All of the above also exist as inherent methods on [`Tree`] for
//! ergonomic call-site syntax (`tree.focus(...)`, etc.).

use std::cell::Cell;

use wgpu_html_events as ev;
use wgpu_html_models as m;

use crate::{
  Element, InteractionState, Modifier, Modifiers, MouseButton, MouseEvent, Node, TextCursor, TextSelection, Tree,
  keyboard_focusable_paths, next_in_order, prev_in_order,
};

// ── Mouse-event helpers ──────────────────────────────────────────────────────

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
#[allow(clippy::too_many_arguments)]
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
  event_phase: ev::EventPhase,
) -> ev::HtmlEvent {
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
        default_prevented: Cell::new(false),
        propagation_stopped: Cell::new(false),
        immediate_propagation_stopped: Cell::new(false),
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

#[derive(Copy, Clone)]
enum Slot {
  MouseDown,
  MouseUp,
  Click,
  DblClick,
  ContextMenu,
  AuxClick,
  DragStart,
  Drag,
  DragOver,
  DragEnd,
  Drop,
}

impl Slot {
  fn event_type_str(self) -> &'static str {
    match self {
      Slot::MouseDown => ev::HtmlEventType::MOUSEDOWN,
      Slot::MouseUp => ev::HtmlEventType::MOUSEUP,
      Slot::Click => ev::HtmlEventType::CLICK,
      Slot::DblClick => ev::HtmlEventType::DBLCLICK,
      Slot::ContextMenu => ev::HtmlEventType::CONTEXTMENU,
      Slot::AuxClick => ev::HtmlEventType::AUXCLICK,
      Slot::DragStart => ev::HtmlEventType::DRAGSTART,
      Slot::Drag => ev::HtmlEventType::DRAG,
      Slot::DragOver => ev::HtmlEventType::DRAGOVER,
      Slot::DragEnd => ev::HtmlEventType::DRAGEND,
      Slot::Drop => ev::HtmlEventType::DROP,
    }
  }
  fn detail(self) -> i32 {
    match self {
      Slot::Click | Slot::DblClick => 1,
      _ => 0,
    }
  }
}

#[derive(Copy, Clone)]
enum HoverSlot {
  Enter,
  Leave,
  DragEnter,
  DragLeave,
}

impl HoverSlot {
  fn event_type_str(self) -> &'static str {
    match self {
      HoverSlot::Enter => ev::HtmlEventType::MOUSEENTER,
      HoverSlot::Leave => ev::HtmlEventType::MOUSELEAVE,
      HoverSlot::DragEnter => ev::HtmlEventType::DRAGENTER,
      HoverSlot::DragLeave => ev::HtmlEventType::DRAGLEAVE,
    }
  }
}

/// Fire the mouse-event callbacks at a single node during
/// a capture/target/bubble walk. Returns `true` if propagation
/// should stop.
#[allow(clippy::too_many_arguments)]
fn fire_mouse_slot(
  tree: &mut Tree,
  target_path: &[usize],
  current_path: &[usize],
  pos: (f32, f32),
  button: Option<MouseButton>,
  slot: Slot,
  time_stamp: f64,
  buttons_down: u16,
  modifiers: Modifiers,
  phase: ev::EventPhase,
) -> bool {
  let current_path = current_path.to_vec();
  let (mouse_cbs, event_cbs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| {
      let mouse_cbs = match slot {
        Slot::MouseDown => node.on_mouse_down.clone(),
        Slot::MouseUp => node.on_mouse_up.clone(),
        Slot::Click => node.on_click.clone(),
        Slot::DblClick => node.on_dblclick.clone(),
        Slot::ContextMenu => node.on_contextmenu.clone(),
        Slot::AuxClick => node.on_auxclick.clone(),
        Slot::DragStart => node.on_dragstart.clone(),
        Slot::Drag => node.on_drag.clone(),
        Slot::DragOver => node.on_dragover.clone(),
        Slot::DragEnd => node.on_dragend.clone(),
        Slot::Drop => node.on_drop.clone(),
      };
      (mouse_cbs, node.on_event.clone())
    })
    .unwrap_or_default();

  let mut ev = MouseEvent {
    pos,
    button,
    modifiers,
    target_path: target_path.to_vec(),
    current_path: current_path.clone(),
  };
  if tree.emit_mouse_event(&mut ev).is_stop() {
    return true;
  }
  if phase != ev::EventPhase::CapturingPhase {
    for cb in &mouse_cbs {
      cb(&ev);
    }
  }

  let mut html_ev = make_mouse_html_event(
    slot.event_type_str(),
    /* bubbles */ true,
    slot.detail(),
    pos,
    button,
    buttons_down,
    modifiers,
    target_path,
    current_path,
    time_stamp,
    phase,
  );
  if tree.emit_event(&mut html_ev).is_stop() {
    return true;
  }
  for cb in &event_cbs {
    cb(&html_ev);
  }
  html_ev.base().propagation_stopped.get()
}

/// Bubble a mouse event up the ancestry chain of `target_path`
/// with capture → target → bubble phases.
fn bubble(tree: &mut Tree, target_path: &[usize], pos: (f32, f32), button: Option<MouseButton>, slot: Slot) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let buttons_down = tree.interaction.buttons_down;
  let modifiers = tree.interaction.modifiers;
  let depth = target_path.len();

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &target_path[..i];
    if fire_mouse_slot(
      tree,
      target_path,
      current_path,
      pos,
      button,
      slot,
      time_stamp,
      buttons_down,
      modifiers,
      ev::EventPhase::CapturingPhase,
    ) {
      return;
    }
  }

  // Target phase
  if fire_mouse_slot(
    tree,
    target_path,
    target_path,
    pos,
    button,
    slot,
    time_stamp,
    buttons_down,
    modifiers,
    ev::EventPhase::AtTarget,
  ) {
    return;
  }

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = &target_path[..i];
    if fire_mouse_slot(
      tree,
      target_path,
      current_path,
      pos,
      button,
      slot,
      time_stamp,
      buttons_down,
      modifiers,
      ev::EventPhase::BubblingPhase,
    ) {
      return;
    }
  }
}

/// Run on each element on `old_path \ new_path` (deepest-first leave),
/// then each on `new_path \ old_path` (root-first enter). Mirrors DOM
/// `mouseenter` / `mouseleave` semantics: outer enters fire before
/// inner enters, inner leaves before outer leaves.
fn update_hover(tree: &mut Tree, old_path: Option<&[usize]>, new_path: Option<&[usize]>, pos: (f32, f32)) {
  let common_len = match (old_path, new_path) {
    (Some(a), Some(b)) => common_prefix(a, b).len(),
    _ => 0,
  };

  if let Some(old) = old_path {
    if old.len() > common_len {
      fire_chain_segment(
        tree,
        old,
        common_len,
        old.len(),
        /* deepest_first */ true,
        pos,
        old_path,
        HoverSlot::Leave,
      );
    }
    if new_path.is_none() {
      fire_root_hover_event(tree, pos, old, HoverSlot::Leave);
    }
  }

  if let Some(new) = new_path {
    if old_path.is_none() {
      fire_root_hover_event(tree, pos, new, HoverSlot::Enter);
    }
    if new.len() > common_len {
      fire_chain_segment(
        tree,
        new,
        common_len,
        new.len(),
        /* deepest_first */ false,
        pos,
        new_path,
        HoverSlot::Enter,
      );
    }
  }
}

fn fire_root_hover_event(tree: &mut Tree, pos: (f32, f32), target_path: &[usize], slot: HoverSlot) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let buttons_down = tree.interaction.buttons_down;
  let modifiers = tree.interaction.modifiers;
  let current_path: Vec<usize> = vec![];

  let (mouse_cbs, event_cbs) = tree
    .root
    .as_ref()
    .map(|root| {
      let mouse_cbs = match slot {
        HoverSlot::Enter => root.on_mouse_enter.clone(),
        HoverSlot::Leave => root.on_mouse_leave.clone(),
        HoverSlot::DragEnter => root.on_dragenter.clone(),
        HoverSlot::DragLeave => root.on_dragleave.clone(),
      };
      (mouse_cbs, root.on_event.clone())
    })
    .unwrap_or_default();

  let mut ev = MouseEvent {
    pos,
    button: None,
    modifiers,
    target_path: target_path.to_vec(),
    current_path: current_path.clone(),
  };
  if tree.emit_mouse_event(&mut ev).is_stop() {
    return;
  }
  for cb in &mouse_cbs {
    cb(&ev);
  }

  let mut html_ev = make_mouse_html_event(
    slot.event_type_str(),
    /* bubbles */ false,
    0,
    pos,
    None,
    buttons_down,
    modifiers,
    target_path,
    current_path,
    time_stamp,
    ev::EventPhase::AtTarget,
  );
  if tree.emit_event(&mut html_ev).is_stop() {
    return;
  }
  for cb in &event_cbs {
    cb(&html_ev);
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
  target_path: Option<&[usize]>,
  slot: HoverSlot,
) {
  if end <= start {
    return;
  }

  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let buttons_down = tree.interaction.buttons_down;
  let modifiers = tree.interaction.modifiers;

  let target = target_path.map(|p| p.to_vec()).unwrap_or_default();
  let count = end - start;
  for idx in 0..count {
    let plen = if deepest_first {
      end - idx
    } else {
      end - (count - 1 - idx)
    };
    let current_path = path[..plen].to_vec();

    let (mouse_cbs, event_cbs) = tree
      .root
      .as_ref()
      .and_then(|root| root.at_path(&current_path))
      .map(|node| {
        let mouse_cbs = match slot {
          HoverSlot::Enter => node.on_mouse_enter.clone(),
          HoverSlot::Leave => node.on_mouse_leave.clone(),
          HoverSlot::DragEnter => node.on_dragenter.clone(),
          HoverSlot::DragLeave => node.on_dragleave.clone(),
        };
        (mouse_cbs, node.on_event.clone())
      })
      .unwrap_or_default();

    let mut ev = MouseEvent {
      pos,
      button: None,
      modifiers,
      target_path: target.clone(),
      current_path: current_path.clone(),
    };
    if tree.emit_mouse_event(&mut ev).is_stop() {
      return;
    }
    for cb in &mouse_cbs {
      cb(&ev);
    }

    let mut html_ev = make_mouse_html_event(
      slot.event_type_str(),
      /* bubbles */ false,
      0,
      pos,
      None,
      buttons_down,
      modifiers,
      &target,
      current_path,
      time_stamp,
      ev::EventPhase::AtTarget,
    );
    if tree.emit_event(&mut html_ev).is_stop() {
      return;
    }
    for cb in &event_cbs {
      cb(&html_ev);
    }
  }
}

/// Return the longest path that is a prefix of both `a` and `b`.
fn common_prefix<'a>(a: &'a [usize], b: &[usize]) -> &'a [usize] {
  let n = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();
  &a[..n]
}

// ── Mouse public API ─────────────────────────────────────────────────────────

/// Update the hover chain to `new_target` and fire any
/// `mouseenter` / `mouseleave` callbacks the change implies.
///
/// `new_target` is the path the host already hit-tested with its
/// layout (`None` = pointer is over no element). `text_cursor`,
/// when supplied, drags the active text selection's focus point if
/// `selecting_text` is set; pass `None` if the position isn't
/// over a text run.
///
/// Modifier state is read from `tree.interaction.modifiers`;
/// hosts update it through [`Tree::set_modifier`].
///
/// Returns `true` if the hover path actually changed.
pub fn dispatch_pointer_move(
  tree: &mut Tree,
  new_target: Option<&[usize]>,
  pos: (f32, f32),
  text_cursor: Option<TextCursor>,
) -> bool {
  tree.interaction.pointer_pos = Some(pos);

  if tree.interaction.selecting_text {
    if let Some(cursor) = text_cursor {
      if let Some(sel) = tree.interaction.selection.as_mut() {
        sel.focus = cursor;
        selectionchange_event(tree);
      }
    }
  }

  let new_owned = new_target.map(<[usize]>::to_vec);
  let changed = tree.interaction.hover_path != new_owned;
  if changed {
    let old = tree.interaction.hover_path.take();
    update_hover(tree, old.as_deref(), new_target, pos);
    tree.interaction.hover_path = new_owned;
  }

  // Fire `mousemove` on the current target regardless of whether
  // the hover path changed (browsers fire it on every pixel move).
  if let Some(target) = tree.interaction.hover_path.clone() {
    bubble_mouse_move(tree, &target, pos);
  }

  // Drag-start detection: if drag_pending and moved ≥ 5 px, fire dragstart.
  if let Some((ref drag_src, (start_x, start_y))) = tree.interaction.drag_pending {
    let dx = pos.0 - start_x;
    let dy = pos.1 - start_y;
    if (dx * dx + dy * dy) >= 25.0 {
      // sqrt(25) = 5, so 5px Euclidean distance
      let src_path = drag_src.clone();
      tree.interaction.drag_pending = None;
      tree.interaction.drag_active_source = Some(src_path.clone());
      bubble(tree, &src_path, pos, Some(MouseButton::Primary), Slot::DragStart);
    }
  }

  // Fire drag/dragover while dragging.
  let drag_src = tree.interaction.drag_active_source.clone();
  let hover = tree.interaction.hover_path.clone();
  if let Some(src) = drag_src {
    bubble(tree, &src, pos, Some(MouseButton::Primary), Slot::Drag);
    if let Some(tgt) = &hover {
      bubble(tree, tgt, pos, Some(MouseButton::Primary), Slot::DragOver);
    }

    // dragenter / dragleave on drag target change.
    let old_drag = tree.interaction.drag_target_path.clone();
    if old_drag != hover {
      let common_len = match (&old_drag, &hover) {
        (Some(a), Some(b)) => common_prefix(a, b).len(),
        _ => 0,
      };
      // dragleave on old \ new (deepest first)
      if let Some(old) = &old_drag {
        if old.len() > common_len {
          fire_chain_segment(
            tree,
            old,
            common_len,
            old.len(),
            true,
            pos,
            Some(old.as_slice()),
            HoverSlot::DragLeave,
          );
        }
        if hover.is_none() {
          fire_root_hover_event(tree, pos, old, HoverSlot::DragLeave);
        }
      }
      // dragenter on new \ old (root first)
      if let Some(new) = &hover {
        if old_drag.is_none() {
          fire_root_hover_event(tree, pos, new, HoverSlot::DragEnter);
        }
        if new.len() > common_len {
          fire_chain_segment(
            tree,
            new,
            common_len,
            new.len(),
            false,
            pos,
            Some(new.as_slice()),
            HoverSlot::DragEnter,
          );
        }
      }
      tree.interaction.drag_target_path = hover;
    }
  } else {
    // Drag ended — clear drag target path.
    if tree.interaction.drag_target_path.is_some() {
      tree.interaction.drag_target_path = None;
    }
  }

  changed
}

/// Bubble a `mousemove` event up the ancestry chain, calling each
/// node's `on_mouse_move` callback and the generic `on_event` slot.
fn bubble_mouse_move(tree: &mut Tree, target_path: &[usize], pos: (f32, f32)) {
  let modifiers = tree.interaction.modifiers;
  let depth = target_path.len();

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = target_path[..i].to_vec();
    fire_mouse_move_at(
      tree,
      target_path,
      &current_path,
      pos,
      modifiers,
      ev::EventPhase::CapturingPhase,
    );
  }

  // Target phase
  fire_mouse_move_at(tree, target_path, target_path, pos, modifiers, ev::EventPhase::AtTarget);

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = target_path[..i].to_vec();
    fire_mouse_move_at(
      tree,
      target_path,
      &current_path,
      pos,
      modifiers,
      ev::EventPhase::BubblingPhase,
    );
  }
}

fn fire_mouse_move_at(
  tree: &mut Tree,
  target_path: &[usize],
  current_path: &[usize],
  pos: (f32, f32),
  modifiers: Modifiers,
  phase: ev::EventPhase,
) {
  let current_path = current_path.to_vec();
  let (mouse_cbs, event_cbs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| (node.on_mouse_move.clone(), node.on_event.clone()))
    .unwrap_or_default();

  let ev = MouseEvent {
    pos,
    button: None,
    modifiers,
    target_path: target_path.to_vec(),
    current_path: current_path.clone(),
  };
  for cb in &mouse_cbs {
    cb(&ev);
  }

  if !event_cbs.is_empty() {
    let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
    let html_ev = make_mouse_html_event(
      ev::HtmlEventType::MOUSEMOVE,
      true,
      0,
      pos,
      None,
      tree.interaction.buttons_down,
      modifiers,
      target_path,
      current_path,
      time_stamp,
      phase,
    );
    for cb in &event_cbs {
      cb(&html_ev);
    }
  }
}

/// The pointer left the surface (e.g. winit `CursorLeft`). Clears
/// the hover path and fires `mouseleave` on the previously-hovered
/// chain.
pub fn dispatch_pointer_leave(tree: &mut Tree) {
  let pos = tree.interaction.pointer_pos.unwrap_or((-1.0, -1.0));
  let old = tree.interaction.hover_path.take();
  update_hover(tree, old.as_deref(), None, pos);
}

/// Primary-button (or any-button) press.
///
/// `target_path` is the result of layout-side hit-testing
/// (`None` = pointer is over no element). `text_cursor` is the
/// text cursor at `pos` if it falls inside a text run, used to
/// start a drag-select.
///
/// Records the active path for click synthesis on the matching
/// release; fires `mousedown` bubbling target → root; on a primary
/// press, also moves keyboard focus to the deepest focusable
/// ancestor of `target_path` (or clears focus if none).
///
/// Returns `true` if a target was hit.
pub fn dispatch_mouse_down(
  tree: &mut Tree,
  target_path: Option<&[usize]>,
  pos: (f32, f32),
  button: MouseButton,
  text_cursor: Option<TextCursor>,
) -> bool {
  tree.interaction.buttons_down |= button_to_bit(button);

  let Some(target_path) = target_path else {
    if button == MouseButton::Primary {
      tree.clear_selection();
    }
    return false;
  };
  if button == MouseButton::Primary {
    tree.interaction.active_path = Some(target_path.to_vec());
    if let Some(cursor) = text_cursor {
      tree.interaction.selection = Some(TextSelection {
        anchor: cursor.clone(),
        focus: cursor,
      });
      tree.interaction.selecting_text = true;
      selectionchange_event(tree);
    } else {
      tree.clear_selection();
    }
  }
  bubble(tree, target_path, pos, Some(button), Slot::MouseDown);

  // Record drag-pending if the target is draggable.
  if button == MouseButton::Primary {
    if element_is_draggable(tree, target_path) {
      tree.interaction.drag_pending = Some((target_path.to_vec(), pos));
    }
  }

  // Browser-style focus update: a primary press on a focusable
  // element (or one of its focusable ancestors) moves focus
  // there. A press anywhere else clears focus. Mirrors the
  // browser order — mousedown fires first, then focus/blur.
  if button == MouseButton::Primary {
    let new_focus = focusable_ancestor(tree, target_path);
    set_focus(tree, new_focus);
  }

  true
}

/// Mouse-up event. Fires `mouseup`; then, if `button` is
/// `Primary` and the release path shares its root with the press
/// path, synthesises a click and fires `click` bubbling.
///
/// `target_path` is the layout-hit-tested target at `pos`
/// (`None` = released over nothing). `text_cursor` is the text
/// cursor at `pos`, used to finalise a drag-select.
pub fn dispatch_mouse_up(
  tree: &mut Tree,
  target_path: Option<&[usize]>,
  pos: (f32, f32),
  button: MouseButton,
  text_cursor: Option<TextCursor>,
) -> bool {
  tree.interaction.buttons_down &= !button_to_bit(button);

  let Some(target_path) = target_path else {
    if button == MouseButton::Primary {
      tree.interaction.active_path = None;
      tree.interaction.selecting_text = false;
    }
    return false;
  };

  bubble(tree, target_path, pos, Some(button), Slot::MouseUp);

  // contextmenu on secondary-button release.
  if button == MouseButton::Secondary {
    // Use the release target (not common ancestor) for contextmenu.
    bubble(tree, target_path, pos, Some(button), Slot::ContextMenu);
  }

  // auxclick on middle-button release.
  if button == MouseButton::Middle {
    bubble(tree, target_path, pos, Some(button), Slot::AuxClick);
  }

  if button == MouseButton::Primary {
    // Drag cleanup: clear pending drag, or finalise active drag.
    tree.interaction.drag_pending = None;
    let drag_was_active = tree.interaction.drag_active_source.take();

    if tree.interaction.selecting_text {
      if let Some(cursor) = text_cursor {
        if let Some(sel) = tree.interaction.selection.as_mut() {
          sel.focus = cursor;
          selectionchange_event(tree);
        }
      }
    }
    let press = tree.interaction.active_path.take();
    let suppress_click = drag_was_active.is_some()
      || tree
        .interaction
        .selection
        .as_ref()
        .is_some_and(|sel| tree.interaction.selecting_text && !sel.is_collapsed());
    tree.interaction.selecting_text = false;

    // Finalise drag: fire dragend on source, drop on release target.
    if let Some(drag_src) = drag_was_active {
      bubble(tree, &drag_src, pos, Some(button), Slot::DragEnd);
      bubble(tree, target_path, pos, Some(button), Slot::Drop);
    }

    if let Some(press_path) = press {
      let click_target = common_prefix(&press_path, target_path);
      if !suppress_click {
        bubble(tree, click_target, pos, Some(button), Slot::Click);

        // Double-click detection: same element, within 300 ms.
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(tree.interaction.last_click_time);
        let same_target = tree.interaction.last_click_path.as_deref() == Some(click_target);
        if elapsed.as_millis() < 300 && same_target {
          bubble(tree, click_target, pos, Some(button), Slot::DblClick);
        }
        tree.interaction.last_click_time = now;
        tree.interaction.last_click_path = Some(click_target.to_vec());
      }
      // Check if click landed on a submit button inside a form.
      if let Some((form_path, submitter_path)) = submit_button_in_form(tree, click_target) {
        bubble_submit_event(tree, &form_path, Some(submitter_path));
      }

      // Checkbox / radio toggle on click.
      toggle_checkable(tree, click_target);
    }
  }
  true
}

// ── Focus ────────────────────────────────────────────────────────────────────

/// Walk `path` from deepest to root and return the first ancestor
/// (inclusive of the target) whose `Element` is focusable per
/// [`crate::is_focusable`]. Returns `None` if no element on the
/// chain is focusable.
fn focusable_ancestor(tree: &Tree, path: &[usize]) -> Option<Vec<usize>> {
  let root = tree.root.as_ref()?;
  let mut chain: Vec<&crate::Element> = Vec::with_capacity(path.len() + 1);
  chain.push(&root.element);
  let mut cursor = root;
  for &i in path {
    let Some(child) = cursor.children.get(i) else {
      break;
    };
    chain.push(&child.element);
    cursor = child;
  }
  for (rev_i, el) in chain.iter().rev().enumerate() {
    if crate::is_focusable(el) {
      let depth = chain.len() - 1 - rev_i;
      return Some(path[..depth].to_vec());
    }
  }
  None
}

/// Move focus to `new_path`, firing `blur` / `focusout` on the
/// previous focus and `focus` / `focusin` on the new one. `None`
/// blurs without focusing anything.
///
/// Returns `true` if the focus path actually changed.
fn set_focus(tree: &mut Tree, new_path: Option<Vec<usize>>) -> bool {
  let old_path = tree.interaction.focus_path.clone();
  if old_path == new_path {
    return false;
  }

  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;

  // Fire change event if the previous focus target had its value mutated.
  if let Some(old) = old_path.as_deref() {
    let old_snapshot = tree.interaction.focus_value_snapshot.take();
    if let Some(snap) = old_snapshot {
      let current_val = tree
        .root
        .as_ref()
        .and_then(|root| root.at_path(old))
        .and_then(|node| read_editable_value(node).map(|(v, ..)| v));
      if let Some(current) = current_val {
        if current != snap {
          fire_focus_event(
            tree,
            old,
            ev::HtmlEventType::CHANGE,
            /* bubbles */ true,
            None,
            time_stamp,
            FocusBubbleKind::Bubble,
          );
        }
      }
    }

    // Date input: validate and parse formatted display value back to ISO on blur.
    // If invalid, the display value is discarded and inp.value stays unchanged (revert).
    if let Some(display) = tree.interaction.date_display_value.take() {
      use wgpu_html_models::common::html_enums::InputType;
      let is_datetime = tree.root.as_ref()
        .and_then(|r| r.at_path(old))
        .map(|n| matches!(&n.element, Element::Input(inp) if matches!(inp.r#type, Some(InputType::DatetimeLocal))))
        .unwrap_or(false);
      let pattern = if is_datetime { tree.locale.datetime_pattern() } else { tree.locale.date_pattern().to_string() };
      let segs = crate::date::parse_pattern_segments(&pattern);
      let valid = crate::date::validate_formatted(&display, &segs);
      if valid {
        if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(old)) {
          if let Element::Input(inp) = &mut node.element {
            if matches!(inp.r#type, Some(InputType::Date)) {
              if let Some((y, m, d)) = crate::date::parse_formatted_date(&display, &pattern) {
                inp.value = Some(crate::date::format_date(y, m, d).into());
                tree.generation += 1;
                tree.dirty_paths.push(old.to_vec());
              }
            } else if matches!(inp.r#type, Some(InputType::DatetimeLocal)) {
              if let Some((y, m, d, h, min)) = crate::date::parse_formatted_datetime(&display, &pattern) {
                inp.value = Some(crate::date::format_datetime_local(y, m, d, h, min).into());
                tree.generation += 1;
                tree.dirty_paths.push(old.to_vec());
              }
            }
          }
        }
      }
      // If not valid, inp.value stays unchanged — effectively reverts.
      tree.generation += 1;
      tree.dirty_paths.push(old.to_vec());
    }

    fire_focus_event(
      tree,
      old,
      ev::HtmlEventType::BLUR,
      /* bubbles */ false,
      new_path.clone(),
      time_stamp,
      FocusBubbleKind::Target,
    );
    fire_focus_event(
      tree,
      old,
      ev::HtmlEventType::FOCUSOUT,
      /* bubbles */ true,
      new_path.clone(),
      time_stamp,
      FocusBubbleKind::Bubble,
    );
  }

  tree.interaction.focus_path = new_path.clone();
  tree.interaction.undo_stack.clear();
  tree.interaction.edit_scroll_x = 0.0;

  // Initialize or clear the edit cursor for the new focus target.
  tree.interaction.edit_cursor = new_path.as_deref().and_then(|path| {
    let node = tree.root.as_ref()?.at_path(path)?;
    match &node.element {
      Element::Input(inp) => {
        use wgpu_html_models::common::html_enums::InputType;
        if matches!(
          inp.r#type,
          Some(
            InputType::Hidden
              | InputType::Checkbox
              | InputType::Radio
              | InputType::Button
              | InputType::Submit
              | InputType::Reset
              | InputType::File
              | InputType::Image
              | InputType::Color
              | InputType::Range
          )
        ) {
          return None;
        }
        let is_date = matches!(inp.r#type, Some(InputType::Date) | Some(InputType::DatetimeLocal));
        if is_date {
          let iso = inp.value.as_deref().unwrap_or("");
          let display = if matches!(inp.r#type, Some(InputType::DatetimeLocal)) {
            if let Some((y, m, d, h, min)) = crate::date::parse_datetime_local(iso) {
              tree.locale.format_datetime(y, m, d, h, min)
            } else {
              tree.locale.datetime_placeholder()
            }
          } else {
            if let Some((y, m, d)) = crate::date::parse_date(iso) {
              tree.locale.format_date(y, m, d)
            } else {
              tree.locale.date_placeholder()
            }
          };
          let pattern = focused_date_pattern_for(tree, inp.r#type.as_ref());
          let segs = crate::date::parse_pattern_segments(&pattern);
          let editable = crate::date::editable_segment_indices(&segs);
          let cursor = if let Some(&first) = editable.first() {
            let s = &segs[first];
            crate::EditCursor { cursor: s.byte_start + s.byte_len, selection_anchor: Some(s.byte_start) }
          } else {
            crate::EditCursor::collapsed(display.len())
          };
          tree.interaction.date_display_value = Some(display);
          return Some(cursor);
        }
        tree.interaction.date_display_value = None;
        let len = inp.value.as_deref().unwrap_or("").len();
        Some(crate::EditCursor::collapsed(len))
      }
      Element::Textarea(ta) => {
        let len = textarea_value(ta, &node.children).len();
        Some(crate::EditCursor::collapsed(len))
      }
      _ => None,
    }
  });
  tree.interaction.caret_blink_epoch = std::time::Instant::now();

  // Snapshot the value for change-event detection.
  tree.interaction.focus_value_snapshot = new_path.as_deref().and_then(|path| {
    let node = tree.root.as_ref()?.at_path(path)?;
    read_editable_value(node).map(|(v, ..)| v)
  });

  if let Some(new) = new_path.as_deref() {
    fire_focus_event(
      tree,
      new,
      ev::HtmlEventType::FOCUS,
      /* bubbles */ false,
      old_path.clone(),
      time_stamp,
      FocusBubbleKind::Target,
    );
    fire_focus_event(
      tree,
      new,
      ev::HtmlEventType::FOCUSIN,
      /* bubbles */ true,
      old_path.clone(),
      time_stamp,
      FocusBubbleKind::Bubble,
    );
  }

  tree.emit_active_element_changed(old_path.as_deref(), new_path.as_deref());

  true
}

#[derive(Copy, Clone)]
enum FocusBubbleKind {
  /// Fire on the target node only.
  Target,
  /// Fire on the target and walk ancestors target → root.
  Bubble,
}

fn fire_focus_event(
  tree: &mut Tree,
  target_path: &[usize],
  event_type: &'static str,
  bubbles: bool,
  related: Option<Vec<usize>>,
  time_stamp: f64,
  kind: FocusBubbleKind,
) {
  let _ = kind;
  let target = target_path.to_vec();
  let depth = target_path.len();

  // Capture phase: root → target parent (fires for all events)
  for i in 0..depth {
    let current_path = &target_path[..i];
    if fire_focus_at(
      tree,
      event_type,
      &target,
      current_path,
      time_stamp,
      related.clone(),
      ev::EventPhase::CapturingPhase,
    ) {
      return;
    }
  }

  // Target phase (always fires)
  {
    if fire_focus_at(
      tree,
      event_type,
      &target,
      target_path,
      time_stamp,
      related.clone(),
      ev::EventPhase::AtTarget,
    ) {
      return;
    }
  }

  // Bubble phase: target parent → root
  if bubbles {
    for i in (0..depth).rev() {
      let current_path = &target_path[..i];
      if fire_focus_at(
        tree,
        event_type,
        &target,
        current_path,
        time_stamp,
        related.clone(),
        ev::EventPhase::BubblingPhase,
      ) {
        return;
      }
    }
  }
}

fn fire_focus_at(
  tree: &mut Tree,
  event_type: &'static str,
  target: &[usize],
  current_path: &[usize],
  time_stamp: f64,
  related: Option<Vec<usize>>,
  phase: ev::EventPhase,
) -> bool {
  // returns true if should stop
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| {
      let d = match event_type {
        ev::HtmlEventType::FOCUS => node.on_focus.clone(),
        ev::HtmlEventType::BLUR => node.on_blur.clone(),
        ev::HtmlEventType::FOCUSIN => node.on_focusin.clone(),
        ev::HtmlEventType::FOCUSOUT => node.on_focusout.clone(),
        ev::HtmlEventType::CHANGE => node.on_change.clone(),
        _ => Vec::new(),
      };
      (d, node.on_event.clone())
    })
    .unwrap_or_default();
  let mut html_ev = ev::HtmlEvent::Focus(ev::events::FocusEvent {
    base: ev::events::UIEvent {
      base: ev::events::Event {
        event_type: ev::HtmlEventType::from(event_type),
        bubbles: true,
        cancelable: false,
        composed: true,
        target: Some(target.to_vec()),
        current_target: Some(current_path),
        event_phase: phase,
        default_prevented: Cell::new(false),
        propagation_stopped: Cell::new(false),
        immediate_propagation_stopped: Cell::new(false),
        is_trusted: true,
        time_stamp,
      },
      detail: 0,
    },
    related_target: related.clone(),
  });
  if tree.emit_event(&mut html_ev).is_stop() {
    return true;
  }
  for cb in &dedicated {
    cb(&html_ev);
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
  false
}

/// Move focus to the element at `path`, walking up to the nearest
/// focusable ancestor if `path` itself is not focusable. Pass
/// `None` to clear focus (equivalent to [`blur`]).
///
/// Returns `true` if the focus path actually changed.
pub fn focus(tree: &mut Tree, path: Option<&[usize]>) -> bool {
  let new = match path {
    Some(p) => focusable_ancestor(tree, p),
    None => None,
  };
  set_focus(tree, new)
}

/// Clear focus and fire `blur` / `focusout` on the previously-
/// focused element. Returns `true` if focus was cleared.
pub fn blur(tree: &mut Tree) -> bool {
  set_focus(tree, None)
}

/// Advance focus to the next keyboard-focusable element in
/// document order, wrapping around. Returns the new focus path,
/// or `None` if the document has no keyboard-focusable elements.
pub fn focus_next(tree: &mut Tree, reverse: bool) -> Option<Vec<usize>> {
  let paths = keyboard_focusable_paths(tree);
  if paths.is_empty() {
    return None;
  }
  let cur = tree.interaction.focus_path.clone();
  let next = if reverse {
    prev_in_order(&paths, cur.as_deref()).map(<[usize]>::to_vec)
  } else {
    next_in_order(&paths, cur.as_deref()).map(<[usize]>::to_vec)
  };
  if let Some(p) = next.as_ref() {
    set_focus(tree, Some(p.clone()));
  }
  next
}

// ── Keyboard ────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn make_keyboard_html_event(
  event_type: &'static str,
  key: &str,
  code: &str,
  repeat: bool,
  modifiers: Modifiers,
  target_path: &[usize],
  current_path: Vec<usize>,
  time_stamp: f64,
  event_phase: ev::EventPhase,
) -> ev::HtmlEvent {
  ev::HtmlEvent::Keyboard(ev::events::KeyboardEvent {
    base: ev::events::UIEvent {
      base: ev::events::Event {
        event_type: ev::HtmlEventType::from(event_type),
        bubbles: true,
        cancelable: true,
        composed: true,
        target: Some(target_path.to_vec()),
        current_target: Some(current_path),
        event_phase,
        default_prevented: Cell::new(false),
        propagation_stopped: Cell::new(false),
        immediate_propagation_stopped: Cell::new(false),
        is_trusted: true,
        time_stamp,
      },
      detail: 0,
    },
    key: key.to_owned(),
    code: code.to_owned(),
    location: ev::enums::KeyboardLocation::Standard,
    ctrl_key: modifiers.ctrl,
    shift_key: modifiers.shift,
    alt_key: modifiers.alt,
    meta_key: modifiers.meta,
    repeat,
    is_composing: false,
  })
}

fn bubble_keyboard(
  tree: &mut Tree,
  target_path: &[usize],
  event_type: &'static str,
  key: &str,
  code: &str,
  repeat: bool,
) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let modifiers = tree.interaction.modifiers;
  let depth = target_path.len();

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &target_path[..i];
    if fire_keyboard_at(
      tree,
      event_type,
      key,
      code,
      repeat,
      modifiers,
      target_path,
      current_path,
      time_stamp,
      ev::EventPhase::CapturingPhase,
    ) {
      return;
    }
  }

  // Target phase
  if fire_keyboard_at(
    tree,
    event_type,
    key,
    code,
    repeat,
    modifiers,
    target_path,
    target_path,
    time_stamp,
    ev::EventPhase::AtTarget,
  ) {
    return;
  }

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = &target_path[..i];
    if fire_keyboard_at(
      tree,
      event_type,
      key,
      code,
      repeat,
      modifiers,
      target_path,
      current_path,
      time_stamp,
      ev::EventPhase::BubblingPhase,
    ) {
      return;
    }
  }
}

#[allow(clippy::too_many_arguments)]
fn fire_keyboard_at(
  tree: &mut Tree,
  event_type: &'static str,
  key: &str,
  code: &str,
  repeat: bool,
  modifiers: Modifiers,
  target_path: &[usize],
  current_path: &[usize],
  time_stamp: f64,
  phase: ev::EventPhase,
) -> bool {
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| {
      let d = if event_type == ev::HtmlEventType::KEYDOWN {
        node.on_keydown.clone()
      } else {
        node.on_keyup.clone()
      };
      (d, node.on_event.clone())
    })
    .unwrap_or_default();
  let mut html_ev = make_keyboard_html_event(
    event_type,
    key,
    code,
    repeat,
    modifiers,
    target_path,
    current_path,
    time_stamp,
    phase,
  );
  if tree.emit_event(&mut html_ev).is_stop() {
    return true;
  }
  for cb in &dedicated {
    cb(&html_ev);
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
  false
}

/// Dispatch `keydown` to the focused element (or document root if
/// nothing is focused), bubbling target → root.
///
/// Modifier state is read from `tree.interaction.modifiers`; hosts
/// keep it in sync via [`Tree::set_modifier`].
///
/// Tab handling: when `key == "Tab"`, the dispatcher first delivers
/// the keydown event so listeners observe the keystroke, then
/// advances focus to the next/previous keyboard-focusable element
/// (Shift held = reverse). Hosts that want to suppress tab
/// navigation should not forward the `Tab` key.
pub fn key_down(tree: &mut Tree, key: &str, code: &str, repeat: bool) -> bool {
  let target = tree.interaction.focus_path.clone().unwrap_or_else(Vec::new);
  bubble_keyboard(tree, &target, ev::HtmlEventType::KEYDOWN, key, code, repeat);

  // Undo / redo (use physical `code` so it works regardless of keyboard language).
  if tree.interaction.modifiers.ctrl {
    let shift = tree.interaction.modifiers.shift;
    if code == "KeyZ" && !shift {
      handle_undo(tree);
      return true;
    }
    if code == "KeyY" || (code == "KeyZ" && shift) {
      handle_redo(tree);
      return true;
    }
  }

  // Number / range ArrowUp/ArrowDown stepping.
  if matches!(key, "ArrowUp" | "ArrowDown") {
    if handle_numeric_step(tree, key) {
      return true;
    }
  }

  // Handle editing keys on focused form controls before Tab.
  if handle_edit_key(tree, key, code) {
    return true;
  }

  if key == "Tab" {
    let reverse = tree.interaction.modifiers.shift;
    focus_next(tree, reverse);
    return true;
  }

  if key == "Enter" || key == " " {
    if key == "Enter" {
      if let Some((form_path, submitter_path)) = enter_in_form_input(tree) {
        bubble_submit_event(tree, &form_path, Some(submitter_path));
      }
    }
    handle_activation_key(tree);
    return true;
  }

  true
}

/// Activate the focused element via Enter or Space. Synthesises a
/// `click` for buttons, links, and checkboxes.
fn handle_activation_key(tree: &mut Tree) {
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return;
  };
  let Some(root) = tree.root.as_ref() else {
    return;
  };
  let Some(node) = root.at_path(&focus_path) else {
    return;
  };
  use wgpu_html_models::common::html_enums::InputType;
  let should_click = match &node.element {
    Element::Button(_) => true,
    Element::A(a) => a.href.is_some(),
    Element::Input(inp) => matches!(
      inp.r#type,
      Some(InputType::Checkbox) | Some(InputType::Submit) | Some(InputType::Reset) | Some(InputType::Button)
    ),
    _ => false,
  };
  if should_click {
    bubble(tree, &focus_path, (0.0, 0.0), Some(MouseButton::Primary), Slot::Click);
    toggle_checkable(tree, &focus_path);
    if let Some((form_path, submitter_path)) = submit_button_in_form(tree, &focus_path) {
      bubble_submit_event(tree, &form_path, Some(submitter_path));
    }
  }
}

/// Check whether the element at `path` has `draggable` set to true.
fn element_is_draggable(tree: &Tree, path: &[usize]) -> bool {
  let Some(root) = tree.root.as_ref() else { return false };
  let Some(node) = root.at_path(path) else { return false };
  node.draggable
}

/// Dispatch `keyup` to the focused element (or document root if
/// nothing is focused), bubbling target → root.
pub fn key_up(tree: &mut Tree, key: &str, code: &str) -> bool {
  let target = tree.interaction.focus_path.clone().unwrap_or_else(Vec::new);
  bubble_keyboard(
    tree,
    &target,
    ev::HtmlEventType::KEYUP,
    key,
    code,
    /* repeat */ false,
  );
  true
}

// ── Input event ──────────────────────────────────────────────────────────────

fn make_input_html_event(
  event_type: &'static str,
  data: Option<String>,
  input_type: ev::enums::InputType,
  value: Option<String>,
  checked: Option<bool>,
  target_path: &[usize],
  current_path: Vec<usize>,
  time_stamp: f64,
  event_phase: ev::EventPhase,
) -> ev::HtmlEvent {
  ev::HtmlEvent::Input(ev::events::InputEvent {
    base: ev::events::UIEvent {
      base: ev::events::Event {
        event_type: ev::HtmlEventType::from(event_type),
        bubbles: true,
        cancelable: true,
        composed: true,
        target: Some(target_path.to_vec()),
        current_target: Some(current_path),
        event_phase,
        default_prevented: Cell::new(false),
        propagation_stopped: Cell::new(false),
        immediate_propagation_stopped: Cell::new(false),
        is_trusted: true,
        time_stamp,
      },
      detail: 0,
    },
    data,
    input_type,
    value,
    checked,
    is_composing: false,
  })
}

fn bubble_input(tree: &mut Tree, target_path: &[usize], data: Option<String>, input_type: ev::enums::InputType) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = target_path.len();
  let (value, checked) = form_control_state(tree, target_path);

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &target_path[..i];
    if fire_input_at(
      tree,
      ev::HtmlEventType::INPUT,
      target_path,
      current_path,
      data.clone(),
      input_type.clone(),
      value.clone(),
      checked,
      time_stamp,
      ev::EventPhase::CapturingPhase,
    ) {
      return;
    }
  }

  // Target phase
  if fire_input_at(
    tree,
    ev::HtmlEventType::INPUT,
    target_path,
    target_path,
    data.clone(),
    input_type.clone(),
    value.clone(),
    checked,
    time_stamp,
    ev::EventPhase::AtTarget,
  ) {
    return;
  }

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = &target_path[..i];
    if fire_input_at(
      tree,
      ev::HtmlEventType::INPUT,
      target_path,
      current_path,
      data.clone(),
      input_type.clone(),
      value.clone(),
      checked,
      time_stamp,
      ev::EventPhase::BubblingPhase,
    ) {
      return;
    }
  }
}

/// Bubble a `beforeinput` event target → root. Returns `true` if
/// `preventDefault()` was called (caller should skip the mutation).
fn bubble_beforeinput(
  tree: &mut Tree,
  target_path: &[usize],
  data: Option<String>,
  input_type: ev::enums::InputType,
) -> bool {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = target_path.len();
  let mut prevented = false;
  let (value, checked) = form_control_state(tree, target_path);

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &target_path[..i];
    let (stop, prev) = fire_input_preventable_at(
      tree,
      ev::HtmlEventType::BEFOREINPUT,
      target_path,
      current_path,
      data.clone(),
      input_type.clone(),
      value.clone(),
      checked,
      time_stamp,
      ev::EventPhase::CapturingPhase,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  // Target phase
  {
    let (stop, prev) = fire_input_preventable_at(
      tree,
      ev::HtmlEventType::BEFOREINPUT,
      target_path,
      target_path,
      data.clone(),
      input_type.clone(),
      value.clone(),
      checked,
      time_stamp,
      ev::EventPhase::AtTarget,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = &target_path[..i];
    let (stop, prev) = fire_input_preventable_at(
      tree,
      ev::HtmlEventType::BEFOREINPUT,
      target_path,
      current_path,
      data.clone(),
      input_type.clone(),
      value.clone(),
      checked,
      time_stamp,
      ev::EventPhase::BubblingPhase,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  prevented
}

#[allow(clippy::too_many_arguments)]
fn fire_input_at(
  tree: &mut Tree,
  event_type: &'static str,
  target_path: &[usize],
  current_path: &[usize],
  data: Option<String>,
  input_type: ev::enums::InputType,
  value: Option<String>,
  checked: Option<bool>,
  time_stamp: f64,
  phase: ev::EventPhase,
) -> bool {
  // returns true if should stop
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| (node.on_input.clone(), node.on_event.clone()))
    .unwrap_or_default();
  let mut html_ev = make_input_html_event(
    event_type,
    data,
    input_type,
    value,
    checked,
    target_path,
    current_path,
    time_stamp,
    phase,
  );
  if tree.emit_event(&mut html_ev).is_stop() {
    return true;
  }
  for cb in &dedicated {
    cb(&html_ev);
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
  false
}

#[allow(clippy::too_many_arguments)]
fn fire_input_preventable_at(
  tree: &mut Tree,
  event_type: &'static str,
  target_path: &[usize],
  current_path: &[usize],
  data: Option<String>,
  input_type: ev::enums::InputType,
  value: Option<String>,
  checked: Option<bool>,
  time_stamp: f64,
  phase: ev::EventPhase,
) -> (bool, bool) {
  // returns (should_stop, prevented)
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| (node.on_beforeinput.clone(), node.on_event.clone()))
    .unwrap_or_default();
  let mut html_ev = make_input_html_event(
    event_type,
    data,
    input_type,
    value,
    checked,
    target_path,
    current_path,
    time_stamp,
    phase,
  );
  if tree.emit_event(&mut html_ev).is_stop() {
    return (true, false);
  }
  for cb in &dedicated {
    cb(&html_ev);
    if html_ev.base().immediate_propagation_stopped.get() {
      break;
    }
  }
  if !html_ev.base().immediate_propagation_stopped.get() {
    for on_ev in &on_evs {
      on_ev(&html_ev);
      if html_ev.base().immediate_propagation_stopped.get() {
        break;
      }
    }
  }
  let prevented = html_ev.base().default_prevented.get();
  (html_ev.base().propagation_stopped.get(), prevented)
}

// ── Wheel event ──────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn make_wheel_html_event(
  event_type: &'static str,
  delta_x: f64,
  delta_y: f64,
  delta_mode: ev::enums::WheelDeltaMode,
  pos: (f32, f32),
  buttons_down: u16,
  modifiers: Modifiers,
  target_path: &[usize],
  current_path: Vec<usize>,
  time_stamp: f64,
  event_phase: ev::EventPhase,
) -> ev::HtmlEvent {
  ev::HtmlEvent::Wheel(ev::events::WheelEvent {
    base: ev::events::MouseEvent {
      base: ev::events::UIEvent {
        base: ev::events::Event {
          event_type: ev::HtmlEventType::from(event_type),
          bubbles: true,
          cancelable: true,
          composed: true,
          target: Some(target_path.to_vec()),
          current_target: Some(current_path),
          event_phase,
          default_prevented: Cell::new(false),
          propagation_stopped: Cell::new(false),
          immediate_propagation_stopped: Cell::new(false),
          is_trusted: true,
          time_stamp,
        },
        detail: 0,
      },
      screen_x: 0.0,
      screen_y: 0.0,
      client_x: pos.0 as f64,
      client_y: pos.1 as f64,
      offset_x: 0.0,
      offset_y: 0.0,
      page_x: pos.0 as f64,
      page_y: pos.1 as f64,
      movement_x: 0.0,
      movement_y: 0.0,
      button: 0,
      buttons: buttons_down,
      ctrl_key: modifiers.ctrl,
      shift_key: modifiers.shift,
      alt_key: modifiers.alt,
      meta_key: modifiers.meta,
      related_target: None,
    },
    delta_x,
    delta_y,
    delta_z: 0.0,
    delta_mode,
  })
}

/// Dispatch a `wheel` event to the element under the pointer (or
/// document root if nothing is hovered), bubbling target → root.
///
/// `delta_mode` should be [`WheelDeltaMode::Pixel`] when the host
/// has already converted the winit delta to screen pixels, or
/// `Line` when using winit's `LineDelta`.
///
/// Returns `true` if `preventDefault()` was called (caller should
/// skip the default scroll action).
pub fn wheel_event(
  tree: &mut Tree,
  pos: (f32, f32),
  delta_x: f64,
  delta_y: f64,
  delta_mode: ev::enums::WheelDeltaMode,
) -> bool {
  let target = tree.interaction.hover_path.clone().unwrap_or_else(Vec::new);
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let buttons_down = tree.interaction.buttons_down;
  let modifiers = tree.interaction.modifiers;
  let depth = target.len();
  let mut prevented = false;

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &target[..i];
    let (stop, prev) = fire_wheel_at(
      tree,
      &target,
      current_path,
      pos,
      delta_x,
      delta_y,
      delta_mode,
      buttons_down,
      modifiers,
      time_stamp,
      ev::EventPhase::CapturingPhase,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  // Target phase
  {
    let (stop, prev) = fire_wheel_at(
      tree,
      &target,
      &target,
      pos,
      delta_x,
      delta_y,
      delta_mode,
      buttons_down,
      modifiers,
      time_stamp,
      ev::EventPhase::AtTarget,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = &target[..i];
    let (stop, prev) = fire_wheel_at(
      tree,
      &target,
      current_path,
      pos,
      delta_x,
      delta_y,
      delta_mode,
      buttons_down,
      modifiers,
      time_stamp,
      ev::EventPhase::BubblingPhase,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  prevented
}

#[allow(clippy::too_many_arguments)]
fn fire_wheel_at(
  tree: &mut Tree,
  target_path: &[usize],
  current_path: &[usize],
  pos: (f32, f32),
  delta_x: f64,
  delta_y: f64,
  delta_mode: ev::enums::WheelDeltaMode,
  buttons_down: u16,
  modifiers: Modifiers,
  time_stamp: f64,
  phase: ev::EventPhase,
) -> (bool, bool) {
  // returns (should_stop, prevented)
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| (node.on_wheel.clone(), node.on_event.clone()))
    .unwrap_or_default();
  let mut html_ev = make_wheel_html_event(
    ev::HtmlEventType::WHEEL,
    delta_x,
    delta_y,
    delta_mode,
    pos,
    buttons_down,
    modifiers,
    target_path,
    current_path,
    time_stamp,
    phase,
  );
  if tree.emit_event(&mut html_ev).is_stop() {
    return (true, false);
  }
  for cb in &dedicated {
    cb(&html_ev);
    if html_ev.base().immediate_propagation_stopped.get() {
      break;
    }
  }
  if !html_ev.base().immediate_propagation_stopped.get() {
    for on_ev in &on_evs {
      on_ev(&html_ev);
      if html_ev.base().immediate_propagation_stopped.get() {
        break;
      }
    }
  }
  let prevented = html_ev.base().default_prevented.get();
  (html_ev.base().propagation_stopped.get(), prevented)
}

// ── Clipboard ────────────────────────────────────────────────────────────────

/// Dispatch a `copy`, `cut`, or `paste` clipboard event to the
/// focused element (or document root if nothing is focused),
/// bubbling target → root.
///
/// Returns `true` if `preventDefault()` was called on the event,
/// signalling the caller to skip the default clipboard operation.
pub fn clipboard_event(tree: &mut Tree, event_type: &'static str) -> bool {
  let target = tree.interaction.focus_path.clone().unwrap_or_else(Vec::new);
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = target.len();
  let mut prevented = false;

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &target[..i];
    let (stop, prev) = fire_clipboard_at(
      tree,
      event_type,
      &target,
      current_path,
      time_stamp,
      ev::EventPhase::CapturingPhase,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  // Target phase
  {
    let (stop, prev) = fire_clipboard_at(tree, event_type, &target, &target, time_stamp, ev::EventPhase::AtTarget);
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = &target[..i];
    let (stop, prev) = fire_clipboard_at(
      tree,
      event_type,
      &target,
      current_path,
      time_stamp,
      ev::EventPhase::BubblingPhase,
    );
    prevented = prevented || prev;
    if stop {
      return prevented;
    }
  }

  prevented
}

fn fire_clipboard_at(
  tree: &mut Tree,
  event_type: &'static str,
  target_path: &[usize],
  current_path: &[usize],
  time_stamp: f64,
  phase: ev::EventPhase,
) -> (bool, bool) {
  // returns (should_stop, prevented)
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| {
      let d = match event_type {
        ev::HtmlEventType::COPY => node.on_copy.clone(),
        ev::HtmlEventType::CUT => node.on_cut.clone(),
        ev::HtmlEventType::PASTE => node.on_paste.clone(),
        _ => Vec::new(),
      };
      (d, node.on_event.clone())
    })
    .unwrap_or_default();
  let mut html_ev = ev::HtmlEvent::Clipboard(ev::events::ClipboardEvent {
    base: ev::events::Event {
      event_type: ev::HtmlEventType::from(event_type),
      bubbles: true,
      cancelable: true,
      composed: true,
      target: Some(target_path.to_vec()),
      current_target: Some(current_path),
      event_phase: phase,
      default_prevented: Cell::new(false),
      propagation_stopped: Cell::new(false),
      immediate_propagation_stopped: Cell::new(false),
      is_trusted: true,
      time_stamp,
    },
    clipboard_data: None,
  });
  if tree.emit_event(&mut html_ev).is_stop() {
    return (true, false);
  }
  for cb in &dedicated {
    cb(&html_ev);
    if html_ev.base().immediate_propagation_stopped.get() {
      break;
    }
  }
  if !html_ev.base().immediate_propagation_stopped.get() {
    for on_ev in &on_evs {
      on_ev(&html_ev);
      if html_ev.base().immediate_propagation_stopped.get() {
        break;
      }
    }
  }
  let prevented = html_ev.base().default_prevented.get();
  (html_ev.base().propagation_stopped.get(), prevented)
}

/// Dispatch a `scroll` event on the element at `path` (non-bubbling,
/// target only). Capture phase fires on ancestors.
pub fn scroll_event(tree: &mut Tree, path: &[usize]) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = path.len();

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &path[..i];
    fire_generic_at(
      tree,
      ev::HtmlEventType::SCROLL,
      path,
      current_path,
      time_stamp,
      ev::EventPhase::CapturingPhase,
    );
  }
  // Target phase
  fire_generic_at(
    tree,
    ev::HtmlEventType::SCROLL,
    path,
    path,
    time_stamp,
    ev::EventPhase::AtTarget,
  );
}

/// Dispatch a `select` event on the element at `path` (non-bubbling,
/// target only). Called when the edit cursor selection range changes
/// inside an `<input>` or `<textarea>`. Capture phase fires on ancestors.
pub fn select_event(tree: &mut Tree, path: &[usize]) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = path.len();

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = &path[..i];
    fire_select_at(tree, path, current_path, time_stamp, ev::EventPhase::CapturingPhase);
  }
  // Target phase
  fire_select_at(tree, path, path, time_stamp, ev::EventPhase::AtTarget);
}

fn fire_generic_at(
  tree: &mut Tree,
  event_type: &str,
  target: &[usize],
  current_path: &[usize],
  time_stamp: f64,
  phase: ev::EventPhase,
) {
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| {
      let d = if event_type == ev::HtmlEventType::SCROLL {
        node.on_scroll.clone()
      } else {
        Vec::new()
      };
      (d, node.on_event.clone())
    })
    .unwrap_or_default();
  let mut html_ev = ev::HtmlEvent::Generic(ev::events::Event {
    event_type: ev::HtmlEventType::from(event_type),
    bubbles: false,
    cancelable: false,
    composed: false,
    target: Some(target.to_vec()),
    current_target: Some(current_path),
    event_phase: phase,
    default_prevented: Cell::new(false),
    propagation_stopped: Cell::new(false),
    immediate_propagation_stopped: Cell::new(false),
    is_trusted: true,
    time_stamp,
  });
  if tree.emit_event(&mut html_ev).is_stop() {
    return;
  }
  for cb in &dedicated {
    cb(&html_ev);
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
}

fn fire_select_at(tree: &mut Tree, target: &[usize], current_path: &[usize], time_stamp: f64, phase: ev::EventPhase) {
  let current_path = current_path.to_vec();
  let (dedicated, on_evs) = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| (node.on_select.clone(), node.on_event.clone()))
    .unwrap_or_default();
  let mut html_ev = ev::HtmlEvent::Generic(ev::events::Event {
    event_type: ev::HtmlEventType::from(ev::HtmlEventType::SELECT),
    bubbles: false,
    cancelable: false,
    composed: false,
    target: Some(target.to_vec()),
    current_target: Some(current_path),
    event_phase: phase,
    default_prevented: Cell::new(false),
    propagation_stopped: Cell::new(false),
    immediate_propagation_stopped: Cell::new(false),
    is_trusted: true,
    time_stamp,
  });
  if tree.emit_event(&mut html_ev).is_stop() {
    return;
  }
  for cb in &dedicated {
    cb(&html_ev);
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
}

/// Dispatch a `selectionchange` event on the document root.
/// Called when text selection changes.
pub fn selectionchange_event(tree: &mut Tree) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let on_evs = tree.root.as_ref().map(|node| node.on_event.clone()).unwrap_or_default();
  let mut html_ev = ev::HtmlEvent::Generic(ev::events::Event {
    event_type: ev::HtmlEventType::from(ev::HtmlEventType::SELECTIONCHANGE),
    bubbles: false,
    cancelable: false,
    composed: false,
    target: Some(vec![]),
    current_target: Some(vec![]),
    event_phase: ev::EventPhase::AtTarget,
    default_prevented: Cell::new(false),
    propagation_stopped: Cell::new(false),
    immediate_propagation_stopped: Cell::new(false),
    is_trusted: true,
    time_stamp,
  });
  if tree.emit_event(&mut html_ev).is_stop() {
    return;
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
}

/// Dispatch a `resize` event on the document root.
pub fn resize_event(tree: &mut Tree) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let on_evs = tree.root.as_ref().map(|node| node.on_event.clone()).unwrap_or_default();
  let html_ev = ev::HtmlEvent::Generic(ev::events::Event {
    event_type: ev::HtmlEventType::from(ev::HtmlEventType::RESIZE),
    bubbles: false,
    cancelable: false,
    composed: false,
    target: Some(vec![]),
    current_target: Some(vec![]),
    event_phase: ev::EventPhase::AtTarget,
    default_prevented: Cell::new(false),
    propagation_stopped: Cell::new(false),
    immediate_propagation_stopped: Cell::new(false),
    is_trusted: true,
    time_stamp,
  });
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
}

// ── Submit / Form ─────────────────────────────────────────────────────────────

/// Find the nearest `<form>` ancestor of the element at `path`.
/// Returns the path to the form element, or `None`.
fn find_ancestor_form(tree: &Tree, path: &[usize]) -> Option<Vec<usize>> {
  let root = tree.root.as_ref()?;
  for prefix_len in (0..=path.len()).rev() {
    let prefix = &path[..prefix_len];
    if let Some(node) = root.at_path(prefix) {
      if let Element::Form(_) = &node.element {
        return Some(prefix.to_vec());
      }
    }
  }
  None
}

/// Fire a `submit` event on the form element at `form_path` with
/// capture → target → bubble phases. `submitter_path` is the button
/// or input that triggered the submission.
fn bubble_submit_event(tree: &mut Tree, form_path: &[usize], submitter_path: Option<Vec<usize>>) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = form_path.len();

  // Capture phase: root → target parent
  for i in 0..depth {
    let current_path = form_path[..i].to_vec();
    fire_submit_at(
      tree,
      form_path,
      &current_path,
      submitter_path.clone(),
      time_stamp,
      ev::EventPhase::CapturingPhase,
    );
  }

  // Target phase
  fire_submit_at(
    tree,
    form_path,
    form_path,
    submitter_path.clone(),
    time_stamp,
    ev::EventPhase::AtTarget,
  );

  // Bubble phase: target parent → root
  for i in (0..depth).rev() {
    let current_path = form_path[..i].to_vec();
    fire_submit_at(
      tree,
      form_path,
      &current_path,
      submitter_path.clone(),
      time_stamp,
      ev::EventPhase::BubblingPhase,
    );
  }
}

fn fire_submit_at(
  tree: &mut Tree,
  form_path: &[usize],
  current_path: &[usize],
  submitter_path: Option<Vec<usize>>,
  time_stamp: f64,
  phase: ev::EventPhase,
) {
  let current_path = current_path.to_vec();
  let on_evs = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(&current_path))
    .map(|node| node.on_event.clone())
    .unwrap_or_default();
  if on_evs.is_empty() {
    return;
  }
  let mut html_ev = ev::HtmlEvent::Submit(ev::events::SubmitEvent {
    base: ev::events::Event {
      event_type: ev::HtmlEventType::from(ev::HtmlEventType::SUBMIT),
      bubbles: true,
      cancelable: true,
      composed: true,
      target: Some(form_path.to_vec()),
      current_target: Some(current_path),
      event_phase: phase,
      default_prevented: Cell::new(false),
      propagation_stopped: Cell::new(false),
      immediate_propagation_stopped: Cell::new(false),
      is_trusted: true,
      time_stamp,
    },
    submitter: submitter_path.clone(),
  });
  if tree.emit_event(&mut html_ev).is_stop() {
    return;
  }
  for on_ev in &on_evs {
    on_ev(&html_ev);
  }
}

/// Check if the element at `path` is a submit button (`<button>` or
/// `<input type="submit">`) and returns its containing form path if
/// found.
fn submit_button_in_form(tree: &Tree, path: &[usize]) -> Option<(Vec<usize>, Vec<usize>)> {
  use wgpu_html_models::common::html_enums::InputType;
  let root = tree.root.as_ref()?;
  let node = root.at_path(path)?;
  let is_submitter = match &node.element {
    Element::Button(btn) => {
      use wgpu_html_models::common::html_enums::ButtonType;
      !matches!(btn.r#type, Some(ButtonType::Reset | ButtonType::Button))
    }
    Element::Input(inp) => matches!(inp.r#type, Some(InputType::Submit)),
    _ => false,
  };
  if !is_submitter {
    return None;
  }
  let form_path = find_ancestor_form(tree, path)?;
  Some((form_path, path.to_vec()))
}

/// Check if `Enter` on the focused element should trigger form
/// submission. Returns the form path if so.
fn enter_in_form_input(tree: &Tree) -> Option<(Vec<usize>, Vec<usize>)> {
  let focus_path = tree.interaction.focus_path.as_deref()?;
  let root = tree.root.as_ref()?;
  let node = root.at_path(focus_path)?;
  match &node.element {
    Element::Input(inp) => {
      use wgpu_html_models::common::html_enums::InputType;
      if matches!(
        inp.r#type,
        Some(
          InputType::Hidden
            | InputType::Checkbox
            | InputType::Radio
            | InputType::File
            | InputType::Image
            | InputType::Color
            | InputType::Range
            | InputType::Button
            | InputType::Submit
            | InputType::Reset
        )
      ) {
        return None;
      }
    }
    Element::Textarea(_) => return None,
    _ => return None,
  }
  let form_path = find_ancestor_form(tree, focus_path)?;
  Some((form_path, focus_path.to_vec()))
}

/// Toggle checkbox/radio state when clicked. Returns `true` if toggled.
fn toggle_checkable(tree: &mut Tree, click_target: &[usize]) -> bool {
  use wgpu_html_models::common::html_enums::InputType;
  let Some(root) = tree.root.as_mut() else {
    return false;
  };
  let Some(node) = root.at_path_mut(click_target) else {
    return false;
  };
  match &mut node.element {
    Element::Input(inp) => match inp.r#type {
      Some(InputType::Checkbox) => {
        let was = inp.checked.unwrap_or(false);
        inp.checked = Some(!was);
        tree.form_control_generation += 1;
        bubble_input(tree, click_target, None, ev::enums::InputType::InsertText);
        fire_change_event_at(tree, click_target);
        return true;
      }
      Some(InputType::Radio) => {
        if inp.checked.unwrap_or(false) {
          return false;
        }
        inp.checked = Some(true);
        tree.form_control_generation += 1;
        let radio_name = inp.name.clone();
        if click_target.len() >= 1 {
          let parent_path = &click_target[..click_target.len() - 1];
          if let Some(parent) = root.at_path_mut(parent_path) {
            let this_idx = *click_target.last().unwrap();
            for (i, sib) in parent.children.iter_mut().enumerate() {
              if i == this_idx {
                continue;
              }
              if let Element::Input(sib_inp) = &mut sib.element {
                if matches!(sib_inp.r#type, Some(InputType::Radio))
                  && sib_inp.name == radio_name
                  && sib_inp.checked.unwrap_or(false)
                {
                  sib_inp.checked = Some(false);
                }
              }
            }
          }
        }
        bubble_input(tree, click_target, None, ev::enums::InputType::InsertText);
        fire_change_event_at(tree, click_target);
        return true;
      }
      _ => {}
    },
    _ => {}
  }
  false
}

/// Handle ArrowUp/ArrowDown on number and range inputs.
fn handle_numeric_step(tree: &mut Tree, key: &str) -> bool {
  use wgpu_html_models::common::html_enums::InputType;
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return false;
  };
  let Some(root) = tree.root.as_mut() else {
    return false;
  };
  let Some(node) = root.at_path_mut(&focus_path) else {
    return false;
  };
  let Element::Input(inp) = &mut node.element else {
    return false;
  };
  let is_number = matches!(inp.r#type, Some(InputType::Number));
  let is_range = matches!(inp.r#type, Some(InputType::Range));
  if !is_number && !is_range {
    return false;
  }

  let min: f64 = inp.min.as_deref().and_then(|s| s.parse().ok()).unwrap_or(if is_range { 0.0 } else { f64::NEG_INFINITY });
  let max: f64 = inp.max.as_deref().and_then(|s| s.parse().ok()).unwrap_or(if is_range { 100.0 } else { f64::INFINITY });
  let step: f64 = inp.step.as_deref().and_then(|s| s.parse().ok()).unwrap_or(1.0);
  let current: f64 = inp.value.as_deref().and_then(|s| s.parse().ok()).unwrap_or(if is_range { (min + max) / 2.0 } else { 0.0 });

  let delta = if key == "ArrowUp" { step } else { -step };
  let new_val = (current + delta).clamp(min, max);

  let formatted = if new_val.fract() == 0.0 && new_val.abs() < i64::MAX as f64 {
    format!("{}", new_val as i64)
  } else {
    format!("{new_val}")
  };
  inp.value = Some(formatted.into());
  if is_range {
    tree.form_control_generation += 1;
  } else {
    tree.generation += 1;
    tree.dirty_paths.push(focus_path.clone());
  }

  if is_number {
    let len = inp.value.as_deref().unwrap_or("").len();
    tree.interaction.edit_cursor = Some(crate::EditCursor::collapsed(len));
    tree.interaction.caret_blink_epoch = std::time::Instant::now();
  }

  bubble_input(tree, &focus_path, None, ev::enums::InputType::InsertText);
  fire_change_event_at(tree, &focus_path);
  true
}

/// Update a `<input type="range">` value to a fraction `[0, 1]` of its range.
/// When `fire_events` is false, only the value and generation are updated
/// (used during drag for performance — events fire on release).
pub fn set_range_value_by_fraction(tree: &mut Tree, path: &[usize], frac: f32) {
  use wgpu_html_models::common::html_enums::InputType;
  let Some(root) = tree.root.as_mut() else {
    return;
  };
  let Some(node) = root.at_path_mut(path) else {
    return;
  };
  if let Element::Input(inp) = &mut node.element {
    if !matches!(inp.r#type, Some(InputType::Range)) {
      return;
    }
    let min: f64 = inp.min.as_deref().and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let max: f64 = inp.max.as_deref().and_then(|s| s.parse().ok()).unwrap_or(100.0);
    let step: f64 = inp.step.as_deref().and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let raw = min + (max - min) * frac.clamp(0.0, 1.0) as f64;
    let stepped = if step > 0.0 {
      (((raw - min) / step).round() * step + min).clamp(min, max)
    } else {
      raw.clamp(min, max)
    };
    let formatted = if stepped.fract() == 0.0 {
      format!("{}", stepped as i64)
    } else {
      format!("{stepped}")
    };
    inp.value = Some(formatted.into());
    tree.form_control_generation += 1;
  }
  bubble_input(tree, path, None, ev::enums::InputType::InsertText);
  fire_change_event_at(tree, path);
}

/// Fire a `change` event on the element at `path` (bubbling).
fn fire_change_event_at(tree: &mut Tree, path: &[usize]) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  fire_focus_event(
    tree,
    path,
    ev::HtmlEventType::CHANGE,
    /* bubbles */ true,
    None,
    time_stamp,
    FocusBubbleKind::Bubble,
  );
}

/// Update a `<input type="color">` value from sRGB 0..255 + alpha 0..1.
pub fn set_color_value(tree: &mut Tree, path: &[usize], r: u8, g: u8, b: u8, a: f32) {
  use wgpu_html_models::common::html_enums::InputType;
  let Some(root) = tree.root.as_mut() else {
    return;
  };
  let Some(node) = root.at_path_mut(path) else {
    return;
  };
  if let Element::Input(inp) = &mut node.element {
    if !matches!(inp.r#type, Some(InputType::Color)) {
      return;
    }
    let a_byte = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    let hex = if a_byte == 255 {
      format!("#{r:02x}{g:02x}{b:02x}")
    } else {
      format!("#{r:02x}{g:02x}{b:02x}{a_byte:02x}")
    };
    inp.value = Some(hex.into());
    tree.form_control_generation += 1;
  }
  bubble_input(tree, path, None, ev::enums::InputType::InsertText);
  fire_change_event_at(tree, path);
}

/// Update a `<input type="date">` or `<input type="datetime-local">` value.
pub fn set_date_value(tree: &mut Tree, path: &[usize], value: &str) {
  use wgpu_html_models::common::html_enums::InputType;
  let Some(root) = tree.root.as_mut() else { return };
  let Some(node) = root.at_path_mut(path) else { return };
  if let Element::Input(inp) = &mut node.element {
    if !matches!(inp.r#type, Some(InputType::Date) | Some(InputType::DatetimeLocal)) {
      return;
    }
    inp.value = Some(value.into());
    tree.form_control_generation += 1;
    tree.generation += 1;
    tree.dirty_paths.push(path.to_vec());
  }
  bubble_input(tree, path, None, ev::enums::InputType::InsertText);
  fire_change_event_at(tree, path);
}

// ── Text editing ─────────────────────────────────────────────────────────────

/// Collect RAWTEXT children of a textarea into a single string.
fn textarea_value(ta: &m::Textarea, children: &[Node]) -> String {
  if let Some(v) = ta.value.as_deref() {
    return v.to_string();
  }
  let mut s = String::new();
  for child in children {
    if let Element::Text(t) = &child.element {
      s.push_str(t);
    }
  }
  s
}

fn focused_date_pattern_for(tree: &Tree, input_type: Option<&wgpu_html_models::common::html_enums::InputType>) -> String {
  use wgpu_html_models::common::html_enums::InputType;
  if matches!(input_type, Some(InputType::DatetimeLocal)) {
    tree.locale.datetime_pattern()
  } else {
    tree.locale.date_pattern().to_string()
  }
}

fn focused_date_pattern(tree: &Tree) -> String {
  use wgpu_html_models::common::html_enums::InputType;
  let is_datetime = tree.interaction.focus_path.as_deref()
    .and_then(|p| tree.root.as_ref()?.at_path(p))
    .map(|n| matches!(&n.element, Element::Input(inp) if matches!(inp.r#type, Some(InputType::DatetimeLocal))))
    .unwrap_or(false);
  if is_datetime {
    tree.locale.datetime_pattern()
  } else {
    tree.locale.date_pattern().to_string()
  }
}

/// Read the current editable value from the focused form control.
/// Returns `(value, is_textarea, is_readonly)`.
fn read_editable_value(node: &Node) -> Option<(String, bool, bool)> {
  use wgpu_html_models::common::html_enums::InputType;
  match &node.element {
    Element::Input(inp) => {
      if matches!(
        inp.r#type,
        Some(
          InputType::Hidden
            | InputType::Checkbox
            | InputType::Radio
            | InputType::Button
            | InputType::Submit
            | InputType::Reset
            | InputType::File
            | InputType::Image
            | InputType::Color
            | InputType::Range
        )
      ) {
        return None;
      }
      let val: String = inp.value.as_deref().unwrap_or_default().to_string();
      let ro = inp.readonly.unwrap_or(false);
      Some((val, false, ro))
    }
    Element::Textarea(ta) => {
      let val = textarea_value(ta, &node.children);
      let ro = ta.readonly.unwrap_or(false);
      Some((val, true, ro))
    }
    _ => None,
  }
}

/// Write a new value back to the focused form control.
fn write_value(node: &mut Node, value: String) {
  match &mut node.element {
    Element::Input(inp) => inp.value = Some(value.into()),
    Element::Textarea(ta) => ta.value = Some(value.into()),
    _ => {}
  }
}

/// Delete the selected text range in the focused form control,
/// returning the removed text. Used by Ctrl+X (cut).
pub fn cut_selection(tree: &mut Tree) -> Option<String> {
  let focus_path = tree.interaction.focus_path.clone()?;
  let cursor = tree.interaction.edit_cursor.clone()?;
  if cursor.selection_anchor.is_none() {
    return None;
  }
  let node = tree.root.as_ref()?.at_path(&focus_path)?;
  let (old_value, _, is_readonly) = read_editable_value(node)?;
  if is_readonly {
    return None;
  }

  let sel_start = cursor.cursor.min(cursor.selection_anchor.unwrap_or(cursor.cursor));
  let sel_end = cursor.cursor.max(cursor.selection_anchor.unwrap_or(cursor.cursor));
  let cut_text = old_value[sel_start..sel_end].to_string();

  let (new_value, new_cursor) = crate::text_edit::delete_selection(&old_value, &cursor);
  if new_value != old_value {
    tree.interaction.undo_stack.push(crate::UndoEntry {
      value: old_value,
      cursor,
    });
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
      write_value(node, new_value);
    }
    tree.generation += 1;
    tree.dirty_paths.push(focus_path.clone());
    bubble_input(tree, &focus_path, None, ev::enums::InputType::DeleteByCut);
  }
  tree.interaction.edit_cursor = Some(new_cursor);
  tree.interaction.caret_blink_epoch = std::time::Instant::now();
  Some(cut_text)
}

fn form_control_state(tree: &Tree, path: &[usize]) -> (Option<String>, Option<bool>) {
  let Some(node) = tree.root.as_ref().and_then(|root| root.at_path(path)) else {
    return (None, None);
  };
  match &node.element {
    Element::Input(inp) => (inp.value.as_deref().map(|s| s.to_string()), inp.checked),
    Element::Textarea(ta) => (Some(textarea_value(ta, &node.children)), None),
    _ => (None, None),
  }
}

/// Process typed text on the currently focused `<input>` or `<textarea>`.
///
/// Inserts `text` at the cursor (replacing any selection), updates
/// `edit_cursor`, and fires `InputEvent`. Returns `true` if the
/// value was mutated.
pub fn text_input(tree: &mut Tree, text: &str) -> bool {
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return false;
  };
  let Some(root) = tree.root.as_ref() else {
    return false;
  };
  let Some(node) = root.at_path(&focus_path) else {
    return false;
  };
  let is_date_display = tree.interaction.date_display_value.is_some();
  let Some((old_value, is_textarea, is_readonly)) = (if is_date_display {
    let dv = tree.interaction.date_display_value.as_ref().unwrap().clone();
    Some((dv, false, false))
  } else {
    read_editable_value(node)
  }) else {
    return false;
  };
  if is_readonly {
    return false;
  }

  // Single-line inputs reject newlines.
  if !is_textarea && text.contains('\n') {
    return false;
  }

  let cursor = tree
    .interaction
    .edit_cursor
    .clone()
    .unwrap_or_else(|| crate::EditCursor::collapsed(old_value.len()));

  // Fire beforeinput — if cancelled, skip the mutation.
  if bubble_beforeinput(
    tree,
    &focus_path,
    Some(text.to_owned()),
    ev::enums::InputType::InsertText,
  ) {
    return true;
  }

  if is_date_display {
    let pattern = focused_date_pattern(tree);
    let segs = crate::date::parse_pattern_segments(&pattern);
    let mut current_text = old_value.clone();
    let mut current_pos = if cursor.has_selection() {
      cursor.selection_range().0
    } else {
      cursor.cursor
    };
    let mut any = false;
    for ch in text.chars() {
      let r = crate::date::date_overwrite_char(&current_text, current_pos, ch, &segs);
      if r.consumed {
        current_text = r.text;
        current_pos = r.cursor;
        any = true;
      }
    }
    if !any { return false; }
    let start_range = cursor.selection_range().0;
    tree.interaction.undo_stack.push(crate::UndoEntry { value: old_value, cursor });
    tree.interaction.date_display_value = Some(current_text);
    let start_seg = crate::date::segment_at(&segs, start_range);
    let end_seg = crate::date::segment_at(&segs, current_pos.saturating_sub(1).min(segs.last().map(|s| s.byte_start + s.byte_len - 1).unwrap_or(0)));
    let advanced = start_seg != end_seg;
    let new_cursor = if advanced {
      if let Some(si) = crate::date::segment_at(&segs, current_pos.min(segs.last().map(|s| s.byte_start + s.byte_len - 1).unwrap_or(0))) {
        let s = &segs[si];
        crate::EditCursor { cursor: s.byte_start + s.byte_len, selection_anchor: Some(s.byte_start) }
      } else {
        crate::EditCursor::collapsed(current_pos)
      }
    } else {
      crate::EditCursor::collapsed(current_pos)
    };
    tree.interaction.edit_cursor = Some(new_cursor);
  } else {
    let (new_value, new_cursor) = crate::text_edit::insert_text(&old_value, &cursor, text);
    tree.interaction.undo_stack.push(crate::UndoEntry { value: old_value, cursor });
    if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
      write_value(node, new_value);
    }
    tree.interaction.edit_cursor = Some(new_cursor);
  }
  tree.interaction.caret_blink_epoch = std::time::Instant::now();
  tree.generation += 1;
  tree.dirty_paths.push(focus_path.clone());

  bubble_input(
    tree,
    &focus_path,
    Some(text.to_owned()),
    ev::enums::InputType::InsertText,
  );

  true
}

fn handle_undo(tree: &mut Tree) -> bool {
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return false;
  };
  let Some(root) = tree.root.as_ref() else {
    return false;
  };
  let Some(node) = root.at_path(&focus_path) else {
    return false;
  };
  let Some((current_value, _, is_readonly)) = read_editable_value(node) else {
    return false;
  };
  if is_readonly {
    return false;
  }
  let current_cursor = tree
    .interaction
    .edit_cursor
    .clone()
    .unwrap_or_else(|| crate::EditCursor::collapsed(current_value.len()));

  let Some(prev) = tree.interaction.undo_stack.undo(crate::UndoEntry {
    value: current_value,
    cursor: current_cursor,
  }) else {
    return false;
  };

  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
    write_value(node, prev.value);
  }
  tree.interaction.edit_cursor = Some(prev.cursor);
  tree.interaction.caret_blink_epoch = std::time::Instant::now();
  tree.generation += 1;
  tree.dirty_paths.push(focus_path.clone());
  bubble_input(tree, &focus_path, None, ev::enums::InputType::HistoryUndo);
  true
}

fn handle_redo(tree: &mut Tree) -> bool {
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return false;
  };
  let Some(root) = tree.root.as_ref() else {
    return false;
  };
  let Some(node) = root.at_path(&focus_path) else {
    return false;
  };
  let Some((current_value, _, is_readonly)) = read_editable_value(node) else {
    return false;
  };
  if is_readonly {
    return false;
  }
  let current_cursor = tree
    .interaction
    .edit_cursor
    .clone()
    .unwrap_or_else(|| crate::EditCursor::collapsed(current_value.len()));

  let Some(next) = tree.interaction.undo_stack.redo(crate::UndoEntry {
    value: current_value,
    cursor: current_cursor,
  }) else {
    return false;
  };

  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
    write_value(node, next.value);
  }
  tree.interaction.edit_cursor = Some(next.cursor);
  tree.interaction.caret_blink_epoch = std::time::Instant::now();
  tree.generation += 1;
  tree.dirty_paths.push(focus_path.clone());
  bubble_input(tree, &focus_path, None, ev::enums::InputType::HistoryRedo);
  true
}

/// Handle editing keys (Backspace, Delete, arrows, Home/End, Enter)
/// when focus is on a form control. Called from `key_down`.
/// Returns `true` if the key was consumed.
fn handle_edit_key(tree: &mut Tree, key: &str, code: &str) -> bool {
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return false;
  };
  let Some(root) = tree.root.as_ref() else {
    return false;
  };
  let Some(node) = root.at_path(&focus_path) else {
    return false;
  };
  let is_date_display = tree.interaction.date_display_value.is_some();
  let Some((old_value, is_textarea, is_readonly)) = (if is_date_display {
    let dv = tree.interaction.date_display_value.as_ref().unwrap().clone();
    Some((dv, false, false))
  } else {
    read_editable_value(node)
  }) else {
    return false;
  };

  let cursor = tree
    .interaction
    .edit_cursor
    .clone()
    .unwrap_or_else(|| crate::EditCursor::collapsed(old_value.len()));

  let shift = tree.interaction.modifiers.shift;
  let ctrl = tree.interaction.modifiers.ctrl;

  // Date inputs: segment-aware navigation.
  if is_date_display {
    let pattern = focused_date_pattern(tree);
    let segs = crate::date::parse_pattern_segments(&pattern);
    let pos = cursor.cursor.min(old_value.len());
    let nav_cursor = match key {
      "ArrowLeft" => Some(crate::EditCursor::collapsed(crate::date::cursor_left(&segs, pos))),
      "ArrowRight" => Some(crate::EditCursor::collapsed(crate::date::cursor_right(&segs, pos, old_value.len()))),
      "Tab" if shift => {
        let (start, end) = crate::date::prev_segment(&segs, pos);
        Some(crate::EditCursor { cursor: end, selection_anchor: Some(start) })
      }
      "Tab" => {
        let (start, end) = crate::date::next_segment(&segs, pos);
        Some(crate::EditCursor { cursor: end, selection_anchor: Some(start) })
      }
      "Home" => Some(crate::EditCursor::collapsed(0)),
      "End" => Some(crate::EditCursor::collapsed(old_value.len())),
      _ => None,
    };
    if let Some(new_cursor) = nav_cursor {
      tree.interaction.edit_cursor = Some(new_cursor);
      tree.interaction.caret_blink_epoch = std::time::Instant::now();
      return true;
    }
    // Date backspace: replace char with '0' instead of deleting.
    if key == "Backspace" || key == "Delete" {
      let r = crate::date::date_backspace(&old_value, pos, &segs);
      if r.consumed {
        tree.interaction.date_display_value = Some(r.text);
        tree.interaction.edit_cursor = Some(crate::EditCursor::collapsed(r.cursor));
        tree.interaction.caret_blink_epoch = std::time::Instant::now();
        tree.generation += 1;
        tree.dirty_paths.push(focus_path.clone());
      }
      return true;
    }
  }

  // Navigation keys (always allowed, even on readonly).
  let nav_cursor = match key {
    "ArrowLeft" if ctrl => Some(crate::text_edit::move_word_left(&old_value, &cursor, shift)),
    "ArrowRight" if ctrl => Some(crate::text_edit::move_word_right(&old_value, &cursor, shift)),
    "ArrowLeft" => Some(crate::text_edit::move_left(&old_value, &cursor, shift)),
    "ArrowRight" => Some(crate::text_edit::move_right(&old_value, &cursor, shift)),
    "Home" => Some(crate::text_edit::move_home(&old_value, &cursor, shift)),
    "End" => Some(crate::text_edit::move_end(&old_value, &cursor, shift)),
    "ArrowUp" if is_textarea => Some(crate::text_edit::move_up(&old_value, &cursor, shift)),
    "ArrowDown" if is_textarea => Some(crate::text_edit::move_down(&old_value, &cursor, shift)),
    _ if ctrl && code == "KeyA" => Some(crate::text_edit::select_all(&old_value)),
    _ => None,
  };

  if let Some(new_cursor) = nav_cursor {
    tree.interaction.edit_cursor = Some(new_cursor);
    tree.interaction.caret_blink_epoch = std::time::Instant::now();
    select_event(tree, &focus_path);
    return true;
  }

  // Mutation keys (blocked by readonly).
  if is_readonly {
    return false;
  }

  let input_type = match key {
    "Backspace" if ctrl => ev::enums::InputType::DeleteWordBackward,
    "Backspace" => ev::enums::InputType::DeleteContentBackward,
    "Delete" if ctrl => ev::enums::InputType::DeleteWordForward,
    "Delete" => ev::enums::InputType::DeleteContentForward,
    "Enter" => ev::enums::InputType::InsertLineBreak,
    _ => return false,
  };
  if bubble_beforeinput(tree, &focus_path, None, input_type.clone()) {
    return true; // cancelled
  }

  let mutation: Option<(String, crate::EditCursor)> = match key {
    "Backspace" if ctrl => Some(crate::text_edit::delete_word_backward(&old_value, &cursor)),
    "Backspace" => Some(crate::text_edit::delete_backward(&old_value, &cursor)),
    "Delete" if ctrl => Some(crate::text_edit::delete_word_forward(&old_value, &cursor)),
    "Delete" => Some(crate::text_edit::delete_forward(&old_value, &cursor)),
    "Enter" if is_textarea => Some(crate::text_edit::insert_line_break(&old_value, &cursor)),
    _ => None,
  };

  if let Some((new_value, new_cursor)) = mutation {
    if new_value != old_value {
      tree.interaction.undo_stack.push(crate::UndoEntry {
        value: old_value,
        cursor,
      });
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
        write_value(node, new_value);
      }
      tree.generation += 1;
      tree.dirty_paths.push(focus_path.clone());

      bubble_input(tree, &focus_path, None, input_type);
    }
    tree.interaction.edit_cursor = Some(new_cursor);
    tree.interaction.caret_blink_epoch = std::time::Instant::now();
    return true;
  }

  false
}

// ── Tree inherent methods ────────────────────────────────────────────────────

impl Tree {
  /// Read-only accessor for the document's interaction state.
  pub fn interaction(&self) -> &InteractionState {
    &self.interaction
  }

  /// Mutable accessor for the document's interaction state.
  pub fn interaction_mut(&mut self) -> &mut InteractionState {
    &mut self.interaction
  }

  /// Currently-tracked modifier state. Updated by
  /// [`Self::set_modifier`]; consumed by the mouse / keyboard
  /// dispatchers when they fire events.
  pub fn modifiers(&self) -> Modifiers {
    self.interaction.modifiers
  }

  /// Flip a single modifier bit in `tree.interaction.modifiers`.
  /// Hosts call this from their key-down / key-up paths so the
  /// dispatchers always have current modifier state and event
  /// emission stays consistent without threading `Modifiers`
  /// through every call site.
  pub fn set_modifier(&mut self, modifier: Modifier, down: bool) {
    self.interaction.modifiers.set(modifier, down);
  }

  /// Move focus to the element at `path`, walking up to the
  /// nearest focusable ancestor. See [`focus`].
  pub fn focus(&mut self, path: Option<&[usize]>) -> bool {
    focus(self, path)
  }

  /// Clear focus. See [`blur`].
  pub fn blur(&mut self) -> bool {
    blur(self)
  }

  /// Advance focus to the next/previous keyboard-focusable
  /// element. See [`focus_next`].
  pub fn focus_next(&mut self, reverse: bool) -> Option<Vec<usize>> {
    focus_next(self, reverse)
  }

  /// Dispatch `keydown` to the focused element. See [`key_down`].
  pub fn key_down(&mut self, key: &str, code: &str, repeat: bool) -> bool {
    key_down(self, key, code, repeat)
  }

  /// Dispatch `keyup` to the focused element. See [`key_up`].
  pub fn key_up(&mut self, key: &str, code: &str) -> bool {
    key_up(self, key, code)
  }

  /// Dispatch a `wheel` event to the hovered element. See [`wheel_event`].
  pub fn wheel_event(
    &mut self,
    pos: (f32, f32),
    delta_x: f64,
    delta_y: f64,
    delta_mode: ev::enums::WheelDeltaMode,
  ) -> bool {
    wheel_event(self, pos, delta_x, delta_y, delta_mode)
  }

  /// Dispatch a clipboard event to the focused element.
  /// See [`clipboard_event`].
  pub fn clipboard_event(&mut self, event_type: &'static str) -> bool {
    clipboard_event(self, event_type)
  }

  /// Dispatch a `scroll` event on an element. See [`scroll_event`].
  pub fn scroll_event(&mut self, path: &[usize]) {
    scroll_event(self, path)
  }

  /// Dispatch a `selectionchange` event on the document root.
  /// See [`selectionchange_event`].
  pub fn selectionchange_event(&mut self) {
    selectionchange_event(self)
  }

  /// Dispatch a `select` event on an element. See [`select_event`].
  pub fn select_event(&mut self, path: &[usize]) {
    select_event(self, path)
  }

  /// Dispatch a `resize` event on the document root.
  pub fn resize_event(&mut self) {
    resize_event(self)
  }

  /// Process typed text on the focused form control. See [`text_input`].
  pub fn text_input(&mut self, text: &str) -> bool {
    text_input(self, text)
  }

  /// The pointer left the surface — clear hover state and fire
  /// `mouseleave`. See [`dispatch_pointer_leave`].
  pub fn pointer_leave(&mut self) {
    dispatch_pointer_leave(self)
  }

  /// Update hover chain to a new layout-hit-tested target.
  /// See [`dispatch_pointer_move`].
  pub fn dispatch_pointer_move(
    &mut self,
    new_target: Option<&[usize]>,
    pos: (f32, f32),
    text_cursor: Option<TextCursor>,
  ) -> bool {
    dispatch_pointer_move(self, new_target, pos, text_cursor)
  }

  /// Mouse-down on a precomputed `target_path`. See
  /// [`dispatch_mouse_down`].
  pub fn dispatch_mouse_down(
    &mut self,
    target_path: Option<&[usize]>,
    pos: (f32, f32),
    button: MouseButton,
    text_cursor: Option<TextCursor>,
  ) -> bool {
    dispatch_mouse_down(self, target_path, pos, button, text_cursor)
  }

  /// Mouse-up on a precomputed `target_path`. See
  /// [`dispatch_mouse_up`].
  pub fn dispatch_mouse_up(
    &mut self,
    target_path: Option<&[usize]>,
    pos: (f32, f32),
    button: MouseButton,
    text_cursor: Option<TextCursor>,
  ) -> bool {
    dispatch_mouse_up(self, target_path, pos, button, text_cursor)
  }
}

// ── Tests (layout-free) ──────────────────────────────────────────────────────
