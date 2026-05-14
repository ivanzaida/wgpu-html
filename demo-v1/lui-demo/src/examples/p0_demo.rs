pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/p0-demo.html");
  let tree = lui_v1::parser::parse(HTML);
  tree
}
