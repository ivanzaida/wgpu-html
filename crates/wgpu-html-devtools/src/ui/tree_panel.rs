use std::collections::HashSet;

use wgpu_html_models::common::{AlignItems, BoxSizing, Cursor, Display, Overflow, WhiteSpace};
use wgpu_html_tree::{Element, Node, Tree};
use wgpu_html_ui::{
  el::{self, div}, style::{self, pct, px, Stylesheet}, Component, Ctx,
  El,
  ShouldRender,
};

use super::lucide_icon::lucide;
use super::store::DevtoolsStore;
use super::theme::Theme;

const ICON_CHEVRON_RIGHT: &str = "\u{E06F}";
const ICON_CHEVRON_DOWN: &str = "\u{E06D}";

const INDENT_PX: f32 = 16.0;
const BASE_PAD_LEFT: f32 = 12.0;
const ROW_HEIGHT: f32 = 18.0;

// ── Props / Msg ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TreePanelProps {
  pub store: DevtoolsStore,
}

#[derive(Clone)]
pub enum TreePanelMsg {
  Select(Vec<usize>),
  ToggleExpand(Vec<usize>),
}

// ── Component ───────────────────────────────────────────────────────

pub struct TreePanel {
  selected: Option<Vec<usize>>,
  expanded: HashSet<Vec<usize>>,
}

impl Component for TreePanel {
  type Props = TreePanelProps;
  type Msg = TreePanelMsg;

  fn create(props: &TreePanelProps) -> Self {
    let mut expanded = HashSet::new();
    if let Some(tree) = &props.store.host_tree.get() {
      if let Some(root) = &tree.root {
        auto_expand(root, &[], 0, 4, &mut expanded);
      }
    }
    TreePanel {
      selected: None,
      expanded,
    }
  }

  fn update(&mut self, msg: TreePanelMsg, props: &TreePanelProps) -> ShouldRender {
    match msg {
      TreePanelMsg::Select(path) => {
        self.selected = Some(path.clone());
        props.store.selected_path.set(Some(path));
        ShouldRender::Yes
      }
      TreePanelMsg::ToggleExpand(path) => {
        if self.expanded.contains(&path) {
          self.expanded.remove(&path);
        } else {
          self.expanded.insert(path.clone());
        }
        self.selected = Some(path.clone());
        props.store.selected_path.set(Some(path));
        ShouldRender::Yes
      }
    }
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".panel")
        .display(Display::Flex)
        .user_select("none")
        .prop("flex-direction", "column")
        .width(pct(44))
        .height(pct(100))
        .background_color(Theme::BG_PRIMARY)
        .border_right(format!("1px solid {}", Theme::BORDER))
        .padding_vh(px(8), px(0))
        .overflow_y(Overflow::Auto)
        .box_sizing(BoxSizing::BorderBox)
        .font_family("monospace")
        .font_size(px(11)),
      style::rule(".rows")
        .flex_grow(1.0)
        .overflow_y(Overflow::Auto),
      style::rule(".row")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(ROW_HEIGHT))
        .padding_right(px(12))
        .white_space(WhiteSpace::Nowrap)
        .cursor(Cursor::Default),
      style::rule(".row:hover")
        .background_color(Theme::BG_HOVER),
      style::rule(".row.selected")
        .background_color(Theme::BG_SELECTED),
      style::rule(".row.selected:hover")
        .background_color(Theme::BG_SELECTED_HOVER),
      style::rule(".chevron")
        .width(px(12))
        .height(px(12))
        .font_size(px(12))
        .color(Theme::TEXT_SECONDARY)
        .cursor(Cursor::Pointer)
        .prop("line-height", "12px")
        .flex_shrink(0.0)
        .margin_right(px(4)),
      style::rule(".chevron.selected")
        .color(Theme::TEXT_PRIMARY),
      style::rule(".spacer")
        .width(px(12))
        .height(px(12))
        .flex_shrink(0.0)
        .margin_right(px(4)),
      style::rule(".tag")
        .color(Theme::TAG_COLOR),
      style::rule(".punct")
        .color(Theme::TAG_BRACKET),
      style::rule(".equals")
        .color(Theme::ATTR_EQUALS),
      style::rule(".attr-name")
        .color(Theme::ATTR_NAME)
        .margin_left("4px"),
      style::rule(".attr-val")
        .color(Theme::ATTR_VALUE),
      style::rule(".text-content")
        .color(Theme::TEXT_CONTENT),
      style::rule(".doctype")
        .color(Theme::COMMENT),
    ]).scoped("tree")
  }

  fn view(&self, props: &TreePanelProps, ctx: &Ctx<TreePanelMsg>) -> El {
    let host = props.store.host_tree.get();

    let mut rows: Vec<El> = Vec::new();
    if let Some(tree) = &host {
      if let Some(root) = &tree.root {
        self.build_rows(root, &[], 0, &mut rows, ctx);
      }
    }

    div().class(ctx.scoped("panel")).children([
      div().class(ctx.scoped("rows")).children(rows),
    ])
  }
}

