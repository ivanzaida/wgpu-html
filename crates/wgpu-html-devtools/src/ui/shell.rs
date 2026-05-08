use wgpu_html_models::common::{Display, Overflow};
use wgpu_html_ui::{el, el::El, style, Component, Ctx, ShouldRender};

use super::store::DevtoolsStore;
use super::styles_panel::{StylesPanel, StylesPanelProps};
use super::top_bar::{Toolbar, ToolbarProps};
use super::tree_panel::{TreePanel, TreePanelProps};
use super::theme::Theme;

#[derive(Clone)]
pub struct DevtoolsProps {
  pub store: DevtoolsStore,
}

#[derive(Clone)]
pub enum DevtoolsMsg {}

pub struct DevtoolsComponent;

impl Component for DevtoolsComponent {
  type Props = DevtoolsProps;
  type Msg = DevtoolsMsg;

  fn create(_props: &DevtoolsProps) -> Self {
    Self
  }

  fn update(&mut self, msg: DevtoolsMsg, _props: &DevtoolsProps) -> ShouldRender {
    match msg {}
  }

  fn styles() -> style::Stylesheet {
    use wgpu_html_models::common::{AlignItems, FontWeight, WhiteSpace};
    style::sheet([
      style::rule(".root")
        .display(Display::Flex)
        .prop("flex-direction", "column")
        .width(style::pct(100))
        .height(style::pct(100))
        .background_color(Theme::BG_PRIMARY)
        .color(Theme::TEXT_PRIMARY)
        .font_family("Inter, system-ui, sans-serif")
        .font_size(style::px(12)),
      style::rule(".body")
        .display(Display::Flex)
        .flex_grow(1.0)
        .overflow(Overflow::Hidden),
      style::rule(".breadcrumb")
        .display(Display::Flex)
        .align_items(AlignItems::Center)
        .height(style::px(22))
        .padding_vh(style::px(0), style::px(12))
        .gap(style::px(4))
        .background_color(Theme::BG_SECONDARY)
        .border_top(format!("1px solid {}", Theme::BORDER))
        .font_size(style::px(10))
        .white_space(WhiteSpace::Nowrap)
        .flex_shrink(0.0),
      style::rule(".crumb")
        .color(Theme::TEXT_MUTED),
      style::rule(".crumb.active")
        .color(Theme::ACCENT_BLUE)
        .font_weight(FontWeight::Weight(600)),
      style::rule(".crumb-sep")
        .width(style::px(10))
        .height(style::px(10))
        .font_size(style::px(10))
        .color(Theme::TEXT_MUTED)
        .prop("line-height", "10px"),
    ]).scoped("devtools")
  }

  fn view(&self, props: &DevtoolsProps, ctx: &Ctx<DevtoolsMsg>) -> El {
    let selected = props.store.selected_path.get();
    let host = props.store.host_tree.get();
    let breadcrumb = super::tree_panel::build_breadcrumb(&selected, host.as_ref(), "devtools");

    el::div().class(ctx.scoped("root")).children([
      ctx.child::<Toolbar>(ToolbarProps),
      el::div().class(ctx.scoped("body")).children([
        ctx.child::<TreePanel>(TreePanelProps {
          store: props.store.clone(),
        }),
        ctx.child::<StylesPanel>(StylesPanelProps {
          store: props.store.clone(),
        }),
      ]),
      breadcrumb,
    ])
  }
}
