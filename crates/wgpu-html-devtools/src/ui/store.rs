use std::sync::{atomic::AtomicBool, Arc};

use wgpu_html_style::CascadedTree;
use wgpu_html_tree::Tree;
use wgpu_html_ui::Observable;

#[derive(Clone)]
pub struct DevtoolsStore {
  pub host_tree: Observable<Option<Tree>>,
  pub cascaded: Observable<Option<CascadedTree>>,
  pub selected_path: Observable<Option<Vec<usize>>>,
  pub hover_path: Observable<Option<Vec<usize>>>,
  pub pick_mode: Arc<AtomicBool>,
  pub pending_pick: Observable<Option<Vec<usize>>>,
}

impl DevtoolsStore {
  pub fn new() -> Self {
    Self {
      host_tree: Observable::new(None),
      cascaded: Observable::new(None),
      selected_path: Observable::new(None),
      hover_path: Observable::new(None),
      pick_mode: Arc::new(AtomicBool::new(false)),
      pending_pick: Observable::new(None),
    }
  }

  pub fn update_host_tree(&self, tree: &Tree) {
    let cascaded = wgpu_html_style::cascade(tree);
    self.host_tree.set(Some(tree.clone()));
    self.cascaded.set(Some(cascaded));
  }
}
