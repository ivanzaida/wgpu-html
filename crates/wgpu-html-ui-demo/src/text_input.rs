use wgpu_html_models::common::JustifyContent::SpaceBetween;
use wgpu_html_models::common::{AlignItems, BoxSizing, Display, FlexDirection, InputType};
use wgpu_html_ui::style::Stylesheet;
use wgpu_html_ui::{el, style, Component, Ctx, El, InputAttrs, ShouldRender};

pub struct TextInput {
  value: String,
}

impl Component for TextInput {
  type Props = ();
  type Msg = ();
  type Env = ();

  fn create(props: &Self::Props) -> Self {
    Self { value: "jopa".into() }
  }

  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender {
    ShouldRender::Yes
  }

  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El {
    el::div().class(ctx.scoped("wrapper"))
      .child(el::input().input_type(InputType::Text).placeholder("test").value(&self.value))
      .child(el::p().class(ctx.scoped("value")).text(&format!("You typed: {}", self.value)))
  }

  fn scope() -> &'static str {
    "text-input"
  }

  fn styles() -> Stylesheet
  where
    Self: Sized,
  {
    style::sheet([
      style::rule(".wrapper")
        .width(style::px(240))
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Center),
      style::rule(".value")
        .width(style::px(240))
        .height(style::px(40))
        .background_color("var(--bg-card)")
        .border("1px solid var(--border-subtle)")
        .border_radius(style::px(12))
        .box_sizing(BoxSizing::BorderBox)
        .padding(style::px(12))
        .color("var(--text-primary)")
        .font_size(style::px(16))
        .font_family("sans-serif")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .justify_content(SpaceBetween),
    ])
  }
}
