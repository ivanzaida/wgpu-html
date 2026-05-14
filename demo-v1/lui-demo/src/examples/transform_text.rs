pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/transform-text.html");
  lui_v1::parser::parse(HTML)
}
