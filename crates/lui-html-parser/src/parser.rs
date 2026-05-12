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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        let doc = parse("");
        assert!(doc.roots.is_empty());
    }

    #[test]
    fn single_div() {
        let doc = parse("<div></div>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Div);
    }

    #[test]
    fn text_content() {
        let doc = parse("<p>hello</p>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::P);
        assert_eq!(doc.roots[0].children.len(), 1);
        assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("hello".into()));
    }

    #[test]
    fn nested() {
        let doc = parse("<div><span>hi</span></div>");
        assert_eq!(doc.roots.len(), 1);
        let div = &doc.roots[0];
        assert_eq!(div.element, HtmlElement::Div);
        assert_eq!(div.children.len(), 1);
        assert_eq!(div.children[0].element, HtmlElement::Span);
        assert_eq!(div.children[0].children[0].element, HtmlElement::Text("hi".into()));
    }

    #[test]
    fn void_element() {
        let doc = parse("<br>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Br);
    }

    #[test]
    fn self_closing() {
        let doc = parse("<br/>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Br);
    }

    #[test]
    fn whitespace_text_dropped() {
        let doc = parse("<div>   </div>");
        assert_eq!(doc.roots.len(), 1);
        assert!(doc.roots[0].children.is_empty());
    }

    #[test]
    fn comment_preserved() {
        let doc = parse("<!-- hello --><div></div>");
        assert_eq!(doc.roots.len(), 2);
        assert_eq!(doc.roots[0].element, HtmlElement::Comment(" hello ".into()));
        assert_eq!(doc.roots[1].element, HtmlElement::Div);
    }

    #[test]
    fn doctype_dropped() {
        let doc = parse("<!doctype html><p></p>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::P);
    }

    #[test]
    fn unknown_tag_preserved() {
        let doc = parse("<foo>hello</foo>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Unknown("foo".into()));
        assert_eq!(doc.roots[0].children.len(), 1);
        assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("hello".into()));
    }

    #[test]
    fn custom_element_with_dash() {
        let doc = parse("<my-widget foo=\"bar\">content</my-widget>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Unknown("my-widget".into()));
        assert_eq!(doc.roots[0].attrs.get("foo").map(|s| &**s), Some("bar"));
        assert_eq!(doc.roots[0].children.len(), 1);
        assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("content".into()));
    }

    #[test]
    fn multi_root() {
        let doc = parse("<div></div><span></span>");
        assert_eq!(doc.roots.len(), 2);
        assert_eq!(doc.roots[0].element, HtmlElement::Div);
        assert_eq!(doc.roots[1].element, HtmlElement::Span);
    }

    #[test]
    fn auto_close_p_before_div() {
        let doc = parse("<p><div>inner</div></p>");
        assert_eq!(doc.roots.len(), 2);
        assert_eq!(doc.roots[0].element, HtmlElement::P);
        assert!(doc.roots[0].children.is_empty());
        assert_eq!(doc.roots[1].element, HtmlElement::Div);
        assert_eq!(doc.roots[1].children[0].element, HtmlElement::Text("inner".into()));
    }

    #[test]
    fn id_and_class() {
        let doc = parse(r#"<div id="main" class="container"></div>"#);
        let div = &doc.roots[0];
        assert_eq!(div.attrs.get("id").map(|s| &**s), Some("main"));
        assert_eq!(div.attrs.get("class").map(|s| &**s), Some("container"));
    }

    #[test]
    fn raw_text_script() {
        let doc = parse("<script>var x = '<div>';</script>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Script);
        assert_eq!(doc.roots[0].children.len(), 1);
        assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("var x = '<div>';".into()));
    }

    #[test]
    fn raw_text_style() {
        let doc = parse("<style>div { color: red; }</style>");
        assert_eq!(doc.roots.len(), 1);
        assert_eq!(doc.roots[0].element, HtmlElement::Style);
        assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("div { color: red; }".into()));
    }

    #[test]
    fn input_attributes() {
        let doc = parse(r#"<input type="text" name="q" placeholder="Search" required>"#);
        let input = &doc.roots[0];
        assert_eq!(input.element, HtmlElement::Input);
        assert_eq!(input.attrs.get("type").map(|s| &**s), Some("text"));
        assert_eq!(input.attrs.get("name").map(|s| &**s), Some("q"));
    }

    #[test]
    fn all_known_elements_parse() {
        for tag in &["html", "head", "body", "title", "base", "link", "meta", "style",
            "article", "section", "nav", "aside", "h1", "h2", "h3", "h4", "h5", "h6",
            "hgroup", "header", "footer", "address",
            "p", "hr", "pre", "blockquote", "ol", "ul", "menu", "li", "dl", "dt", "dd",
            "figure", "figcaption", "main", "search", "div",
            "a", "em", "strong", "small", "s", "cite", "q", "dfn", "abbr",
            "ruby", "rt", "rp", "data", "time", "code", "var", "samp", "kbd",
            "sub", "sup", "i", "b", "u", "mark", "bdi", "bdo", "span", "wbr",
            "ins", "del",
            "picture", "source", "img", "iframe", "embed", "object", "video", "audio", "track",
            "map", "area",
            "table", "caption", "colgroup", "col", "tbody", "thead", "tfoot", "tr", "td", "th",
            "form", "label", "input", "button", "select", "datalist", "optgroup", "option",
            "textarea", "output", "progress", "meter", "fieldset", "legend", "selectedcontent",
            "details", "summary", "dialog",
            "noscript", "template", "slot", "canvas",
            // Obsolete
            "marquee", "blink", "font", "center", "big", "small", "strike", "tt",
            "applet", "acronym", "bgsound", "dir", "frame", "frameset", "noframes",
            "isindex", "keygen", "listing", "menuitem", "nextid", "noembed", "param",
            "plaintext", "rb", "rtc", "xmp", "basefont", "multicol", "nobr", "spacer",
        ] {
            let html = format!("<{}></{}>", tag, tag);
            let doc = parse(&html);
            if doc.roots.is_empty() && HtmlElement::from_name(tag).is_void() {
                // Void elements have no children so <tag></tag> is still ok
                let doc2 = parse(&format!("<{}>", tag));
                assert!(!doc2.roots.is_empty(), "void element <{}> should parse", tag);
            } else {
                assert!(!doc.roots.is_empty(), "<{}> should parse", tag);
            }
        }
    }
}
