//! Builder DSL for constructing element trees.
//!
//! Each HTML element has a corresponding constructor function (e.g.
//! [`div()`], [`button()`], [`input()`]) that returns an [`El`] builder.
//! Chain methods to set attributes, add children, and attach callbacks:
//!
//! ```ignore
//! use wgpu_html_ui::el;
//!
//! el::div().id("app").class("container").children([
//!     el::h1().text("Hello"),
//!     el::button().class("btn").text("Click me")
//!         .on_click(|_| println!("clicked")),
//! ])
//! ```

use std::sync::Arc;

use wgpu_html_models as m;
use wgpu_html_tree::{Element, HtmlEvent, MouseEvent, Node};

/// A node builder. Wraps [`Node`] with chainable setter methods.
///
/// Convert to a raw [`Node`] via [`El::into_node`] or the [`From`] impl.
#[derive(Clone)]
pub struct El {
  pub(crate) node: Node,
}

impl El {
  /// Unwrap into the underlying [`Node`].
  #[inline]
  pub fn into_node(self) -> Node {
    self.node
  }
}

impl From<El> for Node {
  #[inline]
  fn from(el: El) -> Node {
    el.node
  }
}

// ── Children ────────────────────────────────────────────────────────────────

/// A list of child elements that can be passed as a prop for named-slot /
/// content-projection patterns.
///
/// # Example — component that accepts projected children
///
/// ```ignore
/// #[derive(Clone)]
/// struct CardProps {
///     header: El,           // single named slot
///     body:   Children,     // variadic slot
/// }
///
/// // In the parent's view():
/// let card = ctx.child::<Card>(CardProps {
///     header: el::h2().text("Title"),
///     body:   Children::from([
///         el::p().text("paragraph 1"),
///         el::p().text("paragraph 2"),
///     ]),
/// });
/// ```
///
/// Inside `Card::view`, render the slots with `.children(props.body.iter())`.
#[derive(Clone, Default)]
pub struct Children(Vec<El>);

impl Children {
  /// Empty children list.
  pub fn empty() -> Self {
    Self(Vec::new())
  }

  /// Create from any iterator of [`El`].
  pub fn from(iter: impl IntoIterator<Item = El>) -> Self {
    Self(iter.into_iter().collect())
  }

  /// Number of children.
  pub fn len(&self) -> usize {
    self.0.len()
  }

  /// True if the list is empty.
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Iterate over the children.
  pub fn iter(&self) -> impl Iterator<Item = El> + '_ {
    self.0.iter().cloned()
  }
}

impl IntoIterator for Children {
  type Item = El;
  type IntoIter = std::vec::IntoIter<El>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl FromIterator<El> for Children {
  fn from_iter<I: IntoIterator<Item = El>>(iter: I) -> Self {
    Self(iter.into_iter().collect())
  }
}

// ── Variant list ────────────────────────────────────────────────────────────
//
// Duplicated from wgpu-html-tree so we don't need to modify that crate's
// public API.  Keep in sync if new element types are added.

macro_rules! with_all_variants {
  ($mac:ident) => {
    $mac! {
        Html, Head, Body, Title, Meta, Link, StyleElement, Script, Noscript,
        H1, H2, H3, H4, H5, H6, P, Br, Hr, Pre, Blockquote, Address,
        Span, A, Strong, B, Em, I, U, S, Small, Mark, Code, Kbd, Samp,
        Var, Abbr, Cite, Dfn, Sub, Sup, Time,
        Ul, Ol, Li, Dl, Dt, Dd,
        Header, Nav, Main, Section, Article, Aside, Footer, Div,
        Img, Picture, Source, Video, Audio, Track, Iframe, Canvas, Svg, SvgPath,
        Table, Caption, Thead, Tbody, Tfoot, Tr, Th, Td, Colgroup, Col,
        Form, Label, Input, Textarea, Button, Select, OptionElement, Optgroup,
        Fieldset, Legend, Datalist, Output, Progress, Meter,
        Details, Summary, Dialog, Template, Slot, Del, Ins, Bdi, Bdo, Wbr,
        Data, Ruby, Rt, Rp
    }
  };
}

// ── Global attribute setters ────────────────────────────────────────────────
//
// Global attributes (id, class, style, …) are flattened into every model
// struct.  We set them by matching on all Element variants.
//
// We use a single macro that receives the full variant list from
// `with_all_variants!`, avoiding nested macro definitions which can't
// reference outer metavariables.

macro_rules! impl_global_attr_methods {
    ($($V:ident),* $(,)?) => {
        impl El {
            pub fn id(mut self, value: impl Into<String>) -> Self {
                let v = Some(value.into());
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.id = v; })*
                }
                self
            }

