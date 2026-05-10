use super::*;
use crate::tokenizer::tokenize;

#[test]
fn test_simple_tree() {
  let tree = build(tokenize("<div><p>hello</p></div>"));
  let div = tree.root.as_ref().expect("root");
  assert!(matches!(div.element, Element::Div(_)));
  assert_eq!(div.children.len(), 1);
  let p = &div.children[0];
  assert!(matches!(p.element, Element::P(_)));
  assert_eq!(p.children.len(), 1);
  assert!(matches!(p.children[0].element, Element::Text(_)));
}

#[test]
fn test_void_elements() {
  let tree = build(tokenize("<div><br><hr><img></div>"));
  let div = tree.root.as_ref().expect("root");
  assert_eq!(div.children.len(), 3);
}

#[test]
fn test_auto_close_p() {
  // Two sibling <p> with implicit auto-close → wrapped in synthetic body.
  let tree = build(tokenize("<p>one<p>two"));
  let body = tree.root.as_ref().expect("root");
  assert!(matches!(body.element, Element::Body(_)));
  assert_eq!(body.children.len(), 2);
}

#[test]
fn test_unknown_tag_dropped() {
  let tree = build(tokenize("<div><frobnicate>x</frobnicate><p>y</p></div>"));
  let div = tree.root.as_ref().expect("root");
  // Unknown <frobnicate> + its text are gone; only <p> remains.
  assert_eq!(div.children.len(), 1);
  assert!(matches!(div.children[0].element, Element::P(_)));
}

#[test]
fn test_template_contents_are_retained() {
  let tree = build(tokenize(
    "<template id=\"tpl\"><div>hidden</div></template><p>shown</p>",
  ));
  let body = tree.root.as_ref().expect("root");
  assert!(matches!(body.element, Element::Body(_)));
  assert_eq!(body.children.len(), 2);
  let template = &body.children[0];
  assert!(matches!(template.element, Element::Template(_)));
  assert_eq!(template.children.len(), 1);
  assert!(matches!(template.children[0].element, Element::Div(_)));
  assert!(matches!(body.children[1].element, Element::P(_)));
}

#[test]
fn test_comments_and_doctype_dropped() {
  let tree = build(tokenize("<!DOCTYPE html><!--c--><p>hi</p>"));
  // Doctype + comment dropped → only <p> at top level → it becomes the root,
  // no synthetic body wrapper.
  let p = tree.root.as_ref().expect("root");
  assert!(matches!(p.element, Element::P(_)));
  assert_eq!(p.children.len(), 1);
  assert!(matches!(p.children[0].element, Element::Text(_)));
}

#[test]
fn second_body_is_ignored() {
  // HTML spec: a second <body> tag is dropped; content goes
  // into the first <body>.
  let tree = build(tokenize("<body><p>a</p></body><body><p>b</p></body>"));
  let body = tree.root.as_ref().expect("root");
  assert!(matches!(body.element, Element::Body(_)));
  // Both <p> elements end up in the single <body>.
  assert_eq!(body.children.len(), 2);
}

#[test]
fn style_plus_body_adopts_into_body() {
  // A <style> sibling at top-level should be adopted into the
  // existing <body>, not wrapped in a second synthetic <body>.
  let tree = build(tokenize("<style>h1{color:red}</style><body><p>hi</p></body>"));
  let body = tree.root.as_ref().expect("root");
  assert!(matches!(body.element, Element::Body(_)));
  // <style> is adopted before <body>'s own children.
  assert!(matches!(body.children[0].element, Element::StyleElement(_)));
  assert!(matches!(body.children[1].element, Element::P(_)));
}

#[test]
fn second_html_is_ignored() {
  let tree = build(tokenize("<html><body><p>ok</p></body></html><html></html>"));
  let html = tree.root.as_ref().expect("root");
  assert!(matches!(html.element, Element::Html(_)));
}
