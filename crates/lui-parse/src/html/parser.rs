use lui_core::{ArcStr, HtmlDocument, HtmlElement, HtmlNode, Stylesheet, should_auto_close};

use crate::{
  css::stylesheet::{parse_declaration_block, parse_stylesheet},
  html::tokenizer::{Token, tokenize},
};

/// Build an `HtmlNode` from raw attribute pairs, parsing inline styles.
pub fn html_node_with_attrs(mut node: HtmlNode, attrs: Vec<(String, String)>) -> HtmlNode {
  for (k, v) in attrs {
    if k == "id" {
      node.id = Some(ArcStr::from(v.as_str()));
    } else if k == "class" {
      node.class_list = v.split_ascii_whitespace().map(|c| ArcStr::from(c)).collect();
    } else if k == "style" {
      node.styles = parse_declaration_block(&v).unwrap_or_default();
    } else if let Some(rest) = k.strip_prefix("data-") {
      node.data_attrs.insert(ArcStr::from(rest), ArcStr::from(v.as_str()));
    } else if let Some(rest) = k.strip_prefix("aria-") {
      node.aria_attrs.insert(ArcStr::from(rest), ArcStr::from(v.as_str()));
    } else {
      node.attrs.insert(ArcStr::from(k.as_str()), ArcStr::from(v.as_str()));
    }
  }
  node.recompute_hash();
  node
}

/// Parse an HTML string into a `HtmlDocument`.
pub fn parse(html_str: &str) -> HtmlDocument {
  let tokens = tokenize(html_str);
  let mut builder = TreeBuilder::new(tokens);
  builder.run();
  builder.finish()
}

struct TreeBuilder {
  tokens: Vec<Token>,
  pos: usize,
  stack: Vec<(String, Vec<HtmlNode>, Vec<(String, String)>)>,
  document: Vec<HtmlNode>,
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
              self.push_node(html_node_with_attrs(HtmlNode::new(element), attrs));
            }
          } else if name == "body" && self.has_body() {
          } else if name == "html" && self.has_html() {
          } else if element.is_raw_text() {
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
    HtmlDocument::new(root, self.stylesheets)
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
      let css_text: String = children
        .iter()
        .filter_map(|c| match &c.element {
          HtmlElement::Text(s) => Some(s.as_ref()),
          _ => None,
        })
        .collect();
      if let Ok(sheet) = parse_stylesheet(&css_text) {
        self.stylesheets.push(sheet);
      }
    }

    self.push_node(html_node_with_attrs(HtmlNode::new(element), attrs).with_children(children));
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
