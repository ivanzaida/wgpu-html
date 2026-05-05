use std::sync::Arc;

use wgpu_html_models::common::InputType;
use wgpu_html_tree::HtmlEvent;
use wgpu_html_ui::{el, el::El, Component, Ctx, InputAttrs, ShouldRender};

#[derive(Clone)]
pub struct SearchBarProps {
  pub on_change: Arc<dyn Fn(String) + Send + Sync>,
}

#[derive(Clone)]
pub enum SearchBarMsg {
  Input(String),
}

pub struct SearchBar {
  value: String,
}

impl Component for SearchBar {
  type Props = SearchBarProps;
  type Msg = SearchBarMsg;

  fn create(_props: &SearchBarProps) -> Self {
    Self { value: String::new() }
  }

  fn update(&mut self, msg: SearchBarMsg, props: &SearchBarProps) -> ShouldRender {
    match msg {
      SearchBarMsg::Input(v) => {
        self.value = v.clone();
        (props.on_change)(v);
      }
    }
    ShouldRender::Yes
  }

  fn view(&self, _props: &SearchBarProps, ctx: &Ctx<SearchBarMsg>) -> El {
    let sender = ctx.sender();
    el::div().class("style-search").children([
      el::input()
        .input_type(InputType::Text)
        .placeholder("Filter styles")
        .class("ss-input")
        .value(&self.value)
        .on_input(move |ev| {
          println!("input: {:?}", ev);
          if let HtmlEvent::Input(input_ev) = ev {
            if let Some(v) = &input_ev.value {
              sender.send(SearchBarMsg::Input(v.clone()));
            }
          }
        }),
      el::div().class("ss-spacer"),
      el::span().class("ss-btn ss-btn-active").text(":hov"),
      el::span().class("ss-btn").text(".cls"),
      el::span().class("ss-btn icon").text("\u{e13d}"),
    ])
  }
}
