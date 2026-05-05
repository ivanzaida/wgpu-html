//! Host extension trait for tree-level event interception.
//!
//! `Tree` owns the hook list and exposes `emit_*` methods. Hosts/glue crates
//! should call those tree methods at integration boundaries instead of calling
//! hook trait methods directly.

use std::{
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use wgpu_html_events::{HtmlEvent, events};

use crate::{MouseEvent as TreeMouseEvent, Node, Tree};

type HookObject = dyn TreeHook + Send + 'static;

/// Shareable handle to a tree hook stored on [`Tree`].
#[derive(Clone)]
pub struct TreeHookHandle {
  inner: Arc<Mutex<Box<HookObject>>>,
}

impl TreeHookHandle {
  pub fn new(hook: impl TreeHook + Send + 'static) -> Self {
    Self {
      inner: Arc::new(Mutex::new(Box::new(hook))),
    }
  }

  pub fn ptr_eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.inner, &other.inner)
  }
}

impl std::fmt::Debug for TreeHookHandle {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TreeHookHandle").finish_non_exhaustive()
  }
}

/// Coarse lifecycle/profiling stage names.
///
/// The tree crate does not know about layout, paint, or renderer internals, but
/// glue crates can use these shared names when emitting hook events around
/// their pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeLifecycleStage {
  Tree,
  Parse,
  Cascade,
  Layout,
  Paint,
  Render,
  Frame,
  EventDispatch,
  PointerDispatch,
  KeyboardDispatch,
  FocusDispatch,
  TextInput,
  Query,
  AssetPreload,
  Custom,
}

/// Position of a lifecycle event in a measured operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeLifecyclePhase {
  Begin,
  End,
  Instant,
}

/// A lightweight lifecycle/profiling marker.
///
/// `stage` is the broad bucket. `label` can hold a more specific operation
/// name (`"hover-hit-test"`, `"viewport-scroll"`, `"atlas-upload"`, etc.).
/// `duration` is only meaningful for [`TreeLifecyclePhase::End`].
#[derive(Debug, Clone, Copy)]
pub struct TreeLifecycleEvent<'a> {
  pub stage: TreeLifecycleStage,
  pub phase: TreeLifecyclePhase,
  pub label: Option<&'a str>,
  pub at: Instant,
  pub duration: Option<Duration>,
}

impl<'a> TreeLifecycleEvent<'a> {
  pub fn begin(stage: TreeLifecycleStage) -> Self {
    Self::new(stage, TreeLifecyclePhase::Begin)
  }

  pub fn end(stage: TreeLifecycleStage, duration: Duration) -> Self {
    Self {
      duration: Some(duration),
      ..Self::new(stage, TreeLifecyclePhase::End)
    }
  }

  pub fn instant(stage: TreeLifecycleStage) -> Self {
    Self::new(stage, TreeLifecyclePhase::Instant)
  }

  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  fn new(stage: TreeLifecycleStage, phase: TreeLifecyclePhase) -> Self {
    Self {
      stage,
      phase,
      label: None,
      at: Instant::now(),
      duration: None,
    }
  }
}

/// Viewport metadata attached to a render hook.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeRenderViewport {
  /// Logical viewport width in CSS/layout pixels.
  pub width: f32,
  /// Logical viewport height in CSS/layout pixels.
  pub height: f32,
  /// Host scale factor used for layout/paint/render.
  pub scale: f32,
}

impl TreeRenderViewport {
  pub fn new(width: f32, height: f32, scale: f32) -> Self {
    Self { width, height, scale }
  }
}

