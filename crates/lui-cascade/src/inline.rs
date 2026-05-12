use lui_css_parser::{Declaration, parse_declaration};
use lui_html_parser::HtmlNode;

/// Extract and parse inline style declarations from a node's `style` attribute.
/// Returns an empty vec if the node has no `style` attribute.
pub fn node_inline_style(node: &HtmlNode) -> Vec<Declaration> {
    match node.attrs.get("style") {
        Some(css) => parse_inline_style(css),
        None => Vec::new(),
    }
}

/// Parse a `style=""` attribute value into declarations.
pub fn parse_inline_style(css: &str) -> Vec<Declaration> {
    let mut decls = Vec::new();
    for part in css.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some((prop, val)) = part.split_once(':') else {
            continue;
        };
        let prop = prop.trim();
        let val = val.trim();
        if prop.is_empty() || val.is_empty() {
            continue;
        }

        let important = val.ends_with("!important");
        let val = if important {
            val.strip_suffix("!important").unwrap().trim()
        } else {
            val
        };

        match parse_declaration(prop, val) {
            Ok((property, value)) => {
                decls.push(Declaration { property, value, important });
            }
            Err(_) => {}
        }
    }
    decls
}
