use lui_core::{HtmlElement, HtmlNode};

fn parse_root(html: &str) -> HtmlNode {
  lui_parse::parse(html).root
}

// ── text_content ──

#[test]
fn text_content_returns_all_text() {
  let root = parse_root("<html><body><div>hello <span>world</span></div></body></html>");
  let div = root.query_selector("div").unwrap();
  assert_eq!(div.text_content(), "hello world");
}

#[test]
fn text_content_returns_empty_for_no_text() {
  let root = parse_root("<html><body><div><span></span></div></body></html>");
  let div = root.query_selector("div").unwrap();
  assert_eq!(div.text_content(), "");
}

// ── set_text_content ──

#[test]
fn set_text_content_replaces_children() {
  let mut root = parse_root("<html><body><div><span>old</span></div></body></html>");
  let div = root.query_selector_mut("div").unwrap();
  div.set_text_content("new text");
  assert_eq!(div.children().len(), 1);
  assert_eq!(div.text_content(), "new text");
}

#[test]
fn set_text_content_empty_clears_children() {
  let mut root = parse_root("<html><body><div>content</div></body></html>");
  let div = root.query_selector_mut("div").unwrap();
  div.set_text_content("");
  assert!(div.children().is_empty());
}

// ── set_attribute / remove_attribute ──

#[test]
fn set_attribute_id() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("id", "test");
  assert_eq!(node.id(), Some("test"));
}

#[test]
fn set_attribute_class() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("class", "foo bar");
  assert_eq!(node.class_list().len(), 2);
  assert!(node.class_list().contains("foo"));
  assert!(node.class_list().contains("bar"));
}

#[test]
fn set_attribute_data() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("data-value", "42");
  assert_eq!(node.data_attrs().get("value").map(|s| s.as_ref()), Some("42"));
}

#[test]
fn set_attribute_aria() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("aria-label", "close");
  assert_eq!(node.aria_attrs().get("label").map(|s| s.as_ref()), Some("close"));
}

#[test]
fn set_attribute_generic() {
  let mut node = HtmlNode::new(HtmlElement::Input);
  node.set_attribute("type", "text");
  assert_eq!(node.attr("type").map(|s| s.as_ref()), Some("text"));
}

#[test]
fn remove_attribute_id() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.set_attribute("id", "test");
  assert!(node.remove_attribute("id"));
  assert!(node.id().is_none());
}

#[test]
fn remove_attribute_returns_false_when_missing() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  assert!(!node.remove_attribute("id"));
}

// ── class manipulation ──

#[test]
fn add_class_adds_new() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("active");
  assert!(node.class_list().contains("active"));
}

#[test]
fn add_class_does_not_duplicate() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("active");
  node.class_list_mut().add("active");
  assert_eq!(node.class_list().len(), 1);
}

#[test]
fn remove_class_removes_existing() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("a");
  node.class_list_mut().add("b");
  assert!(node.class_list_mut().remove("a"));
  assert!(!node.class_list().contains("a"));
  assert!(node.class_list().contains("b"));
}

#[test]
fn toggle_class_adds_when_absent() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  assert!(node.class_list_mut().toggle("active"));
  assert!(node.class_list().contains("active"));
}

#[test]
fn toggle_class_removes_when_present() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.class_list_mut().add("active");
  assert!(!node.class_list_mut().toggle("active"));
  assert!(!node.class_list().contains("active"));
}

// ── child manipulation ──

#[test]
fn append_child_adds_to_end() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("a"));
  node.append_child(HtmlNode::text("b"));
  assert_eq!(node.children().len(), 2);
  assert_eq!(node.text_content(), "ab");
}

#[test]
fn insert_child_at_beginning() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("b"));
  node.insert_child(0, HtmlNode::text("a"));
  assert_eq!(node.text_content(), "ab");
}

#[test]
fn insert_child_clamps_to_length() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.insert_child(100, HtmlNode::text("end"));
  assert_eq!(node.children().len(), 1);
}

#[test]
fn remove_child_returns_removed() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("a"));
  node.append_child(HtmlNode::text("b"));
  let removed = node.remove_child(0).unwrap();
  assert_eq!(removed.text_content(), "a");
  assert_eq!(node.children().len(), 1);
}

#[test]
fn remove_child_out_of_bounds_returns_none() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  assert!(node.remove_child(0).is_none());
}

#[test]
fn replace_child_swaps() {
  let mut node = HtmlNode::new(HtmlElement::Div);
  node.append_child(HtmlNode::text("old"));
  let old = node.replace_child(0, HtmlNode::text("new")).unwrap();
  assert_eq!(old.text_content(), "old");
  assert_eq!(node.text_content(), "new");
}

// ── inner_html ──

#[test]
fn inner_html_serializes_children() {
  let root = parse_root("<html><body><div><span>hi</span></div></body></html>");
  let div = root.query_selector("div").unwrap();
  assert_eq!(div.inner_html(), "<span>hi</span>");
}

#[test]
fn inner_html_serializes_void_elements() {
  let root = parse_root("<html><body><div><br></div></body></html>");
  let div = root.query_selector("div").unwrap();
  assert_eq!(div.inner_html(), "<br>");
}

#[test]
fn inner_html_serializes_id_and_class() {
  let root = parse_root(r#"<html><body><div><span id="x" class="a b">t</span></div></body></html>"#);
  let div = root.query_selector("div").unwrap();
  let html = div.inner_html();
  assert!(html.contains("id=\"x\""));
  assert!(html.contains("class=\"a b\""));
  assert!(html.contains(">t</span>"));
}

// ── set_inner_html (via lui_parse) ──

#[test]
fn set_inner_html_replaces_children() {
  let mut root = parse_root("<html><body><div>old</div></body></html>");
  let div = root.query_selector_mut("div").unwrap();
  lui_parse::set_inner_html(div, "<span>new</span><em>text</em>");
  assert_eq!(div.children().len(), 2);
  assert_eq!(div.children()[0].tag_name(), "span");
  assert_eq!(div.children()[1].tag_name(), "em");
}

#[test]
fn set_inner_html_with_plain_text() {
  let mut root = parse_root("<html><body><div><span>old</span></div></body></html>");
  let div = root.query_selector_mut("div").unwrap();
  lui_parse::set_inner_html(div, "just text");
  assert_eq!(div.text_content(), "just text");
}
