use std::sync::{
  Arc,
  atomic::{AtomicUsize, Ordering},
};

use wgpu_html_models as m;
use wgpu_html_tree::{Element, MouseButton, MouseEvent, Modifiers, Node, Tree};

fn div_with_id(id: &str) -> m::Div {
  m::Div {
    id: Some(id.to_string()),
    ..m::Div::default()
  }
}

#[test]
fn element_id_reads_global_attribute() {
  let div = Element::Div(div_with_id("hero"));
  assert_eq!(div.id(), Some("hero"));
  let txt = Element::Text("hi".into());
  assert_eq!(txt.id(), None);
}

#[test]
fn get_element_by_id_finds_descendant() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(div_with_id("outer")).with_children(vec![Node::new(div_with_id("inner"))]),
  ]);
  let mut tree = Tree::new(body);
  assert!(tree.get_element_by_id("outer").is_some());
  assert!(tree.get_element_by_id("inner").is_some());
  assert!(tree.get_element_by_id("missing").is_none());
}

#[test]
fn dpi_scale_override_falls_back_to_host_scale() {
  let mut tree = Tree::new(Node::new(m::Body::default()));
  assert_eq!(tree.effective_dpi_scale(1.5), 1.5);

  tree.set_dpi_scale_override(Some(2.0));
  assert_eq!(tree.effective_dpi_scale(1.5), 2.0);

  tree.set_dpi_scale_override(Some(0.0));
  assert_eq!(tree.effective_dpi_scale(1.5), 1.5);

  tree.set_dpi_scale_override(None);
  assert_eq!(tree.effective_dpi_scale(f32::NAN), 1.0);
}

#[test]
fn on_click_field_is_assignable_and_invokable() {
  let mut tree = Tree::new(Node::new(div_with_id("target")));
  let counter = Arc::new(AtomicUsize::new(0));
  let c2 = counter.clone();

  tree
    .get_element_by_id("target")
    .unwrap()
    .on_click
    .push(Arc::new(move |_ev| {
      c2.fetch_add(1, Ordering::Relaxed);
    }));

  let cb = tree.get_element_by_id("target").unwrap().on_click[0].clone();
  let ev = MouseEvent {
    pos: (0.0, 0.0),
    button: Some(MouseButton::Primary),
    modifiers: Modifiers::default(),
    target_path: vec![],
    current_path: vec![],
  };
  cb(&ev);
  cb(&ev);
  assert_eq!(counter.load(Ordering::Relaxed), 2);
}

#[test]
fn first_match_wins_in_document_order() {
  let body =
    Node::new(m::Body::default()).with_children(vec![Node::new(div_with_id("dup")), Node::new(div_with_id("dup"))]);
  let mut tree = Tree::new(body);
  let first = tree.get_element_by_id("dup").unwrap();
  first.on_click.push(Arc::new(|_| {}));
  let body_node = tree.root.as_ref().unwrap();
  assert!(!body_node.children[0].on_click.is_empty());
  assert!(body_node.children[1].on_click.is_empty());
}

#[test]
fn template_content_can_be_cloned_and_appended_by_id() {
  let mut template = m::Template::default();
  template.id = Some("tpl".to_owned());
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(template).with_children(vec![Node::new(div_with_id("from-template"))]),
    Node::new(div_with_id("host")),
  ]);
  let mut tree = Tree::new(body);

  let cloned = tree.clone_template_content_by_id("tpl").expect("template content");
  assert_eq!(cloned.len(), 1);
  assert_eq!(cloned[0].element.id(), Some("from-template"));

  let before = tree.generation;
  let inserted = tree
    .append_template_content_to_id("tpl", "host")
    .expect("inserted range");
  assert_eq!(inserted, 0..1);
  assert_eq!(tree.generation, before + 1);

  let root = tree.root.as_ref().unwrap();
  let template_node = root.children[0].find_by_id("from-template").unwrap();
  assert_eq!(template_node.element.id(), Some("from-template"));
  let host = root.find_by_id("host").unwrap();
  assert_eq!(host.children.len(), 1);
  assert_eq!(host.children[0].element.id(), Some("from-template"));
}

#[test]
fn template_content_can_be_inserted_at_path_index() {
  let mut template = m::Template::default();
  template.id = Some("tpl".to_owned());
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(template).with_children(vec![Node::new(div_with_id("inserted"))]),
    Node::new(div_with_id("host"))
      .with_children(vec![Node::new(div_with_id("before")), Node::new(div_with_id("after"))]),
  ]);
  let mut tree = Tree::new(body);

  let inserted = tree.insert_template_content("tpl", &[1], 1).expect("inserted range");
  assert_eq!(inserted, 1..2);
  let host = tree.root.as_ref().unwrap().children[1].find_by_id("host").unwrap();
  let ids: Vec<_> = host.children.iter().map(|child| child.element.id()).collect();
  assert_eq!(ids, vec![Some("before"), Some("inserted"), Some("after")]);

  assert!(tree.insert_template_content("tpl", &[1], 99).is_none());
}
