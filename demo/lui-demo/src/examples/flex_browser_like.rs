pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/flex-browser-like.html");
  let tree = lui::parser::parse(HTML);
  tree
}