            pub fn class(mut self, value: impl Into<String>) -> Self {
                let v = Some(value.into());
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.class = v; })*
                }
                self
            }

            pub fn style(mut self, value: impl Into<String>) -> Self {
                let v = Some(value.into());
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.style = v; })*
                }
                self
            }

            pub fn attr_title(mut self, value: impl Into<String>) -> Self {
                let v = Some(value.into());
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.title = v; })*
                }
                self
            }

            pub fn hidden(mut self, value: bool) -> Self {
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.hidden = Some(value); })*
                }
                self
            }

            pub fn tabindex(mut self, value: i32) -> Self {
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.tabindex = Some(value); })*
                }
                self
            }

            /// Set a `data-*` attribute.
            pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
                let k = key.into();
                let v = value.into();
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.data_attrs.insert(k, v); })*
                }
                self
            }
        }
    };
}

with_all_variants!(impl_global_attr_methods);

// ── Children & text ─────────────────────────────────────────────────────────

impl El {
  /// Append a text child node.
  pub fn text(mut self, t: impl Into<String>) -> Self {
    self.node.children.push(Node::new(t.into()));
    self
  }

  /// Append multiple children.
  pub fn children(mut self, children: impl IntoIterator<Item = El>) -> Self {
    self.node.children.extend(children.into_iter().map(|el| el.node));
    self
  }

  /// Append a single child.
  pub fn child(mut self, child: El) -> Self {
    self.node.children.push(child.node);
    self
  }
}

// ── Callback setters ────────────────────────────────────────────────────────

/// Type alias for pre-built mouse callbacks (from [`Ctx::msg`], etc.).
pub type MouseCallback = Arc<dyn Fn(&MouseEvent) + Send + Sync>;
/// Type alias for pre-built event callbacks.
pub type EventCallback = Arc<dyn Fn(&HtmlEvent) + Send + Sync>;

impl El {
  pub fn on_click(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_click = Some(Arc::new(f));
    self
  }

  /// Attach a pre-built [`MouseCallback`] (e.g. from [`Ctx::msg`]).
  pub fn on_click_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_click = Some(cb);
    self
  }

  pub fn on_mouse_down(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_down = Some(Arc::new(f));
    self
  }

  pub fn on_mouse_down_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_down = Some(cb);
    self
  }

  pub fn on_mouse_up(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_up = Some(Arc::new(f));
    self
  }

  pub fn on_mouse_up_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_up = Some(cb);
    self
  }

  pub fn on_mouse_move(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_move = Some(Arc::new(f));
    self
  }

  pub fn on_mouse_move_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_move = Some(cb);
    self
  }

  pub fn on_mouse_enter(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_enter = Some(Arc::new(f));
    self
  }

  pub fn on_mouse_enter_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_enter = Some(cb);
    self
  }

  pub fn on_mouse_leave(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_leave = Some(Arc::new(f));
    self
  }

  pub fn on_mouse_leave_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_leave = Some(cb);
    self
  }

  pub fn on_event(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_event = Some(Arc::new(f));
    self
  }

  pub fn on_event_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_event = Some(cb);
    self
  }
}

// ── Element-specific configure ──────────────────────────────────────────────

/// Trait for extracting a mutable reference to a specific model struct
/// from an [`Element`] enum.
pub trait ElementModel: 'static {
  fn from_element_mut(element: &mut Element) -> Option<&mut Self>;
}

macro_rules! impl_element_model {
    ($($V:ident),* $(,)?) => {
        $(
            impl ElementModel for m::$V {
                fn from_element_mut(element: &mut Element) -> Option<&mut Self> {
                    match element {
                        Element::$V(inner) => Some(inner),
                        _ => None,
                    }
                }
            }
        )*
    };
}

with_all_variants!(impl_element_model);

