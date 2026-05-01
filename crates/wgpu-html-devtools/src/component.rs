//! Root devtools component — orchestrates child components.
//!
//! `type Env = Tree` — receives `&Tree` (the host tree) in `view()`.

use std::collections::HashSet;
use std::sync::Arc;

use wgpu_html_tree::Tree;
use wgpu_html_ui::el::El;
use wgpu_html_ui::{el, Component, Ctx, ShouldRender};

use crate::breadcrumb::{BreadcrumbBar, BreadcrumbProps};
use crate::styles_panel::{StylesPanel, StylesPanelProps};
use crate::tree_panel::{TreePanel, TreePanelProps};

// ── Props / Msg ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DevtoolsProps;

#[derive(Clone)]
pub enum DevtoolsMsg {
    SelectRow(Vec<usize>),
    ToggleCollapse(Vec<usize>),
}

// ── Component ───────────────────────────────────────────────────────────────

pub struct DevtoolsComponent {
    selected_path: Option<Vec<usize>>,
    collapsed: HashSet<Vec<usize>>,
}

impl Component for DevtoolsComponent {
    type Props = DevtoolsProps;
    type Msg = DevtoolsMsg;
    type Env = Tree;

    fn create(_props: &DevtoolsProps) -> Self {
        Self {
            selected_path: None,
            collapsed: HashSet::new(),
        }
    }

    fn update(&mut self, msg: DevtoolsMsg, _props: &DevtoolsProps) -> ShouldRender {
        match msg {
            DevtoolsMsg::SelectRow(ref path) => {
                println!("[DevtoolsComponent::update] SelectRow({path:?})");
                self.selected_path = Some(path.clone());
            }
            DevtoolsMsg::ToggleCollapse(ref path) => {
                println!("[DevtoolsComponent::update] ToggleCollapse({path:?})");
                if !self.collapsed.remove(path) {
                    self.collapsed.insert(path.clone());
                }
            }
        }
        ShouldRender::Yes
    }

    fn view(&self, _props: &DevtoolsProps, ctx: &Ctx<DevtoolsMsg>, _env: &Tree) -> El {
        let sender = ctx.sender();
        let select_sender = sender.clone();
        let toggle_sender = sender;

        let tree_props = TreePanelProps {
            selected_path: self.selected_path.clone(),
            collapsed: self.collapsed.clone(),
            on_select: Arc::new(move |path| {
                select_sender.send(DevtoolsMsg::SelectRow(path));
            }),
            on_toggle: Arc::new(move |path| {
                toggle_sender.send(DevtoolsMsg::ToggleCollapse(path));
            }),
        };

        let breadcrumb_props = BreadcrumbProps {
            selected_path: self.selected_path.clone(),
        };

        let styles_props = StylesPanelProps {
            selected_path: self.selected_path.clone(),
        };

        el::html().children([
            el::head().child(
                el::link().configure(|l: &mut wgpu_html_models::Link| {
                    l.rel = Some("stylesheet".into());
                    l.href = Some("devtools.css".into());
                }),
            ),
            el::body().child(
                el::div().class("devtools-root").children([
                    Self::toolbar(),
                    el::div().class("main").children([
                        el::div().class("tree-panel").children([
                            ctx.child::<TreePanel>(tree_props),
                            ctx.child::<BreadcrumbBar>(breadcrumb_props),
                        ]),
                        el::div().class("styles-panel").children([
                            Self::tab_bar(),
                            Self::style_search(),
                            ctx.child::<StylesPanel>(styles_props),
                        ]),
                    ]),
                ]),
            ),
        ])
    }
}

// ── Static view helpers ─────────────────────────────────────────────────────

impl DevtoolsComponent {
    fn toolbar() -> El {
        el::div().class("toolbar").children([
            el::span().class("pick-btn").text("\u{e202}"),
            el::div().class("tb-divider"),
            el::div().class("filter").children([
                el::span().class("filter-icon").text("\u{e0dc}"),
                el::span().class("filter-text").text("Filter"),
            ]),
        ])
    }

    fn tab_bar() -> El {
        el::div().class("tab-bar").children([
            el::div()
                .class("tab tab-active")
                .style("height: 100%;")
                .text("Styles"),
            el::div()
                .class("tab")
                .style("height: 100%;")
                .text("Computed"),
            el::div()
                .class("tab")
                .style("height: 100%;")
                .text("Layout"),
            el::div()
                .class("tab")
                .style("height: 100%;")
                .text("Event Listeners"),
        ])
    }

    fn style_search() -> El {
        el::div().class("style-search").children([
            el::span().class("ss-label").text("Filter"),
            el::div().class("ss-spacer"),
            el::span().class("ss-btn ss-btn-active").text(":hov"),
            el::span().class("ss-btn").text(".cls"),
            el::span().class("ss-btn icon").text("\u{e13d}"),
        ])
    }
}
