use std::sync::Arc;

use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

#[derive(Clone)]
pub struct ToolbarProps {
  pub pick_mode: bool,
  pub on_pick_toggle: Arc<dyn Fn() + Send + Sync>,
}

#[derive(Clone)]
pub enum ToolbarMsg {
  PickToggle,
}

pub struct Toolbar;

impl Component for Toolbar {
  type Props = ToolbarProps;
  type Msg = ToolbarMsg;

  fn create(_props: &ToolbarProps) -> Self {
    Toolbar
  }

  fn update(&mut self, msg: ToolbarMsg, props: &ToolbarProps) -> ShouldRender {
    match msg {
      ToolbarMsg::PickToggle => (props.on_pick_toggle)(),
    }
    ShouldRender::No
  }

  fn view(&self, props: &ToolbarProps, ctx: &Ctx<ToolbarMsg>) -> El {
    let pick_class = if props.pick_mode { "pick-btn pick-active" } else { "pick-btn" };
    let pick_cb = ctx.on_click(ToolbarMsg::PickToggle);
    el::div().class("toolbar").children([
      el::span().class(pick_class).text("\u{e202}").on_click(move |ev| { pick_cb(ev); }),
      el::div().class("tb-divider"),
      el::div().class("filter").children([
        el::span().class("filter-icon").text("\u{e0dc}"),
        el::span().class("filter-text").text("Filter"),
      ]),
    ])
  }
}
