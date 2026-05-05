use wgpu_html_tree::{Element, Node, Tree};

use crate::{attr_parser, tokenizer::Token};

/// HTML void elements that cannot have children and do not need a closing tag.
const VOID_ELEMENTS: &[&str] = &[
  "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr",
];

fn is_void_element(tag: &str) -> bool {
  VOID_ELEMENTS.contains(&tag)
}

/// Build a tree from a list of tokens.
///
/// - Comments and doctypes are dropped.
/// - Unknown tags (and their entire subtree) are dropped.
/// - Whitespace-only text between tags is dropped.
/// - If the parsed tokens yield exactly one top-level node it becomes the tree root; otherwise the children are wrapped
///   in a synthetic `<body>`.
/// - If the tokens yield no nodes at all, the result has `root = None`.
pub fn build(tokens: Vec<Token>) -> Tree {
  let mut builder = TreeBuilder::new(tokens);
  builder.run();

  let mut roots = builder.document;
  let root = match roots.len() {
    0 => None,
    1 => Some(roots.pop().unwrap()),
    _ => {
      // If one of the top-level nodes is already a <body>,
      // adopt the siblings into it instead of wrapping
      // everything in a second synthetic <body>.
      if let Some(body_idx) = roots.iter().position(|n| matches!(&n.element, Element::Body(_))) {
        let mut body = roots.remove(body_idx);
        // Prepend siblings that came before <body>, append
        // those that came after.
        let mut merged = roots.drain(..body_idx).collect::<Vec<_>>();
        let after = roots;
        merged.append(&mut body.children);
        merged.extend(after);
        body.children = merged;
        Some(body)
      } else {
        Some(Node::new(Element::Body(wgpu_html_models::Body::default())).with_children(roots))
      }
    }
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
          } else if name == "body" && self.has_body_on_stack() {
            // HTML spec: a second <body> is ignored.
          } else if name == "html" && self.has_html_on_stack() {
            // HTML spec: a second <html> is ignored.
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

  /// Whether a `<body>` element is already open on the stack
  /// or in the document roots.
  fn has_body_on_stack(&self) -> bool {
    self.stack.iter().any(|(tag, ..)| tag == "body")
      || self.document.iter().any(|n| matches!(&n.element, Element::Body(_)))
  }

  /// Whether an `<html>` element is already open on the stack
  /// or in the document roots.
  fn has_html_on_stack(&self) -> bool {
    self.stack.iter().any(|(tag, ..)| tag == "html")
      || self.document.iter().any(|n| matches!(&n.element, Element::Html(_)))
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
