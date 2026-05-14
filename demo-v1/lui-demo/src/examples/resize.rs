pub fn build() -> lui_v1::tree::Tree {
  const HTML: &str = include_str!("../../html/resize.html");
  let tree = lui_v1::parser::parse(HTML);

  tree
}
