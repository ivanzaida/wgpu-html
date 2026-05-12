pub fn build() -> lui::tree::Tree {
  const HTML: &str = include_str!("../../html/resize.html");
  let tree = lui::parser::parse(HTML);

  tree
}
