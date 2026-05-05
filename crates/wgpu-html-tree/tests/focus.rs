use wgpu_html_models as m;
use wgpu_html_tree::{
  Element, Node, Tree, focusable_paths, is_focusable, is_keyboard_focusable, next_in_order, prev_in_order,
};

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
  assert!(!is_focusable(&el));
  assert!(!is_keyboard_focusable(&el));
}

#[test]
fn focusable_paths_collects_in_document_order() {
  let mut body = Node::new(m::Body::default());
  body
    .children
    .push(Node::new(input(m::common::html_enums::InputType::Text)));
  body.children.push(Node::new(m::Button::default()));
  body
    .children
    .push(Node::new(input(m::common::html_enums::InputType::Hidden)));
  let mut a = m::A::default();
  a.href = Some("/".into());
  body.children.push(Node::new(a));

  let tree = Tree::new(body);
  let paths = focusable_paths(&tree);
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
