//! Root devtools component — orchestrates child components.
//!
//! `type Env = Tree` — receives `&Tree` (the host tree) in `view()`.

use std::{cell::RefCell, collections::HashSet, sync::{Arc, Mutex}};

use wgpu_html_models::Style;
use wgpu_html_style::cascade;
use wgpu_html_tree::Tree;
use wgpu_html_ui::{Component, Ctx, ShouldRender, el, el::El};

use crate::{
  breadcrumb::{BreadcrumbBar, BreadcrumbProps},
  styles_panel::{StylesPanel, StylesPanelProps},
  tree_panel::{TreePanel, TreePanelProps},
};

// ── Props / Msg ─────────────────────────────────────────────────────────────

pub type SharedHoverPath = Arc<Mutex<Option<Vec<usize>>>>;

pub type SharedPickMode = Arc<std::sync::atomic::AtomicBool>;
pub type SharedPendingPick = Arc<Mutex<Option<Vec<usize>>>>;

#[derive(Clone)]
pub struct DevtoolsProps {
  pub shared_hover: SharedHoverPath,
  pub shared_pick_mode: SharedPickMode,
  pub shared_pending_pick: SharedPendingPick,
}

#[derive(Clone)]
pub enum DevtoolsMsg {
  SelectRow(Vec<usize>),
  ToggleCollapse(Vec<usize>),
  HoverRow(Option<Vec<usize>>),
  TogglePickMode,
  DividerDragStart(f32),
  DividerDragMove(f32),
  DividerDragEnd,
}

// ── Component ───────────────────────────────────────────────────────────────

pub struct DevtoolsComponent {
  selected_path: Option<Vec<usize>>,
  pub(crate) hovered_path: Option<Vec<usize>>,
  pick_mode: bool,
  collapsed: HashSet<Vec<usize>>,
  auto_collapse_depth: usize,
  split_ratio: f32,
  drag_start_x: Option<f32>,
  drag_start_ratio: f32,
  cached_style: RefCell<Option<(Vec<usize>, Style)>>,
}

impl Component for DevtoolsComponent {
  type Props = DevtoolsProps;
  type Msg = DevtoolsMsg;
  type Env = Tree;

  fn create(_props: &DevtoolsProps) -> Self {
    Self {
      selected_path: None,
      hovered_path: None,
      pick_mode: false,
      collapsed: HashSet::new(),
      auto_collapse_depth: 2,
      split_ratio: 0.5,
      drag_start_x: None,
      drag_start_ratio: 0.5,
      cached_style: RefCell::new(None),
    }
  }

  fn update(&mut self, msg: DevtoolsMsg, props: &DevtoolsProps) -> ShouldRender {
    match msg {
      DevtoolsMsg::SelectRow(ref path) => {
        self.selected_path = Some(path.clone());
        self.cached_style.borrow_mut().take();
      }
      DevtoolsMsg::ToggleCollapse(ref path) => {
        if !self.collapsed.remove(path) {
          self.collapsed.insert(path.clone());
        }
      }
      DevtoolsMsg::HoverRow(ref path) => {
        self.hovered_path = path.clone();
        if let Ok(mut shared) = props.shared_hover.lock() {
          *shared = path.clone();
        }
        return ShouldRender::No;
      }
      DevtoolsMsg::TogglePickMode => {
        self.pick_mode = !self.pick_mode;
        props.shared_pick_mode.store(self.pick_mode, std::sync::atomic::Ordering::Relaxed);
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

  fn view(&self, props: &DevtoolsProps, ctx: &Ctx<DevtoolsMsg>, env: &Tree) -> El {
    // Check for a pending pick from the host window
    if let Ok(mut pending) = props.shared_pending_pick.lock() {
      if let Some(path) = pending.take() {
        // Can't mutate self in view, so send a message
        let sender = ctx.sender();
        sender.send(DevtoolsMsg::SelectRow(path));
        sender.send(DevtoolsMsg::TogglePickMode);
      }
    }

    let sender = ctx.sender();
    let select_sender = sender.clone();
    let toggle_sender = sender.clone();
    let hover_sender = sender.clone();

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
      on_hover: Arc::new(move |path| {
        hover_sender.send(DevtoolsMsg::HoverRow(path));
      }),
    };

    let breadcrumb_props = BreadcrumbProps {
      selected_path: self.selected_path.clone(),
    };

    let cascaded_style = self.selected_path.as_deref().and_then(|path| {
      // Re-use the cached cascaded style when the selected element
      // hasn't changed.  This avoids a full host-tree cascade on
      // every frame during drag / hover.
      {
        let cache = self.cached_style.borrow();
        if let Some((cached_path, style)) = cache.as_ref() {
          if cached_path.as_slice() == path {
            return Some(style.clone());
          }
        }
      }
      let cascaded = cascade(env);
      let root = cascaded.root.as_ref()?;
      let node = if path.is_empty() { root } else { root.at_path(path)? };
      let style = node.style.clone();
      self.cached_style.borrow_mut().replace((path.to_vec(), style.clone()));
      Some(style)
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
      el::body().child(
        el::div().class("devtools-root").children([
          Self::toolbar(self.pick_mode, ctx),
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
        ]),
      ),
    ])
  }
}

// ── Static view helpers ─────────────────────────────────────────────────────

impl DevtoolsComponent {
  fn toolbar(pick_mode: bool, ctx: &Ctx<DevtoolsMsg>) -> El {
    let pick_class = if pick_mode { "pick-btn pick-active" } else { "pick-btn" };
    let pick_cb = ctx.on_click(DevtoolsMsg::TogglePickMode);
    el::div().class("toolbar").children([
      el::span()
        .class(pick_class)
        .text("\u{e202}")
        .on_click(move |ev| { pick_cb(ev); }),
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
