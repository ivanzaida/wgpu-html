use wgpu_html_models::common::{AlignItems, Display, FontStyle, FontWeight, Overflow, WhiteSpace};
use wgpu_html_models::Style;
use wgpu_html_tree::Element;
use wgpu_html_ui::{
  el::{self, div},
  style::{self, pct, px, Stylesheet},
  Component, Ctx, El, ShouldRender,
};

use super::store::DevtoolsStore;
use super::theme::Theme;

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
        .height(pct(100))
        .background_color(Theme::BG_PRIMARY)
        .font_family("monospace")
        .font_size(px(11))
        .overflow_y(Overflow::Auto),
      // ── Tab bar ──
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
      // ── Scrollable content ──
      style::rule(".content")
        .flex_grow(1.0)
        .overflow_y(Overflow::Auto),
      // ── Rule block ──
      style::rule(".rule")
        .display(Display::Flex)
        .prop("flex-direction", "column")
        .width(pct(100))
        .border_bottom(format!("1px solid {}", Theme::BORDER)),
      style::rule(".rule-hdr")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(22))
        .padding_vh(px(0), px(12))
        .gap(px(6))
        .width(pct(100)),
      style::rule(".rule-end")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(18))
        .padding_vh(px(0), px(12)),
      // ── Declaration row ──
      style::rule(".decl")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(18))
        .padding_left(px(28))
        .padding_right(px(12))
        .white_space(WhiteSpace::Nowrap),
      // ── Syntax colors ──
      style::rule(".sel")
        .color(Theme::SELECTOR)
        .font_weight(FontWeight::Weight(500)),
      style::rule(".brace")
        .color(Theme::PROPERTY),
      style::rule(".prop")
        .color(Theme::PROPERTY),
      style::rule(".val")
        .color(Theme::VALUE),
      style::rule(".unit")
        .color(Theme::UNIT),
      style::rule(".punct")
        .color(Theme::PROPERTY),
      style::rule(".file")
        .color(Theme::ACCENT_BLUE)
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(10)),
      style::rule(".ua-file")
        .color(Theme::TEXT_MUTED)
        .font_family("Inter, system-ui, sans-serif")
        .font_size(px(10))
        .font_style(FontStyle::Italic),
      style::rule(".spacer")
        .flex_grow(1.0),
      style::rule(".chevron")
        .width(px(11))
        .height(px(11))
        .font_size(px(11))
        .color(Theme::TEXT_SECONDARY)
        .prop("line-height", "11px"),
      style::rule(".empty")
        .color(Theme::TEXT_MUTED)
        .padding(px(12))
        .font_family("Inter, system-ui, sans-serif"),
    ])
      .scoped("styles")
  }

  fn view(&self, props: &StylesPanelProps, ctx: &Ctx<StylesPanelMsg>) -> El {
    let selected = props.store.selected_path.get();
    let cascaded = props.store.cascaded.get();

    let content = if let (Some(path), Some(cascaded)) = (&selected, &cascaded) {
      let cnode = cascaded.root.as_ref().and_then(|r| {
        if path.is_empty() {
          Some(r)
        } else {
          r.at_path(path)
        }
      });
      if let Some(cnode) = cnode {
        build_styles_content(&cnode.style, &cnode.element, ctx)
      } else {
        vec![el::span().class(ctx.scoped("empty")).text("No element selected")]
      }
    } else {
      vec![el::span().class(ctx.scoped("empty")).text("No element selected")]
    };

    div().class(ctx.scoped("panel")).children([
      div().class(ctx.scoped("tabs")).children([
        el::span().class(format!("{} {}", ctx.scoped("tab"), ctx.scoped("active"))).text("Styles"),
        el::span().class(ctx.scoped("tab")).text("Computed"),
        el::span().class(ctx.scoped("tab")).text("Layout"),
      ]),
      div().class(ctx.scoped("content")).children(content),
    ])
  }
}

fn build_styles_content(style: &Style, element: &Element, ctx: &Ctx<StylesPanelMsg>) -> Vec<El> {
  let mut blocks: Vec<El> = Vec::new();

  // element.style block (inline styles)
  blocks.push(build_rule_block(
    "element.style",
    None,
    &collect_inline_style(element),
    ctx,
  ));

  // Computed styles as a single rule block
  let computed = collect_computed(style);
  if !computed.is_empty() {
    let selector = element.tag_name();
    blocks.push(build_rule_block(selector, None, &computed, ctx));
  }

  blocks
}

