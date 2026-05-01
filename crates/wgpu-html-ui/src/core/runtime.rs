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

  /// Lifecycle: mounted.
  fn mounted(&mut self);

  /// Lifecycle: destroyed.
  fn destroyed(&mut self);

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
      // Env type mismatch — should not happen in practice.
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
    self.component.mounted();
  }

  fn destroyed(&mut self) {
    self.component.destroyed();
  }
}

// ── Mounted component tree ──────────────────────────────────────────────────

type ChildKey = (usize, TypeId);

pub(crate) struct MountedComponent {
  pub(crate) state: Box<dyn AnyComponent>,
  pub(crate) children: HashMap<ChildKey, MountedComponent>,
}

// ── Runtime ─────────────────────────────────────────────────────────────────

pub(crate) struct Runtime {
  root: MountedComponent,
  wake: Arc<dyn Fn() + Send + Sync>,
  /// TypeIds of components whose styles have been registered.
  registered_styles: std::collections::HashSet<TypeId>,
}

impl Runtime {
  pub(crate) fn root_mounted(&self) -> &MountedComponent {
    &self.root
  }

  /// Create a runtime with a root component.
  pub(crate) fn new<C: Component>(props: &C::Props, wake: Arc<dyn Fn() + Send + Sync>) -> Self
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
      },
      wake,
      registered_styles: std::collections::HashSet::new(),
    }
  }

  /// Perform the initial render of the entire component tree.
  pub(crate) fn initial_render(&mut self, env: &dyn Any) -> Node {
    let node = Self::render_component(&mut self.root, &self.wake, env);
    self.root.state.mounted();
    node
  }

  /// Process all pending messages across the component tree.
  /// Re-renders if any component changed.
  ///
  /// Loops until no more messages are pending so that child→parent
  /// callbacks (which queue messages on the parent during the child's
  /// update) are processed in the same frame.
  ///
  /// Returns `true` if any subtree was re-rendered.
  pub(crate) fn process(&mut self, tree: &mut Tree, env: &dyn Any) -> bool {
    let mut ever_changed = false;
    loop {
      let changed = Self::process_component(&mut self.root, &self.wake);
      if !changed {
        break;
      }
      ever_changed = true;
      let node = Self::render_component(&mut self.root, &self.wake, env);
      Self::replace_component_node(tree, node);
      tree.generation += 1;
    }
    ever_changed
  }

  /// Force a full re-render (e.g. when env changed externally).
  pub(crate) fn force_render(&mut self, tree: &mut Tree, env: &dyn Any) {
    Self::register_styles(tree, &self.root);
    let node = Self::render_component(&mut self.root, &self.wake, env);
    Self::replace_component_node(tree, node);
    tree.generation += 1;
  }

  /// Replace the component's node inside the tree structure.
  /// The tree is `<html><body>{component}</body></html>` — we
  /// replace body's first child, preserving the wrapper.
  fn replace_component_node(tree: &mut Tree, node: Node) {
    if let Some(root) = &mut tree.root {
      // Navigate: html → body (first child) → replace first child
      if let Some(body) = root.children.first_mut() {
        if body.children.is_empty() {
          body.children.push(node);
        } else {
          body.children[0] = node;
        }
        return;
      }
    }
    // Fallback: no wrapper, set root directly.
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

  /// Process a component and its children.  Child `update()` calls
  /// can queue messages on the parent via prop callbacks, so we
  /// drain, process children, then drain again until stable.
  fn process_component(mounted: &mut MountedComponent, wake: &Arc<dyn Fn() + Send + Sync>) -> bool {
    let mut changed = false;

    // Drain own messages first.
    Self::drain_and_update(mounted, &mut changed);

    // Process children — their update() may queue new messages
    // on this component via prop callbacks.
    for (_key, child) in &mut mounted.children {
      if Self::process_component(child, wake) {
        changed = true;
      }
    }

    // Re-drain: children may have queued messages on us.
    Self::drain_and_update(mounted, &mut changed);

    changed
  }

  fn drain_and_update(mounted: &mut MountedComponent, changed: &mut bool) {
    let messages = mounted.state.drain_messages();
    for msg in messages {
      if mounted.state.update_any(msg) == ShouldRender::Yes {
        *changed = true;
      }
    }
  }

  fn render_component(mounted: &mut MountedComponent, wake: &Arc<dyn Fn() + Send + Sync>, env: &dyn Any) -> Node {
    let (mut node, child_slots) = mounted.state.render(env);

    let mut new_children: HashMap<ChildKey, MountedComponent> = HashMap::new();

    for slot in child_slots {
      let key = (slot.index, slot.component_type_id);

      let mut child = if let Some(mut existing) = mounted.children.remove(&key) {
        let _should = existing.state.props_changed_any(slot.props.as_ref());
        existing.state.set_props(slot.props);
        existing
      } else {
        let state = (slot.create)(slot.props.as_ref(), wake.clone());
        let mut child = MountedComponent {
          state,
          children: HashMap::new(),
        };
        child.state.mounted();
        child
      };

      let child_node = Self::render_component(&mut child, wake, env);
      replace_placeholder(&mut node, &slot.marker_id, child_node);

      new_children.insert(key, child);
    }

    for (_key, mut removed) in mounted.children.drain() {
      destroy_recursive(&mut removed);
    }

    mounted.children = new_children;
    node
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
