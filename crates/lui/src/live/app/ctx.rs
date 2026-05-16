use std::{
  any::TypeId,
  collections::{HashMap, HashSet},
  panic::{catch_unwind, AssertUnwindSafe},
  sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc,
  },
};

use lui_core::{ArcStr, HtmlNode, Stylesheet as CoreStylesheet};
use parking_lot::Mutex;

use super::super::{
  builder::{
    el::El,
    style::Stylesheet,
  },
  signal::Signal,
  tracking,
};
use crate::{Lui, StylesheetHandle};

type CleanupFn = Box<dyn FnOnce() + Send>;
type Command = Box<dyn FnOnce(&mut Lui) + Send>;

static NEXT_REF_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct NodeRef {
  id: ArcStr,
}

impl NodeRef {
  pub fn new() -> Self {
    let n = NEXT_REF_ID.fetch_add(1, Ordering::Relaxed);
    Self {
      id: ArcStr::from(format!("__ref_{n}")),
    }
  }

  pub fn id(&self) -> &str {
    &self.id
  }

  pub fn get<'a>(&self, lui: &'a Lui) -> Option<&'a HtmlNode> {
    lui.doc.root.get_element_by_id(self.id.clone())
  }

  pub fn get_mut<'a>(&self, lui: &'a mut Lui) -> Option<&'a mut HtmlNode> {
    lui.doc.root.get_element_by_id_mut(self.id.clone())
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ComponentId(TypeId);

impl ComponentId {
  pub fn of<F: 'static>() -> Self {
    Self(TypeId::of::<F>())
  }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct ChildKey {
  key: String,
  component_id: ComponentId,
  render_order: usize,
}

#[derive(Clone)]
pub struct CommandSender {
  dirty: Arc<AtomicBool>,
  queue: Arc<Mutex<Vec<Command>>>,
}

impl CommandSender {
  pub fn command(&self, f: impl FnOnce(&mut Lui) + Send + 'static) {
    self.queue.lock().push(Box::new(f));
    self.dirty.store(true, Ordering::Relaxed);
  }

  fn command_arc(&self, f: &Arc<dyn Fn(&mut Lui) + Send + Sync>) {
    let f = Arc::clone(f);
    self.queue.lock().push(Box::new(move |lui| f(lui)));
    self.dirty.store(true, Ordering::Relaxed);
  }
}

type LifecycleFn = Box<dyn FnOnce() + Send>;

type ContextMap = HashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>>;

pub(crate) struct StyleRegistry {
  installed: HashMap<ComponentId, StylesheetHandle>,
  pending: Vec<(ComponentId, CoreStylesheet)>,
  active: HashSet<ComponentId>,
}

impl StyleRegistry {
  pub fn new() -> Self {
    Self {
      installed: HashMap::new(),
      pending: Vec::new(),
      active: HashSet::new(),
    }
  }

  pub fn flush(&mut self, lui: &mut Lui) {
    for (id, sheet) in self.pending.drain(..) {
      let handle = lui.add_stylesheet(sheet);
      self.installed.insert(id, handle);
    }
    let to_remove: Vec<ComponentId> = self
      .installed
      .keys()
      .filter(|id| !self.active.contains(id))
      .copied()
      .collect();
    for id in to_remove {
      if let Some(handle) = self.installed.remove(&id) {
        lui.remove_stylesheet(handle);
      }
    }
    self.active.clear();
  }
}

pub struct Ctx {
  component_id: ComponentId,
  dirty: Arc<AtomicBool>,
  commands: Arc<Mutex<Vec<Command>>>,
  style_registry: Arc<Mutex<StyleRegistry>>,
  hooks: Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>,
  hook_pos: AtomicUsize,
  dirty_subs: Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>,
  children: Mutex<HashMap<ChildKey, Arc<Ctx>>>,
  child_order: AtomicUsize,
  active_children: Mutex<Vec<ChildKey>>,
  contexts: Mutex<ContextMap>,
  scope: Mutex<String>,
  mounted_callbacks: Mutex<Vec<LifecycleFn>>,
  cleanups: Mutex<Vec<CleanupFn>>,
}

