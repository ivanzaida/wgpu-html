use lui_core::{HtmlDocument, HtmlElement, HtmlNode};

fn parse_doc(html: &str) -> HtmlDocument {
  lui_parse::parse(html)
}

fn parse_root(html: &str) -> HtmlNode {
  lui_parse::parse(html).root
}

// ── getters return correct values from parsed HTML ──

#[test]
fn getters_reflect_parsed_attributes() {
  let root = parse_root(r#"<html><body><div id="main" class="a b" data-x="1" aria-label="nav">hi</div></body></html>"#);
  let div = root.query_selector("div").unwrap();

  assert_eq!(div.tag_name(), "div");
  assert_eq!(*div.element(), HtmlElement::Div);
  assert_eq!(div.id(), Some("main"));
  assert!(div.class_list().contains("a"));
  assert!(div.class_list().contains("b"));
  assert_eq!(div.class_list().len(), 2);
  assert_eq!(div.data_attrs().get("x").map(|s| s.as_ref()), Some("1"));
  assert_eq!(div.aria_attrs().get("label").map(|s| s.as_ref()), Some("nav"));
  assert_eq!(div.text_content(), "hi");
  assert_eq!(div.children().len(), 1);
}

#[test]
fn getters_reflect_parsed_inline_styles() {
  let root = parse_root(r#"<html><body><div style="color: red; margin: 10px">x</div></body></html>"#);
  let div = root.query_selector("div").unwrap();
  assert!(!div.styles().is_empty());
}

#[test]
fn getters_work_on_text_node() {
  let node = HtmlNode::text("hello");
  assert_eq!(node.tag_name(), "#text");
  assert_eq!(node.id(), None);
  assert!(node.class_list().is_empty());
  assert!(node.children().is_empty());
  assert_eq!(node.text_content(), "hello");
}

// ── every mutation method marks the node dirty ──

#[test]
fn set_attribute_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.set_attribute("href", "/home");
  assert!(node.is_dirty());
}

#[test]
fn remove_attribute_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("href", "/home");
  node.clear_dirty();
  node.remove_attribute("href");
  assert!(node.is_dirty());
}

#[test]
fn set_text_content_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.set_text_content("changed");
  assert!(node.is_dirty());
}

#[test]
fn set_styles_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.set_styles(vec![]);
  assert!(node.is_dirty());
}

#[test]
fn class_list_add_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.class_list_mut().add("x");
  assert!(node.is_dirty());
}

#[test]
fn class_list_remove_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("x");
  node.clear_dirty();
  node.class_list_mut().remove("x");
  assert!(node.is_dirty());
}

#[test]
fn class_list_toggle_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.class_list_mut().toggle("x");
  assert!(node.is_dirty());
}

#[test]
fn class_list_set_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.class_list_mut().set("a b c");
  assert!(node.is_dirty());
}

#[test]
fn class_list_clear_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("x");
  node.clear_dirty();
  node.class_list_mut().clear();
  assert!(node.is_dirty());
}

#[test]
fn append_child_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.append_child(HtmlNode::text("x"));
  assert!(node.is_dirty());
}

#[test]
fn insert_child_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.insert_child(0, HtmlNode::text("x"));
  assert!(node.is_dirty());
}

#[test]
fn remove_child_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("x"));
  node.clear_dirty();
  node.remove_child(0);
  assert!(node.is_dirty());
}

#[test]
fn replace_child_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("x"));
  node.clear_dirty();
  node.replace_child(0, HtmlNode::text("y"));
  assert!(node.is_dirty());
}

#[test]
fn set_children_marks_dirty() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.set_children(vec![HtmlNode::text("a"), HtmlNode::text("b")]);
  assert!(node.is_dirty());
}

// ── no-op mutations do NOT mark dirty ──

#[test]
fn noop_remove_attribute_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.remove_attribute("nonexistent");
  assert!(!node.is_dirty());
}

#[test]
fn noop_class_remove_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.class_list_mut().remove("nonexistent");
  assert!(!node.is_dirty());
}

#[test]
fn noop_class_add_duplicate_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("x");
  node.clear_dirty();
  node.class_list_mut().add("x");
  assert!(!node.is_dirty());
}

#[test]
fn noop_class_clear_on_empty_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.class_list_mut().clear();
  assert!(!node.is_dirty());
}

#[test]
fn noop_remove_child_oob_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.remove_child(0);
  assert!(!node.is_dirty());
}

