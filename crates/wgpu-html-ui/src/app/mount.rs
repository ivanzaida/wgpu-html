//! Public typed handle for driving a component manually.
//!
//! Use [`Mount`] when you need to embed a component in an existing
//! [`Tree`] without the full [`App`](crate::App) harness — e.g. a
//! secondary window, a devtools panel, or an egui overlay.
//!
//! ```ignore
//! let mut mount = Mount::<MyComponent>::new(props);
//! // Initial render:
//! mount.render(&mut tree, &env);
//! // Each frame, after event dispatch:
//! mount.process(&mut tree, &env);
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
/// 3. Calling [`Mount::force_render`] when the environment changes.
pub struct Mount<C: Component> {
  runtime: Runtime,
  _marker: std::marker::PhantomData<C>,
}

impl<C: Component> Mount<C>
where
  C::Msg: Clone + Send + Sync + 'static,
  C::Props: 'static,
  C::Env: 'static,
{
  /// Create a mount with a no-op wake function.
  /// Call [`set_wake`](Mount::set_wake) to install a real one.
  pub fn new(props: C::Props) -> Self {
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    Self {
      runtime: Runtime::new::<C>(&props, wake),
      _marker: std::marker::PhantomData,
    }
  }

  /// Create a mount with a custom wake function that triggers
  /// redraws when callbacks fire.
  pub fn with_wake(props: C::Props, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
    Self {
      runtime: Runtime::new::<C>(&props, wake),
      _marker: std::marker::PhantomData,
    }
  }

  /// Perform the initial render and write the result into `tree.root`.
  pub fn render(&mut self, tree: &mut Tree, env: &C::Env) {
    self.runtime.force_render(tree, env);
  }

  /// Drain pending messages, re-render if needed.
  /// Returns `true` if the tree was updated.
  pub fn process(&mut self, tree: &mut Tree, env: &C::Env) -> bool {
    self.runtime.process(tree, env)
  }

  /// Force a full re-render (e.g. when the environment changed).
  pub fn force_render(&mut self, tree: &mut Tree, env: &C::Env) {
    self.runtime.force_render(tree, env);
  }
}
