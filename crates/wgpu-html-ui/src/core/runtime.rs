//! Component instance management and update loop.

use std::{
  any::{Any, TypeId},
  collections::HashMap,
  sync::Arc,
};

use wgpu_html_tree::{Node, Tree};

use crate::core::{
  component::{Component, ShouldRender},
  ctx::{ChildSlot, Ctx, MsgSender},
};

// ── Type-erased component interface ─────────────────────────────────────────

pub(crate) trait AnyComponent {
  /// Process one message.  Returns whether the view should be rebuilt.
  fn update_any(&mut self, msg: Box<dyn Any>) -> ShouldRender;

  /// Render the component and return the node tree + child slots.
  fn render(&self, env: &dyn Any) -> (Node, Vec<ChildSlot>);

  /// Notify that props changed.
  fn props_changed_any(&mut self, new_props: &dyn Any) -> ShouldRender;

  /// Replace stored props.
  fn set_props(&mut self, props: Box<dyn Any>);

  /// Drain pending messages.
  fn drain_messages(&self) -> Vec<Box<dyn Any>>;

  /// Lifecycle: mounted — passes the sender so the component can subscribe.
  fn mounted(&mut self);

  /// Lifecycle: destroyed.
  fn destroyed(&mut self);

  /// Called after each render pass triggered by the component's own state
  /// change.  Calls [`Component::updated`].
  fn did_render(&mut self);

  /// Component's scope prefix.
  fn scope_prefix(&self) -> &'static str;

  /// Generated scoped CSS, if the component defines styles.
  fn styles_css(&self) -> Option<String>;
}

// ── Concrete typed wrapper ──────────────────────────────────────────────────

pub(crate) struct ComponentState<C: Component> {
  component: C,
  props: C::Props,
  sender: MsgSender<C::Msg>,
}

impl<C: Component> ComponentState<C>
where
  C::Msg: Clone + Send + Sync + 'static,
  C::Props: 'static,
{
  pub(crate) fn new(props: &C::Props, wake: Arc<dyn Fn() + Send + Sync>) -> Self {
    let sender = MsgSender::new(wake);
    let component = C::create(props);
    Self {
      component,
      props: props.clone(),
      sender,
    }
  }
}

impl<C: Component> AnyComponent for ComponentState<C>
where
  C::Msg: Clone + Send + Sync + 'static,
  C::Props: 'static,
  C::Env: 'static,
{
  fn update_any(&mut self, msg: Box<dyn Any>) -> ShouldRender {
    if let Ok(msg) = msg.downcast::<C::Msg>() {
      self.component.update(*msg, &self.props)
    } else {
      ShouldRender::No
    }
  }

  fn render(&self, env: &dyn Any) -> (Node, Vec<ChildSlot>) {
    let ctx = Ctx::new(self.sender.clone(), C::scope());
    if let Some(env) = env.downcast_ref::<C::Env>() {
      let el = self.component.view(&self.props, &ctx, env);
      let children = ctx.children.into_inner();
      (el.into_node(), children)
    } else {
      (Node::new(""), Vec::new())
    }
  }

  fn scope_prefix(&self) -> &'static str {
    C::scope()
  }

  fn styles_css(&self) -> Option<String> {
    let sheet = C::styles();
    if sheet.is_empty() {
      return None;
    }
    let prefix = C::scope();
    if prefix.is_empty() {
      Some(sheet.to_css())
    } else {
      Some(sheet.to_css_scoped(prefix))
    }
  }

  fn props_changed_any(&mut self, new_props: &dyn Any) -> ShouldRender {
    if let Some(new) = new_props.downcast_ref::<C::Props>() {
      let old = self.props.clone();
      self.component.props_changed(&old, new)
    } else {
      ShouldRender::No
    }
  }

  fn set_props(&mut self, props: Box<dyn Any>) {
    if let Ok(p) = props.downcast::<C::Props>() {
      self.props = *p;
    }
  }

  fn drain_messages(&self) -> Vec<Box<dyn Any>> {
    self
      .sender
      .drain()
      .into_iter()
      .map(|m| Box::new(m) as Box<dyn Any>)
      .collect()
  }

  fn mounted(&mut self) {
    self.component.mounted(self.sender.clone());
  }

  fn destroyed(&mut self) {
    self.component.destroyed();
  }

  fn did_render(&mut self) {
    self.component.updated(&self.props);
  }
}

