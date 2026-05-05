use std::sync::Arc;
use wgpu_html_ui::{Component, Ctx, El, ShouldRender};

#[derive(Clone)]
pub struct SearchBarProps {
  on_change: Arc<dyn Fn(String)>,
}

pub struct SearchBar {
  search: String,
}

impl Component for SearchBar {
  type Props = SearchBarProps;
  type Msg = ();
  type Env = ();

  fn create(props: &Self::Props) -> Self {
    Self {
      search: String::new(),
    }
  }

  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender {
    todo!()
  }

  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El {
    todo!()
  }
}
