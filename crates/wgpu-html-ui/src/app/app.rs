//! Application entry point and winit integration.

use std::sync::{Arc, Mutex};

use wgpu_html_models as m;
use wgpu_html_tree::{Node, Tree};
use wgpu_html_winit::{AppHook, EventResponse, FrameTimings, HookContext};
use winit::event::KeyEvent;

use crate::core::{component::Component, runtime::Runtime};

// ── App builder ─────────────────────────────────────────────────────────────

/// Application entry point.
///
/// `State` is shared application state accessible in `Component::view()`
/// via the `env` parameter.  Use `()` (the default) for stateless apps.
///
/// ```ignore
/// // Stateless:
/// App::new::<Counter>(props).run().unwrap();
///
/// // With shared state:
/// let db = MyDatabase::open();
/// App::with_state::<Dashboard>(db, props).run().unwrap();
/// ```
/// Trait for secondary window handlers (devtools, etc.) that need
/// per-frame polling and window event routing from the host app.
pub trait SecondaryWindow: 'static {
  fn poll(&mut self, tree: &Tree, event_loop: &winit::event_loop::ActiveEventLoop);
  fn on_key(&mut self, _tree: &Tree, _event_loop: &winit::event_loop::ActiveEventLoop, _event: &KeyEvent) -> bool {
    false
  }
  fn owns_window(&self, id: winit::window::WindowId) -> bool;
  fn handle_window_event(&mut self, tree: &Tree, event: &winit::event::WindowEvent);
}

pub struct App<State: 'static = ()> {
  state: Arc<State>,
  factory: Box<dyn FnOnce(Arc<dyn Fn() + Send + Sync>) -> Runtime>,
  title: String,
  size: (u32, u32),
  stylesheets: Vec<String>,
  setup: Option<Box<dyn FnOnce(&mut Tree)>>,
  secondary_windows: Vec<Box<dyn FnOnce(&mut Tree) -> Box<dyn SecondaryWindow>>>,
}

impl App<()> {
  /// Create a stateless application rooted at component `C`.
  pub fn new<C: Component<Env = ()>>(props: C::Props) -> Self
  where
    C::Props: Send + Sync + 'static,
    C::Msg: Clone + Send + Sync + 'static,
  {
    Self {
      state: Arc::new(()),
      factory: Box::new(move |wake| Runtime::new::<C>(&props, wake)),
      title: "wgpu-html".into(),
      size: (1280, 720),
      stylesheets: Vec::new(),
      setup: None,
      secondary_windows: Vec::new(),
    }
  }
}

impl<State: Send + Sync + 'static> App<State> {
  /// Create an application with shared state.
  ///
  /// The component's `type Env = State` and receives `&State` in
  /// its `view()` method.
  pub fn with_state<C: Component<Env = State>>(state: State, props: C::Props) -> Self
  where
    C::Props: Send + Sync + 'static,
    C::Msg: Clone + Send + Sync + 'static,
  {
    Self {
      state: Arc::new(state),
      factory: Box::new(move |wake| Runtime::new::<C>(&props, wake)),
      title: "wgpu-html".into(),
      size: (1280, 720),
      stylesheets: Vec::new(),
      setup: None,
      secondary_windows: Vec::new(),
    }
  }

  pub fn title(mut self, title: impl Into<String>) -> Self {
    self.title = title.into();
    self
  }

  pub fn size(mut self, width: u32, height: u32) -> Self {
    self.size = (width, height);
    self
  }

  pub fn stylesheet(mut self, css: impl Into<String>) -> Self {
    self.stylesheets.push(css.into());
    self
  }

  /// Run a setup callback with `&mut Tree` before the window opens.
  /// Use this to attach devtools, register extra fonts, etc.
  pub fn setup_tree(mut self, f: impl FnOnce(&mut Tree) + 'static) -> Self {
    self.setup = Some(Box::new(f));
    self
  }

  /// Add a secondary window (devtools, etc.) that opens on F11 and
  /// receives per-frame polling + window event routing.
  pub fn with_secondary(mut self, factory: impl FnOnce(&mut Tree) -> Box<dyn SecondaryWindow> + 'static) -> Self {
    self.secondary_windows.push(Box::new(factory));
    self
  }

  /// Run the application.  Blocks until the window is closed.
  pub fn run(self) -> Result<(), winit::error::EventLoopError> {
    let wake_slot: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>> = Arc::new(Mutex::new(None));

    let wake_slot_clone = wake_slot.clone();
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
      if let Some(w) = wake_slot_clone.lock().unwrap().as_ref() {
        w();
      }
    });

    let mut runtime = (self.factory)(wake);
    let root_node = runtime.initial_render(&*self.state);

    let body = Node::new(m::Body::default()).with_children(vec![root_node]);
    let html_node = Node::new(m::Html::default()).with_children(vec![body]);
    let mut tree = Tree::new(html_node);

    // Register user-provided global stylesheets.
    for (i, css) in self.stylesheets.iter().enumerate() {
      tree.register_linked_stylesheet(format!("__ui_style_{i}"), css.clone());
    }

    // Register component-level scoped styles.
    Runtime::register_styles(&mut tree, runtime.root_mounted());

    // Register system fonts so text renders.
    crate::register_system_fonts(&mut tree);

    if let Some(setup) = self.setup {
      setup(&mut tree);
    }

    let secondaries: Vec<Box<dyn SecondaryWindow>> = self
      .secondary_windows
      .into_iter()
      .map(|factory| factory(&mut tree))
      .collect();

    let hook = UiHook {
      runtime,
      state: self.state,
      wake_slot,
      secondaries,
    };

    wgpu_html_winit::create_window(&mut tree)
      .with_title(self.title)
      .with_size(self.size.0, self.size.1)
      .with_hook(hook)
      .run()
  }
}

// ── AppHook ─────────────────────────────────────────────────────────────────

struct UiHook<State: 'static> {
  runtime: Runtime,
  state: Arc<State>,
  wake_slot: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
  secondaries: Vec<Box<dyn SecondaryWindow>>,
}

impl<State: Send + Sync + 'static> AppHook for UiHook<State> {
  fn on_frame(&mut self, ctx: HookContext<'_>, _timings: &FrameTimings) {
    {
      let mut slot = self.wake_slot.lock().unwrap();
      if slot.is_none() {
        let window = ctx.window.clone();
        *slot = Some(Arc::new(move || {
          window.request_redraw();
        }));
      }
    }

    if self.runtime.process(ctx.tree, &*self.state) {
      ctx.window.request_redraw();
    }

    for sec in &mut self.secondaries {
      sec.poll(ctx.tree, ctx.event_loop);
    }
  }

  fn on_key(&mut self, ctx: HookContext<'_>, event: &KeyEvent) -> EventResponse {
    if self.runtime.process(ctx.tree, &*self.state) {
      ctx.window.request_redraw();
    }
    EventResponse::Continue
  }

  fn on_window_event(
    &mut self,
    ctx: HookContext<'_>,
    window_id: winit::window::WindowId,
    event: &winit::event::WindowEvent,
  ) -> bool {
    for sec in &mut self.secondaries {
      if sec.owns_window(window_id) {
        sec.handle_window_event(ctx.tree, event);
        return true;
      }
    }
    false
  }
}