// ── Mounted component tree ──────────────────────────────────────────────────

/// Child key: (user/positional key string, TypeId).
/// String keys come from `ctx.keyed_child`; positional keys are
/// `"__pos_{n}"` from `ctx.child`.
type ChildKey = (String, TypeId);

pub(crate) struct MountedComponent {
  pub(crate) state: Box<dyn AnyComponent>,
  pub(crate) children: HashMap<ChildKey, MountedComponent>,
  /// Fully-resolved cached output (placeholders replaced with child subtrees).
  /// Returned on the clean fast-path and on patch-path after substitution.
  pub(crate) last_node: Option<Node>,
  /// Raw output of the last `state.render()` call — child placeholders are
  /// **not** substituted.  Kept because it is much smaller than `last_node`
  /// (placeholder divs instead of full child subtrees) so cloning it to
  /// re-patch dirty children is cheap.  `None` before the first render.
  pub(crate) skeleton_node: Option<Node>,
  /// True if this component's own `update()` returned `Yes` since the last
  /// render, or if its parent flagged it via `props_changed`.
  pub(crate) needs_render: bool,
  /// True if this component OR any descendant is dirty. Propagated by
  /// `process_component`; cleared after a render pass.
  pub(crate) subtree_dirty: bool,
  /// The DOM placeholder id in the parent's node tree (empty for root).
  pub(crate) marker_id: String,
}

// ── Runtime ─────────────────────────────────────────────────────────────────

pub struct Runtime {
  root: MountedComponent,
  wake: Arc<dyn Fn() + Send + Sync>,
  /// TypeIds of components whose styles have been registered.
  #[allow(dead_code)]
  registered_styles: std::collections::HashSet<TypeId>,
  /// When `true`, the component's rendered output replaces `tree.root`
  /// directly.  When `false` (the default), the output is placed inside
  /// the existing `html > body > [0]` structure created by the [`App`]
  /// harness.
  ///
  /// [`Mount`](crate::Mount) sets this to `true` because the component
  /// owns the entire document.
  direct_root: bool,
}

impl Runtime {
  pub(crate) fn root_mounted(&self) -> &MountedComponent {
    &self.root
  }

  /// Create a runtime with a root component.
  pub fn new<C: Component>(props: &C::Props, wake: Arc<dyn Fn() + Send + Sync>) -> Self
  where
    C::Msg: Clone + Send + Sync + 'static,
    C::Props: 'static,
    C::Env: 'static,
  {
    let state = Box::new(ComponentState::<C>::new(props, wake.clone()));
    Self {
      root: MountedComponent {
        state,
        children: HashMap::new(),
        last_node: None,
        skeleton_node: None,
        needs_render: true,
        subtree_dirty: true,
        marker_id: String::new(),
      },
      wake,
      registered_styles: std::collections::HashSet::new(),
      direct_root: false,
    }
  }

  /// When set to `true`, `process` and `force_render` replace
  /// `tree.root` directly instead of navigating into an existing
  /// `html > body` wrapper.
  pub(crate) fn set_direct_root(&mut self, direct: bool) {
    self.direct_root = direct;
  }

  /// Perform the initial render of the entire component tree.
  pub fn initial_render(&mut self, env: &dyn Any) -> Node {
    let node = Self::render_component(&mut self.root, &self.wake, env);
    self.root.state.mounted();
    node
  }

  /// Process all pending messages across the component tree.
  /// Re-renders if any component changed.
  ///
  /// Returns `true` if any subtree was re-rendered.
  pub fn process(&mut self, tree: &mut Tree, env: &dyn Any) -> bool {
    let mut ever_changed = false;
    loop {
      let changed = Self::process_component(&mut self.root, &self.wake);
      if !changed {
        break;
      }
      ever_changed = true;
      let node = Self::render_component(&mut self.root, &self.wake, env);
      self.apply_node(tree, node);
    }
    ever_changed
  }

