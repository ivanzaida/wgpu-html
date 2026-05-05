//! Application entry point and winit integration.

use std::sync::{Arc, Mutex};

use wgpu_html_driver_winit::{WgpuHtml, WinitRuntime, dispatch};
use wgpu_html_models as m;
use wgpu_html_tree::{Node, Tree};
use winit::{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
};

use crate::core::{component::Component, runtime::Runtime};

// ── Secondary window trait ──────────────────────────────────────────────────

pub trait SecondaryWindow: 'static {
  fn poll(&mut self, tree: &Tree, event_loop: &ActiveEventLoop);
  fn on_key(&mut self, _tree: &Tree, _event_loop: &ActiveEventLoop, _event: &winit::event::KeyEvent) -> bool {
    false
  }
  fn owns_window(&self, id: winit::window::WindowId) -> bool;
  fn handle_window_event(&mut self, tree: &Tree, event: &winit::event::WindowEvent);
}

// ── App builder ─────────────────────────────────────────────────────────────

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
  pub fn setup_tree(mut self, f: impl FnOnce(&mut Tree) + 'static) -> Self {
    self.setup = Some(Box::new(f));
    self
  }
  pub fn with_secondary(mut self, factory: impl FnOnce(&mut Tree) -> Box<dyn SecondaryWindow> + 'static) -> Self {
    self.secondary_windows.push(Box::new(factory));
    self
  }

  pub fn run(self) -> Result<(), winit::error::EventLoopError> {
    let wake_slot: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>> = Arc::new(Mutex::new(None));
    let wake_slot_clone = wake_slot.clone();
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
      if let Some(w) = wake_slot_clone.lock().unwrap().as_ref() {
        w();
      }
    });

    let mut ui_runtime = (self.factory)(wake);
    let root_node = ui_runtime.initial_render(&*self.state);

    let body = Node::new(m::Body::default()).with_children(vec![root_node]);
    let html_node = Node::new(m::Html::default()).with_children(vec![body]);
    let mut tree = Tree::new(html_node);

    for (i, css) in self.stylesheets.iter().enumerate() {
      tree.register_linked_stylesheet(format!("__ui_style_{i}"), css.clone());
    }
    Runtime::register_styles(&mut tree, ui_runtime.root_mounted());
    crate::register_system_fonts(&mut tree);

    if let Some(setup) = self.setup {
      setup(&mut tree);
    }

    let secondaries: Vec<Box<dyn SecondaryWindow>> = self
      .secondary_windows
      .into_iter()
      .map(|factory| factory(&mut tree))
      .collect();

    let event_loop = EventLoop::new()?;
    #[allow(deprecated)]
    let window = Arc::new(
      event_loop
        .create_window(
          winit::window::Window::default_attributes()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.size.0, self.size.1)),
        )
        .unwrap(),
    );
    let driver = WgpuHtml { window: window.clone() };
    let rt = WinitRuntime::new(driver, self.size.0, self.size.1);

    let mut app = UiApp {
      tree,
      rt,
      window,
      ui_runtime,
      state: self.state,
      wake_slot,
      secondaries,
    };

    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut app)
  }
}

// ── Application handler ─────────────────────────────────────────────────────

struct UiApp<State: 'static> {
  tree: Tree,
  rt: WinitRuntime,
  window: Arc<winit::window::Window>,
  ui_runtime: Runtime,
  state: Arc<State>,
  wake_slot: Arc<Mutex<Option<Arc<dyn Fn() + Send + Sync>>>>,
  secondaries: Vec<Box<dyn SecondaryWindow>>,
}

impl<State: Send + Sync + 'static> ApplicationHandler for UiApp<State> {
  fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

  fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: winit::window::WindowId, event: WindowEvent) {
    if window_id != self.window.id() {
      for sec in &mut self.secondaries {
        if sec.owns_window(window_id) {
          sec.handle_window_event(&self.tree, &event);
          return;
        }
      }
      return;
    }

    match event {
      WindowEvent::CloseRequested => event_loop.exit(),

      WindowEvent::RedrawRequested => {
        {
          let mut slot = self.wake_slot.lock().unwrap();
          if slot.is_none() {
            let window = self.window.clone();
            *slot = Some(Arc::new(move || {
              window.request_redraw();
            }));
          }
        }
        if self.ui_runtime.process(&mut self.tree, &*self.state) {
          self.window.request_redraw();
        }
        for sec in &mut self.secondaries {
          sec.poll(&self.tree, event_loop);
        }
        self.rt.render_frame(&mut self.tree);
      }

      other => {
        if dispatch(&other, &mut self.rt, &mut self.tree) {
          self.window.request_redraw();
        }
      }
    }
  }
}
