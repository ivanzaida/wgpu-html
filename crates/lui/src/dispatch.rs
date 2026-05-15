use lui_core::{EventPhase, events::DocumentEvent};
use lui_parse::HtmlNode;

/// Find the path (child indices) from `root` to the node at `target` pointer.
pub fn find_node_path(root: &HtmlNode, target: *const HtmlNode) -> Option<Vec<usize>> {
  let mut path = Vec::new();
  if find_path_inner(root, target, &mut path) {
    Some(path)
  } else {
    None
  }
}

fn find_path_inner(node: &HtmlNode, target: *const HtmlNode, path: &mut Vec<usize>) -> bool {
  if std::ptr::eq(node, target) {
    return true;
  }
  for (i, child) in node.children.iter().enumerate() {
    path.push(i);
    if find_path_inner(child, target, path) {
      return true;
    }
    path.pop();
  }
  false
}

/// Dispatch `event` along the DOM path with W3C capture → target → bubble phases.
pub fn dispatch_event(root: &mut HtmlNode, path: &[usize], event: &mut DocumentEvent) {
  // Capture phase: root → parent of target
  for depth in 0..path.len() {
    if event.is_propagation_stopped() {
      return;
    }
    if let Some(node) = node_at_path_mut(root, &path[..depth]) {
      node.dispatch_event_phase(event, EventPhase::Capture);
    }
  }

  // Target phase
  if !event.is_propagation_stopped() {
    if let Some(node) = node_at_path_mut(root, path) {
      node.dispatch_event_phase(event, EventPhase::Target);
    }
  }

  // Bubble phase: parent of target → root
  if event.base().bubbles && !event.is_propagation_stopped() {
    for depth in (0..path.len()).rev() {
      if event.is_propagation_stopped() {
        return;
      }
      if let Some(node) = node_at_path_mut(root, &path[..depth]) {
        node.dispatch_event_phase(event, EventPhase::Bubble);
      }
    }
  }
}

fn node_at_path_mut<'a>(root: &'a mut HtmlNode, path: &[usize]) -> Option<&'a mut HtmlNode> {
  let mut current = root;
  for &idx in path {
    current = current.children.get_mut(idx)?;
  }
  Some(current)
}
