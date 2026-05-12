use lui_tree::Node;

#[allow(dead_code)]
fn first_path_with_class(node: &Node, class: &str, path: &mut Vec<usize>) -> Option<Vec<usize>> {
  if node.class_list.iter().any(|c| c.as_ref() == class) {
    return Some(path.clone());
  }

  for (idx, child) in node.children.iter().enumerate() {
    path.push(idx);
    let found = first_path_with_class(child, class, path);
    path.pop();
    if found.is_some() {
      return found;
    }
  }
  None
}
