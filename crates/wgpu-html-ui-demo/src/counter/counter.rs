use wgpu_html_models::common::{
  AlignItems, BoxSizing, Cursor, Display, FlexDirection, FontWeight, JustifyContent, Overflow, UserSelect,
};
use wgpu_html_ui::{
  Component,
  el::{button, div},
  style::{self, Stylesheet, px},
};

#[derive(Clone)]
pub enum CounterMsg {
  Increment,
  Decrement,
  Reset,
}

pub struct Counter {
  count: i32,
  label: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct CounterProps {
  pub label: &'static str,
}

impl Component for Counter {
  type Props = CounterProps;
  type Msg = CounterMsg;
  type Env = ();

  fn create(props: &CounterProps) -> Self {
    Counter {
      count: 0,
      label: props.label,
    }
  }

  fn update(&mut self, msg: CounterMsg, _props: &CounterProps) -> wgpu_html_ui::ShouldRender {
    match msg {
      CounterMsg::Increment => self.count += 1,
      CounterMsg::Decrement => self.count -= 1,
      CounterMsg::Reset => self.count = 0,
    }
    wgpu_html_ui::ShouldRender::Yes
  }

  fn scope() -> &'static str {
    "counter"
  }

  fn styles() -> Stylesheet {
    style::sheet([
      style::rule(".root")
        .user_select(UserSelect::None)
        .display(Display::Flex)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .flex_direction(FlexDirection::Column)
        .width(px(160))
        .background_color("var(--bg-card)")
        .overflow(Overflow::Hidden)
        .box_sizing(BoxSizing::BorderBox)
        .border("1px solid var(--border-subtle)")
        .border_radius(px(12))
        .gap(px(12))
        .padding(px(24)),
      style::rule(".title").color("var(--text-secondary)").font_size(px(12)),
      style::rule(".value")
        .color("var(--text-primary)")
        .font_size(px(40))
        .font_weight(FontWeight::Bold),
      style::rule(".buttons")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::SpaceBetween)
        .gap(px(8)),
      style::rule("button")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .border_radius(px(6))
        .height(px(36))
        .width(px(36))
        .background_color("var(--bg-elevated)"),
      style::rule("button:hover")
        .background_color("var(--bg-hover)")
        .cursor(Cursor::Pointer),
      style::rule("button.wide").width(px(64)),
    ])
  }

  fn view(&self, _props: &CounterProps, ctx: &wgpu_html_ui::Ctx<CounterMsg>, _env: &()) -> wgpu_html_ui::El {
    div().class(ctx.scoped("root")).children([
      div().class(ctx.scoped("title")).text(self.label),
      div().class(ctx.scoped("value")).text(self.count.to_string()),
      div().class(ctx.scoped("buttons")).children([
        button().text("-").on_click_cb(ctx.on_click(CounterMsg::Decrement)),
        button()
          .class(ctx.scoped("wide"))
          .text("Reset")
          .on_click_cb(ctx.on_click(CounterMsg::Reset)),
        button().text("+").on_click_cb(ctx.on_click(CounterMsg::Increment)),
      ]),
    ])
  }
}
