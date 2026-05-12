pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/box-shadow-demo.html");
  lui::parser::parse(HTML)
}
