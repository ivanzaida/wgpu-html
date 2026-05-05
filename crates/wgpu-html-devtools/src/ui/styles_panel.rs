//! Styles panel component — shows computed styles of the selected element.

use wgpu_html_models::Style;
use wgpu_html_tree::Tree;
use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

use crate::style_extract::extract_grouped;

// ── Props / Msg ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct StylesPanelProps {
  pub selected_path: Option<Vec<usize>>,
  pub cascaded_style: Option<Style>,
}

// Stateless — no interactions.
#[derive(Clone)]
pub enum StylesPanelMsg {}

// ── Component ───────────────────────────────────────────────────────────────

pub struct StylesPanel;

impl Component for StylesPanel {
  type Props = StylesPanelProps;
  type Msg = StylesPanelMsg;
  type Env = Tree;

  fn create(_props: &StylesPanelProps) -> Self {
    StylesPanel
  }

  fn update(&mut self, msg: StylesPanelMsg, _props: &StylesPanelProps) -> ShouldRender {
    match msg {}
  }

  fn view(&self, props: &StylesPanelProps, _ctx: &Ctx<StylesPanelMsg>, env: &Tree) -> El {
    let selected_node = props.selected_path.as_deref().and_then(|path| {
      let root = env.root.as_ref()?;
      if path.is_empty() {
        Some(root)
      } else {
        root.at_path(path)
      }
    });

    let mut container = el::div().class("styles-content");

    if let Some(node) = selected_node {
      // element.style rule (inline styles)
      let mut element_style = el::div()
        .class("rule")
        .children([el::div().class("rule-header").children([
          el::span().class("selector-text").text("element.style"),
          el::span().class("brace").text(" {"),
        ])]);
      if let Some(style_str) = node.element.attr("style") {
        for decl in style_str.split(';') {
          let decl = decl.trim();
          if decl.is_empty() {
            continue;
          }
          if let Some((prop, value)) = decl.split_once(':') {
            element_style = element_style.child(make_decl_el(prop.trim(), value.trim()));
          }
        }
      }
      element_style = element_style.child(el::div().class("rule-end").child(el::text("}")));
      container = container.child(element_style);

      // Cascaded styles grouped by category
      if let Some(ref style) = props.cascaded_style {
        let groups = extract_grouped(style);
        for group in &groups {
          container = container.child(render_style_group(group));
        }
      }

      // Element info
      let tag = node.element.tag_name();
      let mut info_parts: Vec<String> = Vec::new();
      info_parts.push(format!("<{tag}>"));
      if let Some(id) = node.element.id() {
        info_parts.push(format!("id=\"{id}\""));
      }
      if let Some(cls) = node.element.class() {
        info_parts.push(format!("class=\"{cls}\""));
      }
      let info_text = info_parts.join("  ");

      let info_rule = el::div().class("rule").child(
        el::div()
          .class("rule-header")
          .child(el::span().class("selector-text").text(info_text)),
      );
      container = container.child(info_rule);
    } else {
      let placeholder = el::div().class("rule").style("padding: 12px;").child(
        el::span()
          .class("text-node")
          .text("Select an element to inspect its styles"),
      );
      container = container.child(placeholder);
    }

    container
  }
}

fn render_style_group(group: &crate::style_extract::CssDeclGroup) -> El {
  let mut rule = el::div()
    .class("rule")
    .children([el::div().class("rule-header").children([
      el::span().class("selector-text").text(group.label),
      el::span().class("brace").text(" {"),
    ])]);
  for decl in &group.decls {
    rule = rule.child(make_decl_el(&decl.property, &decl.value));
  }
  rule.child(el::div().class("rule-end").child(el::text("}")))
}

fn make_decl_el(prop: &str, value: &str) -> El {
  el::div().class("decl").children([
    el::div().class("cb"),
    el::span().class("prop").text(prop),
    el::span().class("colon").text(": "),
    el::span().class("val").text(value),
    el::span().class("semi").text(";"),
  ])
}