fn build_rule_block(
  selector: &str,
  source: Option<&str>,
  declarations: &[(String, String)],
  ctx: &Ctx<StylesPanelMsg>,
) -> El {
  let mut children: Vec<El> = Vec::new();

  // Header: chevron + selector + { + spacer + source
  let mut hdr_parts: Vec<El> = vec![
    el::span().class(ctx.scoped("sel")).text(selector),
    el::span().class(ctx.scoped("brace")).text(" {"),
  ];
  if let Some(src) = source {
    hdr_parts.push(div().class(ctx.scoped("spacer")));
    hdr_parts.push(el::span().class(ctx.scoped("file")).text(src));
  }
  children.push(div().class(ctx.scoped("rule-hdr")).children(hdr_parts));

  // Declarations
  for (prop, val) in declarations {
    children.push(build_declaration(prop, val, ctx));
  }

  // Closing brace
  children.push(
    div()
      .class(ctx.scoped("rule-end"))
      .children([el::span().class(ctx.scoped("brace")).text("}")]),
  );

  div().class(ctx.scoped("rule")).children(children)
}

fn build_declaration(prop: &str, value: &str, ctx: &Ctx<StylesPanelMsg>) -> El {
  div().class(ctx.scoped("decl")).children([
    el::span().class(ctx.scoped("prop")).text(prop),
    el::span().class(ctx.scoped("punct")).text(": "),
    el::span().class(ctx.scoped("val")).text(value),
    el::span().class(ctx.scoped("punct")).text(";"),
  ])
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

fn collect_computed(style: &Style) -> Vec<(String, String)> {
  let mut out = Vec::new();
  macro_rules! prop {
    ($name:literal, $field:expr) => {
      if let Some(v) = &$field {
        out.push(($name.to_string(), format!("{}", v)));
      }
    };
  }
  prop!("display", style.display);
  prop!("position", style.position);
  prop!("width", style.width);
  prop!("height", style.height);
  prop!("min-width", style.min_width);
  prop!("min-height", style.min_height);
  prop!("max-width", style.max_width);
  prop!("max-height", style.max_height);
  prop!("margin", style.margin);
  prop!("margin-top", style.margin_top);
  prop!("margin-right", style.margin_right);
  prop!("margin-bottom", style.margin_bottom);
  prop!("margin-left", style.margin_left);
  prop!("padding", style.padding);
  prop!("padding-top", style.padding_top);
  prop!("padding-right", style.padding_right);
  prop!("padding-bottom", style.padding_bottom);
  prop!("padding-left", style.padding_left);
  prop!("color", style.color);
  prop!("background-color", style.background_color);
  prop!("border", style.border);
  prop!("font-family", style.font_family);
  prop!("font-size", style.font_size);
  prop!("font-weight", style.font_weight);
  prop!("font-style", style.font_style);
  prop!("line-height", style.line_height);
  prop!("letter-spacing", style.letter_spacing);
  prop!("text-align", style.text_align);
  prop!("text-transform", style.text_transform);
  prop!("white-space", style.white_space);
  prop!("overflow", style.overflow);
  prop!("overflow-x", style.overflow_x);
  prop!("overflow-y", style.overflow_y);
  prop!("opacity", style.opacity);
  prop!("visibility", style.visibility);
  prop!("z-index", style.z_index);
  prop!("flex-direction", style.flex_direction);
  prop!("flex-wrap", style.flex_wrap);
  prop!("justify-content", style.justify_content);
  prop!("align-items", style.align_items);
  prop!("align-self", style.align_self);
  prop!("gap", style.gap);
  prop!("flex-grow", style.flex_grow);
  prop!("flex-shrink", style.flex_shrink);
  prop!("flex-basis", style.flex_basis);
  prop!("cursor", style.cursor);
  prop!("user-select", style.user_select);
  prop!("box-sizing", style.box_sizing);
  prop!("border-radius", style.border_top_left_radius);
  for (k, v) in &style.custom_properties {
    out.push((k.to_string(), v.to_string()));
  }
  out
}
