use std::collections::HashMap;
use std::hash::{Hash, Hasher, DefaultHasher};

use lui_css_parser::{Declaration, Stylesheet, parse_declaration_block, parse_stylesheet};

use crate::{
    element::{should_auto_close, HtmlElement},
    tokenizer::{tokenize, Token},
    ArcStr,
};

/// A node in the parsed HTML tree.
#[derive(Debug, Clone)]
pub struct HtmlNode {
  pub element: HtmlElement,
  pub id: Option<ArcStr>,
  pub class_list: Vec<ArcStr>,
  pub styles: Vec<Declaration>,
  pub attrs: HashMap<ArcStr, ArcStr>,
  pub data_attrs: HashMap<ArcStr, ArcStr>,
  pub aria_attrs: HashMap<ArcStr, ArcStr>,
  pub children: Vec<HtmlNode>,
  /// Pre-computed hash of this node's identity (element + all attributes +
  /// inline styles). Order-independent via XOR for HashMap attrs.
  /// Does NOT include children or positional context.
  pub node_hash: u64,
}

impl PartialEq for HtmlNode {
  fn eq(&self, other: &Self) -> bool {
    self.element == other.element
      && self.id == other.id
      && self.class_list == other.class_list
      && self.styles == other.styles
      && self.attrs == other.attrs
      && self.data_attrs == other.data_attrs
      && self.aria_attrs == other.aria_attrs
      && self.children == other.children
  }
}

impl HtmlNode {
  pub fn new(element: HtmlElement) -> Self {
    let node_hash = hash_tag(element.tag_name());
    Self {
      element,
      id: None,
      class_list: Vec::new(),
      styles: Vec::new(),
      attrs: HashMap::new(),
      data_attrs: HashMap::new(),
      aria_attrs: HashMap::new(),
      children: Vec::new(),
      node_hash,
    }
  }

  pub fn text(content: impl Into<ArcStr>) -> Self {
    Self {
      element: HtmlElement::Text(content.into()),
      id: None,
      class_list: Vec::new(),
      styles: Vec::new(),
      attrs: HashMap::new(),
      data_attrs: HashMap::new(),
      aria_attrs: HashMap::new(),
      children: Vec::new(),
      node_hash: 0,
    }
  }

  /// Convenience: look up any attribute by name, checking attrs first,
  /// then data_attrs and aria_attrs with their prefixes stripped.
  pub fn attr(&self, name: &str) -> Option<&ArcStr> {
    if name == "id" {
      return self.id.as_ref();
    }
    if name == "class" {
      return None; // use class_list instead
    }
    self.attrs.get(name)
      .or_else(|| self.data_attrs.get(name.strip_prefix("data-").unwrap_or("")))
      .or_else(|| self.aria_attrs.get(name.strip_prefix("aria-").unwrap_or("")))
  }

  pub fn with_attrs(mut self, attrs: Vec<(String, String)>) -> Self {
    for (k, v) in attrs {
      if k == "id" {
        self.id = Some(ArcStr::from(v.as_str()));
      } else if k == "class" {
        self.class_list = v.split_ascii_whitespace()
          .map(|c| ArcStr::from(c))
          .collect();
      } else if k == "style" {
        self.styles = parse_declaration_block(&v).unwrap_or_default();
      } else if let Some(rest) = k.strip_prefix("data-") {
        self.data_attrs.insert(ArcStr::from(rest), ArcStr::from(v.as_str()));
      } else if let Some(rest) = k.strip_prefix("aria-") {
        self.aria_attrs.insert(ArcStr::from(rest), ArcStr::from(v.as_str()));
      } else {
        self.attrs.insert(ArcStr::from(k.as_str()), ArcStr::from(v.as_str()));
      }
    }
    self.node_hash = compute_node_hash(&self);
    self
  }

  pub fn with_children(mut self, children: Vec<HtmlNode>) -> Self {
    self.children = children;
    self
  }
}

/// Parsed HTML document — always rooted at a single `<html>` node.
#[derive(Debug, Clone)]
pub struct HtmlDocument {
  pub root: HtmlNode,
  /// Stylesheets extracted from `<style>` elements, in document order.
  pub stylesheets: Vec<Stylesheet>,
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
  /// Stylesheets extracted from `<style>` elements.
  stylesheets: Vec<Stylesheet>,
}

impl TreeBuilder {
  fn new(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
      pos: 0,
      stack: Vec::new(),
      document: Vec::new(),
      stylesheets: Vec::new(),
    }
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
        Token::OpenTag {
          name,
          attrs,
          self_closing,
        } => {
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
    let root = if self.document.len() == 1 && self.document[0].element == HtmlElement::Html {
      self.document.into_iter().next().unwrap()
    } else {
      HtmlNode::new(HtmlElement::Html).with_children(self.document)
    };
    HtmlDocument { root, stylesheets: self.stylesheets }
  }

  fn push_node(&mut self, node: HtmlNode) {
    if let Some(top) = self.stack.last_mut() {
      top.1.push(node);
    } else {
      self.document.push(node);
    }
  }

  fn pop_element(&mut self) {
    let Some((_tag_name, children, attrs)) = self.stack.pop() else {
      return;
    };
    let element = HtmlElement::from_name(&_tag_name);

    if element == HtmlElement::Style {
      let css_text: String = children.iter()
        .filter_map(|c| match &c.element {
          HtmlElement::Text(s) => Some(s.as_ref()),
          _ => None,
        })
        .collect();
      if let Ok(sheet) = parse_stylesheet(&css_text) {
        self.stylesheets.push(sheet);
      }
    }

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

fn hash_tag(tag: &str) -> u64 {
  let mut h = DefaultHasher::new();
  tag.hash(&mut h);
  h.finish()
}

fn hash_kv(k: &str, v: &str) -> u64 {
  let mut h = DefaultHasher::new();
  k.hash(&mut h);
  v.hash(&mut h);
  h.finish()
}

fn compute_node_hash(node: &HtmlNode) -> u64 {
  let mut h = DefaultHasher::new();

  // Tag
  node.element.tag_name().hash(&mut h);

  // ID
  node.id.hash(&mut h);

  // Classes (order matters — hash sequentially)
  node.class_list.len().hash(&mut h);
  for c in &node.class_list {
    c.as_ref().hash(&mut h);
  }

  // Inline styles (order matters — hash sequentially)
  node.styles.len().hash(&mut h);
  for d in &node.styles {
    d.property.hash(&mut h);
    d.value.hash(&mut h);
    d.important.hash(&mut h);
  }

  // Attrs, data_attrs, aria_attrs — order-independent via XOR
  let mut attr_xor = 0u64;
  for (k, v) in &node.attrs {
    attr_xor ^= hash_kv(k.as_ref(), v.as_ref());
  }
  for (k, v) in &node.data_attrs {
    attr_xor ^= hash_kv(k.as_ref(), v.as_ref());
  }
  for (k, v) in &node.aria_attrs {
    attr_xor ^= hash_kv(k.as_ref(), v.as_ref());
  }
  attr_xor.hash(&mut h);
  node.attrs.len().hash(&mut h);
  node.data_attrs.len().hash(&mut h);
  node.aria_attrs.len().hash(&mut h);

  h.finish()
}