/// Per-frame render hook payload.
///
/// `delta` is the elapsed wall time since the previous render callback and is
/// the one field hosts are expected to provide. Everything else is optional so
/// integration crates can fill progressively richer profiling data without
/// forcing renderer/layout types into `wgpu-html-tree`.
#[derive(Debug, Clone, Copy)]
pub struct TreeRenderEvent<'a> {
  pub delta: Duration,
  pub elapsed: Option<Duration>,
  pub frame_index: Option<u64>,
  pub viewport: Option<TreeRenderViewport>,
  pub frame_duration: Option<Duration>,
  pub cascade_duration: Option<Duration>,
  pub layout_duration: Option<Duration>,
  pub paint_duration: Option<Duration>,
  pub render_duration: Option<Duration>,
  pub label: Option<&'a str>,
  pub at: Instant,
}

impl<'a> TreeRenderEvent<'a> {
  pub fn new(delta: Duration) -> Self {
    Self {
      delta,
      elapsed: None,
      frame_index: None,
      viewport: None,
      frame_duration: None,
      cascade_duration: None,
      layout_duration: None,
      paint_duration: None,
      render_duration: None,
      label: None,
      at: Instant::now(),
    }
  }

  pub fn with_elapsed(mut self, elapsed: Duration) -> Self {
    self.elapsed = Some(elapsed);
    self
  }

  pub fn with_frame_index(mut self, frame_index: u64) -> Self {
    self.frame_index = Some(frame_index);
    self
  }

  pub fn with_viewport(mut self, viewport: TreeRenderViewport) -> Self {
    self.viewport = Some(viewport);
    self
  }

  pub fn with_frame_duration(mut self, duration: Duration) -> Self {
    self.frame_duration = Some(duration);
    self
  }

  pub fn with_pipeline_durations(
    mut self,
    cascade: Option<Duration>,
    layout: Option<Duration>,
    paint: Option<Duration>,
    render: Option<Duration>,
  ) -> Self {
    self.cascade_duration = cascade;
    self.layout_duration = layout;
    self.paint_duration = paint;
    self.render_duration = render;
    self
  }

  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TreeHookResponse {
  #[default]
  Continue,
  Stop,
}

impl TreeHookResponse {
  #[inline]
  pub fn is_continue(self) -> bool {
    matches!(self, Self::Continue)
  }

