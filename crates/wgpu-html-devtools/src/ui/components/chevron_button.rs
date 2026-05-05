//! Chevron toggle icon — pure display component driven by props.

use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

#[derive(Clone)]
pub struct ChevronProps {
  pub expanded: bool,
}

#[derive(Clone)]
pub enum ChevronMsg {}

pub struct ChevronButton;

impl Component for ChevronButton {
  type Props = ChevronProps;
  type Msg = ChevronMsg;
  type Env = ();

  fn create(_props: &ChevronProps) -> Self {
    ChevronButton
  }

  fn update(&mut self, msg: ChevronMsg, _props: &ChevronProps) -> ShouldRender {
    match msg {}
  }

  fn view(&self, props: &ChevronProps, _ctx: &Ctx<ChevronMsg>, _env: &()) -> El {
    let icon = if props.expanded { "\u{e06d}" } else { "\u{e06f}" };
    el::span().class("chevron").text(icon)
  }
}
