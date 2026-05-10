use lui_models::common::{
  AlignItems, BoxSizing, Display, FlexDirection, InputType, JustifyContent::SpaceBetween,
};
use lui_ui::{Component, Ctx, El, InputAttrs, ShouldRender, el, style, style::Stylesheet};

pub struct TextInput {
  value: String,
}

impl Component for TextInput {
  type Props = ();
  type Msg = ();

  fn create(_props: &Self::Props) -> Self {
    Self { value: "jopa".into() }
  }

  fn update(&mut self, _msg: Self::Msg, _props: &Self::Props) -> ShouldRender {
    ShouldRender::Yes
  }

  fn view(&self, _props: &Self::Props, ctx: &Ctx<Self::Msg>) -> El {
    el::div()
      .class(ctx.scoped("wrapper"))
      .child(
        el::input()
          .input_type(InputType::Text)
          .placeholder("test")
          .value(self.value.as_str()),
      )
      .child(
        el::p()
          .class(ctx.scoped("value"))
          .text(format!("You typed: {}", self.value).as_str()),
      )
  }

  fn styles() -> Stylesheet {
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
    ]).scoped("text-input")
  }
}