  /// Force a full re-render (e.g. when env changed externally).
  pub fn force_render(&mut self, tree: &mut Tree, env: &dyn Any) {
    Self::register_styles(tree, &self.root);
    Self::mark_all_dirty(&mut self.root);
    let node = Self::render_component(&mut self.root, &self.wake, env);
    self.apply_node(tree, node);
  }

  /// Write the component output into the tree and bump the generation.
  fn apply_node(&self, tree: &mut Tree, node: Node) {
    // Compute selector fingerprint of the new node *before* we
    // throw away the old root.  If the fingerprint is unchanged,
    // only inline styles / text changed, so the cascade can be
    // skipped on the next frame (PipelineAction::LayoutOnly).
    let old_fp = tree.root.as_ref().map(|r| wgpu_html_tree::node_selector_fingerprint(r));
    let new_fp = wgpu_html_tree::node_selector_fingerprint(&node);

    if self.direct_root {
      tree.root = Some(node);
    } else {
      Self::replace_component_node(tree, node);
    }
    tree.generation += 1;
    if Some(new_fp) != old_fp {
      tree.cascade_generation += 1;
    }
  }

  /// Replace the component's node inside the tree structure.
  /// Used by the [`App`] harness where `html > body` already exists.
  fn replace_component_node(tree: &mut Tree, node: Node) {
    if let Some(root) = &mut tree.root {
      if let Some(body) = root.children.first_mut() {
        if body.children.is_empty() {
          body.children.push(node);
        } else {
          body.children[0] = node;
        }
        return;
      }
    }
    tree.root = Some(node);
  }

  /// Walk the mounted tree and register any pending component styles.
  pub(crate) fn register_styles(tree: &mut Tree, mounted: &MountedComponent) {
    let css = mounted.state.styles_css();
    if let Some(css) = css {
      let prefix = mounted.state.scope_prefix();
      let href = if prefix.is_empty() {
        "__component_global".to_string()
      } else {
        format!("__component_{prefix}")
      };
      if !tree.linked_stylesheets.contains_key(&href) {
        tree.register_linked_stylesheet(&href, &css);
      }
    }
    for (_key, child) in &mounted.children {
      Self::register_styles(tree, child);
    }
  }

  /// Mark this component and all descendants as needing a full re-render.
  /// Called by `force_render` when the environment changes.
  fn mark_all_dirty(mounted: &mut MountedComponent) {
    mounted.needs_render = true;
    mounted.subtree_dirty = true;
    for child in mounted.children.values_mut() {
      Self::mark_all_dirty(child);
    }
  }

  /// Drain messages and call `update`.  Sets `needs_render` on the mounted
  /// component when `update` returns `Yes`.
  fn drain_and_update(mounted: &mut MountedComponent, changed: &mut bool) {
    let messages = mounted.state.drain_messages();
    for msg in messages {
      if mounted.state.update_any(msg) == ShouldRender::Yes {
        *changed = true;
        mounted.needs_render = true;
      }
    }
  }

  /// Process a component and its children.  Returns `true` if anything in
  /// the subtree is now dirty.  Sets `subtree_dirty` on `mounted`.
  fn process_component(mounted: &mut MountedComponent, wake: &Arc<dyn Fn() + Send + Sync>) -> bool {
    let mut any_dirty = false;

    Self::drain_and_update(mounted, &mut any_dirty);

    for (_key, child) in &mut mounted.children {
      if Self::process_component(child, wake) {
        any_dirty = true;
      }
    }

    // Re-drain: children may have queued messages on us via prop callbacks.
    Self::drain_and_update(mounted, &mut any_dirty);

    mounted.subtree_dirty = any_dirty;
    any_dirty
  }