impl El {
  /// Access the underlying model struct for element-specific mutation.
  ///
  /// ```ignore
  /// el::input().configure(|i: &mut wgpu_html_models::Input| {
  ///     i.placeholder = Some("type here".into());
  /// })
  /// ```
  ///
  /// Does nothing if the element type doesn't match `M`.
  pub fn configure<M: ElementModel>(mut self, f: impl FnOnce(&mut M)) -> Self {
    if let Some(model) = M::from_element_mut(&mut self.node.element) {
      f(model);
    }
    self
  }
}

// ── Element constructor functions ───────────────────────────────────────────

macro_rules! el_constructors {
    ($($fn_name:ident => $model:ty),* $(,)?) => {
        $(
            #[inline]
            pub fn $fn_name() -> El {
                El { node: Node::new(<$model>::default()) }
            }
        )*
    };
}

el_constructors! {
    // Document
    html     => m::Html,
    head     => m::Head,
    body     => m::Body,
    title    => m::Title,
    meta     => m::Meta,
    link     => m::Link,
    style_el => m::StyleElement,
    script   => m::Script,
    noscript => m::Noscript,

    // Headings & block text
    h1         => m::H1,
    h2         => m::H2,
    h3         => m::H3,
    h4         => m::H4,
    h5         => m::H5,
    h6         => m::H6,
    p          => m::P,
    br         => m::Br,
    hr         => m::Hr,
    pre        => m::Pre,
    blockquote => m::Blockquote,
    address    => m::Address,

    // Inline text
    span   => m::Span,
    a      => m::A,
    strong => m::Strong,
    b      => m::B,
    em     => m::Em,
    i      => m::I,
    u      => m::U,
    s      => m::S,
    small  => m::Small,
    mark   => m::Mark,
    code   => m::Code,
    kbd    => m::Kbd,
    samp   => m::Samp,
    var    => m::Var,
    abbr   => m::Abbr,
    cite   => m::Cite,
    dfn    => m::Dfn,
    sub    => m::Sub,
    sup    => m::Sup,
    time   => m::Time,

    // Lists
    ul => m::Ul,
    ol => m::Ol,
    li => m::Li,
    dl => m::Dl,
    dt => m::Dt,
    dd => m::Dd,

    // Sectioning
    header  => m::Header,
    nav     => m::Nav,
    main_el => m::Main,
    section => m::Section,
    article => m::Article,
    aside   => m::Aside,
    footer  => m::Footer,
    div     => m::Div,

    // Media
    img     => m::Img,
    picture => m::Picture,
    source  => m::Source,
    video   => m::Video,
    audio   => m::Audio,
    track   => m::Track,
    iframe  => m::Iframe,
    canvas  => m::Canvas,
    svg     => m::Svg,
    svg_path => m::SvgPath,

    // Tables
    table    => m::Table,
    caption  => m::Caption,
    thead    => m::Thead,
    tbody    => m::Tbody,
    tfoot    => m::Tfoot,
    tr       => m::Tr,
    th       => m::Th,
    td       => m::Td,
    colgroup => m::Colgroup,
    col      => m::Col,

    // Forms
    form           => m::Form,
    label          => m::Label,
    input          => m::Input,
    textarea       => m::Textarea,
    button         => m::Button,
    select         => m::Select,
    option_element => m::OptionElement,
    optgroup       => m::Optgroup,
    fieldset       => m::Fieldset,
    legend         => m::Legend,
    datalist       => m::Datalist,
    output         => m::Output,
    progress       => m::Progress,
    meter          => m::Meter,

    // Interactive & misc
    details  => m::Details,
    summary  => m::Summary,
    dialog   => m::Dialog,
    template => m::Template,
    slot     => m::Slot,
    del      => m::Del,
    ins      => m::Ins,
    bdi      => m::Bdi,
    bdo      => m::Bdo,
    wbr      => m::Wbr,
    data     => m::Data,
    ruby     => m::Ruby,
    rt       => m::Rt,
    rp       => m::Rp
}

/// Create a text node.
#[inline]
pub fn text(t: impl Into<String>) -> El {
  El {
    node: Node::new(t.into()),
  }
}

// ── Custom properties ───────────────────────────────────────────────────────

impl El {
  /// Set a CSS custom property on this node.
  pub fn custom_property(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
    self.node.custom_properties.insert(name.into(), value.into());
    self
  }
}
