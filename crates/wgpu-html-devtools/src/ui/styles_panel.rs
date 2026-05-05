use wgpu_html_ui::{Component, Ctx, El, ShouldRender};

pub struct StylesPanel;

impl Component for StylesPanel {
  type Props = ();
  type Msg = ();
  type Env = ();

  fn create(props: &Self::Props) -> Self {
    todo!()
  }

  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender {
    todo!()
  }

  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El {
    todo!()
  }
}
