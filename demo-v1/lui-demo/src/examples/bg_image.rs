pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/bg-image.html");
  let tree = lui::parser::parse(HTML);
  tree
}