impl Ctx {
  pub(crate) fn new(
    component_id: ComponentId,
    dirty: Arc<AtomicBool>,
    commands: Arc<Mutex<Vec<Command>>>,
    style_registry: Arc<Mutex<StyleRegistry>>,
    contexts: ContextMap,
  ) -> Self {
    Self {
      component_id,
      dirty,
      commands,
      style_registry,
      hooks: Mutex::new(Vec::new()),
      hook_pos: AtomicUsize::new(0),
      dirty_subs: Mutex::new(Vec::new()),
      children: Mutex::new(HashMap::new()),
      child_order: AtomicUsize::new(0),
      active_children: Mutex::new(Vec::new()),
      contexts: Mutex::new(contexts),
      scope: Mutex::new(String::new()),
      mounted_callbacks: Mutex::new(Vec::new()),
      cleanups: Mutex::new(Vec::new()),
    }
  }

  pub fn component_id(&self) -> ComponentId {
    self.component_id
  }

  pub fn sender(&self) -> CommandSender {
    CommandSender {
      dirty: self.dirty.clone(),
      queue: self.commands.clone(),
    }
  }

  pub fn cmd<E: 'static>(&self, f: impl Fn(&mut Lui) + Send + Sync + 'static) -> impl Fn(&E) + Send + Sync + 'static {
    let sender = self.sender();
    let f: Arc<dyn Fn(&mut Lui) + Send + Sync> = Arc::new(f);
    move |_: &E| {
      sender.command_arc(&f);
    }
  }

  // ── Reactive primitives ──────────────────────────────────────────

  pub fn signal<T: Clone + Send + Sync + 'static>(&self, initial: T) -> Signal<T> {
    let mut hooks = self.hooks.lock();
    let pos = self.hook_pos.fetch_add(1, Ordering::Relaxed);

    if let Some(existing) = hooks.get(pos) {
      return existing.downcast_ref::<Signal<T>>().unwrap().clone();
    }

    let sig = Signal::new(initial);

    let dirty = self.dirty.clone();
    let sub = sig.subscribe(move |_| {
      dirty.store(true, Ordering::Relaxed);
    });
    self.dirty_subs.lock().push(Box::new(sub));

    hooks.push(Box::new(sig.clone()));
    sig
  }

  pub fn memo<T: Clone + Send + Sync + PartialEq + 'static>(
    &self,
    f: impl Fn() -> T + Send + Sync + 'static,
  ) -> Signal<T> {
    let mut hooks = self.hooks.lock();
    let pos = self.hook_pos.fetch_add(1, Ordering::Relaxed);

    if let Some(existing) = hooks.get(pos) {
      return existing.downcast_ref::<Signal<T>>().unwrap().clone();
    }

    let sig = Signal::new(f());

    let dirty = self.dirty.clone();
    let sub = sig.subscribe(move |_| {
      dirty.store(true, Ordering::Relaxed);
    });
    self.dirty_subs.lock().push(Box::new(sub));

    let sig_write = sig.clone();
    let effect = TrackedEffect::new(move || {
      let new_val = f();
      let changed = sig_write.with_untracked(|current| *current != new_val);
      if changed {
        sig_write.set(new_val);
      }
    });
    TrackedEffect::run(&effect);
    self.dirty_subs.lock().push(Box::new(effect));

    hooks.push(Box::new(sig.clone()));
    sig
  }

  pub fn store<T: Clone + Send + Sync + 'static>(&self, initial: T) -> Store<T> {
    Store(self.signal(initial))
  }

  // ── Context ──────────────────────────────────────────────────────

  pub fn provide<T: Send + Sync + 'static>(&self, value: T) {
    self.contexts.lock().insert(TypeId::of::<T>(), Arc::new(value));
  }

  pub fn use_context<T: Send + Sync + Clone + 'static>(&self) -> Option<T> {
    self
      .contexts
      .lock()
      .get(&TypeId::of::<T>())
      .and_then(|v| v.downcast_ref::<T>())
      .cloned()
  }

  // ── Styles ───────────────────────────────────────────────────────

  pub fn styles(&self, sheet: Stylesheet) {
    let scope_name = sheet.scope().to_string();
    *self.scope.lock() = scope_name;

    let mut registry = self.style_registry.lock();
    registry.active.insert(self.component_id);

    if registry.installed.contains_key(&self.component_id) {
      return;
    }
    if registry.pending.iter().any(|(id, _)| *id == self.component_id) {
      return;
    }

    if let Ok(core_sheet) = sheet.try_to_core_stylesheet() {
      registry.pending.push((self.component_id, core_sheet));
    }
  }

  pub fn scoped(&self, class: &str) -> String {
    let scope = self.scope.lock();
    if scope.is_empty() {
      class.to_string()
    } else {
      format!("{}-{}", *scope, class)
    }
  }

  // ── Effects & watchers ───────────────────────────────────────────

  pub fn watch<T: Clone + Send + Sync + 'static>(&self, signal: &Signal<T>, f: impl Fn(&T) + Send + Sync + 'static) {
    let mut hooks = self.hooks.lock();
    let pos = self.hook_pos.fetch_add(1, Ordering::Relaxed);
    if pos < hooks.len() {
      return;
    }

    let sig = signal.clone();
    let handle = signal.watch(move || {
      sig.with_untracked(|val| f(val));
    });
    hooks.push(Box::new(handle));
  }

  pub fn on_effect(&self, f: impl Fn() + Send + Sync + 'static) {
    let mut hooks = self.hooks.lock();
    let pos = self.hook_pos.fetch_add(1, Ordering::Relaxed);

    if pos < hooks.len() {
      return;
    }

    let effect = TrackedEffect::new(f);
    TrackedEffect::run(&effect);
    hooks.push(Box::new(effect));
  }

  // ── Components ───────────────────────────────────────────────────

  pub fn for_each<T, K, F: Fn(&Ctx, &T) -> El + 'static>(
    &self,
    items: &[T],
    key_fn: impl Fn(&T) -> K,
    render: F,
  ) -> Vec<El>
  where
    K: ToString,
  {
    items
      .iter()
      .map(|item| {
        let k = key_fn(item).to_string();
        self.mount_child(&k, ComponentId::of::<F>(), item, |ctx, item| render(ctx, item))
      })
      .collect()
  }

  pub fn component<F: Fn(&Ctx, P) -> El + 'static, P>(&self, f: F, props: P) -> El {
    self.mount_child("", ComponentId::of::<F>(), props, |ctx, p| f(ctx, p))
  }

  pub fn keyed<F: Fn(&Ctx, P) -> El + 'static, P>(&self, key: impl Into<String>, f: F, props: P) -> El {
    self.mount_child(&key.into(), ComponentId::of::<F>(), props, |ctx, p| f(ctx, p))
  }

  pub fn error_boundary<F: Fn(&Ctx) -> El + 'static>(&self, f: F, fallback: impl Fn(&str) -> El + 'static) -> El {
    let (child_ctx, _first_render) = {
      let mut hooks = self.hooks.lock();
      let pos = self.hook_pos.fetch_add(1, Ordering::Relaxed);

      if let Some(existing) = hooks.get(pos) {
        (existing.downcast_ref::<Arc<Ctx>>().unwrap().clone(), false)
      } else {
        let id = ComponentId::of::<F>();
        let inherited_contexts = self.contexts.lock().clone();
        let child = Arc::new(Ctx::new(
          id,
          self.dirty.clone(),
          self.commands.clone(),
          self.style_registry.clone(),
          inherited_contexts,
        ));
        hooks.push(Box::new(child.clone()));
        (child, true)
      }
    };

    child_ctx.begin_render();
    let result = catch_unwind(AssertUnwindSafe(|| f(&child_ctx)));
    child_ctx.end_render();

    match result {
      Ok(el) => el,
      Err(panic) => {
        let msg = if let Some(s) = panic.downcast_ref::<&str>() {
          s.to_string()
        } else if let Some(s) = panic.downcast_ref::<String>() {
          s.clone()
        } else {
          "component panicked".to_string()
        };
        fallback(&msg)
      }
    }
  }

  fn mount_child<P>(&self, key: &str, cid: ComponentId, props: P, render: impl FnOnce(&Ctx, P) -> El) -> El {
    let order = self.child_order.fetch_add(1, Ordering::Relaxed);
    let child_key = ChildKey {
      key: key.to_string(),
      component_id: cid,
      render_order: order,
    };

    let (child_ctx, first_render) = {
      let mut children = self.children.lock();

      let found = if !key.is_empty() {
        children
          .iter()
          .find(|(k, _)| k.key == child_key.key && k.component_id == child_key.component_id)
          .map(|(_, v)| v.clone())
      } else {
        children.get(&child_key).cloned()
      };

      if let Some(existing) = found {
        self.active_children.lock().push(child_key);
        (existing, false)
      } else {
        let inherited_contexts = self.contexts.lock().clone();
        let child = Arc::new(Ctx::new(
          cid,
          self.dirty.clone(),
          self.commands.clone(),
          self.style_registry.clone(),
          inherited_contexts,
        ));
        children.insert(child_key.clone(), child.clone());
        self.active_children.lock().push(child_key);
        (child, true)
      }
    };

    child_ctx.begin_render();
    let el = render(&child_ctx, props);
    child_ctx.end_render();

    if first_render {
      for cb in child_ctx.drain_mounted_callbacks() {
        cb();
      }
    }

    el
  }

  // ── Refs & lifecycle ─────────────────────────────────────────────

  pub fn node_ref(&self) -> NodeRef {
    let mut hooks = self.hooks.lock();
    let pos = self.hook_pos.fetch_add(1, Ordering::Relaxed);

    if let Some(existing) = hooks.get(pos) {
      return existing.downcast_ref::<NodeRef>().unwrap().clone();
    }

    let r = NodeRef::new();
    hooks.push(Box::new(r.clone()));
    r
  }

  pub fn on_mounted(&self, f: impl FnOnce() + Send + 'static) {
    self.mounted_callbacks.lock().push(Box::new(f));
  }

  pub fn on_unmounted(&self, f: impl FnOnce() + Send + 'static) {
    self.cleanups.lock().push(Box::new(f));
  }

  pub fn on_cleanup(&self, f: impl FnOnce() + Send + 'static) {
    self.cleanups.lock().push(Box::new(f));
  }

  // ── Internal ─────────────────────────────────────────────────────

  pub(crate) fn is_dirty(&self) -> bool {
    self.dirty.load(Ordering::Relaxed)
  }

  pub(crate) fn clear_dirty(&self) {
    self.dirty.store(false, Ordering::Relaxed);
  }

  pub(crate) fn begin_render(&self) {
    self.hook_pos.store(0, Ordering::Relaxed);
    self.child_order.store(0, Ordering::Relaxed);
    self.active_children.lock().clear();
  }

  pub(crate) fn end_render(&self) {
    let active = self.active_children.lock();
    let mut children = self.children.lock();

    children.retain(|k, _| {
      if k.key.is_empty() {
        active.contains(k)
      } else {
        active
          .iter()
          .any(|a| a.key == k.key && a.component_id == k.component_id)
      }
    });
  }

  pub(crate) fn drain_commands(&self) -> Vec<Command> {
    std::mem::take(&mut *self.commands.lock())
  }

  pub(crate) fn drain_mounted_callbacks(&self) -> Vec<LifecycleFn> {
    std::mem::take(&mut *self.mounted_callbacks.lock())
  }

  pub(crate) fn flush_styles(&self, lui: &mut Lui) {
    self.style_registry.lock().flush(lui);
  }
}

