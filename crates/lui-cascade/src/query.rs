use lui_parse::{HtmlNode, parse_selector_list};

use crate::matching::{AncestorEntry, MatchContext, matches_selector};

/// Find all nodes in the subtree matching a CSS selector string.
pub fn query_selector_all<'a>(root: &'a HtmlNode, selector: &str) -> Vec<&'a HtmlNode> {
  let sel = match parse_selector_list(selector) {
    Ok(s) => s,
    Err(_) => return vec![],
  };
  let mut results = Vec::new();
  let mut path = Vec::new();
  walk_collect(root, root, &sel.0, &[], &mut path, &mut results);
  results
}

/// Find the first node in the subtree matching a CSS selector string.
pub fn query_selector<'a>(root: &'a HtmlNode, selector: &str) -> Option<&'a HtmlNode> {
  let sel = match parse_selector_list(selector) {
    Ok(s) => s,
    Err(_) => return None,
  };
  let mut path = Vec::new();
  walk_first(root, root, &sel.0, &[], &mut path)
}

/// Test whether a node matches a CSS selector string.
pub fn matches(node: &HtmlNode, selector: &str) -> bool {
  let sel = match parse_selector_list(selector) {
    Ok(s) => s,
    Err(_) => return false,
  };
  let ctx = MatchContext::default();
  sel
    .0
    .iter()
    .any(|complex| matches_selector(complex, node, &ctx, &[], None))
}

/// Walk up the ancestor chain and return the first ancestor (or self) matching the selector.
pub fn closest<'a>(
  node: &'a HtmlNode,
  selector: &str,
  ancestors: &[AncestorEntry<'a>],
  parent_node: Option<&'a HtmlNode>,
) -> Option<&'a HtmlNode> {
  let sel = match parse_selector_list(selector) {
    Ok(s) => s,
    Err(_) => return None,
  };

  let ctx = MatchContext::default();
  if sel
    .0
    .iter()
    .any(|complex| matches_selector(complex, node, &ctx, ancestors, parent_node))
  {
    return Some(node);
  }

  for (i, entry) in ancestors.iter().enumerate() {
    let further = &ancestors[i + 1..];
    let parent = further.first().map(|e| e.node);
    if sel
      .0
      .iter()
      .any(|complex| matches_selector(complex, entry.node, &entry.ctx, further, parent))
    {
      return Some(entry.node);
    }
  }

  None
}

// ---------------------------------------------------------------------------

use lui_core::selector::ComplexSelector;

fn walk_collect<'a>(
  tree_root: &'a HtmlNode,
  node: &'a HtmlNode,
  selectors: &[ComplexSelector],
  ancestors: &[AncestorEntry<'a>],
  path: &mut Vec<usize>,
  results: &mut Vec<&'a HtmlNode>,
) {
  let count = node.children().len();
  for (i, child) in node.children().iter().enumerate() {
    let ctx = child_ctx(i, count);

    let mut child_ancestors = vec![AncestorEntry {
      node,
      ctx: child_ctx(i, count),
    }];
    child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
      node: a.node,
      ctx: a.ctx.clone(),
    }));

    if selectors
      .iter()
      .any(|sel| matches_selector(sel, child, &ctx, &child_ancestors, Some(node)))
    {
      results.push(child);
    }

    path.push(i);
    walk_collect(tree_root, child, selectors, &child_ancestors, path, results);
    path.pop();
  }
}

fn walk_first<'a>(
  tree_root: &'a HtmlNode,
  node: &'a HtmlNode,
  selectors: &[ComplexSelector],
  ancestors: &[AncestorEntry<'a>],
  path: &mut Vec<usize>,
) -> Option<&'a HtmlNode> {
  let count = node.children().len();
  for (i, child) in node.children().iter().enumerate() {
    let ctx = child_ctx(i, count);

    let mut child_ancestors = vec![AncestorEntry {
      node,
      ctx: child_ctx(i, count),
    }];
    child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
      node: a.node,
      ctx: a.ctx.clone(),
    }));

    if selectors
      .iter()
      .any(|sel| matches_selector(sel, child, &ctx, &child_ancestors, Some(node)))
    {
      return Some(child);
    }

    path.push(i);
    if let Some(found) = walk_first(tree_root, child, selectors, &child_ancestors, path) {
      return Some(found);
    }
    path.pop();
  }
  None
}

fn child_ctx(index: usize, count: usize) -> MatchContext<'static> {
  MatchContext {
    is_first_child: index == 0,
    is_last_child: index == count - 1,
    is_only_child: count == 1,
    sibling_index: index,
    sibling_count: count,
    ..Default::default()
  }
}
