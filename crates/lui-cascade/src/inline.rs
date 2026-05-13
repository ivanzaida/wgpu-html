use lui_core::Declaration;
use lui_parse::{parse_declaration, HtmlNode};

/// Return inline style declarations for a node.
/// The parser already parses `style=""` into `node.styles`.
pub fn node_inline_style(node: &HtmlNode) -> Vec<Declaration> {
    node.styles.clone()
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