#[test]
fn noop_replace_child_oob_stays_clean() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.clear_dirty();
  node.replace_child(5, HtmlNode::text("x"));
  assert!(!node.is_dirty());
}

// ── dirty propagates through collect_dirty_paths ──

#[test]
fn collect_dirty_paths_finds_deep_mutation() {
  let mut doc = parse_doc(
    r#"<html><body><div><span><em id="deep">text</em></span></div></body></html>"#,
  );
  doc.collect_dirty_paths();

  let em = doc.root.query_selector_mut("#deep").unwrap();
  em.set_attribute("class", "changed");

  assert!(doc.collect_dirty_paths());
  let found = doc.dirty_paths.iter().any(|p| {
    doc.root.at_path(p).and_then(|n| n.id()) == Some("deep")
  });
  assert!(found, "should find deep mutated node in dirty_paths");
}

#[test]
fn collect_dirty_paths_clears_all_flags() {
  let mut doc = parse_doc("<html><body><div id=\"a\"></div><div id=\"b\"></div></body></html>");
  doc.collect_dirty_paths();

  doc.root.query_selector_mut("#a").unwrap().set_attribute("class", "x");
  doc.root.query_selector_mut("#b").unwrap().class_list_mut().add("y");

  doc.collect_dirty_paths();

  // Second collect should find nothing
  assert!(!doc.collect_dirty_paths());
  assert!(doc.dirty_paths.is_empty());
}

#[test]
fn generation_only_bumps_when_actually_dirty() {
  let mut doc = parse_doc("<html><body><div>text</div></body></html>");
  doc.collect_dirty_paths();
  let after_initial = doc.generation;

  // No mutations — generation stays
  doc.collect_dirty_paths();
  assert_eq!(doc.generation, after_initial);

  // Mutation — generation bumps
  doc.root.query_selector_mut("div").unwrap().set_text_content("new");
  doc.collect_dirty_paths();
  assert_eq!(doc.generation, after_initial + 1);

  // No mutations again — stays
  doc.collect_dirty_paths();
  assert_eq!(doc.generation, after_initial + 1);
}

// ── event handler mutations tracked via dirty flags ──

#[test]
fn event_handler_mutation_detected_by_collect() {
  use std::sync::Arc;

  let mut doc = parse_doc(r#"<html><body><div id="btn">click me</div></body></html>"#);
  doc.collect_dirty_paths();

  let btn = doc.root.query_selector_mut("#btn").unwrap();
  btn.add_event_listener(
    "click",
    Arc::new(|node, _| {
      node.class_list_mut().add("clicked");
    }),
  );

  let mut event = lui_core::events::DocumentEvent::Event(lui_core::events::EventInit {
    event_type: "click".into(),
    bubbles: true,
    cancelable: true,
    ..Default::default()
  });
  let btn = doc.root.query_selector_mut("#btn").unwrap();
  btn.dispatch_event(&mut event);

  assert!(doc.collect_dirty_paths());
  assert!(doc.root.query_selector("#btn").unwrap().class_list().contains("clicked"));
}

// ── set_inner_html marks dirty ──

#[test]
fn set_inner_html_marks_parent_dirty() {
  let mut doc = parse_doc("<html><body><div id=\"container\">old</div></body></html>");
  doc.collect_dirty_paths();

  let container = doc.root.query_selector_mut("#container").unwrap();
  lui_parse::set_inner_html(container, "<span>new</span>");

  assert!(doc.collect_dirty_paths());
}

// ── new nodes start dirty and get collected ──

#[test]
fn appended_node_collected_as_dirty() {
  let mut doc = parse_doc("<html><body><div id=\"parent\"></div></body></html>");
  doc.collect_dirty_paths();

  let parent = doc.root.query_selector_mut("#parent").unwrap();
  let mut child = HtmlNode::new(HtmlElement::Span);
  child.set_attribute("id", "new-child");
  parent.append_child(child);

  doc.collect_dirty_paths();

  let found_parent = doc.dirty_paths.iter().any(|p| {
    doc.root.at_path(p).and_then(|n| n.id()) == Some("parent")
  });
  let found_child = doc.dirty_paths.iter().any(|p| {
    doc.root.at_path(p).and_then(|n| n.id()) == Some("new-child")
  });
  assert!(found_parent, "parent should be dirty (DIRTY_CHILDREN)");
  assert!(found_child, "new child should be dirty (starts as DIRTY_ALL)");
}
