use wgpu_html_ui::{Component, Ctx, El, ShouldRender};

#[derive(Clone)]
pub struct TabBarTab {
  name: String,
}

pub struct TabBar {
  tabs: Vec<TabBarTab>,
  selected_tab_idx: u32,
}

impl Component for TabBar {
  type Props = Vec<TabBarTab>;
  type Msg = ();

  fn create(props: &Self::Props) -> Self {
    todo!()
  }

  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender {
    todo!()
  }

  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>) -> El {
    todo!()
  }
}
