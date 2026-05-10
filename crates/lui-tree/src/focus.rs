//! Focus model: which elements can take keyboard focus, and
//! enumerating them in tab order.
//!
//! The HTML focus model is layered:
//!
//! - Form controls (`<input>`, `<textarea>`, `<select>`, `<button>`) are focusable by default unless `disabled` or
//!   `type=hidden`.
//! - `<a>` is focusable iff it has an `href`.
//! - `<summary>` (the first child of an open `<details>`) is focusable. We don't model the open/closed bit yet, so we
//!   treat `<summary>` as always focusable.
//! - Any element with `tabindex >= 0` is focusable; with `-1` it is focusable only via script (pointer /
//!   `Tree::focus`), not tab navigation.
//!
//! Tab order: stable source order across all focusable elements
//! whose `tabindex >= 0` (ignoring positive tabindex precedence
//! tiers, which we punt on; positive values are treated as `0`).

use lui_models as m;

use crate::{Element, Node, Tree};

/// Whether `element` accepts focus from any source (pointer,
/// script, keyboard navigation).
///
/// Returns `false` for `disabled` form controls, `<input
/// type="hidden">`, anchors without an `href`, and any element
/// whose `tabindex` is `Some(t < 0)`.
pub fn is_focusable(element: &Element) -> bool {
  match element {
    // Form controls: focusable unless disabled.
    Element::Input(i) => {
      if i.disabled.unwrap_or(false) {
        return false;
      }
      // type="hidden" is never focusable; everything else is.
      !matches!(i.r#type, Some(m::common::html_enums::InputType::Hidden)) && !is_negative_tabindex(i.tabindex)
    }
    Element::Textarea(t) => !t.disabled.unwrap_or(false) && !is_negative_tabindex(t.tabindex),
    Element::Select(s) => !s.disabled.unwrap_or(false) && !is_negative_tabindex(s.tabindex),
    Element::Button(b) => !b.disabled.unwrap_or(false) && !is_negative_tabindex(b.tabindex),
    // Anchor: focusable iff it has an href.
    Element::A(a) => a.href.is_some() && !is_negative_tabindex(a.tabindex),
    // <summary>: always focusable (we don't track <details> open state).
    Element::Summary(s) => !is_negative_tabindex(s.tabindex),
    // Anything else: focusable only via tabindex >= 0.
    _ => element_tabindex(element).map(|t| t >= 0).unwrap_or(false),
  }
}

/// Whether `element` accepts focus from sequential (Tab) navigation.
///
/// Same as [`is_focusable`] but excludes elements with `tabindex == -1`,
/// matching browser behaviour where `tabindex="-1"` means
/// "scriptable focus only".
pub fn is_keyboard_focusable(element: &Element) -> bool {
  if !is_focusable(element) {
    return false;
  }
  // is_focusable already filters tabindex < 0, so we're done.
  true
}

fn is_negative_tabindex(t: Option<i32>) -> bool {
  matches!(t, Some(v) if v < 0)
}

/// Read the `tabindex` HTML attribute off any element variant
/// that carries it. `Text` returns `None`.
///
/// Thin re-export of [`Element::tabindex`] for callers that
/// prefer free-function form.
pub fn element_tabindex(element: &Element) -> Option<i32> {
  element.tabindex()
}

/// Walk the tree in document order and return the path of every
/// focusable element (including those with `tabindex < 0`, which
/// can still be focused via pointer / `Tree::focus`).
pub fn focusable_paths(tree: &Tree) -> Vec<Vec<usize>> {
  let mut out = Vec::new();
  if let Some(root) = tree.root.as_ref() {
    collect(root, &mut Vec::new(), &mut out, /* kbd_only */ false);
  }
  out
}

/// Like [`focusable_paths`] but only the elements reachable via Tab.
pub fn keyboard_focusable_paths(tree: &Tree) -> Vec<Vec<usize>> {
  let mut out = Vec::new();
  if let Some(root) = tree.root.as_ref() {
    collect(root, &mut Vec::new(), &mut out, /* kbd_only */ true);
  }
  out
}

fn collect(node: &Node, path: &mut Vec<usize>, out: &mut Vec<Vec<usize>>, kbd_only: bool) {
  let pickable = if kbd_only {
    is_keyboard_focusable(&node.element)
  } else {
    is_focusable(&node.element)
  };
  if pickable {
    out.push(path.clone());
  }
  for (i, child) in node.children.iter().enumerate() {
    path.push(i);
    collect(child, path, out, kbd_only);
    path.pop();
  }
}

/// Find the next path in `paths` strictly after `from`, wrapping
/// to the first entry. Returns `None` if `paths` is empty.
///
/// `from = None` returns the first path.
pub fn next_in_order<'a>(paths: &'a [Vec<usize>], from: Option<&[usize]>) -> Option<&'a [usize]> {
  if paths.is_empty() {
    return None;
  }
  match from {
    None => Some(paths[0].as_slice()),
    Some(cur) => {
      let idx = paths.iter().position(|p| p.as_slice() == cur);
      let next_idx = match idx {
        Some(i) => (i + 1) % paths.len(),
        // Current focus is no longer focusable (e.g. its
        // tabindex changed) — fall through to the start.
        None => 0,
      };
      Some(paths[next_idx].as_slice())
    }
  }
}

/// Find the previous path in `paths` (Shift+Tab), wrapping to the
/// last entry. Returns `None` if `paths` is empty.
pub fn prev_in_order<'a>(paths: &'a [Vec<usize>], from: Option<&[usize]>) -> Option<&'a [usize]> {
  if paths.is_empty() {
    return None;
  }
  match from {
    None => Some(paths[paths.len() - 1].as_slice()),
    Some(cur) => {
      let idx = paths.iter().position(|p| p.as_slice() == cur);
      let prev_idx = match idx {
        Some(0) => paths.len() - 1,
        Some(i) => i - 1,
        None => paths.len() - 1,
      };
      Some(paths[prev_idx].as_slice())
    }
  }
}
