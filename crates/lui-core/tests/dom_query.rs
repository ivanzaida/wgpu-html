use lui_core::ArcStr;

fn parse_root(html: &str) -> lui_core::HtmlNode {
  lui_parse::parse(html).root
}

// ── get_element_by_id ──

#[test]
fn get_element_by_id_finds_direct_child() {
  let root = parse_root(r#"<html><body><div id="target">hello</div></body></html>"#);
  let node = root.get_element_by_id(ArcStr::from("target"));
  assert!(node.is_some(), "should find element with id=target");
  assert_eq!(node.unwrap().tag_name(), "div");
}

#[test]
fn get_element_by_id_finds_nested() {
  let root = parse_root(
    r#"<html><body><div><span><p id="deep">text</p></span></div></body></html>"#,
  );
  let node = root.get_element_by_id(ArcStr::from("deep"));
  assert!(node.is_some());
  assert_eq!(node.unwrap().tag_name(), "p");
}

#[test]
fn get_element_by_id_returns_none_when_missing() {
  let root = parse_root(r#"<html><body><div>no id here</div></body></html>"#);
  assert!(root.get_element_by_id(ArcStr::from("absent")).is_none());
}

#[test]
fn get_element_by_id_mut_can_mutate() {
  let mut root = parse_root(r#"<html><body><div id="box"></div></body></html>"#);
  let node = root.get_element_by_id_mut(ArcStr::from("box")).unwrap();
  node.set_attribute("id", "changed");
  assert!(root.get_element_by_id(ArcStr::from("changed")).is_some());
}

// ── get_elements_by_class_name ──

#[test]
fn get_elements_by_class_name_finds_multiple() {
  let root = parse_root(
    r#"<html><body>
      <div class="item">a</div>
      <div class="other">b</div>
      <div class="item">c</div>
    </body></html>"#,
  );
  let items = root.get_elements_by_class_name(ArcStr::from("item"));
  assert_eq!(items.len(), 2, "should find 2 elements with class=item");
}

#[test]
fn get_elements_by_class_name_returns_empty_when_none() {
  let root = parse_root(r#"<html><body><div>plain</div></body></html>"#);
  let items = root.get_elements_by_class_name(ArcStr::from("absent"));
  assert!(items.is_empty());
}

#[test]
fn get_elements_by_class_name_matches_multi_class_elements() {
  let root = parse_root(
    r#"<html><body><div class="foo bar">text</div></body></html>"#,
  );
  assert_eq!(root.get_elements_by_class_name(ArcStr::from("foo")).len(), 1);
  assert_eq!(root.get_elements_by_class_name(ArcStr::from("bar")).len(), 1);
}

// ── get_elements_by_tag_name ──

#[test]
fn get_elements_by_tag_name_finds_all_matching() {
  let root = parse_root(
    r#"<html><body>
      <p>one</p>
      <div><p>two</p></div>
      <p>three</p>
    </body></html>"#,
  );
  let ps = root.get_elements_by_tag_name(ArcStr::from("p"));
  assert_eq!(ps.len(), 3);
}

#[test]
fn get_elements_by_tag_name_returns_empty_for_absent_tag() {
  let root = parse_root(r#"<html><body><div>hi</div></body></html>"#);
  assert!(root.get_elements_by_tag_name(ArcStr::from("span")).is_empty());
}

// ── query_selector ──

#[test]
fn query_selector_by_tag() {
  let root = parse_root(
    r#"<html><body><div><span>hello</span></div></body></html>"#,
  );
  let node = root.query_selector("span");
  assert!(node.is_some());
  assert_eq!(node.unwrap().tag_name(), "span");
}

#[test]
fn query_selector_by_id() {
  let root = parse_root(
    r#"<html><body><div id="main">content</div></body></html>"#,
  );
  let node = root.query_selector("#main");
  assert!(node.is_some());
  assert_eq!(node.unwrap().id(), Some("main"));
}

#[test]
fn query_selector_by_class() {
  let root = parse_root(
    r#"<html><body><div class="highlight">text</div></body></html>"#,
  );
  let node = root.query_selector(".highlight");
  assert!(node.is_some());
}

#[test]
fn query_selector_returns_first_match() {
  let root = parse_root(
    r#"<html><body>
      <p id="first">one</p>
      <p id="second">two</p>
    </body></html>"#,
  );
  let node = root.query_selector("p").unwrap();
  assert_eq!(node.id(), Some("first"));
}

#[test]
fn query_selector_returns_none_when_no_match() {
  let root = parse_root(r#"<html><body><div>hi</div></body></html>"#);
  assert!(root.query_selector("span").is_none());
}

// ── query_selector_all ──

#[test]
fn query_selector_all_by_tag() {
  let root = parse_root(
    r#"<html><body>
      <div>a</div>
      <span>b</span>
      <div>c</div>
    </body></html>"#,
  );
  let divs = root.query_selector_all("div");
  assert_eq!(divs.len(), 2);
}

#[test]
fn query_selector_all_by_class() {
  let root = parse_root(
    r#"<html><body>
      <div class="a">1</div>
      <div class="b">2</div>
      <div class="a">3</div>
    </body></html>"#,
  );
  let nodes = root.query_selector_all(".a");
  assert_eq!(nodes.len(), 2);
}

#[test]
fn query_selector_all_returns_empty_for_no_match() {
  let root = parse_root(r#"<html><body><div>hi</div></body></html>"#);
  assert!(root.query_selector_all(".missing").is_empty());
}

// ── compound selectors ──

#[test]
fn query_selector_compound_tag_and_class() {
  let root = parse_root(
    r#"<html><body>
      <div class="a">1</div>
      <span class="a">2</span>
    </body></html>"#,
  );
  let node = root.query_selector("div.a").unwrap();
  assert_eq!(node.tag_name(), "div");
}

#[test]
fn query_selector_compound_tag_id_class() {
  let root = parse_root(
    r#"<html><body>
      <div id="x" class="y">target</div>
      <div class="y">other</div>
    </body></html>"#,
  );
  let node = root.query_selector("div#x.y").unwrap();
  assert_eq!(node.id(), Some("x"));
}

// ── combinators ──

#[test]
fn query_selector_child_combinator() {
  let root = parse_root(
    r#"<html><body>
      <div><p>direct</p></div>
      <span><div><p>nested</p></div></span>
    </body></html>"#,
  );
  let nodes = root.query_selector_all("div > p");
  assert_eq!(nodes.len(), 2);
}

#[test]
fn query_selector_descendant_combinator() {
  let root = parse_root(
    r#"<html><body>
      <div><span><p id="deep">text</p></span></div>
    </body></html>"#,
  );
  let node = root.query_selector("div p").unwrap();
  assert_eq!(node.id(), Some("deep"));
}

// ── pseudo-classes ──

#[test]
fn query_selector_first_child() {
  let root = parse_root(
    r#"<html><body>
      <p>first</p>
      <p>second</p>
    </body></html>"#,
  );
  let nodes = root.query_selector_all("p:first-child");
  assert_eq!(nodes.len(), 1);
}

#[test]
fn query_selector_not_pseudo() {
  let root = parse_root(
    r#"<html><body>
      <div class="a">1</div>
      <div class="b">2</div>
      <div class="a">3</div>
    </body></html>"#,
  );
  let nodes = root.query_selector_all("div:not(.a)");
  assert_eq!(nodes.len(), 1);
}

// ── attribute selectors ──

#[test]
fn query_selector_attribute_equals() {
  let root = parse_root(
    r#"<html><body>
      <input type="text">
      <input type="hidden">
    </body></html>"#,
  );
  let nodes = root.query_selector_all("input[type='text']");
  assert_eq!(nodes.len(), 1);
}

// ── matches() ──

#[test]
fn matches_returns_true_for_matching_node() {
  let root = parse_root(r#"<html><body><div class="foo">x</div></body></html>"#);
  let div = root.query_selector("div.foo").unwrap();
  assert!(div.matches("div.foo"));
  assert!(div.matches(".foo"));
  assert!(!div.matches(".bar"));
}
