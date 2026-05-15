use lui_cascade::inline::node_inline_style;
use lui_core::CssProperty;
use lui_parse::parse;

#[test]
fn extracts_style_from_node() {
  let doc = parse(r#"<div style="color: red; display: block"></div>"#);
  let div = &doc.root.children()[0];
  let decls = node_inline_style(div);
  assert_eq!(decls.len(), 2);
  assert_eq!(decls[0].property, CssProperty::Color);
  assert_eq!(decls[1].property, CssProperty::Display);
}

#[test]
fn returns_empty_when_no_style_attr() {
  let doc = parse(r#"<div class="foo"></div>"#);
  let div = &doc.root.children()[0];
  let decls = node_inline_style(div);
  assert!(decls.is_empty());
}

#[test]
fn handles_important_in_inline() {
  let doc = parse(r#"<p style="color: red !important"></p>"#);
  let p = &doc.root.children()[0];
  let decls = node_inline_style(p);
  assert_eq!(decls.len(), 1);
  assert!(decls[0].important);
}

#[test]
fn handles_empty_style_attr() {
  let doc = parse(r#"<div style=""></div>"#);
  let div = &doc.root.children()[0];
  let decls = node_inline_style(div);
  assert!(decls.is_empty());
}

#[test]
fn complex_inline_style() {
  let doc = parse(r#"<div style="margin: 10px; padding: 5px; background-color: blue; font-size: 14px"></div>"#);
  let div = &doc.root.children()[0];
  let decls = node_inline_style(div);
  assert_eq!(decls.len(), 4);
}
