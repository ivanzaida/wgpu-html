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
}

impl HoverSlot {
  fn event_type_str(self) -> &'static str {
    match self {
      HoverSlot::Enter => ev::HtmlEventType::MOUSEENTER,
      HoverSlot::Leave => ev::HtmlEventType::MOUSELEAVE,
    }
  }
}

/// Bubble a mouse event up the ancestry chain of `target_path`.
fn bubble(tree: &mut Tree, target_path: &[usize], pos: (f32, f32), button: Option<MouseButton>, slot: Slot) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let buttons_down = tree.interaction.buttons_down;
  let modifiers = tree.interaction.modifiers;

  let depth = target_path.len();
  for i in 0..=depth {
    let current_path = target_path[..depth.saturating_sub(i)].to_vec();

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
      return;
    }
    for cb in &mouse_cbs {
      cb(&ev);
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
    );
    if tree.emit_event(&mut html_ev).is_stop() {
      return;
    }
    for cb in &event_cbs {
      cb(&html_ev);
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

  changed
}

/// Bubble a `mousemove` event up the ancestry chain, calling each
/// node's `on_mouse_move` callback and the generic `on_event` slot.
fn bubble_mouse_move(tree: &mut Tree, target_path: &[usize], pos: (f32, f32)) {
  let modifiers = tree.interaction.modifiers;
  let depth = target_path.len();
  for i in 0..=depth {
    let current_path = target_path[..depth.saturating_sub(i)].to_vec();
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

    // Also fire through on_event as a mousemove HtmlEvent.
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
      );
      for cb in &event_cbs {
        cb(&html_ev);
      }
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
        .and_then(|node| read_editable_value(node).map(|(v, _, _)| v));
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
  tree.interaction.focus_value_snapshot = new_path
    .as_deref()
    .and_then(|path| {
      let node = tree.root.as_ref()?.at_path(path)?;
      read_editable_value(node).map(|(v, _, _)| v)
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
  let depth = target_path.len();
  let target = target_path.to_vec();
  for i in 0..=depth {
    if matches!(kind, FocusBubbleKind::Target) && i != 0 {
      break;
    }
    let current_path = target_path[..depth.saturating_sub(i)].to_vec();
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
    let event_phase = if current_path == target {
      ev::EventPhase::AtTarget
    } else {
      ev::EventPhase::BubblingPhase
    };
    let mut html_ev = ev::HtmlEvent::Focus(ev::events::FocusEvent {
      base: ev::events::UIEvent {
        base: ev::events::Event {
          event_type: ev::HtmlEventType::from(event_type),
          bubbles,
          cancelable: false,
          composed: true,
          target: Some(target.clone()),
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
      related_target: related.clone(),
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
) -> ev::HtmlEvent {
  let event_phase = if current_path.as_slice() == target_path {
    ev::EventPhase::AtTarget
  } else {
    ev::EventPhase::BubblingPhase
  };
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
  for i in 0..=depth {
    let current_path = target_path[..depth.saturating_sub(i)].to_vec();
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
    );
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

  // Handle editing keys on focused form controls before Tab.
  handle_edit_key(tree, key);

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
    Element::Input(inp) => matches!(inp.r#type, Some(InputType::Checkbox) | Some(InputType::Submit) | Some(InputType::Reset) | Some(InputType::Button)),
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
  target_path: &[usize],
  current_path: Vec<usize>,
  time_stamp: f64,
) -> ev::HtmlEvent {
  let event_phase = if current_path.as_slice() == target_path {
    ev::EventPhase::AtTarget
  } else {
    ev::EventPhase::BubblingPhase
  };
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
    is_composing: false,
  })
}

fn bubble_input(
  tree: &mut Tree,
  target_path: &[usize],
  data: Option<String>,
  input_type: ev::enums::InputType,
) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = target_path.len();
  for i in 0..=depth {
    let current_path = target_path[..depth.saturating_sub(i)].to_vec();
    let (dedicated, on_evs) = tree
      .root
      .as_ref()
      .and_then(|root| root.at_path(&current_path))
      .map(|node| (node.on_input.clone(), node.on_event.clone()))
      .unwrap_or_default();
    let mut html_ev = make_input_html_event(
      ev::HtmlEventType::INPUT,
      data.clone(),
      input_type.clone(),
      target_path,
      current_path,
      time_stamp,
    );
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
) -> ev::HtmlEvent {
  let event_phase = if current_path.as_slice() == target_path {
    ev::EventPhase::AtTarget
  } else {
    ev::EventPhase::BubblingPhase
  };
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
  for i in 0..=depth {
    let current_path = target[..depth.saturating_sub(i)].to_vec();
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
      &target,
      current_path,
      time_stamp,
    );
    if tree.emit_event(&mut html_ev).is_stop() {
      return prevented;
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
    prevented = prevented || html_ev.base().default_prevented.get();
    if html_ev.base().propagation_stopped.get() {
      break;
    }
  }
  prevented
}

// ── Clipboard ────────────────────────────────────────────────────────────────

/// Dispatch a `copy`, `cut`, or `paste` clipboard event to the
/// focused element (or document root if nothing is focused),
/// bubbling target → root.
///
/// Returns `true` if `preventDefault()` was called on the event,
/// signalling the caller to skip the default clipboard operation.
pub fn clipboard_event(
  tree: &mut Tree,
  event_type: &'static str,
) -> bool {
  let target = tree.interaction.focus_path.clone().unwrap_or_else(Vec::new);
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let depth = target.len();
  let mut prevented = false;
  for i in 0..=depth {
    let current_path = target[..depth.saturating_sub(i)].to_vec();
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
    let event_phase = if current_path.as_slice() == target.as_slice() {
      ev::EventPhase::AtTarget
    } else {
      ev::EventPhase::BubblingPhase
    };
    let mut html_ev = ev::HtmlEvent::Clipboard(ev::events::ClipboardEvent {
      base: ev::events::Event {
        event_type: ev::HtmlEventType::from(event_type),
        bubbles: true,
        cancelable: true,
        composed: true,
        target: Some(target.clone()),
        current_target: Some(current_path),
        event_phase,
        default_prevented: Cell::new(false),
        propagation_stopped: Cell::new(false),
        immediate_propagation_stopped: Cell::new(false),
        is_trusted: true,
        time_stamp,
      },
      clipboard_data: None,
    });
    if tree.emit_event(&mut html_ev).is_stop() {
      return prevented;
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
    prevented = prevented || html_ev.base().default_prevented.get();
    if html_ev.base().propagation_stopped.get() {
      break;
    }
  }
  prevented
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

/// Bubble a `submit` event on the form element at `form_path`,
/// with `submitter_path` as the submitter (the button or input
/// that triggered the submission).
fn bubble_submit_event(tree: &mut Tree, form_path: &[usize], submitter_path: Option<Vec<usize>>) {
  let time_stamp = tree.interaction.time_origin.elapsed().as_secs_f64() * 1000.0;
  let on_evs = tree
    .root
    .as_ref()
    .and_then(|root| root.at_path(form_path))
    .map(|node| node.on_event.clone())
    .unwrap_or_default();
  let mut html_ev = ev::HtmlEvent::Submit(ev::events::SubmitEvent {
    base: ev::events::Event {
      event_type: ev::HtmlEventType::from(ev::HtmlEventType::SUBMIT),
      bubbles: true,
      cancelable: true,
      composed: true,
      target: Some(form_path.to_vec()),
      current_target: Some(form_path.to_vec()),
      event_phase: ev::EventPhase::AtTarget,
      default_prevented: Cell::new(false),
      propagation_stopped: Cell::new(false),
      immediate_propagation_stopped: Cell::new(false),
      is_trusted: true,
      time_stamp,
    },
    submitter: submitter_path,
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
        Some(InputType::Hidden | InputType::Checkbox | InputType::Radio | InputType::File | InputType::Image | InputType::Color | InputType::Range | InputType::Button | InputType::Submit | InputType::Reset)
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
        tree.generation += 1;
        bubble_input(tree, click_target, None, ev::enums::InputType::InsertText);
        fire_change_event_at(tree, click_target);
        return true;
      }
      Some(InputType::Radio) => {
        if inp.checked.unwrap_or(false) {
          return false;
        }
        inp.checked = Some(true);
        tree.generation += 1;
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
      let val = inp.value.clone().unwrap_or_default();
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
    Element::Input(inp) => inp.value = Some(value),
    Element::Textarea(ta) => ta.value = Some(value),
    _ => {}
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
  let Some((old_value, is_textarea, is_readonly)) = read_editable_value(node) else {
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

  let (new_value, new_cursor) = crate::text_edit::insert_text(&old_value, &cursor, text);

  // Write back.
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
    write_value(node, new_value);
  }
  tree.interaction.edit_cursor = Some(new_cursor);
  tree.interaction.caret_blink_epoch = std::time::Instant::now();
  tree.generation += 1;

  bubble_input(tree, &focus_path, Some(text.to_owned()), ev::enums::InputType::InsertText);

  true
}

/// Handle editing keys (Backspace, Delete, arrows, Home/End, Enter)
/// when focus is on a form control. Called from `key_down`.
/// Returns `true` if the key was consumed.
fn handle_edit_key(tree: &mut Tree, key: &str) -> bool {
  let Some(focus_path) = tree.interaction.focus_path.clone() else {
    return false;
  };
  let Some(root) = tree.root.as_ref() else {
    return false;
  };
  let Some(node) = root.at_path(&focus_path) else {
    return false;
  };
  let Some((old_value, is_textarea, is_readonly)) = read_editable_value(node) else {
    return false;
  };

  let cursor = tree
    .interaction
    .edit_cursor
    .clone()
    .unwrap_or_else(|| crate::EditCursor::collapsed(old_value.len()));

  let shift = tree.interaction.modifiers.shift;
  let ctrl = tree.interaction.modifiers.ctrl;

  // Navigation keys (always allowed, even on readonly).
  let nav_cursor = match key {
    "ArrowLeft" => Some(crate::text_edit::move_left(&old_value, &cursor, shift)),
    "ArrowRight" => Some(crate::text_edit::move_right(&old_value, &cursor, shift)),
    "Home" => Some(crate::text_edit::move_home(&old_value, &cursor, shift)),
    "End" => Some(crate::text_edit::move_end(&old_value, &cursor, shift)),
    "ArrowUp" if is_textarea => Some(crate::text_edit::move_up(&old_value, &cursor, shift)),
    "ArrowDown" if is_textarea => Some(crate::text_edit::move_down(&old_value, &cursor, shift)),
    _ if ctrl && matches!(key, "a" | "A") => Some(crate::text_edit::select_all(&old_value)),
    _ => None,
  };

  if let Some(new_cursor) = nav_cursor {
    tree.interaction.edit_cursor = Some(new_cursor);
    tree.interaction.caret_blink_epoch = std::time::Instant::now();
    return true;
  }

  // Mutation keys (blocked by readonly).
  if is_readonly {
    return false;
  }

  let mutation: Option<(String, crate::EditCursor)> = match key {
    "Backspace" => Some(crate::text_edit::delete_backward(&old_value, &cursor)),
    "Delete" => Some(crate::text_edit::delete_forward(&old_value, &cursor)),
    "Enter" if is_textarea => Some(crate::text_edit::insert_line_break(&old_value, &cursor)),
    _ => None,
  };

  if let Some((new_value, new_cursor)) = mutation {
    if new_value != old_value {
      if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(&focus_path)) {
        write_value(node, new_value);
      }
      tree.generation += 1;

      let input_type = match key {
        "Backspace" => ev::enums::InputType::DeleteContentBackward,
        "Delete" => ev::enums::InputType::DeleteContentForward,
        "Enter" => ev::enums::InputType::InsertLineBreak,
        _ => ev::enums::InputType::InsertText,
      };
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

#[cfg(test)]
#[path = "dispatch_tests.rs"]
mod tests_dispatch;