impl Drop for Ctx {
  fn drop(&mut self) {
    let cleanups: Vec<CleanupFn> = std::mem::take(self.cleanups.get_mut());
    for cleanup in cleanups {
      cleanup();
    }
  }
}

// ── Store ────────────────────────────────────────────────────────────

pub struct Store<T>(Signal<T>);

impl<T: Clone + Send + Sync + 'static> Store<T> {
  pub fn get(&self) -> T {
    self.0.get()
  }

  pub fn get_untracked(&self) -> T {
    self.0.get_untracked()
  }

  pub fn set(&self, value: T) {
    self.0.set(value);
  }

  pub fn update(&self, f: impl FnOnce(&mut T)) {
    self.0.update(f);
  }

  pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R
  where
    T: Send + Sync + 'static,
  {
    self.0.with(f)
  }

  pub fn lens<F, R>(&self, field: F) -> Lens<R>
  where
    F: Fn(&T) -> R + Send + Sync + 'static,
    R: Clone + Send + Sync + PartialEq + 'static,
    T: Send + Sync + 'static,
  {
    let store = self.0.clone();
    let initial = store.with(|v| field(v));
    let sig = Signal::new(initial);

    let sig_write = sig.clone();
    let effect = TrackedEffect::new(move || {
      let new_val = store.get();
      let projected = field(&new_val);
      let changed = sig_write.with_untracked(|current| *current != projected);
      if changed {
        sig_write.set(projected);
      }
    });
    TrackedEffect::run(&effect);

    Lens {
      signal: sig,
      _effect: effect,
    }
  }

  pub fn signal(&self) -> Signal<T> {
    self.0.clone()
  }
}

