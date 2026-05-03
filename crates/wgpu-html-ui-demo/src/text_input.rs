use wgpu_html_models::common::{AlignItems, Display};
use wgpu_html_ui::style::Stylesheet;
use wgpu_html_ui::{el, style, Component, Ctx, El, ShouldRender};

pub struct TextInput {
  value: String,
}

impl Component for TextInput {
  type Props = ();
  type Msg = ();
  type Env = ();

  fn create(props: &Self::Props) -> Self {
    Self { value: String::new() }
  }

  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender {
    ShouldRender::Yes
  }

  fn scope() -> &'static str {
    "text-input"
  }

  fn styles() -> Stylesheet
  where
    Self: Sized,
  {
    style::sheet([
      style::rule("wrapper")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .gap(style::px(12)),
    ])
  }

  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El {
    el::div().class(ctx.scoped("wrapper"))
      .child(
        el::input()
      )
      .child(el::p().class(ctx.scoped("value")).text(&format!("You typed: {}", self.value)))
  }
}
