use crate::{
  ArcStr, HtmlNode,
  selector_match::{AncestorEntry, MatchContext, matches_selector},
  selector_parse::parse_selector_list,
};

impl HtmlNode {
  pub fn get_element_by_id(&self, id: ArcStr) -> Option<&HtmlNode> {
    if self.id.as_ref() == Some(&id) {
      return Some(self);
    }
    for child in &self.children {
      if let Some(found) = child.get_element_by_id(id.clone()) {
        return Some(found);
      }
    }
    None
  }

  pub fn get_element_by_id_mut(&mut self, id: ArcStr) -> Option<&mut HtmlNode> {
    if self.id.as_ref() == Some(&id) {
      return Some(self);
    }
    for child in &mut self.children {
      if let Some(found) = child.get_element_by_id_mut(id.clone()) {
        return Some(found);
      }
    }
    None
  }

  pub fn get_elements_by_class_name(&self, class_name: ArcStr) -> Vec<&HtmlNode> {
    let mut out = Vec::new();
    collect_by_class(self, &class_name, &mut out);
    out
  }

  pub fn get_elements_by_class_name_mut(&mut self, class_name: ArcStr) -> Vec<&mut HtmlNode> {
    let mut out = Vec::new();
    collect_by_class_mut(self, &class_name, &mut out);
    out
  }

  pub fn get_elements_by_tag_name(&self, tag_name: ArcStr) -> Vec<&HtmlNode> {
    let mut out = Vec::new();
    collect_by_tag(self, tag_name.as_ref(), &mut out);
    out
  }

  pub fn get_elements_by_tag_name_mut(&mut self, tag_name: ArcStr) -> Vec<&mut HtmlNode> {
    let mut out = Vec::new();
    collect_by_tag_mut(self, tag_name.as_ref(), &mut out);
    out
  }

  pub fn query_selector(&self, selector: &str) -> Option<&HtmlNode> {
    let sel = parse_selector_list(selector).ok()?;
    walk_first(self, &sel.0, &[])
  }

  pub fn query_selector_mut(&mut self, selector: &str) -> Option<&mut HtmlNode> {
    let sel = parse_selector_list(selector).ok()?;
    let path = walk_first_path(self, &sel.0, &[])?;
    self.at_path_mut(&path)
  }

  pub fn query_selector_all(&self, selector: &str) -> Vec<&HtmlNode> {
    let sel = match parse_selector_list(selector) {
      Ok(s) => s,
      Err(_) => return vec![],
    };
    let mut results = Vec::new();
    walk_collect(self, &sel.0, &[], &mut results);
    results
  }

  pub fn query_selector_all_paths(&self, selector: &str) -> Vec<Vec<usize>> {
    let sel = match parse_selector_list(selector) {
      Ok(s) => s,
      Err(_) => return vec![],
    };
    let mut paths = Vec::new();
    walk_collect_paths(self, &sel.0, &[], &mut Vec::new(), &mut paths);
    paths
  }

  pub fn matches(&self, selector: &str) -> bool {
    let sel = match parse_selector_list(selector) {
      Ok(s) => s,
      Err(_) => return false,
    };
    let ctx = MatchContext::default();
    sel.0.iter().any(|complex| matches_selector(complex, self, &ctx, &[], None))
  }
}

fn collect_by_class<'a>(node: &'a HtmlNode, class: &ArcStr, out: &mut Vec<&'a HtmlNode>) {
  if node.class_list.contains(class) {
    out.push(node);
  }
  for child in &node.children {
    collect_by_class(child, class, out);
  }
}

fn collect_by_class_mut<'a>(node: &'a mut HtmlNode, class: &ArcStr, out: &mut Vec<&'a mut HtmlNode>) {
  let matches = node.class_list.contains(class);
  if matches {
    out.push(node);
  } else {
    for child in &mut node.children {
      collect_by_class_mut(child, class, out);
    }
  }
}

fn collect_by_tag<'a>(node: &'a HtmlNode, tag: &str, out: &mut Vec<&'a HtmlNode>) {
  if node.element.tag_name() == tag {
    out.push(node);
  }
  for child in &node.children {
    collect_by_tag(child, tag, out);
  }
}

