use super::helpers::*;
use crate::*;

#[test]
fn content_property_parsed() {
  let style = wgpu_html_parser::parse_inline_style(r#"content: "hello""#);
  assert!(style.content.is_some(), "content should be parsed");
}

#[test]
fn cascade_pseudo_element_style_computed() {
  let tree = wgpu_html_parser::parse(
    r#"<style>p::before { content: "X"; color: red; }</style><p>test</p>"#,
  );
  let cascaded = wgpu_html_style::cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  fn find_p(n: &wgpu_html_style::CascadedNode) -> Option<&wgpu_html_style::CascadedNode> {
    if matches!(n.element, wgpu_html_tree::Element::P(_)) {
      return Some(n);
    }
    n.children.iter().find_map(find_p)
  }
  let p = find_p(root).expect("found p");
  assert!(p.before.is_some(), "p::before should have style");
  let pe = p.before.as_ref().unwrap();
  assert_eq!(pe.content_text.as_ref(), "X");
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
