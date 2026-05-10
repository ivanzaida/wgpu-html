use lui_models as m;

use super::*;

fn div(id: Option<&str>, class: Option<&str>) -> m::Div {
  m::Div {
    id: id.map(str::to_owned),
    class: class.map(str::to_owned),
    ..m::Div::default()
  }
}

fn span(id: Option<&str>, class: Option<&str>) -> m::Span {
  m::Span {
    id: id.map(str::to_owned),
    class: class.map(str::to_owned),
    ..m::Span::default()
  }
}

fn sample() -> Tree {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(div(Some("outer"), Some("box hero"))).with_children(vec![
      Node::new(span(None, Some("label"))).with_children(vec![Node::new("hi")]),
      Node::new(div(None, Some("box"))).with_children(vec![
        Node::new(span(None, Some("label"))).with_children(vec![Node::new("two")]),
      ]),
    ]),
    Node::new(span(Some("solo"), Some("label primary"))),
  ]);
  Tree::new(body)
}

#[test]
fn parse_compound_keeps_old_grammar() {
  let s = CompoundSelector::parse("div.box#outer.hero").unwrap();
  assert_eq!(s.tag.as_deref(), Some("div"));
  assert_eq!(s.id.as_deref(), Some("outer"));
  assert_eq!(s.classes, vec!["box".to_string(), "hero".to_string()]);

  let s = CompoundSelector::parse("*.label").unwrap();
  assert!(s.tag.is_none());
  assert_eq!(s.classes, vec!["label".to_string()]);

  assert!(CompoundSelector::parse("div span").is_err());
  assert!(CompoundSelector::parse("a, b").is_err());
  assert!(CompoundSelector::parse("a > b").is_err());
}

#[test]
fn parse_attribute_operators() {
  let list = SelectorList::parse("[a][b=v][c~=v][d|=en][e^=p][f$=q][g*=r]").unwrap();
  let cs = &list.selectors[0].compounds[0];
  let ops: Vec<_> = cs.attrs.iter().map(|f| f.op).collect();
  assert_eq!(
    ops,
    vec![
      AttrOp::Exists,
      AttrOp::Equals,
      AttrOp::Includes,
      AttrOp::DashMatch,
      AttrOp::Prefix,
      AttrOp::Suffix,
      AttrOp::Substring,
    ]
  );
}

#[test]
fn parse_attribute_case_flags() {
  let list = SelectorList::parse("[type=PASSWORD i]").unwrap();
  let f = &list.selectors[0].compounds[0].attrs[0];
  assert_eq!(f.value, "PASSWORD");
  assert!(f.case_insensitive);

  let list = SelectorList::parse("[type=password s]").unwrap();
  let f = &list.selectors[0].compounds[0].attrs[0];
  assert!(!f.case_insensitive);
}

#[test]
fn attribute_op_includes_matches_class_token() {
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(div(None, Some("foo bar baz"))),
    Node::new(div(None, Some("foobar"))),
  ]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("[class~=\"bar\"]");
  assert_eq!(hits, vec![vec![0]]);
}

#[test]
fn attribute_op_dashmatch_for_lang() {
  let mut e = m::Div::default();
  e.lang = Some("en-US".to_owned());
  let mut e2 = m::Div::default();
  e2.lang = Some("english".to_owned());
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(e), Node::new(e2)]);
  let mut tree = Tree::new(body);
  let hits = tree.query_selector_all_paths("[lang|=en]");
  assert_eq!(hits, vec![vec![0]]);
}

#[test]
fn attribute_op_prefix_suffix_substring() {
  let mut a1 = m::A::default();
  a1.href = Some("https://example.com/path".to_owned());
  let mut a2 = m::A::default();
  a2.href = Some("/local".to_owned());
  let mut a3 = m::A::default();
  a3.href = Some("https://example.com/file.pdf".to_owned());
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(a1), Node::new(a2), Node::new(a3)]);
  let mut tree = Tree::new(body);

  assert_eq!(
    tree.query_selector_all_paths("a[href^=\"https://\"]"),
    vec![vec![0], vec![2]]
  );
  assert_eq!(tree.query_selector_all_paths("a[href$=\".pdf\"]"), vec![vec![2]]);
  assert_eq!(
    tree.query_selector_all_paths("a[href*=\"example\"]"),
    vec![vec![0], vec![2]]
  );
}

