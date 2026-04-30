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

#[test]
#[ignore = "requires windowed context (HtmlWindow needs ActiveEventLoop)"]
fn pointer_hover_repaints_tree_row_background() {
    let inspected = Tree::new(Node::new(wgpu_html_models::Div {
        id: Some("app".to_owned()),
        ..Default::default()
    }));
    let mut devtools = Devtools::new(false);
    devtools.poll(&inspected);
    if wgpu_html_tree::register_system_fonts(&mut devtools.tree, "DemoSans") == 0 {
        return;
    }

    let _ = first_path_with_class(
        devtools.tree.root.as_ref().expect("devtools root"),
        "tree-row",
        &mut Vec::new(),
    );
}
