use wgpu_html_tree::{Element, Node, Tree};

use crate::attr_parser;
use crate::tokenizer::Token;

/// HTML void elements that cannot have children and do not need a closing tag.
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

fn is_void_element(tag: &str) -> bool {
    VOID_ELEMENTS.contains(&tag)
}

/// Build a tree from a list of tokens.
///
/// - Comments and doctypes are dropped.
/// - Unknown tags (and their entire subtree) are dropped.
/// - Whitespace-only text between tags is dropped.
/// - If the parsed tokens yield exactly one top-level node it becomes the
///   tree root; otherwise the children are wrapped in a synthetic `<body>`.
/// - If the tokens yield no nodes at all, the result has `root = None`.
pub fn build(tokens: Vec<Token>) -> Tree {
    let mut builder = TreeBuilder::new(tokens);
    builder.run();

    let mut roots = builder.document;
    let root = match roots.len() {
        0 => None,
        1 => Some(roots.pop().unwrap()),
        _ => Some(Node::new(Element::Body(wgpu_html_models::Body::default())).with_children(roots)),
    };
    Tree {
        root,
        ..Tree::default()
    }
}

struct TreeBuilder {
    tokens: Vec<Token>,
    pos: usize,
    /// Stack of open elements. `Option<Element>` is `None` for an unknown
    /// tag — its subtree is parsed but discarded on close.
    stack: Vec<(String, Option<Element>, Vec<Node>)>,
    document: Vec<Node>,
}

impl TreeBuilder {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            stack: Vec::new(),
            document: Vec::new(),
        }
    }

    fn run(&mut self) {
        while self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;

            match token {
                // Comments and doctypes are dropped.
                Token::Doctype(_) | Token::Comment(_) => {}
                Token::Text(text) => {
                    if !text.trim().is_empty() {
                        self.push_node(Node::new(Element::Text(text)));
                    }
                }
                Token::OpenTag {
                    name,
                    attrs,
                    self_closing,
                } => {
                    let element = attr_parser::parse_element(&name, &attrs);

                    if self_closing || is_void_element(&name) {
                        if let Some(el) = element {
                            self.push_node(Node::new(el));
                        }
                        // Unknown void → silently dropped.
                    } else {
                        // Auto-close certain elements before opening a new one.
                        self.auto_close_before(&name);
                        self.stack.push((name, element, Vec::new()));
                    }
                }
                Token::CloseTag(name) => self.close_tag(&name),
            }
        }

        // Close any remaining open elements.
        while !self.stack.is_empty() {
            self.pop_element();
        }
    }

    /// Push a node into the current open element's children, or into the document root.
    fn push_node(&mut self, node: Node) {
        if let Some(top) = self.stack.last_mut() {
            top.2.push(node);
        } else {
            self.document.push(node);
        }
    }

    /// Pop the top element from the stack and add it as a child to its parent.
    /// If the popped element is `None` (unknown tag), the subtree is discarded.
    fn pop_element(&mut self) {
        let Some((_tag_name, element, children)) = self.stack.pop() else {
            return;
        };
        if let Some(el) = element {
            self.push_node(Node::new(el).with_children(children));
        }
        // else: drop unknown subtree silently
    }

    /// Close a tag by name, popping elements from the stack.
    fn close_tag(&mut self, name: &str) {
        // Find the matching open tag in the stack (innermost match).
        let mut found = None;
        for (i, entry) in self.stack.iter().enumerate().rev() {
            if entry.0 == name {
                found = Some(i);
                break;
            }
        }

        if let Some(idx) = found {
            let count = self.stack.len() - idx;
            for _ in 0..count {
                self.pop_element();
            }
        }
        // If not found, silently ignore the orphan closing tag.
    }

    /// Auto-close certain elements based on HTML nesting rules
    /// (e.g. `<p>` is implicitly closed when another block element opens).
    fn auto_close_before(&mut self, opening_tag: &str) {
        loop {
            let should_close = if let Some(top) = self.stack.last() {
                should_auto_close(&top.0, opening_tag)
            } else {
                false
            };
            if should_close {
                self.pop_element();
            } else {
                break;
            }
        }
    }
}

