//! Root devtools component — orchestrates child components.
//!
//! `type Env = Tree` — receives `&Tree` (the host tree) in `view()`.

use std::{collections::HashSet, sync::Arc};

use wgpu_html_style::cascade;
use wgpu_html_tree::Tree;
use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

use crate::{
  breadcrumb::{BreadcrumbBar, BreadcrumbProps},
  styles_panel::{StylesPanel, StylesPanelProps},
  tree_panel::{TreePanel, TreePanelProps},
};

// ── Props / Msg ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DevtoolsProps;

#[derive(Clone)]
pub enum DevtoolsMsg {
  SelectRow(Vec<usize>),
  ToggleCollapse(Vec<usize>),
  DividerDragStart(f32),
  DividerDragMove(f32),
  DividerDragEnd,
}

// ── Component ───────────────────────────────────────────────────────────────

pub struct DevtoolsComponent {
  selected_path: Option<Vec<usize>>,
  collapsed: HashSet<Vec<usize>>,
  auto_collapse_depth: usize,
  /// Tree panel width as a fraction of the .main container (0.0–1.0).
  split_ratio: f32,
  /// X position when drag started (for computing delta).
  drag_start_x: Option<f32>,
  /// split_ratio at the moment drag started.
  drag_start_ratio: f32,
}

impl Component for DevtoolsComponent {
  type Props = DevtoolsProps;
  type Msg = DevtoolsMsg;
  type Env = Tree;

  fn create(_props: &DevtoolsProps) -> Self {
    Self {
      selected_path: None,
      collapsed: HashSet::new(),
      auto_collapse_depth: 2,
      split_ratio: 0.5,
      drag_start_x: None,
      drag_start_ratio: 0.5,
    }
  }

  fn update(&mut self, msg: DevtoolsMsg, _props: &DevtoolsProps) -> ShouldRender {
    match msg {
      DevtoolsMsg::SelectRow(ref path) => {
        self.selected_path = Some(path.clone());
      }
      DevtoolsMsg::ToggleCollapse(ref path) => {
        if !self.collapsed.remove(path) {
          self.collapsed.insert(path.clone());
        }
      }
      DevtoolsMsg::DividerDragStart(x) => {
        self.drag_start_x = Some(x);
        self.drag_start_ratio = self.split_ratio;
        return ShouldRender::No;
      }
      DevtoolsMsg::DividerDragMove(x) => {
        if let Some(start_x) = self.drag_start_x {
          // Estimate container width from viewport (close enough).
          let container_w = 1280.0_f32;
          let dx = x - start_x;
          let new_ratio = (self.drag_start_ratio + dx / container_w).clamp(0.15, 0.85);
          self.split_ratio = new_ratio;
        } else {
          return ShouldRender::No;
        }
      }
      DevtoolsMsg::DividerDragEnd => {
        self.drag_start_x = None;
        return ShouldRender::No;
      }
    }
    ShouldRender::Yes
  }

  fn view(&self, _props: &DevtoolsProps, ctx: &Ctx<DevtoolsMsg>, env: &Tree) -> El {
    let sender = ctx.sender();
    let select_sender = sender.clone();
    let toggle_sender = sender.clone();

    let tree_props = TreePanelProps {
      selected_path: self.selected_path.clone(),
      collapsed: self.collapsed.clone(),
      auto_collapse_depth: self.auto_collapse_depth,
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

    let cascaded_style = self.selected_path.as_deref().and_then(|path| {
      let cascaded = cascade(env);
      let root = cascaded.root.as_ref()?;
      let node = if path.is_empty() { root } else { root.at_path(path)? };
      Some(node.style.clone())
    });

    let styles_props = StylesPanelProps {
      selected_path: self.selected_path.clone(),
      cascaded_style,
    };

    // Divider callbacks
    let drag_start_sender = sender.clone();
    let drag_move_sender = sender.clone();
    let drag_end_sender = sender;

    let tree_width_pct = format!("{}%", self.split_ratio * 100.0);

    el::html().children([
      el::head().child(el::link().configure(|l: &mut wgpu_html_models::Link| {
        l.rel = Some("stylesheet".into());
        l.href = Some("devtools.css".into());
      })),
      el::body().child(el::div().class("devtools-root").children([
        Self::toolbar(),
        el::div()
          .class("main")
          // Track mouse move/up on the whole container so drag
          // continues even after the pointer leaves the 4px divider.
          .on_mouse_move(move |ev| {
            drag_move_sender.send(DevtoolsMsg::DividerDragMove(ev.pos.0));
          })
          .on_mouse_up(move |_| {
            drag_end_sender.send(DevtoolsMsg::DividerDragEnd);
          })
          .children([
            el::div()
              .class("tree-panel")
              .style(format!("width: {tree_width_pct}; min-width: 0;"))
              .children([
                ctx.child::<TreePanel>(tree_props),
                ctx.child::<BreadcrumbBar>(breadcrumb_props),
              ]),
            el::div()
              .class("divider")
              .on_mouse_down(move |ev| {
                drag_start_sender.send(DevtoolsMsg::DividerDragStart(ev.pos.0));
              }),
            el::div().class("styles-panel").children([
              Self::tab_bar(),
              Self::style_search(),
              ctx.child::<StylesPanel>(styles_props),
            ]),
          ]),
      ])),
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
      el::div().class("tab tab-active").style("height: 100%;").text("Styles"),
      el::div().class("tab").style("height: 100%;").text("Computed"),
      el::div().class("tab").style("height: 100%;").text("Layout"),
      el::div().class("tab").style("height: 100%;").text("Event Listeners"),
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