impl<T> Clone for Store<T> {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

pub struct Lens<T> {
  signal: Signal<T>,
  _effect: Arc<TrackedEffect>,
}

impl<T: Clone + Send + Sync + 'static> Lens<T> {
  pub fn get(&self) -> T {
    self.signal.get()
  }

  pub fn get_untracked(&self) -> T {
    self.signal.get_untracked()
  }

  pub fn signal(&self) -> Signal<T> {
    self.signal.clone()
  }
}

impl<T> Clone for Lens<T> {
  fn clone(&self) -> Self {
    Self {
      signal: self.signal.clone(),
      _effect: self._effect.clone(),
    }
  }
}

// ── TrackedEffect ────────────────────────────────────────────────────

struct TrackedEffect {
  f: Arc<dyn Fn() + Send + Sync>,
  subscriptions: Mutex<Vec<Box<dyn std::any::Any + Send + Sync>>>,
  generation: AtomicUsize,
}

impl TrackedEffect {
  fn new(f: impl Fn() + Send + Sync + 'static) -> Arc<Self> {
    Arc::new(Self {
      f: Arc::new(f),
      subscriptions: Mutex::new(Vec::new()),
      generation: AtomicUsize::new(0),
    })
  }

  fn run(this: &Arc<Self>) {
    let current_gen = this.generation.fetch_add(1, Ordering::Relaxed) + 1;
    this.subscriptions.lock().clear();

    tracking::start_tracking();
    (this.f)();
    let deps = tracking::stop_tracking();

    let mut subs = this.subscriptions.lock();
    for (_, subscribe_fn) in deps {
      let weak = Arc::downgrade(this);
      let captured_gen = current_gen;
      let handle = subscribe_fn(Arc::new(move || {
        if let Some(effect) = weak.upgrade() {
          if effect.generation.load(Ordering::Relaxed) == captured_gen {
            TrackedEffect::run(&effect);
          }
        }
      }));
      subs.push(handle);
    }
  }
}
