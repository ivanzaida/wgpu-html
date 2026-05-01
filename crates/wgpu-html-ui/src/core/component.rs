//! Component trait and related types.

use crate::{
  core::ctx::{Ctx, MsgSender},
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
/// A component owns private state, receives immutable [`Props`](Component::Props)
/// from its parent, and communicates via [`Msg`](Component::Msg) messages dispatched
/// through callbacks.
///
/// [`Env`](Component::Env) is external data provided by the mount site at
/// render time (e.g. a reference to an inspected tree). Standalone apps
/// use `Env = ()`.
///
/// # Example
///
/// ```ignore
/// use wgpu_html_ui::{Component, Ctx, ShouldRender, el, El};
///
/// struct Counter { count: i32 }
///
/// #[derive(Clone)]
/// struct Props { label: String }
///
/// #[derive(Clone)]
/// enum Msg { Inc, Dec }
///
/// impl Component for Counter {
///     type Props = Props;
///     type Msg = Msg;
///     type Env = ();
///
///     fn create(_props: &Props) -> Self { Counter { count: 0 } }
///
///     fn update(&mut self, msg: Msg, _props: &Props) -> ShouldRender {
///         match msg {
///             Msg::Inc => self.count += 1,
///             Msg::Dec => self.count -= 1,
///         }
///         ShouldRender::Yes
///     }
///
///     fn view(&self, props: &Props, ctx: &Ctx<Msg>, _env: &()) -> El {
///         el::div().children([
///             el::span().text(&props.label),
///             el::button().text("-").on_click(ctx.on_click(Msg::Dec)),
///             el::span().text(&self.count.to_string()),
///             el::button().text("+").on_click(ctx.on_click(Msg::Inc)),
///         ])
///     }
/// }
/// ```
pub trait Component: 'static {
  /// Immutable configuration passed from the parent component.
  type Props: Clone + 'static;

  /// Messages produced by user interactions or other events.
  type Msg: Clone + Send + Sync + 'static;

  /// External data provided by the mount site at render time.
  /// Use `()` for standalone applications.
  type Env: 'static;

  /// Create a new component instance from initial props.
  fn create(props: &Self::Props) -> Self;

  /// Handle a message.  Return [`ShouldRender::Yes`] to trigger
  /// a call to [`view`](Component::view) and subtree replacement.
  fn update(&mut self, msg: Self::Msg, props: &Self::Props) -> ShouldRender;

  /// Produce the element tree for the current state, props, and
  /// environment.
  fn view(&self, props: &Self::Props, ctx: &Ctx<Self::Msg>, env: &Self::Env) -> El;

  /// Called when the parent passes new props.  Default: always re-render.
  fn props_changed(&mut self, _old: &Self::Props, _new: &Self::Props) -> ShouldRender {
    ShouldRender::Yes
  }

  /// Called once after the component is first mounted.
  ///
  /// `sender` can be stored and used to subscribe to [`Store`](crate::Store)
  /// updates or to send messages from background threads.
  fn mounted(&mut self, _sender: MsgSender<Self::Msg>) {}

  /// Called after every re-render triggered by this component's own
  /// [`update`](Component::update) returning [`ShouldRender::Yes`].
  /// Analogous to React's `componentDidUpdate` / Vue's `updated` hook.
  /// Not called after the initial mount render.
  fn updated(&mut self, _props: &Self::Props) {}

  /// Called before the component is destroyed.
  fn destroyed(&mut self) {}

  /// Scope prefix for this component's styles. When non-empty, class
  /// names in [`styles()`](Component::styles) are auto-prefixed with
  /// this value, and [`Ctx::scoped`] prepends it in view().
  ///
  /// Return `""` (default) for no scoping / global styles.
  fn scope() -> &'static str
  where
    Self: Sized,
  {
    ""
  }

  /// Component-level stylesheet. Registered once when the component
  /// type is first mounted. If [`scope()`](Component::scope) is
  /// non-empty, all `.class` selectors are prefixed automatically.
  fn styles() -> Stylesheet
  where
    Self: Sized,
  {
    Stylesheet::empty()
  }
}
