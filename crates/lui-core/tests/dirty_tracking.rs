use lui_core::{HtmlDocument, HtmlElement, HtmlNode};

fn parse_doc(html: &str) -> HtmlDocument {
  lui_parse::parse(html)
}

// ── per-node dirty flags ──

#[test]
fn new_node_starts_dirty() {
  let node = HtmlNode::new(HtmlElement::Div);
  assert!(node.is_dirty());
}

#[test]
fn clear_dirty_resets_flags() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  assert!(!node.is_dirty());
}

#[test]
fn set_attribute_marks_dirty_attrs() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.set_attribute("id", "test");
  assert!(node.is_dirty());
}

#[test]
fn remove_attribute_marks_dirty_attrs() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("id", "test");
  node.clear_dirty();
  node.remove_attribute("id");
  assert!(node.is_dirty());
}

#[test]
fn add_class_marks_class_list_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.class_list_mut().add("active");
  assert!(node.class_list().is_dirty());
  assert!(node.is_dirty());
}

#[test]
fn set_text_content_marks_dirty_text_and_children() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.set_text_content("hello");
  assert!(node.is_dirty());
}

#[test]
fn append_child_marks_dirty_children() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.append_child(HtmlNode::text("x"));
  assert!(node.is_dirty());
}

#[test]
fn remove_child_marks_dirty_children() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("x"));
  node.clear_dirty();
  node.remove_child(0);
  assert!(node.is_dirty());
}

#[test]
fn replace_child_marks_dirty_children() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("old"));
  node.clear_dirty();
  node.replace_child(0, HtmlNode::text("new"));
  assert!(node.is_dirty());
}

#[test]
fn no_op_remove_attribute_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.remove_attribute("nonexistent");
  assert!(!node.is_dirty());
}

// ── document collect_dirty_paths ──

#[test]
fn fresh_document_has_dirty_root() {
  let mut doc = parse_doc("<html><body><div>text</div></body></html>");
  assert!(doc.collect_dirty_paths());
  assert!(!doc.dirty_paths.is_empty());
}

#[test]
fn clean_document_returns_false() {
  let mut doc = parse_doc("<html><body><div>text</div></body></html>");
  doc.collect_dirty_paths();
  let prev_gen = doc.generation;
  assert!(!doc.collect_dirty_paths());
  assert_eq!(doc.generation, prev_gen);
}

#[test]
fn mutation_after_clean_bumps_generation() {
  let mut doc = parse_doc("<html><body><div id=\"target\">text</div></body></html>");
  doc.collect_dirty_paths();
  let prev_gen = doc.generation;

  let node = doc.root.query_selector_mut("div").unwrap();
  node.set_attribute("class", "updated");

  assert!(doc.collect_dirty_paths());
  assert_eq!(doc.generation, prev_gen + 1);
}

#[test]
fn dirty_path_points_to_mutated_node() {
  let mut doc = parse_doc(r#"<html><body><div id="a"></div><div id="b"></div></body></html>"#);
  doc.collect_dirty_paths();

  let node = doc.root.query_selector_mut("#b").unwrap();
  node.class_list_mut().add("changed");

  doc.collect_dirty_paths();
  let has_b_path = doc.dirty_paths.iter().any(|p| {
    doc.root.at_path(p).and_then(|n| n.id()) == Some("b")
  });
  assert!(has_b_path, "dirty_paths should contain the path to #b, got: {:?}", doc.dirty_paths);
}

#[test]
fn multiple_mutations_collected_in_one_pass() {
  let mut doc = parse_doc(r#"<html><body><div id="a"></div><div id="b"></div></body></html>"#);
  doc.collect_dirty_paths();

  doc.root.query_selector_mut("#a").unwrap().class_list_mut().add("x");
  doc.root.query_selector_mut("#b").unwrap().class_list_mut().add("y");

  doc.collect_dirty_paths();
  assert!(doc.dirty_paths.len() >= 2);
}

#[test]
fn structural_mutation_detected() {
  let mut doc = parse_doc("<html><body><div></div></body></html>");
  doc.collect_dirty_paths();

  let div = doc.root.query_selector_mut("div").unwrap();
  div.append_child(HtmlNode::new(HtmlElement::Span));

  assert!(doc.collect_dirty_paths());
}