  #[inline]
  pub fn is_stop(self) -> bool {
    matches!(self, Self::Stop)
  }
}

#[allow(unused_variables)]
pub trait TreeHook {
  /// Called once per rendered frame when a host chooses to expose render
  /// progress through this hook.
  fn on_render(&mut self, tree: &mut Tree, event: &TreeRenderEvent<'_>) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  /// Called for coarse lifecycle/profiling markers.
  ///
  /// The default implementation fans out by phase. Hosts that only care
  /// about profiling can override this one method and ignore DOM events.
  fn on_lifecycle_event(&mut self, tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    match event.phase {
      TreeLifecyclePhase::Begin => self.on_lifecycle_begin(tree, event),
      TreeLifecyclePhase::End => self.on_lifecycle_end(tree, event),
      TreeLifecyclePhase::Instant => self.on_lifecycle_instant(tree, event),
    }
  }

  fn on_lifecycle_begin(&mut self, tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_lifecycle_end(&mut self, tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_lifecycle_instant(&mut self, tree: &mut Tree, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  /// Called with the tree crate's lightweight mouse payload.
  ///
  /// This preserves the original hook shape used by low-level tree
  /// dispatchers (`crate::MouseEvent`), distinct from the DOM-style
  /// `wgpu_html_events::events::MouseEvent` handled by
  /// [`Self::on_dom_mouse_event`].
  fn on_mouse_event(&mut self, tree: &mut Tree, event: &mut TreeMouseEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  /// Called with a fully typed DOM-style event.
  ///
  /// The default implementation fans out to the most specific typed hook.
  /// Returning [`TreeHookResponse::Stop`] from any typed hook stops this
  /// fan-out and gives future dispatch integration a single propagation
  /// decision to honor.
  fn on_event(&mut self, tree: &mut Tree, event: &mut HtmlEvent) -> TreeHookResponse {
    match event {
      HtmlEvent::Mouse(event) => self.on_dom_mouse_event(tree, event),
      HtmlEvent::Pointer(event) => self.on_pointer_event(tree, event),
      HtmlEvent::Wheel(event) => self.on_wheel_event(tree, event),
      HtmlEvent::Keyboard(event) => self.on_keyboard_event(tree, event),
      HtmlEvent::Focus(event) => self.on_focus_event(tree, event),
      HtmlEvent::Input(event) => self.on_input_event(tree, event),
      HtmlEvent::Composition(event) => self.on_composition_event(tree, event),
      HtmlEvent::Clipboard(event) => self.on_clipboard_event(tree, event),
      HtmlEvent::Drag(event) => self.on_drag_event(tree, event),
      HtmlEvent::Touch(event) => self.on_touch_event(tree, event),
      HtmlEvent::Animation(event) => self.on_animation_event(tree, event),
      HtmlEvent::Transition(event) => self.on_transition_event(tree, event),
      HtmlEvent::Submit(event) => self.on_submit_event(tree, event),
      HtmlEvent::FormData(event) => self.on_form_data_event(tree, event),
      HtmlEvent::Toggle(event) => self.on_toggle_event(tree, event),
      HtmlEvent::BeforeToggle(event) => self.on_before_toggle_event(tree, event),
      HtmlEvent::Progress(event) => self.on_progress_event(tree, event),
      HtmlEvent::Generic(event) => self.on_generic_event(tree, event),
    }
  }

  fn on_dom_mouse_event(&mut self, tree: &mut Tree, event: &mut events::MouseEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_pointer_event(&mut self, tree: &mut Tree, event: &mut events::PointerEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_wheel_event(&mut self, tree: &mut Tree, event: &mut events::WheelEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_keyboard_event(&mut self, tree: &mut Tree, event: &mut events::KeyboardEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_focus_event(&mut self, tree: &mut Tree, event: &mut events::FocusEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_input_event(&mut self, tree: &mut Tree, event: &mut events::InputEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_composition_event(&mut self, tree: &mut Tree, event: &mut events::CompositionEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_clipboard_event(&mut self, tree: &mut Tree, event: &mut events::ClipboardEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_drag_event(&mut self, tree: &mut Tree, event: &mut events::DragEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_touch_event(&mut self, tree: &mut Tree, event: &mut events::TouchEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_animation_event(&mut self, tree: &mut Tree, event: &mut events::AnimationEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_transition_event(&mut self, tree: &mut Tree, event: &mut events::TransitionEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_submit_event(&mut self, tree: &mut Tree, event: &mut events::SubmitEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_form_data_event(&mut self, tree: &mut Tree, event: &mut events::FormDataEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_toggle_event(&mut self, tree: &mut Tree, event: &mut events::ToggleEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_before_toggle_event(&mut self, tree: &mut Tree, event: &mut events::BeforeToggleEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_progress_event(&mut self, tree: &mut Tree, event: &mut events::ProgressEvent) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  fn on_generic_event(&mut self, tree: &mut Tree, event: &mut events::Event) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  /// Called when a node has been added to the tree.
  ///
  /// The `element` reference points to the node **before** it is
  /// inserted (the caller emits this just before or just after
  /// pushing it into the tree, depending on which avoids borrow
  /// conflicts in the host code).
  fn on_element_added(&mut self, tree: &mut Tree, element: &Node) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  /// Called when a node has been removed from the tree.
  ///
  /// The `element` reference points to the node that was just
  /// removed. It is no longer reachable through the tree's root.
  fn on_element_removed(&mut self, tree: &mut Tree, element: &Node) -> TreeHookResponse {
    TreeHookResponse::Continue
  }

  /// Called when the focused (active) element changes.
  ///
  /// `old_path` is the path to the previously focused element
  /// (`None` if nothing was focused). `new_path` is the path to
  /// the newly focused element (`None` if focus was cleared).
  /// Use `tree.root.as_ref()?.at_path(path)` to look up nodes.
  fn on_active_element_changed(
    &mut self,
    tree: &mut Tree,
    old_path: Option<&[usize]>,
    new_path: Option<&[usize]>,
  ) -> TreeHookResponse {
    TreeHookResponse::Continue
  }
}

impl Tree {
  /// Register a tree hook and return a handle that can be used to remove it.
  pub fn add_hook(&mut self, hook: impl TreeHook + Send + 'static) -> TreeHookHandle {
    let handle = TreeHookHandle::new(hook);
    self.add_hook_handle(handle.clone());
    handle
  }

  /// Register an existing hook handle.
  ///
  /// This is useful when the same hook instance should be shared across
  /// cloned trees or external owner state.
  pub fn add_hook_handle(&mut self, handle: TreeHookHandle) {
    self.hooks.push(handle);
  }

  /// Remove a previously registered hook by handle.
  ///
  /// Returns `true` when a hook was removed.
  pub fn remove_hook(&mut self, handle: &TreeHookHandle) -> bool {
    let old_len = self.hooks.len();
    self.hooks.retain(|h| !h.ptr_eq(handle));
    self.hooks.len() != old_len
  }

  /// Remove all registered hooks.
  pub fn clear_hooks(&mut self) {
    self.hooks.clear();
  }

  /// Number of hooks currently registered on the tree.
  pub fn hook_count(&self) -> usize {
    self.hooks.len()
  }

  /// Emit a render-frame event to all registered hooks.
  pub fn emit_render(&mut self, event: &TreeRenderEvent<'_>) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_render(tree, event))
  }

  /// Emit a lifecycle/profiling marker to all registered hooks.
  pub fn emit_lifecycle_event(&mut self, event: &TreeLifecycleEvent<'_>) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_lifecycle_event(tree, event))
  }

  pub fn emit_lifecycle_begin(&mut self, stage: TreeLifecycleStage) -> TreeHookResponse {
    self.emit_lifecycle_event(&TreeLifecycleEvent::begin(stage))
  }

  pub fn emit_lifecycle_end(&mut self, stage: TreeLifecycleStage, duration: Duration) -> TreeHookResponse {
    self.emit_lifecycle_event(&TreeLifecycleEvent::end(stage, duration))
  }

  pub fn emit_lifecycle_instant(&mut self, stage: TreeLifecycleStage) -> TreeHookResponse {
    self.emit_lifecycle_event(&TreeLifecycleEvent::instant(stage))
  }

  /// Emit a DOM-style event to all registered hooks.
  pub fn emit_event(&mut self, event: &mut HtmlEvent) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_event(tree, event))
  }

  /// Emit the tree crate's lightweight mouse event to all registered hooks.
  pub fn emit_mouse_event(&mut self, event: &mut TreeMouseEvent) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_mouse_event(tree, event))
  }

  /// Notify hooks that a node was added to the tree.
  pub fn emit_element_added(&mut self, element: &Node) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_element_added(tree, element))
  }

  /// Notify hooks that a node was removed from the tree.
  pub fn emit_element_removed(&mut self, element: &Node) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_element_removed(tree, element))
  }

  /// Notify hooks that the focused (active) element changed.
  pub fn emit_active_element_changed(
    &mut self,
    old_path: Option<&[usize]>,
    new_path: Option<&[usize]>,
  ) -> TreeHookResponse {
    self.emit_hooks(|hook, tree| hook.on_active_element_changed(tree, old_path, new_path))
  }

  fn emit_hooks(&mut self, mut emit: impl FnMut(&mut HookObject, &mut Tree) -> TreeHookResponse) -> TreeHookResponse {
    let hooks = self.hooks.clone();
    for handle in hooks {
      let Ok(mut hook) = handle.inner.lock() else {
        continue;
      };
      if emit(hook.as_mut(), self).is_stop() {
        return TreeHookResponse::Stop;
      }
    }
    TreeHookResponse::Continue
  }
}
