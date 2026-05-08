use std::sync::{
  atomic::{AtomicBool, AtomicPtr, Ordering},
  Arc,
};

use wgpu_html_style::CascadedTree;
use wgpu_html_tree::Tree;
use wgpu_html_ui::Observable;

#[derive(Clone)]
pub struct DevtoolsStore {
  host_tree_ptr: Arc<AtomicPtr<Tree>>,
  pub cascaded: Observable<Option<CascadedTree>>,
  pub selected_path: Observable<Option<Vec<usize>>>,
  pub hover_path: Observable<Option<Vec<usize>>>,
  pub pick_mode: Arc<AtomicBool>,
  pub pending_pick: Observable<Option<Vec<usize>>>,
}

impl DevtoolsStore {
  pub fn new() -> Self {
    Self {
      host_tree_ptr: Arc::new(AtomicPtr::new(std::ptr::null_mut())),
      cascaded: Observable::new(None),
      selected_path: Observable::new(None),
      hover_path: Observable::new(None),
      pick_mode: Arc::new(AtomicBool::new(false)),
      pending_pick: Observable::new(None),
    }
  }

  /// Scoped access to the live host tree. Only valid during
  /// [`Devtools::poll`] — returns `None` outside that window.
  pub fn host_tree(&self) -> Option<&Tree> {
    let p = self.host_tree_ptr.load(Ordering::Relaxed);
    if p.is_null() {
      None
    } else {
      // SAFETY: the pointer is set from a `&Tree` at the start of
      // `poll()` and cleared at the end. All component rendering
      // runs synchronously within that window on the same thread.
      Some(unsafe { &*p })
    }
  }

  pub(crate) fn bind_host_tree(&self, tree: &Tree) {
    self.host_tree_ptr.store(tree as *const Tree as *mut Tree, Ordering::Relaxed);
  }

  pub(crate) fn unbind_host_tree(&self) {
    self.host_tree_ptr.store(std::ptr::null_mut(), Ordering::Relaxed);
  }

  pub fn update_cascade(&self, tree: &Tree) {
    let cascaded = wgpu_html_style::cascade(tree);
    self.cascaded.set(Some(cascaded));
  }
}