#[test]
fn attribute_case_insensitive_flag() {
  use m::common::html_enums::InputType;
  let body = Node::new(m::Body::default()).with_children(vec![Node::new(m::Input {
    id: Some("pw".to_owned()),
    r#type: Some(InputType::Password),
    ..m::Input::default()
  })]);
  let mut tree = Tree::new(body);
  assert!(tree.query_selector("input[type=PASSWORD]").is_none());
  assert!(tree.query_selector("input[type=PASSWORD i]").is_some());
}

#[test]
fn descendant_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("body div span");
  assert_eq!(hits.len(), 2);
}

#[test]
fn child_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("div > span");
  assert_eq!(hits.len(), 2);
}

#[test]
fn next_sibling_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("span + div");
  assert_eq!(hits.len(), 1);
}

#[test]
fn subsequent_sibling_combinator() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("span ~ div");
  assert_eq!(hits.len(), 1);
}

#[test]
fn selector_list_unions_matches() {
  let mut tree = sample();
  let hits = tree.query_selector_all_paths("#outer, #solo");
  assert_eq!(hits.len(), 2);

  let hits = tree.query_selector_all_paths("div, [class~=primary]");
  assert_eq!(hits.len(), 3);
}

#[test]
fn query_selector_by_id() {
  let mut tree = sample();
  let n = tree.query_selector("#outer").unwrap();
  assert_eq!(n.element.tag_name(), "div");
  assert_eq!(n.element.id(), Some("outer"));
  assert!(tree.query_selector("#missing").is_none());
}

#[test]
fn query_selector_by_tag() {
  let mut tree = sample();
  let first = tree.query_selector("div").unwrap();
  assert_eq!(first.element.id(), Some("outer"));
}

#[test]
fn query_selector_by_class_compound() {
  let mut tree = sample();
  let n = tree.query_selector("span.primary").unwrap();
  assert_eq!(n.element.id(), Some("solo"));
  assert!(tree.query_selector("span.box").is_none());
}

#[test]
fn universal_selector_includes_root_self() {
  let mut tree = sample();
  let all = tree.query_selector_all("*");
  assert_eq!(all.len(), 6);
}

#[test]
fn empty_tree_is_safe() {
  let mut tree = Tree::default();
  assert!(tree.query_selector("div").is_none());
  assert!(tree.query_selector_all("div").is_empty());
  assert!(tree.query_selector_all("a, b, c").is_empty());
}

#[test]
fn pre_parsed_selector_reuses_across_calls() {
  let sel = SelectorList::parse(".label, #solo").unwrap();
  let mut tree = sample();
  assert_eq!(tree.query_selector_all(&sel).len(), 3);
  assert_eq!(tree.query_selector_all(sel).len(), 3);
}

#[test]
fn compound_selector_into_list() {
  let cs = CompoundSelector::parse("span.label").unwrap();
  let mut tree = sample();
  assert_eq!(tree.query_selector_all(cs).len(), 3);
}

#[test]
fn input_type_password_user_case() {
  use m::common::html_enums::InputType;
  let body = Node::new(m::Body::default()).with_children(vec![
    Node::new(m::Input {
      id: Some("user".to_owned()),
      r#type: Some(InputType::Text),
      ..m::Input::default()
    }),
    Node::new(m::Input {
      id: Some("pass".to_owned()),
      r#type: Some(InputType::Password),
      ..m::Input::default()
    }),
  ]);
  let mut tree = Tree::new(body);
  let hit = tree.query_selector("input[type=\"password\"]").unwrap();
  assert_eq!(hit.element.id(), Some("pass"));
}
