use crate::{ArcStr, HtmlNode};

#[derive(Debug, Clone)]
pub enum QuerySelector {
  Tag(String),
  Id(String),
  Class(String),
}

impl QuerySelector {
  pub fn matches(&self, node: &HtmlNode) -> bool {
    match self {
      QuerySelector::Tag(tag) => node.element.tag_name() == tag.as_str(),
      QuerySelector::Id(id) => node.id.as_deref() == Some(id.as_str()),
      QuerySelector::Class(class) => node.class_list.iter().any(|c| c.as_ref() == class.as_str()),
    }
  }
}

impl From<&str> for QuerySelector {
  fn from(selector: &str) -> Self {
    parse_simple_selector(selector)
  }
}

impl From<ArcStr> for QuerySelector {
  fn from(selector: ArcStr) -> Self {
    parse_simple_selector(selector.as_ref())
  }
}

impl From<String> for QuerySelector {
  fn from(selector: String) -> Self {
    parse_simple_selector(&selector)
  }
}

impl From<&QuerySelector> for QuerySelector {
  fn from(selector: &QuerySelector) -> Self {
    selector.clone()
  }
}

fn parse_simple_selector(s: &str) -> QuerySelector {
  let s = s.trim();
  if let Some(id) = s.strip_prefix('#') {
    QuerySelector::Id(id.to_string())
  } else if let Some(class) = s.strip_prefix('.') {
    QuerySelector::Class(class.to_string())
  } else {
    QuerySelector::Tag(s.to_string())
  }
}
