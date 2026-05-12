use std::collections::HashMap;

use crate::ArcStr;
use crate::element::{HtmlElement, should_auto_close};
use crate::tokenizer::{Token, tokenize};

/// A node in the parsed HTML tree.
#[derive(Debug, Clone, PartialEq)]
pub struct HtmlNode {
    pub element: HtmlElement,
    pub attrs: HashMap<ArcStr, ArcStr>,
    pub children: Vec<HtmlNode>,
}

impl HtmlNode {
    pub fn new(element: HtmlElement) -> Self {
        Self { element, attrs: HashMap::new(), children: Vec::new() }
    }

    pub fn text(content: impl Into<ArcStr>) -> Self {
        Self { element: HtmlElement::Text(content.into()), attrs: HashMap::new(), children: Vec::new() }
    }

    pub fn with_attrs(mut self, attrs: Vec<(String, String)>) -> Self {
        self.attrs = attrs.into_iter().map(|(k, v)| (ArcStr::from(k), ArcStr::from(v))).collect();
        self
    }

    pub fn with_children(mut self, children: Vec<HtmlNode>) -> Self {
        self.children = children;
        self
    }
}

/// Parsed HTML document — one or more root nodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HtmlDocument {
    pub roots: Vec<HtmlNode>,
}

/// Parse an HTML string into a `HtmlDocument`.
///
/// Doctypes and whitespace-only text between tags are dropped.
/// Comments are preserved as `HtmlElement::Comment`.
/// Unknown tags are preserved as `HtmlElement::Unknown(tag_name)`.
pub fn parse(html_str: &str) -> HtmlDocument {
    let tokens = tokenize(html_str);
    let mut builder = TreeBuilder::new(tokens);
    builder.run();
    builder.finish()
}

// ---------------------------------------------------------------------------
// Tree builder
// ---------------------------------------------------------------------------

struct TreeBuilder {
    tokens: Vec<Token>,
    pos: usize,
    /// (tag_name, children, raw_attrs). tag_name is used for matching close tags.
    stack: Vec<(String, Vec<HtmlNode>, Vec<(String, String)>)>,
    /// Nodes that will become document roots.
    document: Vec<HtmlNode>,
}

impl TreeBuilder {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0, stack: Vec::new(), document: Vec::new() }
    }

    fn run(&mut self) {
        while self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;

            match token {
                Token::Doctype(_) => {}
                Token::Comment(text) => {
                    self.push_node(HtmlNode::new(HtmlElement::Comment(ArcStr::from(text))));
                }
                Token::Text(text) => {
                    if !text.trim().is_empty() {
                        self.push_node(HtmlNode::text(text));
                    }
                }
                Token::OpenTag { name, attrs, self_closing } => {
                    let element = HtmlElement::from_name(&name);

                    if self_closing || element.is_void() {
                        if !element.is_text() {
                            self.push_node(HtmlNode::new(element).with_attrs(attrs));
                        }
                    } else if name == "body" && self.has_body() {
                        // Ignore duplicate <body>
                    } else if name == "html" && self.has_html() {
                        // Ignore duplicate <html>
                    } else if element.is_raw_text() {
                        // Raw text elements go on the stack — the tokenizer's
                        // emitted Text token becomes their child, and the
                        // matching CloseTag pops them.
                        self.stack.push((name, Vec::new(), attrs));
                    } else {
                        self.auto_close_before(&name);
                        self.stack.push((name, Vec::new(), attrs));
                    }
                }
                Token::CloseTag(name) => self.close_tag(&name),
            }
        }

        while !self.stack.is_empty() {
            self.pop_element();
        }
    }

    fn finish(self) -> HtmlDocument {
        HtmlDocument { roots: self.document }
    }

    fn push_node(&mut self, node: HtmlNode) {
        if let Some(top) = self.stack.last_mut() {
            top.1.push(node);
        } else {
            self.document.push(node);
        }
    }

    fn pop_element(&mut self) {
        let Some((_tag_name, children, attrs)) = self.stack.pop() else { return };
        let element = HtmlElement::from_name(&_tag_name);
        self.push_node(HtmlNode::new(element).with_attrs(attrs).with_children(children));
    }

    fn close_tag(&mut self, name: &str) {
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
    }

    fn has_body(&self) -> bool {
        self.stack.iter().any(|(t, ..)| t == "body")
    }

    fn has_html(&self) -> bool {
        self.stack.iter().any(|(t, ..)| t == "html")
    }

    fn auto_close_before(&mut self, opening_tag: &str) {
        loop {
            let should_close = self
                .stack
                .last()
                .map(|top| should_auto_close(&top.0, opening_tag))
                .unwrap_or(false);
            if should_close {
                self.pop_element();
            } else {
                break;
            }
        }
    }
}

