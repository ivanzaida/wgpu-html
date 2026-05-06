//! Root devtools component.

use std::sync::{
  atomic::AtomicBool,
  Arc, Mutex, RwLock,
};
use wgpu_html_tree::Tree;
use wgpu_html_ui::{el, el::El, Component, Ctx, ShouldRender};

// ── Shared state ───────────────────────────────────────────────────────────

pub type SharedHoverPath = Arc<Mutex<Option<Vec<usize>>>>;
pub type SharedPickMode = Arc<AtomicBool>;
pub type SharedPendingPick = Arc<Mutex<Option<Vec<usize>>>>;
pub type SharedHostTree = Arc<RwLock<Option<Tree>>>;

// ── Props / Msg ────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DevtoolsProps {
  pub shared_hover: SharedHoverPath,
  pub shared_pick_mode: SharedPickMode,
  pub shared_pending_pick: SharedPendingPick,
  pub host_tree: SharedHostTree,
}

#[derive(Clone)]
pub enum DevtoolsMsg {}

// ── Component ──────────────────────────────────────────────────────────────

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

  fn view(&self, _props: &DevtoolsProps, _ctx: &Ctx<DevtoolsMsg>) -> El {
    el::html().children([
      el::head().child(el::link().configure(|l: &mut wgpu_html_models::Link| {
        l.rel = Some("stylesheet".into());
        l.href = Some("devtools.css".into());
      })),
      el::body(),
    ])
  }
}
