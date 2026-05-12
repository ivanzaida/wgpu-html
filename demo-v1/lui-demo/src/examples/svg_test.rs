pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/svg-test.html");
  let tree = lui::parser::parse(HTML);
  tree
}