fn collect_by_tag_mut<'a>(node: &'a mut HtmlNode, tag: &str, out: &mut Vec<&'a mut HtmlNode>) {
  let matches = node.element.tag_name() == tag;
  if matches {
    out.push(node);
  } else {
    for child in &mut node.children {
      collect_by_tag_mut(child, tag, out);
    }
  }
}

fn child_ctx(index: usize, count: usize) -> MatchContext<'static> {
  MatchContext {
    is_first_child: index == 0,
    is_last_child: index == count.saturating_sub(1),
    is_only_child: count == 1,
    sibling_index: index,
    sibling_count: count,
    ..Default::default()
  }
}

use crate::selector::ComplexSelector;

fn walk_first<'a>(
  node: &'a HtmlNode,
  selectors: &[ComplexSelector],
  ancestors: &[AncestorEntry<'a>],
) -> Option<&'a HtmlNode> {
  let count = node.children.len();
  for (i, child) in node.children.iter().enumerate() {
    let ctx = child_ctx(i, count);
    let mut child_ancestors = vec![AncestorEntry { node, ctx: child_ctx(i, count) }];
    child_ancestors.extend(ancestors.iter().cloned());

    if selectors.iter().any(|sel| matches_selector(sel, child, &ctx, &child_ancestors, Some(node))) {
      return Some(child);
    }
    if let Some(found) = walk_first(child, selectors, &child_ancestors) {
      return Some(found);
    }
  }
  None
}

fn walk_first_path(
  node: &HtmlNode,
  selectors: &[ComplexSelector],
  ancestors: &[AncestorEntry<'_>],
) -> Option<Vec<usize>> {
  let count = node.children.len();
  for (i, child) in node.children.iter().enumerate() {
    let ctx = child_ctx(i, count);
    let mut child_ancestors = vec![AncestorEntry { node, ctx: child_ctx(i, count) }];
    child_ancestors.extend(ancestors.iter().cloned());

    if selectors.iter().any(|sel| matches_selector(sel, child, &ctx, &child_ancestors, Some(node))) {
      return Some(vec![i]);
    }
    if let Some(mut path) = walk_first_path(child, selectors, &child_ancestors) {
      path.insert(0, i);
      return Some(path);
    }
  }
  None
}

fn walk_collect<'a>(
  node: &'a HtmlNode,
  selectors: &[ComplexSelector],
  ancestors: &[AncestorEntry<'a>],
  results: &mut Vec<&'a HtmlNode>,
) {
  let count = node.children.len();
  for (i, child) in node.children.iter().enumerate() {
    let ctx = child_ctx(i, count);
    let mut child_ancestors = vec![AncestorEntry { node, ctx: child_ctx(i, count) }];
    child_ancestors.extend(ancestors.iter().cloned());

    if selectors.iter().any(|sel| matches_selector(sel, child, &ctx, &child_ancestors, Some(node))) {
      results.push(child);
    }
    walk_collect(child, selectors, &child_ancestors, results);
  }
}

fn walk_collect_paths(
  node: &HtmlNode,
  selectors: &[ComplexSelector],
  ancestors: &[AncestorEntry<'_>],
  current_path: &mut Vec<usize>,
  paths: &mut Vec<Vec<usize>>,
) {
  let count = node.children.len();
  for (i, child) in node.children.iter().enumerate() {
    let ctx = child_ctx(i, count);
    let mut child_ancestors = vec![AncestorEntry { node, ctx: child_ctx(i, count) }];
    child_ancestors.extend(ancestors.iter().cloned());

    current_path.push(i);
    if selectors.iter().any(|sel| matches_selector(sel, child, &ctx, &child_ancestors, Some(node))) {
      paths.push(current_path.clone());
    }
    walk_collect_paths(child, selectors, &child_ancestors, current_path, paths);
    current_path.pop();
  }
}

fn at_path_split<'a>(root: &'a mut HtmlNode, path: &[usize]) -> Option<&'a mut HtmlNode> {
  let mut node = root;
  for &idx in path {
    node = node.children.get_mut(idx)?;
  }
  Some(node)
}
