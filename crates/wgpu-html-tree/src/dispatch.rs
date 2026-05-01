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
  fn detail(self) -> i32 {
    match self {
      Slot::Click => 1,
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

    let (mouse_cb, event_cb) = tree
      .root
      .as_ref()
      .and_then(|root| root.at_path(&current_path))
      .map(|node| {
        let mouse_cb = match slot {
          Slot::MouseDown => node.on_mouse_down.clone(),
          Slot::MouseUp => node.on_mouse_up.clone(),
          Slot::Click => node.on_click.clone(),
        };
        (mouse_cb, node.on_event.clone())
      })
      .unwrap_or((None, None));

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
    if let Some(cb) = mouse_cb {
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
    if let Some(on_ev) = event_cb {
      on_ev(&html_ev);
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

  let (mouse_cb, event_cb) = tree
    .root
    .as_ref()
    .map(|root| {
      let mouse_cb = match slot {
        HoverSlot::Enter => root.on_mouse_enter.clone(),
        HoverSlot::Leave => root.on_mouse_leave.clone(),
      };
      (mouse_cb, root.on_event.clone())
    })
    .unwrap_or((None, None));

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
  if let Some(cb) = mouse_cb {
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
  if let Some(on_ev) = event_cb {
    on_ev(&html_ev);
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

    let (mouse_cb, event_cb) = tree
      .root
      .as_ref()
      .and_then(|root| root.at_path(&current_path))
      .map(|node| {
        let mouse_cb = match slot {
          HoverSlot::Enter => node.on_mouse_enter.clone(),
          HoverSlot::Leave => node.on_mouse_leave.clone(),
        };
        (mouse_cb, node.on_event.clone())
      })
      .unwrap_or((None, None));

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
    if let Some(cb) = mouse_cb {
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
    if let Some(on_ev) = event_cb {
      on_ev(&html_ev);
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
  changed
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

  if button == MouseButton::Primary {
    if tree.interaction.selecting_text {
      if let Some(cursor) = text_cursor {
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
      let click_target = common_prefix(&press_path, target_path);
      if !suppress_click {
        bubble(tree, click_target, pos, Some(button), Slot::Click);
      }
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

  if let Some(old) = old_path.as_deref() {
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
    let on_ev = tree
      .root
      .as_ref()
      .and_then(|root| root.at_path(&current_path))
      .and_then(|node| node.on_event.clone());
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
          default_prevented: false,
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
    if let Some(on_ev) = on_ev {
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
        default_prevented: false,
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
    let on_ev = tree
      .root
      .as_ref()
      .and_then(|root| root.at_path(&current_path))
      .and_then(|node| node.on_event.clone());
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
    if let Some(on_ev) = on_ev {
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
  }
  true
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
mod tests {
  use std::sync::{Arc, Mutex};

  use wgpu_html_models as m;

  use super::*;
  use crate::{Node, TreeHook, TreeHookResponse};

  /// Build a tree with a body containing children of mixed
  /// focusability:
  ///   [0] <input type="text">          focusable, kbd
  ///   [1] <div>                        not focusable
  ///   [2] <button>                     focusable, kbd
  ///   [3] <input type="hidden">        not focusable
  ///   [4] <a href="#">                 focusable, kbd
  fn focus_test_tree() -> Tree {
    let mut root = Node::new(m::Body::default());
    let mut input_text = m::Input::default();
    input_text.r#type = Some(m::common::html_enums::InputType::Text);
    root.children.push(Node::new(input_text));
    root.children.push(Node::new(m::Div::default()));
    root.children.push(Node::new(m::Button::default()));
    let mut input_hidden = m::Input::default();
    input_hidden.r#type = Some(m::common::html_enums::InputType::Hidden);
    root.children.push(Node::new(input_hidden));
    let mut anchor = m::A::default();
    anchor.href = Some("#".into());
    root.children.push(Node::new(anchor));
    Tree::new(root)
  }

  struct RecordingHook {
    events: Arc<Mutex<Vec<String>>>,
    mouse_paths: Arc<Mutex<Vec<Vec<usize>>>>,
  }

  impl TreeHook for RecordingHook {
    fn on_event(&mut self, _tree: &mut Tree, event: &mut ev::HtmlEvent) -> TreeHookResponse {
      self.events.lock().unwrap().push(event.event_type().to_string());
      TreeHookResponse::Continue
    }

    fn on_mouse_event(&mut self, _tree: &mut Tree, event: &mut MouseEvent) -> TreeHookResponse {
      self.mouse_paths.lock().unwrap().push(event.current_path.clone());
      TreeHookResponse::Continue
    }
  }

  #[test]
  fn tree_hook_receives_keyboard_event_without_node_callback() {
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let mouse_paths = Arc::new(Mutex::new(Vec::<Vec<usize>>::new()));
    let mut tree = focus_test_tree();
    tree.add_hook(RecordingHook {
      events: events.clone(),
      mouse_paths,
    });

    tree.key_down("a", "KeyA", false);

    let events = events.lock().unwrap().clone();
    assert!(events.contains(&"keydown".to_owned()), "got {events:?}");
  }

  #[test]
  fn tree_hook_receives_mouse_events_without_node_callback() {
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let mouse_paths = Arc::new(Mutex::new(Vec::<Vec<usize>>::new()));
    let mut root = Node::new(m::Body::default());
    root.children.push(Node::new(m::Div::default()));
    let mut tree = Tree::new(root);
    tree.add_hook(RecordingHook {
      events: events.clone(),
      mouse_paths: mouse_paths.clone(),
    });

    tree.dispatch_mouse_down(Some(&[0]), (1.0, 1.0), MouseButton::Primary, None);

    let events = events.lock().unwrap().clone();
    let mouse_paths = mouse_paths.lock().unwrap().clone();
    assert!(events.contains(&"mousedown".to_owned()), "got {events:?}");
    assert!(
      mouse_paths.iter().any(|p| p.as_slice() == [0usize]),
      "got {mouse_paths:?}"
    );
  }

  #[test]
  fn focus_sets_focus_path_and_fires_focus_focusin() {
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let r = received.clone();
    let mut tree = focus_test_tree();
    if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(0)) {
      n.on_event = Some(Arc::new(move |ev| {
        r.lock().unwrap().push(ev.event_type().to_string());
      }));
    }
    assert!(tree.focus(Some(&[0])));
    assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
    let evs = received.lock().unwrap().clone();
    assert!(evs.contains(&"focus".into()), "expected focus in {evs:?}");
    assert!(evs.contains(&"focusin".into()), "expected focusin in {evs:?}");
  }

  #[test]
  fn focus_change_fires_blur_with_related_target() {
    let received = Arc::new(Mutex::new(Vec::<(String, Option<Vec<usize>>)>::new()));
    let r = received.clone();
    let mut tree = focus_test_tree();
    if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(0)) {
      n.on_event = Some(Arc::new(move |ev| {
        if let ev::HtmlEvent::Focus(fe) = ev {
          r.lock()
            .unwrap()
            .push((ev.event_type().to_string(), fe.related_target.clone()));
        }
      }));
    }
    tree.focus(Some(&[0]));
    received.lock().unwrap().clear();
    tree.focus(Some(&[2]));
    let evs = received.lock().unwrap().clone();
    let blur_evs: Vec<_> = evs.iter().filter(|(t, _)| t == "blur").collect();
    assert_eq!(blur_evs.len(), 1, "got {evs:?}");
    assert_eq!(blur_evs[0].1.as_deref(), Some([2usize].as_slice()));
  }

  #[test]
  fn blur_clears_focus_and_fires_blur() {
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let r = received.clone();
    let mut tree = focus_test_tree();
    if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(2)) {
      n.on_event = Some(Arc::new(move |ev| {
        r.lock().unwrap().push(ev.event_type().to_string());
      }));
    }
    tree.focus(Some(&[2]));
    received.lock().unwrap().clear();
    assert!(tree.blur());
    assert_eq!(tree.interaction.focus_path, None);
    let evs = received.lock().unwrap().clone();
    assert!(evs.contains(&"blur".into()));
    assert!(evs.contains(&"focusout".into()));
  }

  #[test]
  fn focus_walks_up_to_focusable_ancestor() {
    let mut button = m::Button::default();
    button.id = Some("ok".into());
    let mut btn_node = Node::new(button);
    btn_node.children.push(Node::new("OK"));
    let mut root = Node::new(m::Body::default());
    root.children.push(btn_node);
    let mut tree = Tree::new(root);
    assert!(tree.focus(Some(&[0, 0])));
    assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
  }

  #[test]
  fn focus_next_cycles_in_document_order() {
    let mut tree = focus_test_tree();
    assert_eq!(tree.focus_next(false).as_deref(), Some([0usize].as_slice()));
    assert_eq!(tree.focus_next(false).as_deref(), Some([2usize].as_slice()));
    assert_eq!(tree.focus_next(false).as_deref(), Some([4usize].as_slice()));
    assert_eq!(tree.focus_next(false).as_deref(), Some([0usize].as_slice()));
  }

  #[test]
  fn focus_next_reverse_cycles_backward() {
    let mut tree = focus_test_tree();
    assert_eq!(tree.focus_next(true).as_deref(), Some([4usize].as_slice()));
    assert_eq!(tree.focus_next(true).as_deref(), Some([2usize].as_slice()));
    assert_eq!(tree.focus_next(true).as_deref(), Some([0usize].as_slice()));
  }

  #[test]
  fn key_down_dispatches_to_focused_element_on_event() {
    let received = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
    let r = received.clone();
    let mut tree = focus_test_tree();
    if let Some(n) = tree.root.as_mut().and_then(|r| r.children.get_mut(0)) {
      n.on_event = Some(Arc::new(move |ev| {
        if let ev::HtmlEvent::Keyboard(ke) = ev {
          r.lock().unwrap().push((ev.event_type().to_string(), ke.key.clone()));
        }
      }));
    }
    tree.focus(Some(&[0]));
    tree.key_down("a", "KeyA", false);
    let evs = received.lock().unwrap().clone();
    assert!(evs.iter().any(|(t, k)| t == "keydown" && k == "a"), "got {evs:?}");
  }

  #[test]
  fn key_down_tab_advances_focus() {
    let mut tree = focus_test_tree();
    tree.focus(Some(&[0]));
    tree.key_down("Tab", "Tab", false);
    assert_eq!(tree.interaction.focus_path.as_deref(), Some([2usize].as_slice()));
  }

  #[test]
  fn key_down_shift_tab_retreats_focus() {
    let mut tree = focus_test_tree();
    tree.focus(Some(&[2]));
    tree.set_modifier(Modifier::Shift, true);
    tree.key_down("Tab", "Tab", false);
    assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
  }

  #[test]
  fn focus_returns_false_when_target_has_no_focusable_ancestor() {
    let mut root = Node::new(m::Body::default());
    root.children.push(Node::new(m::Div::default()));
    let mut tree = Tree::new(root);
    assert!(!tree.focus(Some(&[0])));
    assert_eq!(tree.interaction.focus_path, None);
  }

  #[test]
  fn dispatch_mouse_down_with_no_target_clears_selection() {
    let mut tree = Tree::new(Node::new("text"));
    tree.dispatch_mouse_down(None, (0.0, 0.0), MouseButton::Primary, None);
    assert!(tree.interaction.selection.is_none());
  }

  #[test]
  fn dispatch_mouse_down_focuses_focusable_target() {
    // Body → [0] = Button. Pressing the button should focus it.
    let mut root = Node::new(m::Body::default());
    root.children.push(Node::new(m::Button::default()));
    let mut tree = Tree::new(root);
    tree.dispatch_mouse_down(Some(&[0]), (0.0, 0.0), MouseButton::Primary, None);
    assert_eq!(tree.interaction.focus_path.as_deref(), Some([0usize].as_slice()));
  }

  #[test]
  fn dispatch_mouse_down_then_up_synthesises_click() {
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let r = received.clone();
    let mut node = Node::new("text");
    node.on_event = Some(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
    let mut tree = Tree::new(node);
    let path: &[usize] = &[];
    tree.dispatch_mouse_down(Some(path), (1.0, 1.0), MouseButton::Primary, None);
    tree.dispatch_mouse_up(Some(path), (1.0, 1.0), MouseButton::Primary, None);
    let evs = received.lock().unwrap().clone();
    assert!(evs.contains(&"mousedown".into()), "got {evs:?}");
    assert!(evs.contains(&"mouseup".into()), "got {evs:?}");
    assert!(evs.contains(&"click".into()), "got {evs:?}");
  }

  #[test]
  fn dispatch_pointer_move_fires_enter_then_leave() {
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let r = received.clone();
    let mut node = Node::new("text");
    node.on_event = Some(Arc::new(move |ev| {
      r.lock().unwrap().push(ev.event_type().to_string());
    }));
    let mut tree = Tree::new(node);
    tree.dispatch_pointer_move(Some(&[]), (1.0, 1.0), None);
    tree.pointer_leave();
    let evs = received.lock().unwrap().clone();
    assert!(evs.contains(&"mouseenter".into()), "got {evs:?}");
    assert!(evs.contains(&"mouseleave".into()), "got {evs:?}");
  }

  #[test]
  fn buttons_down_bitmask_tracks_press_and_release() {
    let mut tree = Tree::new(Node::new("text"));
    let path: &[usize] = &[];
    assert_eq!(tree.interaction.buttons_down, 0);
    tree.dispatch_mouse_down(Some(path), (0.0, 0.0), MouseButton::Primary, None);
    assert_eq!(tree.interaction.buttons_down, 1);
    tree.dispatch_mouse_up(Some(path), (0.0, 0.0), MouseButton::Primary, None);
    assert_eq!(tree.interaction.buttons_down, 0);
  }

  #[test]
  fn set_modifier_updates_interaction_state() {
    let mut tree = Tree::new(Node::new("text"));
    assert!(!tree.modifiers().shift);
    tree.set_modifier(Modifier::Shift, true);
    assert!(tree.modifiers().shift);
    tree.set_modifier(Modifier::Ctrl, true);
    assert!(tree.modifiers().ctrl);
    assert!(tree.modifiers().shift); // unchanged by previous call
    tree.set_modifier(Modifier::Shift, false);
    assert!(!tree.modifiers().shift);
    assert!(tree.modifiers().ctrl);
  }
}
