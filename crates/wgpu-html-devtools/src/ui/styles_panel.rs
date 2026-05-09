use super::store::DevtoolsStore;
use super::style_rule::{StyleRule, StyleRuleProps};
use super::theme::Theme;
use wgpu_html_models::common::{AlignItems, Display, FontWeight, Overflow};
use wgpu_html_tree::Element;
use wgpu_html_ui::style::Val;
use wgpu_html_ui::{
  el::{self, div},
  style::{self, pct, px, Stylesheet},
  Component, Ctx, El, ShouldRender,
};

const ELEM_MARGIN: Val = Val::Px(4f32);

#[derive(Clone)]
pub struct StylesPanelProps {
  pub store: DevtoolsStore,
}

#[derive(Clone)]
pub enum StylesPanelMsg {}

pub struct StylesPanel;

impl Component for StylesPanel {
  type Props = StylesPanelProps;
  type Msg = StylesPanelMsg;

  fn create(_props: &StylesPanelProps) -> Self {
    Self
  }

  fn update(&mut self, msg: StylesPanelMsg, _props: &StylesPanelProps) -> ShouldRender {
    match msg {}
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".panel")
        .display(Display::Flex)
        .prop("flex-direction", "column")
        .flex_grow(1.0)
        .min_width(px(0))
        .height(pct(100))
        .background_color(Theme::BG_PRIMARY)
        .font_family("monospace")
        .font_size(px(11))
        .overflow_x(Overflow::Hidden)
        .overflow_y(Overflow::Auto),
      style::rule(".tabs")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(28))
        .padding_vh(px(0), px(12))
        .gap(px(16))
        .background_color(Theme::BG_SECONDARY)
        .border_bottom(format!("1px solid {}", Theme::BORDER))
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(11))
        .flex_shrink(0.0),
      style::rule(".tab")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(pct(100))
        .padding_vh(px(0), px(4))
        .color(Theme::TEXT_SECONDARY)
        .font_weight(FontWeight::Weight(500)),
      style::rule(".tab.active")
        .color(Theme::TEXT_PRIMARY)
        .font_weight(FontWeight::Weight(600))
        .border_bottom(format!("2px solid {}", Theme::ACCENT_BLUE)),
      style::rule(".content")
        .flex_grow(1.0)
        .min_width(px(0))
        .overflow_x(Overflow::Hidden)
        .overflow_y(Overflow::Auto),
      style::rule(".inherited-hdr")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(22))
        .padding_vh(px(0), px(12))
        .color(Theme::TEXT_MUTED)
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(10))
        .border_bottom(format!("1px solid {}", Theme::BORDER)),
      style::rule(".inherited-tag")
        .color(Theme::TEXT_SECONDARY)
        .margin_left(ELEM_MARGIN),
      style::rule(".empty")
        .color(Theme::TEXT_MUTED)
        .padding(px(12))
        .font_family("Inter, system-ui, sans-serif"),
    ])
    .scoped("styles")
  }

  fn view(&self, props: &StylesPanelProps, ctx: &Ctx<StylesPanelMsg>) -> El {
    let selected = props.store.selected_path.get();

    let content = if let Some(path) = &selected {
      if let Some(host_tree) = props.store.host_tree() {
        let inspection = wgpu_html_style::InspectionContext::new(host_tree);
        build_styles_content(host_tree, &inspection, path, ctx)
      } else {
        vec![el::span().class(ctx.scoped("empty")).text("No element selected")]
      }
    } else {
      vec![el::span().class(ctx.scoped("empty")).text("No element selected")]
    };

    div().class(ctx.scoped("panel")).children([
      div().class(ctx.scoped("tabs")).children([
        el::span()
          .class(format!("{} {}", ctx.scoped("tab"), ctx.scoped("active")))
          .text("Styles"),
        el::span().class(ctx.scoped("tab")).text("Computed"),
        el::span().class(ctx.scoped("tab")).text("Layout"),
      ]),
      div().class(ctx.scoped("content")).children(content),
    ])
  }
}

