use crate::{ArcStr, HtmlElement, HtmlNode, node::html_node::{DIRTY_ATTRS, DIRTY_CHILDREN, DIRTY_TEXT}};

impl HtmlNode {
  pub fn text_content(&self) -> String {
    let mut out = String::new();
    collect_text(self, &mut out);
    out
  }

  pub fn set_text_content(&mut self, text: &str) {
    self.children.clear();
    if !text.is_empty() {
      self.children.push(HtmlNode::text(text));
    }
    self.dirty |= DIRTY_TEXT | DIRTY_CHILDREN;
  }

  pub fn set_attribute(&mut self, name: &str, value: &str) {
    match name {
      "id" => self.id = Some(ArcStr::from(value)),
      "class" => self.class_list.set(value),
      _ if name.starts_with("data-") => {
        self.data_attrs.insert(ArcStr::from(&name[5..]), ArcStr::from(value));
      }
      _ if name.starts_with("aria-") => {
        self.aria_attrs.insert(ArcStr::from(&name[5..]), ArcStr::from(value));
      }
      _ => {
        self.attrs.insert(ArcStr::from(name), ArcStr::from(value));
      }
    }
    self.dirty |= DIRTY_ATTRS;
    self.recompute_hash();
  }

  pub fn remove_attribute(&mut self, name: &str) -> bool {
    let removed = match name {
      "id" => self.id.take().is_some(),
      "class" => {
        let had = !self.class_list.is_empty();
        self.class_list.clear();
        had
      }
      _ if name.starts_with("data-") => self.data_attrs.remove(&name[5..]).is_some(),
      _ if name.starts_with("aria-") => self.aria_attrs.remove(&name[5..]).is_some(),
      _ => self.attrs.remove(name).is_some(),
    };
    if removed {
      self.dirty |= DIRTY_ATTRS;
      self.recompute_hash();
    }
    removed
  }

  pub fn set_children(&mut self, children: Vec<HtmlNode>) {
    self.children = children;
    self.dirty |= DIRTY_CHILDREN;
  }

  pub fn append_child(&mut self, child: HtmlNode) {
    self.children.push(child);
    self.dirty |= DIRTY_CHILDREN;
  }

  pub fn insert_child(&mut self, index: usize, child: HtmlNode) {
    let idx = index.min(self.children.len());
    self.children.insert(idx, child);
    self.dirty |= DIRTY_CHILDREN;
  }

  pub fn remove_child(&mut self, index: usize) -> Option<HtmlNode> {
    if index < self.children.len() {
      self.dirty |= DIRTY_CHILDREN;
      Some(self.children.remove(index))
    } else {
      None
    }
  }

  pub fn replace_child(&mut self, index: usize, new_child: HtmlNode) -> Option<HtmlNode> {
    if index < self.children.len() {
      self.dirty |= DIRTY_CHILDREN;
      Some(std::mem::replace(&mut self.children[index], new_child))
    } else {
      None
    }
  }

  pub fn inner_html(&self) -> String {
    let mut out = String::new();
    for child in &self.children {
      serialize_node(child, &mut out);
    }
    out
  }
}

fn collect_text(node: &HtmlNode, out: &mut String) {
  if let HtmlElement::Text(s) = &node.element {
    out.push_str(s);
  } else {
    for child in &node.children {
      collect_text(child, out);
    }
  }
}

fn serialize_node(node: &HtmlNode, out: &mut String) {
  match &node.element {
    HtmlElement::Text(s) => out.push_str(s),
    HtmlElement::Comment(s) => {
      out.push_str("<!--");
      out.push_str(s);
      out.push_str("-->");
    }
    _ => {
      let tag = node.element.tag_name();
      out.push('<');
      out.push_str(tag);
      if let Some(id) = &node.id {
        out.push_str(" id=\"");
        out.push_str(id);
        out.push('"');
      }
      if !node.class_list.is_empty() {
        out.push_str(" class=\"");
        for (i, c) in node.class_list.iter().enumerate() {
          if i > 0 {
            out.push(' ');
          }
          out.push_str(c);
        }
        out.push('"');
      }
      for (k, v) in &node.attrs {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(v);
        out.push('"');
      }
      for (k, v) in &node.data_attrs {
        out.push_str(" data-");
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(v);
        out.push('"');
      }
      for (k, v) in &node.aria_attrs {
        out.push_str(" aria-");
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(v);
        out.push('"');
      }
      out.push('>');
      if !node.element.is_void() {
        for child in &node.children {
          serialize_node(child, out);
        }
        out.push_str("</");
        out.push_str(tag);
        out.push('>');
      }
    }
  }
}