// ── Tree walking ────────────────────────────────────────────────────

impl TreePanel {
  fn build_rows(
    &self,
    node: &Node,
    path: &[usize],
    depth: usize,
    rows: &mut Vec<El>,
    ctx: &Ctx<TreePanelMsg>,
  ) {
    let tag = node.element.tag_name();
    if tag == "#text" || tag == "style" {
      return;
    }

    let is_selected = self.selected.as_deref() == Some(path);
    let has_visible_children = node.children.iter().any(|c| {
      let t = c.element.tag_name();
      t != "#text" && t != "style"
    });
    let is_expanded = self.expanded.contains(path);
    let path_vec = path.to_vec();

    if has_visible_children {
      rows.push(self.render_open_tag(node, &path_vec, depth, is_selected, is_expanded, ctx));

      if is_expanded {
        for (i, child) in node.children.iter().enumerate() {
          let mut child_path = path_vec.clone();
          child_path.push(i);
          self.build_rows(child, &child_path, depth + 1, rows, ctx);
        }
        rows.push(self.render_close_tag(node, &path_vec, depth, is_selected, ctx));
      }
    } else {
      let text_content = node.children.iter().find_map(|c| {
        if let Element::Text(t) = &c.element { Some(t.as_ref()) } else { None }
      });
      rows.push(self.render_leaf_tag(node, &path_vec, depth, is_selected, text_content, ctx));
    }
  }

  fn render_open_tag(
    &self,
    node: &Node,
    path: &[usize],
    depth: usize,
    selected: bool,
    expanded: bool,
    ctx: &Ctx<TreePanelMsg>,
  ) -> El {
    let pad_left = BASE_PAD_LEFT + depth as f32 * INDENT_PX;
    let tag = node.element.tag_name();

    let chevron_icon = if expanded { ICON_CHEVRON_DOWN } else { ICON_CHEVRON_RIGHT };

    let mut parts: Vec<El> = Vec::new();
    parts.push(
      lucide(chevron_icon)
        .class(format!("{}{}", ctx.scoped("chevron"), sel_class(selected, ctx)))
        .on_click_cb(ctx.on_click(TreePanelMsg::ToggleExpand(path.to_vec()))),
    );
    parts.push(punct("<", selected, ctx));
    parts.push(tag_span(tag, selected, ctx));
    push_attrs(node, selected, &mut parts, ctx);
    parts.push(punct(">", selected, ctx));

    if !expanded {
      parts.push(punct("\u{2026}", selected, ctx));
      push_close_tag(tag, selected, &mut parts, ctx);
    }

    row(parts, pad_left, selected, ctx)
      .on_click_cb(ctx.on_click(TreePanelMsg::Select(path.to_vec())))
  }

  fn render_close_tag(
    &self,
    node: &Node,
    path: &[usize],
    depth: usize,
    selected: bool,
    ctx: &Ctx<TreePanelMsg>,
  ) -> El {
    let pad_left = BASE_PAD_LEFT + depth as f32 * INDENT_PX;
    let tag = node.element.tag_name();

    let mut parts: Vec<El> = Vec::new();
    parts.push(div().class(ctx.scoped("spacer")));
    push_close_tag(tag, selected, &mut parts, ctx);

    row(parts, pad_left, selected, ctx)
      .on_click_cb(ctx.on_click(TreePanelMsg::Select(path.to_vec())))
  }

