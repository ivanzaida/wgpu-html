//! Elm-architecture component system.
//!
//! Each component owns private state, receives immutable props from its
//! parent, and communicates via typed messages dispatched through
//! callbacks. The runtime manages mounting, rendering, reconciliation,
//! and destruction.
//!
//! # Lifecycle
//!
//! ```text
//! create(props)          — construct instance from initial props
//!     ↓
//! view(props, ctx)       — produce the element tree (first render)
//!     ↓
//! mounted(sender)        — called once; store sender for async work
//!     ↓
//! subscribe(sender, subs)— set up Observable subscriptions (auto-cleaned)
//!     ↓
//! ┌─ update(msg, props)  — handle a message, return ShouldRender
//! │      ↓ Yes
//! │  view(props, ctx)    — re-render
//! │      ↓
//! │  updated(props)      — post-render hook (not called on initial mount)
//! └──────┘
//!     ↓ (parent removes component)
//! destroyed()            — cleanup; subscriptions auto-dropped
//! ```
//!
//! # Rendering optimization
//!
//! The runtime uses three render paths per component:
//!
//! - **Path 1 (clean):** Component and subtree unchanged — returns
//!   cached output instantly. No `view()` call.
//! - **Path 2 (patch):** Component unchanged but a child is dirty —
//!   `view()` is skipped; only dirty children re-render.
//! - **Path 3 (full):** Component's own `update()` returned `Yes` —
//!   `view()` is called and children are reconciled by key.
//!
//! `ctx.scoped()` class names are cached across renders — after the
//! first frame, repeated calls return a cheap `ArcStr` clone.

use crate::{
  core::ctx::{Ctx, MsgSender},
  core::observable::Subscriptions,
  el::El,
  style::Stylesheet,
};

/// Whether a component's view should be re-rendered after an update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShouldRender {
  Yes,
  No,
}

/// Elm-architecture component.
///
/// # Example
///
/// ```ignore
/// use wgpu_html_ui::{Component, Ctx, ShouldRender, el, El, Observable, Subscriptions, MsgSender};
///
/// struct Counter {
///     count: i32,
///     shared: Observable<i32>,
/// }
///
/// #[derive(Clone)]
/// struct Props { label: String, shared: Observable<i32> }
///
/// #[derive(Clone)]
/// enum Msg { Inc, Dec, Synced(i32) }
///
/// impl Component for Counter {
///     type Props = Props;
///     type Msg = Msg;
///
///     fn create(props: &Props) -> Self {
///         Counter { count: 0, shared: props.shared.clone() }
///     }
///
///     fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
///         match msg {
///             Msg::Inc => { self.count += 1; self.shared.set(self.count); }
///             Msg::Dec => { self.count -= 1; self.shared.set(self.count); }
///             Msg::Synced(v) => { self.count = v; }
///         }
///         ShouldRender::Yes
///     }
///
///     fn view(&self, props: &Props, ctx: &Ctx<Msg>) -> El {
///         el::div().children([
///             el::span().text(&props.label),
///             el::button().text("-").on_click_cb(ctx.on_click(Msg::Dec)),
///             el::span().text(&self.count.to_string()),
///             el::button().text("+").on_click_cb(ctx.on_click(Msg::Inc)),
///         ])
///     }
///
///     fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
///         subs.add(self.shared.subscribe_msg(sender, |v| Msg::Synced(*v)));
///     }
///
///     fn styles() -> wgpu_html_ui::style::Stylesheet {
///         wgpu_html_ui::style::sheet([
///             wgpu_html_ui::style::rule(".root")
///                 .display(wgpu_html_models::common::Display::Flex)
///                 .gap(wgpu_html_ui::style::px(8)),
///         ]).scoped("counter")
///     }
/// }
/// ```
pub trait Component: 'static {
  /// Immutable configuration passed from the parent component.
  type Props: Clone + 'static;

  /// Messages produced by user interactions or other events.
  type Msg: Clone + Send + Sync + 'static;

  /// Construct a new instance from initial props. Called once when
  /// the component is first added to the tree. Do not perform side
  /// effects here — use [`mounted`](Component::mounted) instead.
  fn create(props: &Self::Props) -> Self;

  /// Handle a message. Return [`ShouldRender::Yes`] to trigger a
  /// `view()` call and DOM patch.
  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;

  /// Produce the element tree for the current state and props.
  /// Called on first render and after each `update()` that returns
  /// `Yes`. Use `ctx.scoped("class")` for scoped class names
  /// (cached across renders — zero allocation after the first frame).
  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>) -> El;

  /// Called when the parent passes new props. Default: always re-render.
  fn props_changed(&mut self, _old: &Self::Props, _new: &Self::Props) -> ShouldRender {
    ShouldRender::Yes
  }

  /// Called once after the initial render. Use to store the sender
  /// for async work or background threads.
  fn mounted(&mut self, _sender: MsgSender<Self::Msg>) {}

  /// Called after every re-render triggered by this component's own
  /// `update()` returning `Yes`. Not called after the initial mount.
  fn updated(&mut self, _props: &Self::Props) {}

  /// Set up [`Observable`](crate::Observable) subscriptions that
  /// should live as long as the component. Called once after
  /// `mounted()`. Subscriptions added to `subs` are automatically
  /// cancelled when the component is destroyed — no need to store
  /// them as struct fields.
  ///
  /// ```ignore
  /// fn subscribe(&self, sender: &MsgSender<Msg>, subs: &mut Subscriptions) {
  ///     subs.add(self.value.subscribe_msg(sender, |v| Msg::ValueChanged(v.clone())));
  /// }
  /// ```
  fn subscribe(&self, _sender: &MsgSender<Self::Msg>, _subs: &mut Subscriptions) {}

  /// Called before the component is removed from the tree.
  /// Runtime-managed subscriptions are already cancelled at this
  /// point.
  fn destroyed(&mut self) {}

  /// Component-level stylesheet, registered once when the component
  /// type is first mounted. Use `.scoped("prefix")` on the
  /// stylesheet to auto-prefix class selectors; `ctx.scoped()` in
  /// `view()` produces matching class names.
  fn styles() -> Stylesheet
  where
    Self: Sized,
  {
    Stylesheet::empty()
  }
}
