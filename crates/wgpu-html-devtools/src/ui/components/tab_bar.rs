use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

#[derive(Clone)]
pub struct TabBarProps {
  pub active: usize,
}

#[derive(Clone)]
pub enum TabBarMsg {}

pub struct TabBar;

impl Component for TabBar {
  type Props = TabBarProps;
  type Msg = TabBarMsg;

  fn create(_props: &TabBarProps) -> Self {
    TabBar
  }

  fn update(&mut self, msg: TabBarMsg, _props: &TabBarProps) -> ShouldRender {
    match msg {}
  }

  fn view(&self, props: &TabBarProps, _ctx: &Ctx<TabBarMsg>) -> El {
    let tabs = ["Styles", "Computed", "Layout", "Event Listeners"];
    let mut bar = el::div().class("tab-bar");
    for (i, label) in tabs.iter().enumerate() {
      let class = if i == props.active { "tab tab-active" } else { "tab" };
      bar = bar.child(el::div().class(class).style("height: 100%;").text(*label));
    }
    bar
  }
}