  fn render_leaf_tag(
    &self,
    node: &Node,
    path: &[usize],
    depth: usize,
    selected: bool,
    text: Option<&str>,
    ctx: &Ctx<TreePanelMsg>,
  ) -> El {
    let pad_left = BASE_PAD_LEFT + depth as f32 * INDENT_PX;
    let tag = node.element.tag_name();

    let mut parts: Vec<El> = Vec::new();
    parts.push(div().class(ctx.scoped("spacer")));
    parts.push(punct("<", selected, ctx));
    parts.push(tag_span(tag, selected, ctx));
    push_attrs(node, selected, &mut parts, ctx);

    if is_void_element(tag) {
      parts.push(punct(">", selected, ctx));
    } else if let Some(txt) = text {
      let short = if txt.len() > 30 { &txt[..30] } else { txt };
      parts.push(punct(">", selected, ctx));
      parts.push(el::span().class(ctx.scoped("text-content")).text(short));
      push_close_tag(tag, selected, &mut parts, ctx);
    } else {
      parts.push(punct(">", selected, ctx));
      push_close_tag(tag, selected, &mut parts, ctx);
    }

    row(parts, pad_left, selected, ctx)
      .on_click_cb(ctx.on_click(TreePanelMsg::Select(path.to_vec())))
  }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn sel_class(selected: bool, ctx: &Ctx<TreePanelMsg>) -> String {
  if selected {
    format!(" {}", ctx.scoped("selected"))
  } else {
    String::new()
  }
}

fn row(children: Vec<El>, pad_left: f32, selected: bool, ctx: &Ctx<TreePanelMsg>) -> El {
  div()
    .class(if selected {
      format!("{} {}", ctx.scoped("row"), ctx.scoped("selected"))
    } else {
      ctx.scoped("row").to_string()
    })
    .style(format!("padding-left: {}px", pad_left))
    .children(children)
}

fn punct(text: &str, selected: bool, ctx: &Ctx<TreePanelMsg>) -> El {
  el::span()
    .class(format!("{}{}", ctx.scoped("punct"), sel_class(selected, ctx)))
    .text(text)
}

fn push_close_tag(tag: &str, selected: bool, parts: &mut Vec<El>, ctx: &Ctx<TreePanelMsg>) {
  parts.push(punct("</", selected, ctx));
  parts.push(tag_span(tag, selected, ctx));
  parts.push(punct(">", selected, ctx));
}

fn tag_span(tag: &str, selected: bool, ctx: &Ctx<TreePanelMsg>) -> El {
  el::span()
    .class(format!("{}{}", ctx.scoped("tag"), sel_class(selected, ctx)))
    .text(tag)
}

fn push_attrs(node: &Node, selected: bool, parts: &mut Vec<El>, ctx: &Ctx<TreePanelMsg>) {
  if !node.raw_attrs.is_empty() {
    for (name, value) in &node.raw_attrs {
      push_attr(name, value, selected, parts, ctx);
    }
    return;
  }
  if let Some(id) = node.element.id() {
    push_attr("id", id, selected, parts, ctx);
  }
  if let Some(class) = node.element.class() {
    push_attr("class", class, selected, parts, ctx);
  }
  if let Some(v) = node.element.attr("type") {
    push_attr("type", &v, selected, parts, ctx);
  }
  if let Some(v) = node.element.attr("href") {
    push_attr("href", &v, selected, parts, ctx);
  }
  if let Some(v) = node.element.attr("src") {
    push_attr("src", &v, selected, parts, ctx);
  }
  if let Some(v) = node.element.attr("name") {
    push_attr("name", &v, selected, parts, ctx);
  }
  if let Some(v) = node.element.attr("value") {
    push_attr("value", &v, selected, parts, ctx);
  }
  if let Some(v) = node.element.attr("placeholder") {
    push_attr("placeholder", &v, selected, parts, ctx);
  }
}

fn push_attr(name: &str, value: &str, _selected: bool, parts: &mut Vec<El>, ctx: &Ctx<TreePanelMsg>) {
  parts.push(el::span().class(ctx.scoped("attr-name")).text(format!(" {}", name)));
  if !value.is_empty() {
    parts.push(el::span().class(ctx.scoped("equals")).text("="));
    parts.push(el::span().class(ctx.scoped("attr-val")).text(format!("\"{}\"", value)));
  }
}

fn is_void_element(tag: &str) -> bool {
  matches!(
    tag,
    "area" | "base" | "br" | "col" | "embed" | "hr" | "img"
      | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr"
  )
}

fn auto_expand(node: &Node, path: &[usize], depth: usize, max_depth: usize, out: &mut HashSet<Vec<usize>>) {
  let tag = node.element.tag_name();
  if tag == "#text" || tag == "style" || tag == "head" {
    return;
  }
  if depth >= max_depth {
    return;
  }
  out.insert(path.to_vec());
  for (i, child) in node.children.iter().enumerate() {
    let mut child_path = path.to_vec();
    child_path.push(i);
    auto_expand(child, &child_path, depth + 1, max_depth, out);
  }
}

pub fn build_breadcrumb(
  selected: &Option<Vec<usize>>,
  tree: Option<&Tree>,
  scope: &str,
) -> El {
  let mut crumbs: Vec<El> = Vec::new();

  if let (Some(path), Some(tree)) = (selected, tree) {
    for i in 0..=path.len() {
      let ancestor_path = &path[..i];
      let node = if ancestor_path.is_empty() {
        tree.root.as_ref()
      } else {
        tree.root.as_ref().and_then(|r| r.at_path(ancestor_path))
      };
      if let Some(n) = node {
        let tag = n.element.tag_name();
        if tag == "#text" || tag == "head" || tag == "style" {
          continue;
        }
        if !crumbs.is_empty() {
          crumbs.push(
            lucide(ICON_CHEVRON_RIGHT)
              .class(format!("{scope}__crumb-sep")),
          );
        }
        let is_last = i == path.len();
        let label = crumb_label(n);
        crumbs.push(
          el::span()
            .class(if is_last {
              format!("{scope}__crumb {scope}__active")
            } else {
              format!("{scope}__crumb")
            })
            .text(label),
        );
      }
    }
  }

  div().class(format!("{scope}__breadcrumb")).children(crumbs)
}

fn crumb_label(node: &Node) -> String {
  let tag = node.element.tag_name();
  let mut label = tag.to_string();
  if let Some(id) = node.element.id() {
    label.push('#');
    label.push_str(id);
  } else if let Some(class) = node.element.class() {
    if let Some(first) = class.split_ascii_whitespace().next() {
      label.push('.');
      label.push_str(first);
    }
  }
  label
}
