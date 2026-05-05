//! Search / filter bar component for the styles panel.

use std::sync::Arc;

use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

#[derive(Clone)]
pub struct SearchBarProps {
  pub on_change: Arc<dyn Fn(String) + Send + Sync>,
}

pub struct SearchBar {
  search: String,
}

impl Component for SearchBar {
  type Props = SearchBarProps;
  type Msg = ();
  type Env = ();

  fn create(_props: &SearchBarProps) -> Self {
    Self { search: String::new() }
  }

  fn update(&mut self, _msg: (), _props: &SearchBarProps) -> ShouldRender {
    ShouldRender::No
  }

  fn view(&self, _props: &SearchBarProps, _ctx: &Ctx<()>, _env: &()) -> El {
    el::div().class("style-search").children([
      el::span().class("ss-label").text("Filter"),
      el::div().class("ss-spacer"),
      el::span().class("ss-btn ss-btn-active").text(":hov"),
      el::span().class("ss-btn").text(".cls"),
      el::span().class("ss-btn icon").text("\u{e13d}"),
    ])
  }
}
