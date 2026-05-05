use wgpu_html_ui::{Component, Ctx, El, ShouldRender};

pub struct ChevronButton {
  expanded: bool,
}

#[derive(Clone)]
pub struct ChevronProps {
  pub expanded: bool,
}

#[derive(Clone)]
pub enum ChevronMsg {
  Expanded(bool)
}

impl Component for ChevronButton {
  type Props = ChevronProps;
  type Msg = ChevronMsg;
  type Env = ();

  fn create(props: &Self::Props) -> Self {
    Self {
      expanded: props.expanded
    }
  }

  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender {
    ShouldRender::Yes
  }

  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El {
    todo!()
  }
}
