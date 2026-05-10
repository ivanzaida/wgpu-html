use lui_models::{ArcStr, Div, Span};
use lui_tree::{Element, Node};

pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/custom-elements.html");
  let mut tree = lui::parser::parse(HTML);

  tree.register_custom_element("app-card", |node| {
    let title = node.element.attr("title").unwrap_or(ArcStr::from("Untitled"));
    let status = node.element.attr("status").unwrap_or(ArcStr::from("active"));

    let mut card = Node::new(Div::default());
    if let Element::Div(ref mut d) = card.element {
      d.class = Some("card".into());
    }

    let mut h2 = Node::new(lui_models::H2::default());
    h2.children.push(Node::new(Element::Text(title)));

    let mut badge = Node::new(Span::default());
    if let Element::Span(ref mut s) = badge.element {
      s.class = Some(ArcStr::from(format!("badge status-{status}")));
    }
    badge.children.push(Node::new(Element::Text(status)));

    card.children.push(h2);
    for child in &node.children {
      card.children.push(child.clone());
    }
    card.children.push(badge);
    card
  });

  tree.register_custom_element("app-alert", |node| {
    let level = node.element.attr("level").unwrap_or(ArcStr::from("info"));

    let mut alert = Node::new(Div::default());
    if let Element::Div(ref mut d) = alert.element {
      d.class = Some(ArcStr::from(format!("alert-box alert-{level}")));
    }
    alert.children = node.children.clone();
    alert
  });

  tree.resolve_custom_elements();
  tree
}
