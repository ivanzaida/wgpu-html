use lui_parser::{parse, parse_inline_style};
use lui_tree::{Element, Node};

#[track_caller]
fn root(html: &str) -> Node {
  parse(html).root.expect("expected a root node")
}

#[track_caller]
fn text_of(node: &Node) -> &str {
  match &node.element {
    Element::Text(s) => s,
    other => panic!("expected Element::Text, got {other:?}"),
  }
}

#[test]
fn empty_input_has_no_root() {
  let tree = parse("");
  assert!(tree.root.is_none());
}

#[test]
fn whitespace_only_input_has_no_root() {
  let tree = parse("   \n\t  ");
  assert!(tree.root.is_none());
}

#[test]
fn pure_text_input_becomes_text_root() {
  let r = root("hello");
  assert_eq!(text_of(&r), "hello");
  assert!(r.children.is_empty());
}

#[test]
fn doctype_only_has_no_root() {
  let tree = parse("<!DOCTYPE html>");
  assert!(tree.root.is_none());
}

#[test]
fn comment_only_has_no_root() {
  let tree = parse("<!-- nothing here -->");
  assert!(tree.root.is_none());
}

#[test]
fn paragraph_with_text() {
  let r = root("<p>hello</p>");
  assert!(matches!(r.element, Element::P(_)));
  assert_eq!(r.children.len(), 1);
  assert_eq!(text_of(&r.children[0]), "hello");
}

#[test]
fn span_with_text() {
  let r = root("<span>x</span>");
  assert!(matches!(r.element, Element::Span(_)));
  assert_eq!(r.children.len(), 1);
}

#[test]
fn div_p_text_nesting() {
  let r = root("<div><p>hi</p></div>");
  assert!(matches!(r.element, Element::Div(_)));
  let p = &r.children[0];
  assert!(matches!(p.element, Element::P(_)));
  assert_eq!(text_of(&p.children[0]), "hi");
}

#[test]
fn three_level_nesting() {
  let r = root("<section><article><p>x</p></article></section>");
  let a = &r.children[0];
  let p = &a.children[0];
  assert!(matches!(r.element, Element::Section(_)));
  assert!(matches!(a.element, Element::Article(_)));
  assert!(matches!(p.element, Element::P(_)));
}

#[test]
fn mixed_inline_in_paragraph() {
  let r = root("<p>hello <strong>bold</strong> and <em>italic</em>!</p>");
  assert!(matches!(r.element, Element::P(_)));
  assert_eq!(r.children.len(), 5);
  assert_eq!(text_of(&r.children[0]), "hello ");
  assert!(matches!(r.children[1].element, Element::Strong(_)));
  assert_eq!(text_of(&r.children[2]), " and ");
  assert!(matches!(r.children[3].element, Element::Em(_)));
  assert_eq!(text_of(&r.children[4]), "!");
}

#[test]
fn deep_inline_nesting() {
  let r = root("<p>a<b>b<i>i<u>u</u></i></b></p>");
  assert!(matches!(r.element, Element::P(_)));
  let b = &r.children[1];
  let i = &b.children[1];
  let u = &i.children[1];
  assert!(matches!(b.element, Element::B(_)));
  assert!(matches!(i.element, Element::I(_)));
  assert!(matches!(u.element, Element::U(_)));
  assert_eq!(text_of(&u.children[0]), "u");
}

#[test]
fn full_document_skeleton() {
  let html = "<!DOCTYPE html>\
        <html>\
            <head><title>T</title></head>\
            <body><h1>Hi</h1><p>p</p></body>\
        </html>";
  let r = root(html);
  assert!(matches!(r.element, Element::Html(_)));
  assert_eq!(r.children.len(), 2);
  let head = &r.children[0];
  let body = &r.children[1];
  assert!(matches!(head.element, Element::Head(_)));
  assert!(matches!(body.element, Element::Body(_)));
  let title = &head.children[0];
  assert!(matches!(title.element, Element::Title(_)));
  assert_eq!(text_of(&title.children[0]), "T");
  assert!(matches!(body.children[0].element, Element::H1(_)));
  assert!(matches!(body.children[1].element, Element::P(_)));
}

#[test]
fn multiple_top_level_get_synthetic_body() {
  let r = root("<p>one</p><p>two</p>");
  assert!(matches!(r.element, Element::Body(_)));
  assert_eq!(r.children.len(), 2);
  assert!(matches!(r.children[0].element, Element::P(_)));
  assert!(matches!(r.children[1].element, Element::P(_)));
}

