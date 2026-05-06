mod counter;
mod text_input;

use std::sync::Arc;

use wgpu_html_driver_winit::{WinitDriver, WindowEvent};
use wgpu_html_models::common::css_enums::*;
use wgpu_html_tree::Tree;
use wgpu_html_ui::{
  Component, Ctx, El, Mount, ShouldRender, el,
  style::{self, px},
};
use winit::{
  application::ApplicationHandler,
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  window::{Window, WindowId},
};

use crate::{
  counter::counter::{Counter, CounterProps},
  text_input::TextInput,
};

// ── Root App Component ──────────────────────────────────────────────────────

struct DemoApp;

#[derive(Clone)]
struct DemoProps;

#[derive(Clone)]
enum DemoMsg {}

impl Component for DemoApp {
  type Props = DemoProps;
  type Msg = DemoMsg;

  fn scope() -> &'static str {
    "app"
  }

  fn styles() -> style::Stylesheet {
    style::sheet([
      style::rule(".root")
        .display(Display::Flex)
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .flex_grow(1.0)
        .background_color("#1a1a1a")
        .font_family("sans-serif"),
      style::rule(".title")
        .font_size(px(20))
        .font_weight(FontWeight::Weight(600))
        .color("#8AB4F8")
        .margin_bottom(px(40))
        .white_space(WhiteSpace::Nowrap),
      style::rule(".counters").display(Display::Flex).gap(px(24)),
    ])
  }

  fn create(_props: &DemoProps) -> Self {
    DemoApp
  }

  fn update(&mut self, _msg: DemoMsg, _props: &DemoProps) -> ShouldRender {
    ShouldRender::No
  }

  fn view(&self, _props: &DemoProps, ctx: &Ctx<DemoMsg>) -> El {
    el::div().class(ctx.scoped("root")).children([
      el::div().class(ctx.scoped("title")).text("wgpu-html-ui Demo"),
      ctx.child::<TextInput>(()),
      el::div().class(ctx.scoped("counters")).children([
        ctx.child::<Counter>(CounterProps { label: "Clicks" }),
        ctx.child::<Counter>(CounterProps { label: "Score" }),
      ]),
    ])
  }
}

// ── Winit harness ──────────────────────────────────────────────────────────

struct UiDemoApp {
  mount: Mount<DemoApp>,
  tree: Tree,
  devtools: wgpu_html_devtools::Devtools,
  driver: Option<WinitDriver>,
  devtools_driver: Option<WinitDriver>,
}

impl ApplicationHandler for UiDemoApp {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.driver.is_some() {
      return;
    }
    let attrs = Window::default_attributes()
      .with_title("wgpu-html-ui demo")
      .with_inner_size(winit::dpi::PhysicalSize::new(800u32, 500u32));
    let window = Arc::new(event_loop.create_window(attrs).unwrap());
    let tree = std::mem::replace(&mut self.tree, Tree::default());
    self.driver = Some(WinitDriver::bind(window, tree));
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
    let Some(driver) = &mut self.driver else { return };

    if window_id != driver.window_id() {
      if let Some(dd) = &mut self.devtools_driver {
        if dd.window_id() == window_id {
          match &event {
            WindowEvent::CloseRequested => {
              self.devtools.disable();
              dd.window().set_visible(false);
            }
            WindowEvent::RedrawRequested => {
              dd.render(self.devtools.tree_mut());
              self.devtools.frame_rendered();
            }
            other => {
              if dd.dispatch_to(other, self.devtools.tree_mut()) {
                dd.request_redraw();
              }
            }
          }
        }
      }
      return;
    }

    match event {
      WindowEvent::CloseRequested => event_loop.exit(),
      WindowEvent::RedrawRequested => {
        if self.mount.process(&mut driver.tree) {
          driver.request_redraw();
        }
        driver.rt.render_frame(&mut driver.tree);
      }
      other => {
        driver.handle_event(&other);
      }
    }
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    let Some(driver) = &self.driver else { return };

    self.devtools.poll(&driver.tree);

    if self.devtools.is_enabled() {
      if self.devtools_driver.is_none() {
        let attrs = Window::default_attributes()
          .with_title("DevTools")
          .with_inner_size(winit::dpi::PhysicalSize::new(1280u32, 720u32));
        let win = Arc::new(event_loop.create_window(attrs).expect("devtools window"));
        self.devtools.tree_mut().register_system_fonts("sans-serif");
        self.devtools_driver = Some(WinitDriver::bind(win, Tree::default()));
      }
      if let Some(dd) = &self.devtools_driver {
        dd.window().set_visible(true);
        if self.devtools.needs_redraw() {
          dd.request_redraw();
        }
      }
    } else if let Some(dd) = &self.devtools_driver {
      dd.window().set_visible(false);
    }
  }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
  let mut mount = Mount::<DemoApp>::new(DemoProps);
  let mut tree = Tree::default();
  tree.register_system_fonts("sans-serif");
  tree.register_linked_stylesheet("base", "html, body { height: 100%; margin: 0; background: #1a1a1a; }");
  tree.register_linked_stylesheet("theme", include_str!("theme.css"));
  mount.render(&mut tree);

  let devtools = wgpu_html_devtools::Devtools::attach(&mut tree, false);

  let event_loop = EventLoop::new().unwrap();
  let mut app = UiDemoApp {
    mount,
    tree,
    devtools,
    driver: None,
    devtools_driver: None,
  };
  event_loop.set_control_flow(ControlFlow::Wait);
  event_loop.run_app(&mut app).unwrap();
}
