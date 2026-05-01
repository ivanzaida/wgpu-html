//! Component framework for `wgpu-html`.
//!
//! Provides an Elm-architecture component model with an ergonomic
//! builder DSL for constructing element trees.
//!
//! # Quick start
//!
//! ```ignore
//! use wgpu_html_ui::{el, App, Component, Ctx, El, ShouldRender};
//!
//! struct Counter { count: i32 }
//!
//! #[derive(Clone)]
//! struct Props { label: String }
//!
//! #[derive(Clone)]
//! enum Msg { Inc, Dec }
//!
//! impl Component for Counter {
//!     type Props = Props;
//!     type Msg = Msg;
//!     type Env = ();
//!
//!     fn create(_props: &Props) -> Self { Counter { count: 0 } }
//!
//!     fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
//!         match msg {
//!             Msg::Inc => self.count += 1,
//!             Msg::Dec => self.count -= 1,
//!         }
//!         ShouldRender::Yes
//!     }
//!
//!     fn view(&self, props: &Props, ctx: &Ctx<Msg>, _env: &()) -> El {
//!         el::div().children([
//!             el::span().text(&props.label),
//!             el::button().text("-").on_click(ctx.msg(Msg::Dec)),
//!             el::span().text(&self.count.to_string()),
//!             el::button().text("+").on_click(ctx.msg(Msg::Inc)),
//!         ])
//!     }
//! }
//!
//! fn main() {
//!     App::new::<Counter>(Props { label: "Count".into() })
//!         .title("Counter")
//!         .size(400, 300)
//!         .run()
//!         .unwrap();
//! }
//! ```

pub mod el;
pub mod style;

mod app;
mod component;
mod ctx;
mod mount;
pub(crate) mod runtime;

pub use app::{App, SecondaryWindow};
pub use component::{Component, ShouldRender};
pub use ctx::{Ctx, MsgSender};
pub use el::El;
pub use mount::Mount;

/// Register platform system fonts with the tree under the given family alias.
/// Call this once during app setup so text renders correctly.
pub(crate) fn register_system_fonts(tree: &mut wgpu_html_tree::Tree) {
    wgpu_html_winit::register_system_fonts(tree, "sans-serif");
}
