//! Tree panel component — renders the inspected DOM as expandable rows.

use std::{collections::HashSet, sync::Arc};

use wgpu_html_tree::{Element, Node, Tree};
use wgpu_html_ui::{el, el::El, Component, Ctx, ShouldRender};

use crate::tags::*;

// ── Props / Msg ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TreePanelProps {
  pub selected_path: Option<Vec<usize>>,
  pub collapsed: HashSet<Vec<usize>>,
  /// Depth beyond which nodes are treated as collapsed unless
  /// explicitly expanded by the user.
  pub auto_collapse_depth: usize,
  pub on_select: Arc<dyn Fn(Vec<usize>) + Send + Sync>,
  pub on_toggle: Arc<dyn Fn(Vec<usize>) + Send + Sync>,
}

#[derive(Clone)]
pub enum TreePanelMsg {
  Select(Vec<usize>),
  Toggle(Vec<usize>),
}

// ── Component ───────────────────────────────────────────────────────────────

pub struct TreePanel;

impl Component for TreePanel {
  type Props = TreePanelProps;
  type Msg = TreePanelMsg;
  type Env = Tree;

  fn create(_props: &TreePanelProps) -> Self {
    TreePanel
  }

  fn update(&mut self, msg: TreePanelMsg, props: &TreePanelProps) -> ShouldRender {
    match msg {
      TreePanelMsg::Select(ref path) => {
        println!("[TreePanel::update] Select({path:?})");
        (props.on_select)(path.clone());
      }
      TreePanelMsg::Toggle(ref path) => {
        println!("[TreePanel::update] Toggle({path:?})");
        (props.on_toggle)(path.clone());
      }
    }
    // Parent will re-render us with new props.
    ShouldRender::No
  }

  fn view(&self, props: &TreePanelProps, ctx: &Ctx<TreePanelMsg>, env: &Tree) -> El {
    let root = env.root.as_ref();
    let mut container = el::div().class("tree-rows");
    if let Some(root) = root {
      let mut path = Vec::new();
      let mut rows = Vec::new();
      emit_tree_node(
        &mut rows,
        root,
        0,
        &mut path,
        props.selected_path.as_deref(),
        &props.collapsed,
        props.auto_collapse_depth,
        ctx,
      );
      for row in rows {
        container = container.child(row);
      }
    }
    container
  }
}

// ── Tree node rendering ─────────────────────────────────────────────────────

fn emit_tree_node(
  out: &mut Vec<El>,
  node: &Node,
  depth: usize,
  path: &mut Vec<usize>,
  selected_path: Option<&[usize]>,
  collapsed: &HashSet<Vec<usize>>,
  auto_collapse_depth: usize,
  ctx: &Ctx<TreePanelMsg>,
) {
  if depth > 32 {
    return;
  }

  match &node.element {
    Element::Text(t) => {
      let trimmed = t.trim();
      if trimmed.is_empty() {
        return;
      }
      let display = truncate(trimmed, 60);
      let row = make_tree_row(depth, path, selected_path, ctx)
        .child(el::span().class("text-node").text(format!("\"{display}\"")));
      out.push(row);
    }
    _ => {
      let tag = node.element.tag_name();
      if matches!(tag, "style" | "script" | "meta" | "link" | "title") {
        return;
      }

      let has_vis = has_visible_children(node);

      if has_vis {
        // Auto-collapse beyond the configured depth unless
        // the user explicitly expanded this node.
        // let explicitly_collapsed = collapsed.contains(path.as_slice());
        // let is_collapsed =
        //   explicitly_collapsed || (depth >= auto_collapse_depth && !collapsed.contains(path.as_slice()));
        // If user toggled an auto-collapsed node, treat it as expanded.
        // We use a trick: if it's in the collapsed set AND beyond auto depth,
        // that means the user toggled it open (the set acts as an override).
        let is_collapsed = if depth >= auto_collapse_depth {
          // Beyond auto-collapse depth: collapsed unless user toggled it open
          !collapsed.contains(path.as_slice())
        } else {
          // Within auto-collapse depth: expanded unless user collapsed it
          collapsed.contains(path.as_slice())
        };

        let icon = if is_collapsed {
          ICON_CHEVRON_RIGHT
        } else {
          ICON_CHEVRON_DOWN
        };

        let mut row = make_tree_row(depth, path, selected_path, ctx).child(chevron_button(icon, path, ctx));
        row = push_open_tag_el(row, node, tag);

        if is_collapsed {
          row = row
            .child(el::span().class("text-node").text("\u{2026}"))
            .child(el::span().class("bracket").text("</"))
            .child(el::span().class("tag").text(tag))
            .child(el::span().class("bracket").text(">"));
          out.push(row);
        } else {
          out.push(row);

          for (i, child) in node.children.iter().enumerate() {
            path.push(i);
            emit_tree_node(
              out,
              child,
              depth + 1,
              path,
              selected_path,
              collapsed,
              auto_collapse_depth,
              ctx,
            );
            path.pop();
          }

          let close = plain_row(depth).children([
            el::span().class("bracket").text("</"),
            el::span().class("tag").text(tag),
            el::span().class("bracket").text(">"),
          ]);
          out.push(close);
        }
      } else {
        let mut row = make_tree_row(depth, path, selected_path, ctx);
        row = push_open_tag_el(row, node, tag);

        if let Some(txt) = text_only_content(node) {
          row = row.child(el::span().class("text-node").text(truncate(&txt, 40)));
        }

        row = row
          .child(el::span().class("bracket").text("</"))
          .child(el::span().class("tag").text(tag))
          .child(el::span().class("bracket").text(">"));
        out.push(row);
      }
    }
  }
}

fn make_tree_row(depth: usize, path: &[usize], selected_path: Option<&[usize]>, ctx: &Ctx<TreePanelMsg>) -> El {
  let px = 12 + depth * 16;
  let is_selected = selected_path == Some(path);
  let class = if is_selected {
    "tree-row tree-row-selected"
  } else {
    "tree-row"
  };
  let path_owned = path.to_vec();
  let msg_cb = ctx.on_click(TreePanelMsg::Select(path_owned.clone()));
  el::div()
    .class(class)
    .style(format!("padding-left: {px}px;"))
    .data("path", encode_path(path))
    .on_click(move |ev| {
      println!("[on_click] row path={path_owned:?}");
      msg_cb(ev);
    })
}

fn plain_row(depth: usize) -> El {
  let px = 12 + depth * 16;
  el::div().class("tree-row").style(format!("padding-left: {px}px;"))
}

fn chevron_button(icon: &str, path: &[usize], ctx: &Ctx<TreePanelMsg>) -> El {
  let path_owned = path.to_vec();
  el::span()
    .class("chevron")
    .text(icon)
    .on_click_cb(ctx.on_click(TreePanelMsg::Toggle(path_owned)))
}

/// Append `<tag id="…" class="…">` spans to an El builder.
fn push_open_tag_el(mut row: El, node: &Node, tag: &str) -> El {
  row = row
    .child(el::span().class("bracket").text("<"))
    .child(el::span().class("tag").text(tag));

  if let Some(id) = node.element.id() {
    row = row
      .child(el::span().class("attr-n").text(" id"))
      .child(el::span().class("bracket").text("="))
      .child(el::span().class("attr-v").text(format!("\"{id}\"")));
  }
  if let Some(cls) = node.element.class() {
    row = row
      .child(el::span().class("attr-n").text(" class"))
      .child(el::span().class("bracket").text("="))
      .child(el::span().class("attr-v").text(format!("\"{cls}\"")));
  }

  row.child(el::span().class("bracket").text(">"))
}
