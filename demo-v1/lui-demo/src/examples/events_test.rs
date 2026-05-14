pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/events-test.html");
  let tree = lui_v1::parser::parse(HTML);
  tree
}