  /// Render a component and return its resolved node.
  ///
  /// Three paths, from cheapest to most expensive:
  ///
  /// 1. **Clean fast-path** (`!needs_render && !subtree_dirty`): Return `last_node` directly.  No allocations, no
  ///    `view()` call.
  ///
  /// 2. **Patch path** (`!needs_render && subtree_dirty`): The component itself is unchanged so `view()` is
  ///    **skipped**. Clone the stored `skeleton_node` (tiny — contains placeholder divs instead of full child subtrees)
  ///    and re-substitute every child: dirty children re-render recursively; clean children return their own
  ///    `last_node`.  Saves one `view()` call per ancestor of every updated leaf.
  ///
  /// 3. **Full render** (`needs_render`, or first render): Call `view()`, reconcile the child set (add/remove/update),
  ///    store the raw output as `skeleton_node`, substitute children, cache the resolved result as `last_node`.
  fn render_component(mounted: &mut MountedComponent, wake: &Arc<dyn Fn() + Send + Sync>, env: &dyn Any) -> Node {
    // ── Path 1: clean fast-path ─────────────────────────────────────
    if !mounted.needs_render && !mounted.subtree_dirty {
      if let Some(cached) = &mounted.last_node {
        return cached.clone();
      }
    }

    // ── Path 2: patch path — parent clean, children dirty ───────────
    if !mounted.needs_render && mounted.subtree_dirty {
      if let Some(skeleton) = &mounted.skeleton_node {
        // Clone the skeleton (placeholder divs only — much smaller than
        // last_node which contains full child subtrees).
        let mut resolved = skeleton.clone();

        // Re-substitute every child.  Dirty children re-render; clean
        // children hit path 1 and return their last_node cheaply.
        for child in mounted.children.values_mut() {
          let child_node = Self::render_component(child, wake, env);
          replace_placeholder(&mut resolved, &child.marker_id, child_node);
        }

        mounted.subtree_dirty = false;
        mounted.last_node = Some(resolved.clone());
        return resolved;
      }
      // No skeleton yet (shouldn't happen in steady state) — fall through
      // to a full render.
    }

    // ── Path 3: full render ──────────────────────────────────────────
    let was_dirty = mounted.needs_render;
    let is_first_render = mounted.skeleton_node.is_none();
    let (skeleton, child_slots) = mounted.state.render(env);
    // Resolved starts as a clone of the skeleton; children are then
    // substituted in.  The original skeleton is kept intact for future
    // patch-path passes.
    let mut resolved = skeleton.clone();

    let mut new_children: HashMap<ChildKey, MountedComponent> = HashMap::new();

    for slot in child_slots {
      let key: ChildKey = (slot.key.clone(), slot.component_type_id);

      let mut child = if let Some(mut existing) = mounted.children.remove(&key) {
        let props_changed = existing.state.props_changed_any(slot.props.as_ref());
        if props_changed == ShouldRender::Yes {
          existing.needs_render = true;
          existing.subtree_dirty = true;
        }
        existing.state.set_props(slot.props);
        existing
      } else {
        // New child — always render and call mounted.
        let state = (slot.create)(slot.props.as_ref(), wake.clone());
        let mut child = MountedComponent {
          state,
          children: HashMap::new(),
          last_node: None,
          skeleton_node: None,
          needs_render: true,
          subtree_dirty: true,
          marker_id: slot.marker_id.clone(),
        };
        child.state.mounted();
        child
      };

      child.marker_id = slot.marker_id.clone();

      let child_node = Self::render_component(&mut child, wake, env);
      replace_placeholder(&mut resolved, &slot.marker_id, child_node);

      new_children.insert(key, child);
    }

    // Destroy removed children.
    for (_key, mut removed) in mounted.children.drain() {
      destroy_recursive(&mut removed);
    }
    mounted.children = new_children;

    // Post-render lifecycle hook (skip on first render — initial mount
    // is not considered a "re-render").
    if was_dirty && !is_first_render {
      mounted.state.did_render();
    }

    mounted.skeleton_node = Some(skeleton);
    mounted.needs_render = false;
    mounted.subtree_dirty = false;
    mounted.last_node = Some(resolved.clone());
    resolved
  }
}

fn replace_placeholder(node: &mut Node, marker_id: &str, replacement: Node) -> bool {
  for i in 0..node.children.len() {
    if node.children[i].element.id() == Some(marker_id) {
      node.children[i] = replacement;
      return true;
    }
    if replace_placeholder(&mut node.children[i], marker_id, replacement.clone()) {
      return true;
    }
  }
  false
}

fn destroy_recursive(mounted: &mut MountedComponent) {
  for (_key, child) in &mut mounted.children {
    destroy_recursive(child);
  }
  mounted.state.destroyed();
}