#[test]
fn single_top_level_does_not_get_synthetic_body() {
  let r = root("<p>only</p>");
  assert!(matches!(r.element, Element::P(_)));
}

#[test]
fn div_has_id_class() {
  let r = root(r#"<div id="hero" class="card big"></div>"#);
  let Element::Div(div) = &r.element else {
    panic!("expected Div")
  };
  assert_eq!(div.id.as_deref(), Some("hero"));
  assert_eq!(r.has_class("card"), true);
  assert_eq!(r.has_class("big"), true);
}

#[test]
fn p_has_inline_style_string() {
  let r = root(r#"<p style="color: red; padding: 8px;">x</p>"#);
  let Element::P(p) = &r.element else {
    panic!("expected P")
  };
  assert_eq!(p.style.as_deref(), Some("color: red; padding: 8px;"));
}

#[test]
fn anchor_href() {
  let r = root(r#"<a href="https://example.com">link</a>"#);
  let Element::A(a) = &r.element else {
    panic!("expected A")
  };
  assert_eq!(a.href.as_deref(), Some("https://example.com"));
}

#[test]
fn img_src_alt() {
  let r = root(r#"<img src="/x.png" alt="x">"#);
  let Element::Img(img) = &r.element else {
    panic!("expected Img")
  };
  assert_eq!(img.src.as_deref(), Some("/x.png"));
  assert_eq!(img.alt.as_deref(), Some("x"));
}

#[test]
fn input_type_value() {
  let r = root(r#"<input type="email" value="a@b.com">"#);
  let Element::Input(inp) = &r.element else {
    panic!("expected Input")
  };
  assert_eq!(inp.value.as_deref(), Some("a@b.com"));
  assert!(inp.r#type.is_some());
}

#[test]
fn unordered_list_with_items() {
  let r = root("<ul><li>a</li><li>b</li><li>c</li></ul>");
  assert!(matches!(r.element, Element::Ul(_)));
  assert_eq!(r.children.len(), 3);
  for li in &r.children {
    assert!(matches!(li.element, Element::Li(_)));
  }
}

#[test]
fn definition_list() {
  let r = root("<dl><dt>k</dt><dd>v</dd></dl>");
  assert!(matches!(r.element, Element::Dl(_)));
  assert!(matches!(r.children[0].element, Element::Dt(_)));
  assert!(matches!(r.children[1].element, Element::Dd(_)));
}

#[test]
fn table_with_thead_tbody() {
  let r = root(
    "<table>\
           <thead><tr><th>h1</th><th>h2</th></tr></thead>\
           <tbody><tr><td>a</td><td>b</td></tr></tbody>\
         </table>",
  );
  assert!(matches!(r.element, Element::Table(_)));
  let thead = &r.children[0];
  let tbody = &r.children[1];
  assert!(matches!(thead.element, Element::Thead(_)));
  assert!(matches!(tbody.element, Element::Tbody(_)));
  let head_row = &thead.children[0];
  assert!(matches!(head_row.element, Element::Tr(_)));
  assert_eq!(head_row.children.len(), 2);
  assert!(matches!(head_row.children[0].element, Element::Th(_)));
  let body_row = &tbody.children[0];
  assert_eq!(body_row.children.len(), 2);
  assert!(matches!(body_row.children[0].element, Element::Td(_)));
}

#[test]
fn form_with_inputs_and_button() {
  let r = root(
    r#"<form>
            <label for="name">Name</label>
            <input type="text" name="name">
            <button type="submit">Send</button>
        </form>"#,
  );
  assert!(matches!(r.element, Element::Form(_)));
  assert!(r.children.iter().any(|c| matches!(c.element, Element::Label(_))));
  assert!(r.children.iter().any(|c| matches!(c.element, Element::Input(_))));
  assert!(r.children.iter().any(|c| matches!(c.element, Element::Button(_))));
}

#[test]
fn select_with_options() {
  let r = root("<select><option>a</option><option>b</option><optgroup></optgroup></select>");
  assert!(matches!(r.element, Element::Select(_)));
  assert!(matches!(r.children[0].element, Element::OptionElement(_)));
  assert!(matches!(r.children[1].element, Element::OptionElement(_)));
  assert!(matches!(r.children[2].element, Element::Optgroup(_)));
}

#[test]
fn p_auto_closes_when_div_opens() {
  let r = root("<p>a<div>b</div>");
  assert!(matches!(r.element, Element::Body(_)));
  assert_eq!(r.children.len(), 2);
  assert!(matches!(r.children[0].element, Element::P(_)));
  assert!(matches!(r.children[1].element, Element::Div(_)));
}

#[test]
fn li_auto_closes_when_next_li_opens() {
  let r = root("<ul><li>a<li>b<li>c</ul>");
  assert!(matches!(r.element, Element::Ul(_)));
  assert_eq!(r.children.len(), 3);
}

#[test]
fn dt_auto_closes_dd() {
  let r = root("<dl><dt>k1<dd>v1<dt>k2<dd>v2</dl>");
  assert!(matches!(r.element, Element::Dl(_)));
  let kinds: Vec<&Element> = r.children.iter().map(|c| &c.element).collect();
  assert_eq!(kinds.len(), 4);
  assert!(matches!(kinds[0], Element::Dt(_)));
  assert!(matches!(kinds[1], Element::Dd(_)));
  assert!(matches!(kinds[2], Element::Dt(_)));
  assert!(matches!(kinds[3], Element::Dd(_)));
}

#[test]
fn tr_auto_closes_tr() {
  let r = root("<table><tbody><tr><td>a</td><tr><td>b</td></tbody></table>");
  let tbody = &r.children[0];
  assert_eq!(tbody.children.len(), 2);
  for tr in &tbody.children {
    assert!(matches!(tr.element, Element::Tr(_)));
  }
}

#[test]
fn void_elements_are_leaves() {
  let r = root("<div><br><hr><img></div>");
  assert_eq!(r.children.len(), 3);
  assert!(matches!(r.children[0].element, Element::Br(_)));
  assert!(matches!(r.children[1].element, Element::Hr(_)));
  assert!(matches!(r.children[2].element, Element::Img(_)));
  for c in &r.children {
    assert!(c.children.is_empty());
  }
}

#[test]
fn self_closing_syntax_works_for_known_tags() {
  let r = root("<div><br /><hr /></div>");
  assert_eq!(r.children.len(), 2);
}

#[test]
fn unclosed_tags_at_eof_are_closed_implicitly() {
  let r = root("<div><span>hello");
  assert!(matches!(r.element, Element::Div(_)));
  let span = &r.children[0];
  assert!(matches!(span.element, Element::Span(_)));
  assert_eq!(text_of(&span.children[0]), "hello");
}

#[test]
fn unknown_tag_subtree_is_dropped() {
  let r = root("<div>before<frob>nested<p>inside</p></frob>after</div>");
  let kinds: Vec<&Element> = r.children.iter().map(|c| &c.element).collect();
  assert!(matches!(r.element, Element::Div(_)));
  assert_eq!(kinds.len(), 2);
  assert!(matches!(kinds[0], Element::Text(_)));
  assert!(matches!(kinds[1], Element::Text(_)));
}

#[test]
fn doctype_and_comments_are_stripped() {
  let r = root("<!DOCTYPE html><!--top--><div><!--inside-->ok<!--end--></div>");
  assert!(matches!(r.element, Element::Div(_)));
  assert_eq!(r.children.len(), 1);
  assert_eq!(text_of(&r.children[0]), "ok");
}

#[test]
fn inline_style_color_padding() {
  let style = parse_inline_style("color: red; padding: 8px;");
  assert!(style.color.is_some(), "color should be set");
  assert!(style.padding.is_some(), "padding should be set");
}

#[test]
fn inline_style_ignores_unknown_props() {
  let _ = parse_inline_style("nonsense: 123; color: blue;");
}

#[test]
fn multiple_data_attrs_are_all_preserved() {
  let r = root(r#"<div data-id="42" data-name="hello" data-foo="bar"></div>"#);
  assert_eq!(r.data_attrs.len(), 3);
  assert_eq!(r.data_attr("id").map(|s| s.as_ref()), Some("42"));
  assert_eq!(r.data_attr("name").map(|s| s.as_ref()), Some("hello"));
  assert_eq!(r.data_attr("foo").map(|s| s.as_ref()), Some("bar"));
  assert!(r.aria_attrs.is_empty());
}

#[test]
fn multiple_aria_attrs_are_all_preserved() {
  let r = root(r#"<button aria-label="Close" aria-pressed="false" aria-controls="menu"></button>"#);
  assert_eq!(r.aria_attrs.len(), 3);
  assert_eq!(r.aria_attr("label").map(|s| s.as_ref()), Some("Close"));
  assert_eq!(r.aria_attr("pressed").map(|s| s.as_ref()), Some("false"));
  assert_eq!(r.aria_attr("controls").map(|s| s.as_ref()), Some("menu"));
}
