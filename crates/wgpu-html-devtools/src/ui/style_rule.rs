use super::theme::Theme;
use wgpu_html_models::common::{AlignItems, Display, FontStyle, FontWeight, Overflow, WhiteSpace};
use wgpu_html_ui::style::Val;
use wgpu_html_ui::{
  el::{self, div},
  style::{self, px, Stylesheet},
  Component, Ctx, El, ShouldRender,
};

const ELEM_MARGIN: Val = Val::Px(4f32);

#[derive(Clone)]
pub struct StyleRuleProps {
  pub selector: String,
  pub source: Option<String>,
  pub is_ua: bool,
  pub declarations: Vec<(String, String)>,
}

#[derive(Clone)]
pub enum StyleRuleMsg {}

pub struct StyleRule;

impl Component for StyleRule {
  type Props = StyleRuleProps;
  type Msg = StyleRuleMsg;

  fn create(_props: &StyleRuleProps) -> Self {
    Self
  }

  fn update(&mut self, msg: StyleRuleMsg, _props: &StyleRuleProps) -> ShouldRender {
    match msg {}
  }

  fn view(&self, props: &StyleRuleProps, ctx: &Ctx<StyleRuleMsg>) -> El {
    let mut children: Vec<El> = Vec::new();

    let mut hdr_parts: Vec<El> = vec![
      el::span().class(ctx.scoped("sel")).text(props.selector.as_str()),
      el::span().class(ctx.scoped("brace")).text("{"),
    ];
    if let Some(label) = &props.source {
      hdr_parts.push(div().class(ctx.scoped("spacer")));
      let class = if props.is_ua {
        ctx.scoped("ua-file")
      } else {
        ctx.scoped("file")
      };
      hdr_parts.push(el::span().class(class).text(label.as_str()));
    }
    children.push(div().class(ctx.scoped("rule-hdr")).children(hdr_parts));

    for (prop, val) in &props.declarations {
      children.push(
        div().class(ctx.scoped("decl")).children([
          el::span().class(ctx.scoped("prop")).text(prop.as_str()),
          el::span().class(ctx.scoped("punct")).text(":"),
          el::span().class(ctx.scoped("val")).text(val.as_str()),
          el::span().class(ctx.scoped("punct")).text(";"),
        ]),
      );
    }

    children.push(
      div()
        .class(ctx.scoped("rule-end"))
        .children([el::span().class(ctx.scoped("brace")).text("}")]),
    );

    div().class(ctx.scoped("rule")).children(children)
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".rule")
        .display(Display::Flex)
        .prop("flex-direction", "column")
        .border_bottom(format!("1px solid {}", Theme::BORDER)),
      style::rule(".rule-hdr")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(22))
        .padding_vh(px(0), px(12))
        .overflow_x(Overflow::Hidden),
      style::rule(".rule-end")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(18))
        .padding_vh(px(0), px(12)),
      style::rule(".decl")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(18))
        .padding_left(px(28))
        .padding_right(px(12))
        .white_space(WhiteSpace::Nowrap)
        .overflow_x(Overflow::Hidden),
      style::rule(".sel")
        .color(Theme::SELECTOR)
        .font_weight(FontWeight::Weight(500)),
      style::rule(".brace")
        .color(Theme::PROPERTY)
        .margin_left(ELEM_MARGIN.clone()),
      style::rule(".prop")
        .color(Theme::PROPERTY),
      style::rule(".val")
        .color(Theme::VALUE)
        .margin_left(ELEM_MARGIN.clone()),
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
    ])
      .scoped("srule")
  }
}
