use super::lucide_icon::lucide;
use super::theme::Theme;
use wgpu_html_models::common::{AlignItems, BoxSizing, Display, UserSelect};
use wgpu_html_models::ArcStr;
use wgpu_html_ui::{el::{self, div}, style::{self, pct, px, Stylesheet}, Component, Ctx, El, InputAttrs, MsgSender, Observable, ShouldRender, Subscriptions};

#[derive(Clone)]
pub struct ToolbarProps;

#[derive(Clone)]
pub enum ToolbarMsg {}

pub struct Toolbar {
  value: Observable<ArcStr>,
}

impl Component for Toolbar {
  type Props = ToolbarProps;
  type Msg = ToolbarMsg;

  fn create(_props: &ToolbarProps) -> Self {
    Toolbar {
      value: Observable::default(),
    }
  }

  fn update(&mut self, msg: ToolbarMsg, _props: &ToolbarProps) -> ShouldRender {
    match msg {}
  }


  fn view(&self, _props: &ToolbarProps, ctx: &Ctx<ToolbarMsg>) -> El {
    div().class(ctx.scoped("bar")).children([
      lucide("\u{E11F}").class(ctx.scoped("icon")),
      div().class(ctx.scoped("divider")),
      div().class(ctx.scoped("filter")).children([
        lucide("\u{E151}").class(ctx.scoped("filter-icon")),
        el::input()
          .bind(self.value.clone())
          .class(ctx.scoped("filter-input"))
          .placeholder("Filter"),
      ]),
    ])
  }

  fn subscribe(&self, _sender: &MsgSender<Self::Msg>, subs: &mut Subscriptions) {
    subs.add(self.value.subscribe(|value| {
      println!("Filter value changed: {}", value);
    }));
  }

  fn mounted(&mut self, _sender: MsgSender<Self::Msg>) {
    println!("Toolbar mounted");
  }

  fn destroyed(&mut self) {
    println!("Toolbar destroyed");
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".bar")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(30))
        .width(pct(100))
        .padding_vh(px(0), px(8))
        .gap(px(8))
        .background_color(Theme::BG_SECONDARY)
        .border_bottom(format!("1px solid {}", Theme::BORDER))
        .box_sizing(BoxSizing::BorderBox)
        .user_select(UserSelect::None)
        .font_size(px(11)),
      style::rule(".icon")
        .width(px(14))
        .height(px(14))
        .font_size(px(14))
        .color(Theme::ACCENT_BLUE)
        .prop("line-height", "14px"),
      style::rule(".divider")
        .width(px(1))
        .height(px(16))
        .background_color(Theme::DIVIDER),
      style::rule(".filter")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(px(22))
        .width(px(200))
        .padding_vh(px(0), px(8))
        .gap(px(6))
        .background_color(Theme::BG_TERTIARY)
        .border_radius(px(3))
        .box_sizing(BoxSizing::BorderBox),
      style::rule(".filter-icon")
        .width(px(12))
        .height(px(12))
        .font_size(px(12))
        .color(Theme::TEXT_MUTED)
        .prop("line-height", "12px"),
      style::rule(".filter-input")
        .flex_grow(1.0)
        .height(pct(100))
        .border("none")
        .background_color("transparent")
        .color(Theme::TEXT_PRIMARY)
        .font_size(px(11))
        .prop("outline", "none")
        .padding(px(0)),
    ]).scoped("toolbar")
  }
}
