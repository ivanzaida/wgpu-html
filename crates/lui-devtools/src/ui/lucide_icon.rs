use lui_ui::{el, El};

pub fn lucide(icon: &str) -> El {
  el::span().style("font-family: lucide").text(icon)
}
