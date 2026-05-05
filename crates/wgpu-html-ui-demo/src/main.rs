mod counter;
mod text_input;

use wgpu_html_models::common::css_enums::*;
use wgpu_html_ui::{
  App, Component, Ctx, El, ShouldRender, el,
  style::{self, px},
};

use crate::{
  counter::counter::{Counter, CounterProps},
  text_input::TextInput,
};
// ── Root App Component ──────────────────────────────────────────────────────

struct DemoApp;

#[derive(Clone)]
struct DemoProps;

#[derive(Clone)]
enum DemoMsg {}

impl Component for DemoApp {
  type Props = DemoProps;
  type Msg = DemoMsg;

  fn scope() -> &'static str {
    "app"
  }

  fn styles() -> style::Stylesheet {
    style::sheet([
      style::rule(".root")
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .flex_grow(1.0)
        .background_color("#1a1a1a")
        .font_family("sans-serif"),
      style::rule(".title")
        .font_size(px(20))
        .font_weight(FontWeight::Weight(600))
        .color("#8AB4F8")
        .margin_bottom(px(40))
        .white_space(WhiteSpace::Nowrap),
      style::rule(".counters").display(Display::Flex).gap(px(24)),
    ])
  }

  fn create(_props: &DemoProps) -> Self {
    DemoApp
  }

  fn update(&mut self, _msg: DemoMsg, _props: &DemoProps) -> ShouldRender {
    ShouldRender::No
  }

  fn view(&self, _props: &DemoProps, ctx: &Ctx<DemoMsg>) -> El {
    el::div().class(ctx.scoped("root")).children([
      el::div().class(ctx.scoped("title")).text("wgpu-html-ui Demo"),
      ctx.child::<TextInput>(()),
      el::div().class(ctx.scoped("counters")).children([
        ctx.child::<Counter>(CounterProps { label: "Clicks" }),
        ctx.child::<Counter>(CounterProps { label: "Score" }),
      ]),
    ])
  }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
  App::new::<DemoApp>(DemoProps)
    .title("wgpu-html-ui demo")
    .size(800, 500)
    .stylesheet("html, body { height: 100%; margin: 0; background: #1a1a1a; }")
    .stylesheet(include_str!("theme.css"))
    .with_secondary(|tree| Box::new(wgpu_html_devtools::Devtools::attach(tree, false)))
    .run()
    .unwrap();
}
