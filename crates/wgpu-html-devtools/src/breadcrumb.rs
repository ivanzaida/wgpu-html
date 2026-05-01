//! Breadcrumb bar component — shows the path to the selected element.

use wgpu_html_tree::Tree;
use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

use crate::tags::tag_label;

// ── Props / Msg ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct BreadcrumbProps {
  pub selected_path: Option<Vec<usize>>,
}

// Stateless — no interactions.
#[derive(Clone)]
pub enum BreadcrumbMsg {}

// ── Component ───────────────────────────────────────────────────────────────

pub struct BreadcrumbBar;

impl Component for BreadcrumbBar {
  type Props = BreadcrumbProps;
  type Msg = BreadcrumbMsg;
  type Env = Tree;

  fn create(_props: &BreadcrumbProps) -> Self {
    BreadcrumbBar
  }

  fn update(&mut self, msg: BreadcrumbMsg, _props: &BreadcrumbProps) -> ShouldRender {
    match msg {}
  }

  fn view(&self, props: &BreadcrumbProps, _ctx: &Ctx<BreadcrumbMsg>, env: &Tree) -> El {
    let root = env.root.as_ref();
    let mut bc = el::div().class("breadcrumb");

    if let (Some(root), Some(path)) = (root, props.selected_path.as_deref()) {
      let mut current = root;
      let len = path.len();

      if len == 0 {
        bc = bc.child(el::span().class("bc-active").text(tag_label(current)));
      } else {
        bc = bc.child(el::span().class("bracket").text(tag_label(current)));
      }

      for (i, &idx) in path.iter().enumerate() {
        bc = bc.child(el::text(" \u{203A} "));
        if let Some(child) = current.children.get(idx) {
          let label = tag_label(child);
          let cls = if i == len - 1 { "bc-active" } else { "bracket" };
          bc = bc.child(el::span().class(cls).text(label));
          current = child;
        } else {
          break;
        }
      }
    } else {
      bc = bc.child(el::span().class("bc-active").text("document"));
    }

    bc
  }
}
