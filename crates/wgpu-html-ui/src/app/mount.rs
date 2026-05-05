//! Public typed handle for driving a component manually.
//!
//! Use [`Mount`] when you need to embed a component in an existing
//! [`Tree`] without the full [`App`](crate::App) harness — e.g. a
//! secondary window, a devtools panel, or an egui overlay.
//!
//! ```ignore
//! let mut mount = Mount::<MyComponent>::new(props);
//! // Initial render:
//! mount.render(&mut tree);
//! // Each frame, after event dispatch:
//! mount.process(&mut tree);
//! ```

use std::sync::Arc;

use wgpu_html_tree::Tree;

use crate::core::{component::Component, runtime::Runtime};

/// A typed handle for driving a single component tree manually.
///
/// Unlike [`App`](crate::App), `Mount` does not own an event loop
/// or a window. The caller is responsible for:
///
/// 1. Calling [`Mount::render`] once to populate the initial tree.
/// 2. Calling [`Mount::process`] each frame (or after events) to drain pending messages and re-render if needed.
/// 3. Calling [`Mount::force_render`] when props or external state changed.
pub struct Mount<C: Component> {
  runtime: Runtime,
  _marker: std::marker::PhantomData<C>,
}

impl<C: Component> Mount<C>
where
  C::Msg: Clone + Send + Sync + 'static,
  C::Props: 'static,
{
  /// Create a mount with a no-op wake function.
  /// Call [`set_wake`](Mount::set_wake) to install a real one.
  pub fn new(props: C::Props) -> Self {
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let mut runtime = Runtime::new::<C>(&props, wake);
    runtime.set_direct_root(true);
    Self {
      runtime,
      _marker: std::marker::PhantomData,
    }
  }

  /// Create a mount with a custom wake function that triggers
  /// redraws when callbacks fire.
  pub fn with_wake(props: C::Props, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
    let mut runtime = Runtime::new::<C>(&props, wake);
    runtime.set_direct_root(true);
    Self {
      runtime,
      _marker: std::marker::PhantomData,
    }
  }

  /// Perform the initial render and write the result into `tree.root`.
  pub fn render(&mut self, tree: &mut Tree) {
    self.runtime.force_render(tree);
  }

  /// Drain pending messages, re-render if needed.
  /// Returns `true` if the tree was updated.
  pub fn process(&mut self, tree: &mut Tree) -> bool {
    self.runtime.process(tree)
  }

  /// Force a full re-render of every component in the tree.
  pub fn force_render(&mut self, tree: &mut Tree) {
    self.runtime.force_render(tree);
  }

  /// Render the component tree and return the result as an HTML
  /// string. Useful for debugging layout in a real browser.
  pub fn generate_html(&mut self, tree: &mut Tree) -> String {
    self.runtime.force_render(tree);
    tree.to_html()
  }
}