/// Determines if the current open element should be auto-closed when
/// a new tag is being opened.
fn should_auto_close(current: &str, opening: &str) -> bool {
    match current {
        "p" => matches!(
            opening,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "details"
                | "div"
                | "dl"
                | "fieldset"
                | "figcaption"
                | "figure"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hgroup"
                | "hr"
                | "main"
                | "menu"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "ul"
        ),
        "li" => opening == "li",
        "dt" => matches!(opening, "dt" | "dd"),
        "dd" => matches!(opening, "dt" | "dd"),
        "thead" => matches!(opening, "tbody" | "tfoot"),
        "tbody" => matches!(opening, "tbody" | "tfoot"),
        "tr" => opening == "tr",
        "th" => matches!(opening, "td" | "th" | "tr"),
        "td" => matches!(opening, "td" | "th" | "tr"),
        "option" => matches!(opening, "option" | "optgroup"),
        "optgroup" => opening == "optgroup",
        "rt" => matches!(opening, "rt" | "rp"),
        "rp" => matches!(opening, "rt" | "rp"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    fn test_simple_tree() {
        let tree = build(tokenize("<div><p>hello</p></div>"));
        let div = tree.root.as_ref().expect("root");
        assert!(matches!(div.element, Element::Div(_)));
        assert_eq!(div.children.len(), 1);
        let p = &div.children[0];
        assert!(matches!(p.element, Element::P(_)));
        assert_eq!(p.children.len(), 1);
        assert!(matches!(p.children[0].element, Element::Text(_)));
    }

    #[test]
    fn test_void_elements() {
        let tree = build(tokenize("<div><br><hr><img></div>"));
        let div = tree.root.as_ref().expect("root");
        assert_eq!(div.children.len(), 3);
    }

    #[test]
    fn test_auto_close_p() {
        // Two sibling <p> with implicit auto-close → wrapped in synthetic body.
        let tree = build(tokenize("<p>one<p>two"));
        let body = tree.root.as_ref().expect("root");
        assert!(matches!(body.element, Element::Body(_)));
        assert_eq!(body.children.len(), 2);
    }

    #[test]
    fn test_unknown_tag_dropped() {
        let tree = build(tokenize("<div><frobnicate>x</frobnicate><p>y</p></div>"));
        let div = tree.root.as_ref().expect("root");
        // Unknown <frobnicate> + its text are gone; only <p> remains.
        assert_eq!(div.children.len(), 1);
        assert!(matches!(div.children[0].element, Element::P(_)));
    }

    #[test]
    fn test_template_contents_are_retained() {
        let tree = build(tokenize("<template id=\"tpl\"><div>hidden</div></template><p>shown</p>"));
        let body = tree.root.as_ref().expect("root");
        assert!(matches!(body.element, Element::Body(_)));
        assert_eq!(body.children.len(), 2);
        let template = &body.children[0];
        assert!(matches!(template.element, Element::Template(_)));
        assert_eq!(template.children.len(), 1);
        assert!(matches!(template.children[0].element, Element::Div(_)));
        assert!(matches!(body.children[1].element, Element::P(_)));
    }

    #[test]
    fn test_comments_and_doctype_dropped() {
        let tree = build(tokenize("<!DOCTYPE html><!--c--><p>hi</p>"));
        // Doctype + comment dropped → only <p> at top level → it becomes the root,
        // no synthetic body wrapper.
        let p = tree.root.as_ref().expect("root");
        assert!(matches!(p.element, Element::P(_)));
        assert_eq!(p.children.len(), 1);
        assert!(matches!(p.children[0].element, Element::Text(_)));
    }
}
