use lui_ui::{El, el};

pub fn lucide(icon: &str) -> El {
  el::span().style("font-family: lucide").text(icon)
}
