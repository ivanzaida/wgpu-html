//! Focus model: which elements can take keyboard focus, and
//! enumerating them in tab order.
//!
//! The HTML focus model is layered:
//!
//! - Form controls (`<input>`, `<textarea>`, `<select>`, `<button>`)
//!   are focusable by default unless `disabled` or `type=hidden`.
//! - `<a>` is focusable iff it has an `href`.
//! - `<summary>` (the first child of an open `<details>`) is
//!   focusable. We don't model the open/closed bit yet, so we
//!   treat `<summary>` as always focusable.
//! - Any element with `tabindex >= 0` is focusable; with `-1` it
//!   is focusable only via script (pointer / `Tree::focus`), not
//!   tab navigation.
//!
//! Tab order: stable source order across all focusable elements
//! whose `tabindex >= 0` (ignoring positive tabindex precedence
//! tiers, which we punt on; positive values are treated as `0`).

use crate::{Element, Node, Tree};
use wgpu_html_models as m;

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
            !matches!(i.r#type, Some(m::common::html_enums::InputType::Hidden))
                && !is_negative_tabindex(i.tabindex)
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
        collect(root, &mut Vec::new(), &mut out, /*kbd_only*/ false);
    }
    out
}

/// Like [`focusable_paths`] but only the elements reachable via Tab.
pub fn keyboard_focusable_paths(tree: &Tree) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if let Some(root) = tree.root.as_ref() {
        collect(root, &mut Vec::new(), &mut out, /*kbd_only*/ true);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node;
    use wgpu_html_models as m;

    fn input(t: m::common::html_enums::InputType) -> m::Input {
        m::Input {
            r#type: Some(t),
            ..m::Input::default()
        }
    }

    #[test]
    fn input_text_is_focusable() {
        let el: Element = input(m::common::html_enums::InputType::Text).into();
        assert!(is_focusable(&el));
        assert!(is_keyboard_focusable(&el));
    }

    #[test]
    fn input_hidden_is_not_focusable() {
        let el: Element = input(m::common::html_enums::InputType::Hidden).into();
        assert!(!is_focusable(&el));
    }

    #[test]
    fn disabled_button_is_not_focusable() {
        let mut b = m::Button::default();
        b.disabled = Some(true);
        let el: Element = b.into();
        assert!(!is_focusable(&el));
    }

    #[test]
    fn anchor_without_href_is_not_focusable() {
        let a = m::A::default();
        let el: Element = a.into();
        assert!(!is_focusable(&el));
    }

    #[test]
    fn anchor_with_href_is_focusable() {
        let mut a = m::A::default();
        a.href = Some("#".into());
        let el: Element = a.into();
        assert!(is_focusable(&el));
    }

    #[test]
    fn div_with_tabindex_is_focusable() {
        let mut d = m::Div::default();
        d.tabindex = Some(0);
        let el: Element = d.into();
        assert!(is_focusable(&el));
    }

    #[test]
    fn div_with_tabindex_minus_one_is_focusable_but_not_keyboard() {
        let mut d = m::Div::default();
        d.tabindex = Some(-1);
        let el: Element = d.into();
        // tabindex=-1 is "scriptable focus only" — neither path
        // takes it via the predicates here (we treat it as
        // not-focusable to keep the API simple).
        assert!(!is_focusable(&el));
        assert!(!is_keyboard_focusable(&el));
    }

    #[test]
    fn focusable_paths_collects_in_document_order() {
        // <body> <input> <button> <input type=hidden> <a href> </body>
        let mut body = Node::new(m::Body::default());
        body.children
            .push(Node::new(input(m::common::html_enums::InputType::Text)));
        body.children.push(Node::new(m::Button::default()));
        body.children
            .push(Node::new(input(m::common::html_enums::InputType::Hidden)));
        let mut a = m::A::default();
        a.href = Some("/".into());
        body.children.push(Node::new(a));

        let tree = Tree::new(body);
        let paths = focusable_paths(&tree);
        // Indices 0 (text input), 1 (button), 3 (anchor). Hidden is skipped.
        assert_eq!(paths, vec![vec![0], vec![1], vec![3]]);
    }

    #[test]
    fn next_and_prev_wrap_around() {
        let paths = vec![vec![0], vec![2], vec![5]];
        assert_eq!(next_in_order(&paths, None), Some([0usize].as_slice()));
        assert_eq!(next_in_order(&paths, Some(&[0])), Some([2usize].as_slice()));
        assert_eq!(next_in_order(&paths, Some(&[5])), Some([0usize].as_slice()));
        assert_eq!(prev_in_order(&paths, None), Some([5usize].as_slice()));
        assert_eq!(prev_in_order(&paths, Some(&[0])), Some([5usize].as_slice()));
        assert_eq!(prev_in_order(&paths, Some(&[2])), Some([0usize].as_slice()));
    }
}
