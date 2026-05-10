use super::helpers::*;
use crate::*;

#[test]
fn content_property_parsed() {
  let style = lui_parser::parse_inline_style(r#"content: "hello""#);
  assert!(style.content.is_some(), "content should be parsed");
}

#[test]
fn cascade_pseudo_element_style_computed() {
  let tree = lui_parser::parse(
    r#"<style>p::before { content: "X"; color: red; }</style><p>test</p>"#,
  );
  let cascaded = lui_style::cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  fn find_p(n: &lui_style::CascadedNode) -> Option<&lui_style::CascadedNode> {
    if matches!(n.element, lui_tree::Element::P(_)) {
      return Some(n);
    }
    n.children.iter().find_map(find_p)
  }
  let p = find_p(root).expect("found p");
  assert!(p.before.is_some(), "p::before should have style");
  let pe = p.before.as_ref().unwrap();
  assert_eq!(pe.content_text.as_ref(), "X");
}

// ---------------------------------------------------------------------------
// ::marker
// ---------------------------------------------------------------------------

#[test]
fn ul_list_items_get_bullet_markers() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <ul><li>one</li><li>two</li></ul>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains("\u{2022}"), "should contain bullet marker, got: {text}");
  assert!(text.contains("one"), "should contain list text, got: {text}");
}

#[test]
fn ol_list_items_get_numbered_markers() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <ol><li>first</li><li>second</li><li>third</li></ol>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains("1."), "should contain number 1, got: {text}");
  assert!(text.contains("2."), "should contain number 2, got: {text}");
  assert!(text.contains("3."), "should contain number 3, got: {text}");
}

#[test]
fn ol_start_attribute_offsets_numbering() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <ol start="5"><li>a</li><li>b</li></ol>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains("5."), "should start at 5, got: {text}");
  assert!(text.contains("6."), "should have 6, got: {text}");
}

#[test]
fn list_style_none_hides_markers() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <ul style="list-style-type: none;"><li>item</li></ul>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(!text.contains("\u{2022}"), "should not contain bullet, got: {text}");
  assert!(text.contains("item"), "should contain text, got: {text}");
}

#[test]
fn cascade_marker_computed_for_list_item() {
  let tree = lui_parser::parse(
    r#"<ul><li>test</li></ul>"#,
  );
  let cascaded = lui_style::cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  fn find_li(n: &lui_style::CascadedNode) -> Option<&lui_style::CascadedNode> {
    if matches!(n.element, lui_tree::Element::Li(_)) {
      return Some(n);
    }
    n.children.iter().find_map(find_li)
  }
  let li = find_li(root).expect("found li");
  assert!(li.marker.is_some(), "li should have marker pseudo-element");
  let marker = li.marker.as_ref().unwrap();
  assert!(marker.content_text.contains("\u{2022}"), "marker should be bullet, got: {}", marker.content_text);
}

fn find_text_content(b: &LayoutBox) -> String {
  let mut out = String::new();
  if let Some(ref run) = b.text_run {
    out.push_str(&run.text);
  }
  for child in &b.children {
    out.push_str(&find_text_content(child));
  }
  out
}

#[test]
fn before_pseudo_element_renders_content() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <style>p::before { content: ">> "; }</style>
      <p>hello</p>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains(">>"), "should contain before content, got: {text}");
  assert!(text.contains("hello"), "should contain original text, got: {text}");
}

#[test]
fn after_pseudo_element_renders_content() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <style>p::after { content: " <<"; }</style>
      <p>hello</p>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains("<<"), "should contain after content, got: {text}");
}

#[test]
fn before_and_after_both_render() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <style>.tag::before { content: "["; } .tag::after { content: "]"; }</style>
      <span class="tag">hello</span>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains("["), "should contain before content, got: {text}");
  assert!(text.contains("]"), "should contain after content, got: {text}");
  assert!(text.contains("hello"), "should contain original text, got: {text}");
}

#[test]
fn pseudo_element_content_none_no_extra_text() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <style>p::before { content: none; }</style>
      <p>hello</p>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert_eq!(text.trim(), "hello");
}

#[test]
fn pseudo_element_without_content_no_extra_text() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <style>p::before { color: red; }</style>
      <p>hello</p>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert_eq!(text.trim(), "hello");
}

#[test]
fn pseudo_element_block_display_creates_children() {
  let root = layout_with_fonts(
    r#"<body style="margin: 0; font-family: sans-serif;">
      <style>div::before { content: "top"; display: block; } div::after { content: "bottom"; display: block; }</style>
      <div style="width: 200px;">middle</div>
    </body>"#,
    800.0,
    600.0,
  );
  let text = find_text_content(&root);
  assert!(text.contains("top"), "should contain before content, got: {text}");
  assert!(text.contains("middle"), "should contain original text, got: {text}");
  assert!(text.contains("bottom"), "should contain after content, got: {text}");
}
