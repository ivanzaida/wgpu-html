use crate::{Element, Node};

#[derive(Debug, Clone, Copy, Default)]
pub struct PatchResult {
  pub selector_changed: bool,
  pub any_change: bool,
}

impl PatchResult {
  fn merge(&mut self, other: PatchResult) {
    self.selector_changed |= other.selector_changed;
    self.any_change |= other.any_change;
  }
}

pub fn patch_node(old: &mut Node, new: Node) -> PatchResult {
  if !old.element.same_tag(&new.element) {
    *old = new;
    return PatchResult { selector_changed: true, any_change: true };
  }

  let mut result = PatchResult::default();

  let old_id = old.element.id().map(|s| s as *const str);
  let old_class = old.element.class().map(|s| s as *const str);

  result.merge(patch_element(&mut old.element, new.element));

  let new_id = old.element.id().map(|s| s as *const str);
  let new_class = old.element.class().map(|s| s as *const str);
  if old_id != new_id || old_class != new_class {
    result.selector_changed = true;
  }

  old.on_click = new.on_click;
  old.on_mouse_down = new.on_mouse_down;
  old.on_mouse_up = new.on_mouse_up;
  old.on_mouse_move = new.on_mouse_move;
  old.on_mouse_enter = new.on_mouse_enter;
  old.on_mouse_leave = new.on_mouse_leave;
  old.on_keydown = new.on_keydown;
  old.on_keyup = new.on_keyup;
  old.on_focus = new.on_focus;
  old.on_blur = new.on_blur;
  old.on_focusin = new.on_focusin;
  old.on_focusout = new.on_focusout;
  old.on_input = new.on_input;
  old.on_beforeinput = new.on_beforeinput;
  old.on_change = new.on_change;
  old.on_wheel = new.on_wheel;
  old.on_dblclick = new.on_dblclick;
  old.on_contextmenu = new.on_contextmenu;
  old.on_auxclick = new.on_auxclick;
  old.on_dragstart = new.on_dragstart;
  old.on_dragend = new.on_dragend;
  old.on_drop = new.on_drop;
  old.on_drag = new.on_drag;
  old.on_dragover = new.on_dragover;
  old.on_dragenter = new.on_dragenter;
  old.on_dragleave = new.on_dragleave;
  old.draggable = new.draggable;
  old.on_copy = new.on_copy;
  old.on_cut = new.on_cut;
  old.on_paste = new.on_paste;
  old.on_scroll = new.on_scroll;
  old.on_select = new.on_select;
  old.on_event = new.on_event;

  old.custom_properties = new.custom_properties;
  old.raw_attrs = new.raw_attrs;

  result.merge(patch_children(&mut old.children, new.children));

  result
}

fn patch_element(old: &mut Element, new: Element) -> PatchResult {
  let mut result = PatchResult::default();

  match (old, new) {
    (Element::Text(old_t), Element::Text(new_t)) => {
      if **old_t != *new_t {
        *old_t = new_t;
        result.any_change = true;
      }
    }
    (Element::Input(old_inp), Element::Input(new_inp)) => {
      let live_value = old_inp.value.take();
      *old_inp = new_inp;
      if live_value.is_some() {
        old_inp.value = live_value;
      }
      result.any_change = true;
    }
    (Element::Textarea(old_ta), Element::Textarea(new_ta)) => {
      let live_value = old_ta.value.take();
      *old_ta = new_ta;
      if live_value.is_some() {
        old_ta.value = live_value;
      }
      result.any_change = true;
    }
    (old_el, new_el) => {
      *old_el = new_el;
      result.any_change = true;
    }
  }

  result
}

fn patch_children(old_children: &mut Vec<Node>, new_children: Vec<Node>) -> PatchResult {
  let mut result = PatchResult::default();
  let old_len = old_children.len();
  let new_len = new_children.len();
  let common = old_len.min(new_len);

  let mut new_iter = new_children.into_iter();

  for i in 0..common {
    let new_child = new_iter.next().unwrap();
    result.merge(patch_node(&mut old_children[i], new_child));
  }

  for new_child in new_iter {
    old_children.push(new_child);
    result.any_change = true;
    result.selector_changed = true;
  }

  if old_len > new_len {
    old_children.truncate(new_len);
    result.any_change = true;
    result.selector_changed = true;
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use wgpu_html_models::html as m;

  fn with_kids(mut node: Node, children: Vec<Node>) -> Node {
    node.children = children;
    node
  }

  fn div_node() -> Node {
    Node::new(Element::Div(m::Div::default()))
  }

  fn span_node() -> Node {
    Node::new(Element::Span(m::Span::default()))
  }

  fn text_node(s: &str) -> Node {
    Node::new(Element::Text(s.into()))
  }

  fn input_with_value(v: &str) -> Node {
    let mut inp = m::Input::default();
    inp.value = Some(v.into());
    Node::new(Element::Input(inp))
  }

  fn input_no_value() -> Node {
    Node::new(Element::Input(m::Input::default()))
  }

  #[test]
  fn same_structure_patches_text() {
    let mut old = with_kids(div_node(), vec![text_node("hello")]);
    let new = with_kids(div_node(), vec![text_node("world")]);
    let r = patch_node(&mut old, new);
    assert!(r.any_change);
    assert!(!r.selector_changed);
    match &old.children[0].element {
      Element::Text(t) => assert_eq!(&**t, "world"),
      _ => panic!("expected text"),
    }
  }

  #[test]
  fn different_tag_replaces_wholesale() {
    let mut old = div_node();
    let new = span_node();
    let r = patch_node(&mut old, new);
    assert!(r.selector_changed);
    assert_eq!(old.element.tag_name(), "span");
  }

  #[test]
  fn preserves_rect_on_same_tag() {
    let mut old = div_node();
    old.rect = Some(crate::NodeRect { x: 1.0, y: 2.0, width: 3.0, height: 4.0 });
    let new = div_node();
    patch_node(&mut old, new);
    assert!(old.rect.is_some());
  }

  #[test]
  fn input_value_preserved_when_new_is_none() {
    let mut old = input_with_value("user typed");
    let new = input_no_value();
    patch_node(&mut old, new);
    match &old.element {
      Element::Input(inp) => assert_eq!(inp.value.as_deref(), Some("user typed")),
      _ => panic!("expected input"),
    }
  }

  #[test]
  fn input_live_value_always_preserved() {
    let mut old = input_with_value("user typed");
    let new = input_with_value("stale");
    patch_node(&mut old, new);
    match &old.element {
      Element::Input(inp) => assert_eq!(inp.value.as_deref(), Some("user typed")),
      _ => panic!("expected input"),
    }
  }

  #[test]
  fn children_appended() {
    let mut old = with_kids(div_node(), vec![text_node("a")]);
    let new = with_kids(div_node(), vec![text_node("a"), text_node("b")]);
    let r = patch_node(&mut old, new);
    assert!(r.selector_changed);
    assert_eq!(old.children.len(), 2);
  }

  #[test]
  fn children_truncated() {
    let mut old = with_kids(div_node(), vec![text_node("a"), text_node("b")]);
    let new = with_kids(div_node(), vec![text_node("a")]);
    let r = patch_node(&mut old, new);
    assert!(r.selector_changed);
    assert_eq!(old.children.len(), 1);
  }
}
