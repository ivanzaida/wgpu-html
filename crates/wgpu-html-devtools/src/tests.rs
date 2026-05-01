use wgpu_html_tree::{Node, Tree};

use crate::Devtools;

#[allow(dead_code)]
fn first_path_with_class(node: &Node, class: &str, path: &mut Vec<usize>) -> Option<Vec<usize>> {
    if node
        .element
        .class()
        .is_some_and(|classes| classes.split_ascii_whitespace().any(|c| c == class))
    {
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
