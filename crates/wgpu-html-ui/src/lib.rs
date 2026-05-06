//! Platform-agnostic component framework for `wgpu-html`.
//!
//! Provides an Elm-architecture component model with an ergonomic
//! builder DSL for constructing element trees.
//!
//! # Quick start
//!
//! ```ignore
//! use wgpu_html_ui::{el, Component, Ctx, El, Mount, ShouldRender};
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
//!     fn view(&self, props: &Props, ctx: &Ctx<Msg>) -> El {
//!         el::div().children([
//!             el::span().text(&props.label),
//!             el::button().text("-").on_click(ctx.on_click(Msg::Dec)),
//!             el::span().text(&self.count.to_string()),
//!             el::button().text("+").on_click(ctx.on_click(Msg::Inc)),
//!         ])
//!     }
//! }
//! ```

// ── DSL modules (public) ─────────────────────────────────────────────────────
pub mod el;
pub mod style;

// ── Shared reactive state ────────────────────────────────────────────────────
mod store;

// ── Core Elm-architecture machinery ─────────────────────────────────────────
pub mod core;

// ── Mount (manual component driver) ─────────────────────────────────────────
mod app;

// ── Public re-exports ────────────────────────────────────────────────────────
pub use core::{
  component::{Component, ShouldRender},
  ctx::{Ctx, MsgSender},
  observable::{Observable, Subscription},
  runtime::Runtime,
};

pub use app::mount::Mount;

pub use el::{
  AnchorAttrs, AudioAttrs, BlockquoteAttrs, ButtonAttrs, CanvasAttrs, Children, ColAttrs, ColgroupAttrs, DataElAttrs,
  DelAttrs, DetailsAttrs, DialogAttrs, El, FieldsetAttrs, FormAttrs, IframeAttrs, ImgAttrs, InputAttrs, InsAttrs,
  LabelAttrs, LinkAttrs, MetaAttrs, MeterAttrs, OlAttrs, OptgroupAttrs, OptionAttrs, OutputAttrs, ProgressAttrs,
  ScriptAttrs, SelectAttrs, SourceAttrs, SvgAttrs, SvgPathAttrs, TdAttrs, TextareaAttrs, ThAttrs, TimeAttrs,
  TrackAttrs, VideoAttrs,
};
pub use store::Store;

/// Re-export commonly-used HTML attribute enum types.
pub mod html {
  pub use wgpu_html_models::common::html_enums::{
    AutoComplete, ButtonType, CaptureMode, CrossOrigin, FormEncoding, FormMethod, ImageDecoding, InputType, LinkAs,
    LinkTarget, Loading, OlType, Preload, ReferrerPolicy, SvgLength, TableHeaderScope, TextareaWrap, TrackKind,
  };
}
