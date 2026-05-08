use wgpu_html_models as m;
use wgpu_html_tree::{Element, Node, Tree, wrap_in_document};

#[test]
fn wrap_in_document_wraps_div_in_html_head_body() {
  let div = Node::new(m::Div::default());
  let wrapped = wrap_in_document(div);

  assert_eq!(wrapped.element.tag_name(), "html");
  assert_eq!(wrapped.children.len(), 2);
  assert_eq!(wrapped.children[0].element.tag_name(), "head");
  assert_eq!(wrapped.children[1].element.tag_name(), "body");
  assert_eq!(wrapped.children[1].children.len(), 1);
  assert_eq!(wrapped.children[1].children[0].element.tag_name(), "div");
}

#[test]
fn wrap_in_document_preserves_html_root() {
  let html = Node::new(m::Html::default()).with_children(vec![
    Node::new(m::Head::default()),
    Node::new(m::Body::default()),
  ]);
  let wrapped = wrap_in_document(html);

  assert_eq!(wrapped.element.tag_name(), "html");
  assert_eq!(wrapped.children.len(), 2);
}

#[test]
fn wrap_in_document_preserves_body_root_inside_html() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Div::default()),
  ]);
  let wrapped = wrap_in_document(body);

  assert_eq!(wrapped.element.tag_name(), "html");
  assert_eq!(wrapped.children[1].element.tag_name(), "body");
  assert_eq!(wrapped.children[1].children.len(), 1);
}

#[test]
fn set_root_wraps_when_wrap_body_is_true() {
  let mut tree = Tree::default();
  assert!(tree.wrap_body);
  tree.set_root(Node::new(m::Div::default()));

  let root = tree.root.as_ref().unwrap();
  assert_eq!(root.element.tag_name(), "html");
  assert_eq!(root.children[1].children[0].element.tag_name(), "div");
}

#[test]
fn set_root_skips_wrap_when_disabled() {
  let mut tree = Tree::default();
  tree.with_body(false);
  tree.set_root(Node::new(m::Div::default()));

  let root = tree.root.as_ref().unwrap();
  assert_eq!(root.element.tag_name(), "div");
}

#[test]
fn set_root_does_not_double_wrap_html() {
  let mut tree = Tree::default();
  let html = Node::new(m::Html::default()).with_children(vec![
    Node::new(m::Head::default()),
    Node::new(m::Body::default()),
  ]);
  tree.set_root(html);

  let root = tree.root.as_ref().unwrap();
  assert_eq!(root.element.tag_name(), "html");
  assert_ne!(root.children[0].element.tag_name(), "html");
}

#[test]
fn set_root_increments_generation() {
  let mut tree = Tree::default();
  let gen_before = tree.generation;
  tree.set_root(Node::new(m::Div::default()));
  assert!(tree.generation > gen_before);
}

#[test]
fn set_root_preserves_children_of_wrapped_node() {
  let mut tree = Tree::default();
  let mut inner = m::Div::default();
  inner.id = Some("inner".into());
  let div = Node::new(m::Div::default()).with_children(vec![
    Node::new(inner),
    Node::new(Element::Text("hello".into())),
  ]);
  tree.set_root(div);

  let body = &tree.root.as_ref().unwrap().children[1];
  let user_div = &body.children[0];
  assert_eq!(user_div.children.len(), 2);
  assert_eq!(user_div.children[0].element.id(), Some("inner"));
  assert!(matches!(&user_div.children[1].element, Element::Text(t) if t.as_ref() == "hello"));
}