fn rule_el(
  selector: &str,
  source: Option<&str>,
  is_ua: bool,
  declarations: &[(String, String)],
  ctx: &Ctx<StylesPanelMsg>,
) -> El {
  ctx.child::<StyleRule>(StyleRuleProps {
    selector: selector.to_string(),
    source: source.map(|s| s.to_string()),
    is_ua,
    declarations: declarations.to_vec(),
  })
}

fn build_styles_content(
  tree: &wgpu_html_tree::Tree,
  inspection: &wgpu_html_style::InspectionContext,
  path: &[usize],
  ctx: &Ctx<StylesPanelMsg>,
) -> Vec<El> {
  let mut blocks: Vec<El> = Vec::new();

  if let Some(node) = tree.root.as_ref().and_then(|r| r.at_path(path)) {
    let inline = collect_inline_style(&node.element);
    blocks.push(rule_el("element.style", None, false, &inline, ctx));
  }

  let matched = inspection.matched_rules(tree, path);
  for rule in &matched {
    blocks.push(rule_el(
      &rule.selector,
      Some(&rule.source),
      rule.is_ua,
      &rule.declarations,
      ctx,
    ));
  }

  let mut ancestor_path = path.to_vec();
  while !ancestor_path.is_empty() {
    ancestor_path.pop();
    let ancestor_node = tree.root.as_ref().and_then(|r| r.at_path(&ancestor_path));
    let Some(ancestor_node) = ancestor_node else { break };

    let ancestor_matched = inspection.matched_rules(tree, &ancestor_path);
    let inherited_rules: Vec<&wgpu_html_style::MatchedRuleInfo> = ancestor_matched
      .iter()
      .filter(|r| r.declarations.iter().any(|(p, _)| is_inherited_property(p)))
      .collect();
    if inherited_rules.is_empty() {
      continue;
    }

    let tag = ancestor_node.element.tag_name();
    let class_attr = ancestor_node.element.attr("class");
    let label = match &class_attr {
      Some(c) if !c.is_empty() => {
        format!("{}.{}", tag, c.split_whitespace().collect::<Vec<_>>().join("."))
      }
      _ => tag.to_string(),
    };

    blocks.push(
      div().class(ctx.scoped("inherited-hdr")).children([
        el::span().text("Inherited from"),
        el::span()
          .class(ctx.scoped("inherited-tag"))
          .text(label.as_str()),
      ]),
    );

    for rule in inherited_rules {
      let inherited_decls: Vec<(String, String)> = rule
        .declarations
        .iter()
        .filter(|(p, _)| is_inherited_property(p))
        .cloned()
        .collect();
      blocks.push(rule_el(
        &rule.selector,
        Some(&rule.source),
        rule.is_ua,
        &inherited_decls,
        ctx,
      ));
    }

    let ancestor_inline = collect_inline_style(&ancestor_node.element);
    let inherited_inline: Vec<(String, String)> = ancestor_inline
      .into_iter()
      .filter(|(p, _)| is_inherited_property(p))
      .collect();
    if !inherited_inline.is_empty() {
      blocks.push(rule_el("element.style", None, false, &inherited_inline, ctx));
    }
  }

  blocks
}

fn is_inherited_property(prop: &str) -> bool {
  matches!(
    prop,
    "color"
      | "accent-color"
      | "font-family"
      | "font-size"
      | "font-weight"
      | "font-style"
      | "line-height"
      | "letter-spacing"
      | "text-align"
      | "text-transform"
      | "text-decoration"
      | "white-space"
      | "visibility"
      | "cursor"
      | "user-select"
      | "pointer-events"
      | "list-style-type"
      | "list-style-position"
  )
}

fn collect_inline_style(element: &Element) -> Vec<(String, String)> {
  let style_str = match element {
    Element::Text(_) => return Vec::new(),
    other => other.attr("style"),
  };
  let Some(raw) = style_str else {
    return Vec::new();
  };
  raw
    .split(';')
    .filter_map(|decl| {
      let decl = decl.trim();
      if decl.is_empty() {
        return None;
      }
      let (p, v) = decl.split_once(':')?;
      Some((p.trim().to_string(), v.trim().to_string()))
    })
    .collect()
}
