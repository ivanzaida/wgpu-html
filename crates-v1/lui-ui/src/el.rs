//! Builder DSL for constructing element trees.
//!
//! Each HTML element has a corresponding constructor function (e.g.
//! [`div()`], [`button()`], [`input()`]) that returns an [`El`] builder.
//! Chain methods to set attributes, add children, and attach callbacks:
//!
//! ```ignore
//! use lui_ui::el;
//!
//! el::div().id("app").class("container").children([
//!     el::h1().text("Hello"),
//!     el::button().class("btn").text("Click me")
//!         .on_click(|_| println!("clicked")),
//! ])
//! ```

use std::sync::Arc;

use lui_models as m;
use lui_models::ArcStr;
use lui_tree::{Element, HtmlEvent, MouseEvent, Node};

use crate::Observable;

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
// Duplicated from lui-tree so we don't need to modify that crate's
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
        Img, Picture, Source, Video, Audio, Track, Iframe, Canvas, Svg, SvgPath, SvgElement,
        Table, Caption, Thead, Tbody, Tfoot, Tr, Th, Td, Colgroup, Col,
        Form, Label, Input, Textarea, Button, Select, OptionElement, Optgroup,
        Fieldset, Legend, Datalist, Output, Progress, Meter,
        Details, Summary, Dialog, Template, Slot, Del, Ins, Bdi, Bdo, Wbr,
        Data, Ruby, Rt, Rp, CustomElement
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
            pub fn id(mut self, value: impl Into<ArcStr>) -> Self {
                let v = Some(value.into());
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.id = v; })*
                }
                self
            }

            pub fn class(mut self, value: impl Into<ArcStr>) -> Self {
                let v = value.into();
                self.node.class_list = v
                    .split_ascii_whitespace()
                    .map(|c| ArcStr::from(c))
                    .collect();
                self
            }

            pub fn style(mut self, value: impl Into<ArcStr>) -> Self {
                let v = Some(value.into());
                match &mut self.node.element {
                    Element::Text(_) => {}
                    $(Element::$V(e) => { e.style = v; })*
                }
                self
            }

            pub fn attr_title(mut self, value: impl Into<ArcStr>) -> Self {
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

            /// Set a `data-*` attribute (stored on `Node.data_attrs`).
            pub fn data(mut self, key: impl Into<ArcStr>, value: impl Into<ArcStr>) -> Self {
                self.node.data_attrs.insert(key.into(), value.into());
                self
            }

            /// Set an `aria-*` attribute (stored on `Node.aria_attrs`).
            pub fn aria(mut self, key: impl Into<ArcStr>, value: impl Into<ArcStr>) -> Self {
                self.node.aria_attrs.insert(key.into(), value.into());
                self
            }
        }
    };
}

with_all_variants!(impl_global_attr_methods);

// ── Children & text ─────────────────────────────────────────────────────────

impl El {
  /// Append a text child node.
  pub fn text(mut self, t: impl Into<ArcStr>) -> Self {
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

  /// Set a raw HTML attribute and reflect common typed fields.
  /// `class`, `aria-*`, and `data-*` are routed to dedicated
  /// Node fields; other attributes land in `raw_attrs`.
  pub fn attribute(mut self, name: impl Into<ArcStr>, value: impl Into<ArcStr>) -> Self {
    let name: ArcStr = name.into();
    let value: ArcStr = value.into();
    reflect_attribute(&mut self.node.element, &name, &value);
    match name.as_ref() {
      "class" => {
        self.node.class_list = value.split_ascii_whitespace().map(|c| ArcStr::from(c)).collect();
      }
      _ if name.starts_with("aria-") => {
        let suffix = ArcStr::from(&name[5..]);
        self.node.aria_attrs.insert(suffix, value);
      }
      _ if name.starts_with("data-") => {
        let suffix = ArcStr::from(&name[5..]);
        self.node.data_attrs.insert(suffix, value);
      }
      _ => {
        self.node.raw_attrs.push((name, value));
      }
    }
    self
  }
}

// ── Callback setters ────────────────────────────────────────────────────────

/// Type alias for pre-built mouse callbacks (from [`Ctx::msg`], etc.).
pub type MouseCallback = Arc<dyn Fn(&MouseEvent) + Send + Sync>;
/// Type alias for pre-built event callbacks.
pub type EventCallback = Arc<dyn Fn(&HtmlEvent) + Send + Sync>;

impl El {
  /// Two-way bind an input/textarea value.
  pub fn bind(self, value: Observable<ArcStr>) -> Self {
    self.bind_value(value)
  }

  /// Two-way bind an input/textarea value.
  pub fn bind_value(mut self, value: Observable<ArcStr>) -> Self {
    let current: ArcStr = value.get();
    match &mut self.node.element {
      Element::Input(input) => input.value = Some(current),
      Element::Textarea(textarea) => textarea.value = Some(current),
      _ => {}
    }
    self.on_input(move |ev| {
      if let HtmlEvent::Input(input) = ev {
        if let Some(next) = &input.value {
          value.set(ArcStr::from(next.as_str()));
        }
      }
    })
  }

  /// Two-way bind a checkbox/radio checked state.
  pub fn bind_checked(mut self, checked: Observable<bool>) -> Self {
    let current = checked.get();
    if let Element::Input(input) = &mut self.node.element {
      input.checked = Some(current);
    }
    self.on_input(move |ev| {
      if let HtmlEvent::Input(input) = ev {
        if let Some(next) = input.checked {
          checked.set(next);
        }
      }
    })
  }

  pub fn on_click(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_click.push(Arc::new(f));
    self
  }

  /// Attach a pre-built [`MouseCallback`] (e.g. from [`Ctx::msg`]).
  pub fn on_click_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_click.push(cb);
    self
  }

  pub fn on_mouse_down(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_down.push(Arc::new(f));
    self
  }

  pub fn on_mouse_down_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_down.push(cb);
    self
  }

  pub fn on_mouse_up(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_up.push(Arc::new(f));
    self
  }

  pub fn on_mouse_up_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_up.push(cb);
    self
  }

  pub fn on_mouse_move(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_move.push(Arc::new(f));
    self
  }

  pub fn on_mouse_move_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_move.push(cb);
    self
  }

  pub fn on_mouse_enter(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_enter.push(Arc::new(f));
    self
  }

  pub fn on_mouse_enter_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_enter.push(cb);
    self
  }

  pub fn on_mouse_leave(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_mouse_leave.push(Arc::new(f));
    self
  }

  pub fn on_mouse_leave_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_mouse_leave.push(cb);
    self
  }

  pub fn on_event(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_event.push(Arc::new(f));
    self
  }

  pub fn on_event_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_event.push(cb);
    self
  }

  pub fn on_keydown(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_keydown.push(Arc::new(f));
    self
  }

  pub fn on_keydown_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_keydown.push(cb);
    self
  }

  pub fn on_keyup(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_keyup.push(Arc::new(f));
    self
  }

  pub fn on_keyup_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_keyup.push(cb);
    self
  }

  pub fn on_focus(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_focus.push(Arc::new(f));
    self
  }

  pub fn on_focus_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_focus.push(cb);
    self
  }

  pub fn on_blur(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_blur.push(Arc::new(f));
    self
  }

  pub fn on_blur_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_blur.push(cb);
    self
  }

  pub fn on_focusin(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_focusin.push(Arc::new(f));
    self
  }

  pub fn on_focusin_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_focusin.push(cb);
    self
  }

  pub fn on_focusout(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_focusout.push(Arc::new(f));
    self
  }

  pub fn on_focusout_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_focusout.push(cb);
    self
  }

  pub fn on_input(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_input.push(Arc::new(f));
    self
  }

  pub fn on_input_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_input.push(cb);
    self
  }

  pub fn on_beforeinput(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_beforeinput.push(Arc::new(f));
    self
  }

  pub fn on_beforeinput_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_beforeinput.push(cb);
    self
  }

  pub fn on_change(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_change.push(Arc::new(f));
    self
  }

  pub fn on_change_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_change.push(cb);
    self
  }

  pub fn on_wheel(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_wheel.push(Arc::new(f));
    self
  }

  pub fn on_wheel_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_wheel.push(cb);
    self
  }

  pub fn on_dblclick(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_dblclick.push(Arc::new(f));
    self
  }

  pub fn on_dblclick_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_dblclick.push(cb);
    self
  }

  pub fn on_contextmenu(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_contextmenu.push(Arc::new(f));
    self
  }

  pub fn on_contextmenu_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_contextmenu.push(cb);
    self
  }

  pub fn on_auxclick(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_auxclick.push(Arc::new(f));
    self
  }

  pub fn on_auxclick_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_auxclick.push(cb);
    self
  }

  pub fn on_dragstart(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_dragstart.push(Arc::new(f));
    self
  }

  pub fn on_dragstart_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_dragstart.push(cb);
    self
  }

  pub fn on_dragend(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_dragend.push(Arc::new(f));
    self
  }

  pub fn on_dragend_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_dragend.push(cb);
    self
  }

  pub fn on_drop(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_drop.push(Arc::new(f));
    self
  }

  pub fn on_drop_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_drop.push(cb);
    self
  }

  pub fn on_drag(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_drag.push(Arc::new(f));
    self
  }

  pub fn on_drag_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_drag.push(cb);
    self
  }

  pub fn on_dragover(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_dragover.push(Arc::new(f));
    self
  }

  pub fn on_dragover_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_dragover.push(cb);
    self
  }

  pub fn on_dragenter(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_dragenter.push(Arc::new(f));
    self
  }

  pub fn on_dragenter_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_dragenter.push(cb);
    self
  }

  pub fn on_dragleave(mut self, f: impl Fn(&MouseEvent) + Send + Sync + 'static) -> Self {
    self.node.on_dragleave.push(Arc::new(f));
    self
  }

  pub fn on_dragleave_cb(mut self, cb: MouseCallback) -> Self {
    self.node.on_dragleave.push(cb);
    self
  }

  pub fn on_copy(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_copy.push(Arc::new(f));
    self
  }

  pub fn on_copy_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_copy.push(cb);
    self
  }

  pub fn on_cut(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_cut.push(Arc::new(f));
    self
  }

  pub fn on_cut_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_cut.push(cb);
    self
  }

  pub fn on_paste(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_paste.push(Arc::new(f));
    self
  }

  pub fn on_paste_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_paste.push(cb);
    self
  }

  pub fn on_scroll(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_scroll.push(Arc::new(f));
    self
  }

  pub fn on_scroll_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_scroll.push(cb);
    self
  }

  pub fn on_select(mut self, f: impl Fn(&HtmlEvent) + Send + Sync + 'static) -> Self {
    self.node.on_select.push(Arc::new(f));
    self
  }

  pub fn on_select_cb(mut self, cb: EventCallback) -> Self {
    self.node.on_select.push(cb);
    self
  }
}

fn reflect_attribute(element: &mut Element, name: &str, value: &str) {
  let name = name.to_ascii_lowercase();
  match (name.as_str(), element) {
    ("type", Element::Input(input)) => input.r#type = parse_input_type(value),
    ("value", Element::Input(input)) => input.value = Some(ArcStr::from(value)),
    ("value", Element::Textarea(textarea)) => textarea.value = Some(ArcStr::from(value)),
    ("checked", Element::Input(input)) => input.checked = Some(parse_bool_attr(value)),
    _ => {}
  }
}

fn parse_bool_attr(value: &str) -> bool {
  !matches!(value.trim().to_ascii_lowercase().as_str(), "" | "false" | "0")
}

fn parse_input_type(value: &str) -> Option<InputType> {
  Some(match value.trim().to_ascii_lowercase().as_str() {
    "button" => InputType::Button,
    "checkbox" => InputType::Checkbox,
    "color" => InputType::Color,
    "date" => InputType::Date,
    "datetime-local" => InputType::DatetimeLocal,
    "email" => InputType::Email,
    "file" => InputType::File,
    "hidden" => InputType::Hidden,
    "image" => InputType::Image,
    "month" => InputType::Month,
    "number" => InputType::Number,
    "password" => InputType::Password,
    "radio" => InputType::Radio,
    "range" => InputType::Range,
    "reset" => InputType::Reset,
    "search" => InputType::Search,
    "submit" => InputType::Submit,
    "tel" => InputType::Tel,
    "text" => InputType::Text,
    "time" => InputType::Time,
    "url" => InputType::Url,
    "week" => InputType::Week,
    _ => return None,
  })
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
  /// el::input().configure(|i: &mut lui_models::Input| {
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

// ── Element-specific attribute traits ───────────────────────────────────────
//
// Each trait groups the setters for one (or a few related) element types.
// All are implemented on `El`; a setter silently no-ops when the underlying
// element doesn't match (same semantics as `configure()`).

/// Helper macro that generates a trait + impl-on-El for element-specific attrs.
///
/// Supported field kinds:
///   `string`      — `Option<String>`, setter takes `impl Into<String>`
///   `bool`        — `Option<bool>`, setter takes `bool`
///   `u32`         — `Option<u32>`, setter takes `u32`
///   `i32`         — `Option<i32>`, setter takes `i32`
///   `f64`         — `Option<f64>`, setter takes `f64`
///   `enum(T)`     — `Option<T>`, setter takes `T`
macro_rules! element_attrs {
    (
        $(#[$trait_meta:meta])*
        $trait_name:ident for $model:ty {
            $(
                $(#[$meta:meta])*
                $method:ident ($field:ident) : $kind:ident $(($inner:ty))?
            );* $(;)?
        }
    ) => {
        $(#[$trait_meta])*
        pub trait $trait_name: Sized {
            $(
                $(#[$meta])*
                element_attrs!(@sig $method $kind $(($inner))?);
            )*
        }

        impl $trait_name for El {
            $(
                element_attrs!(@impl_method $model, $method, $field, $kind $(($inner))?);
            )*
        }
    };

    // ── Signature arms ────────────────────────────────────────────────────

    (@sig $method:ident string) => {
        fn $method(self, value: impl Into<ArcStr>) -> Self;
    };
    (@sig $method:ident bool) => {
        fn $method(self, value: bool) -> Self;
    };
    (@sig $method:ident u32) => {
        fn $method(self, value: u32) -> Self;
    };
    (@sig $method:ident i32) => {
        fn $method(self, value: i32) -> Self;
    };
    (@sig $method:ident f64) => {
        fn $method(self, value: f64) -> Self;
    };
    (@sig $method:ident enum($inner:ty)) => {
        fn $method(self, value: $inner) -> Self;
    };

    // ── Implementation arms ───────────────────────────────────────────────

    (@impl_method $model:ty, $method:ident, $field:ident, string) => {
        fn $method(mut self, value: impl Into<ArcStr>) -> Self {
            if let Some(m) = <$model as ElementModel>::from_element_mut(&mut self.node.element) {
                m.$field = Some(value.into());
            }
            self
        }
    };
    (@impl_method $model:ty, $method:ident, $field:ident, bool) => {
        fn $method(mut self, value: bool) -> Self {
            if let Some(m) = <$model as ElementModel>::from_element_mut(&mut self.node.element) {
                m.$field = Some(value);
            }
            self
        }
    };
    (@impl_method $model:ty, $method:ident, $field:ident, u32) => {
        fn $method(mut self, value: u32) -> Self {
            if let Some(m) = <$model as ElementModel>::from_element_mut(&mut self.node.element) {
                m.$field = Some(value);
            }
            self
        }
    };
    (@impl_method $model:ty, $method:ident, $field:ident, i32) => {
        fn $method(mut self, value: i32) -> Self {
            if let Some(m) = <$model as ElementModel>::from_element_mut(&mut self.node.element) {
                m.$field = Some(value);
            }
            self
        }
    };
    (@impl_method $model:ty, $method:ident, $field:ident, f64) => {
        fn $method(mut self, value: f64) -> Self {
            if let Some(m) = <$model as ElementModel>::from_element_mut(&mut self.node.element) {
                m.$field = Some(value);
            }
            self
        }
    };
    (@impl_method $model:ty, $method:ident, $field:ident, enum($inner:ty)) => {
        fn $method(mut self, value: $inner) -> Self {
            if let Some(m) = <$model as ElementModel>::from_element_mut(&mut self.node.element) {
                m.$field = Some(value);
            }
            self
        }
    };
}

use m::common::html_enums::{
  AutoComplete, ButtonType, CaptureMode, CrossOrigin, FormEncoding, FormMethod, ImageDecoding, InputType, LinkAs,
  LinkTarget, Loading, OlType, Preload, ReferrerPolicy, SvgLength, TableHeaderScope, TextareaWrap, TrackKind,
};

// ── Form elements ─────────────────────────────────────────────────────────

element_attrs! {
    /// Attribute setters for `<input>` elements.
    InputAttrs for m::Input {
        input_type(r#type): enum(InputType);
        name(name): string;
        value(value): string;
        placeholder(placeholder): string;
        required(required): bool;
        disabled(disabled): bool;
        readonly(readonly): bool;
        checked(checked): bool;
        min(min): string;
        max(max): string;
        step(step): string;
        minlength(minlength): u32;
        maxlength(maxlength): u32;
        pattern(pattern): string;
        autocomplete(autocomplete): string;
        autofocus(autofocus): bool;
        multiple(multiple): bool;
        accept(accept): string;
        capture(capture): enum(CaptureMode);
        size(size): u32;
        list(list): string;
        form_attr(form): string
    }
}

element_attrs! {
    /// Attribute setters for `<textarea>` elements.
    TextareaAttrs for m::Textarea {
        name(name): string;
        value(value): string;
        placeholder(placeholder): string;
        required(required): bool;
        disabled(disabled): bool;
        readonly(readonly): bool;
        rows(rows): u32;
        cols(cols): u32;
        minlength(minlength): u32;
        maxlength(maxlength): u32;
        wrap(wrap): enum(TextareaWrap);
        autocomplete(autocomplete): string;
        autofocus(autofocus): bool;
        form_attr(form): string
    }
}

element_attrs! {
    /// Attribute setters for `<button>` elements.
    ButtonAttrs for m::Button {
        button_type(r#type): enum(ButtonType);
        name(name): string;
        value(value): string;
        disabled(disabled): bool;
        autofocus(autofocus): bool;
        form_attr(form): string;
        formaction(formaction): string;
        formenctype(formenctype): enum(FormEncoding);
        formmethod(formmethod): enum(FormMethod);
        formnovalidate(formnovalidate): bool;
        formtarget(formtarget): enum(LinkTarget)
    }
}

element_attrs! {
    /// Attribute setters for `<select>` elements.
    SelectAttrs for m::Select {
        name(name): string;
        required(required): bool;
        disabled(disabled): bool;
        multiple(multiple): bool;
        size(size): u32;
        autofocus(autofocus): bool;
        form_attr(form): string
    }
}

element_attrs! {
    /// Attribute setters for `<option>` elements.
    OptionAttrs for m::OptionElement {
        value(value): string;
        label(label): string;
        selected(selected): bool;
        disabled(disabled): bool
    }
}

element_attrs! {
    /// Attribute setters for `<optgroup>` elements.
    OptgroupAttrs for m::Optgroup {
        label(label): string;
        disabled(disabled): bool
    }
}

element_attrs! {
    /// Attribute setters for `<form>` elements.
    FormAttrs for m::Form {
        action(action): string;
        method(method): enum(FormMethod);
        enctype(enctype): enum(FormEncoding);
        target(target): enum(LinkTarget);
        form_autocomplete(autocomplete): enum(AutoComplete);
        novalidate(novalidate): bool;
        name(name): string;
        rel(rel): string
    }
}

element_attrs! {
    /// Attribute setters for `<label>` elements.
    LabelAttrs for m::Label {
        label_for(r#for): string
    }
}

element_attrs! {
    /// Attribute setters for `<fieldset>` elements.
    FieldsetAttrs for m::Fieldset {
        disabled(disabled): bool;
        form_attr(form): string;
        name(name): string
    }
}

element_attrs! {
    /// Attribute setters for `<output>` elements.
    OutputAttrs for m::Output {
        form_attr(form): string;
        name(name): string
    }
}

element_attrs! {
    /// Attribute setters for `<progress>` elements.
    ProgressAttrs for m::Progress {
        progress_value(value): f64;
        progress_max(max): f64
    }
}

element_attrs! {
    /// Attribute setters for `<meter>` elements.
    MeterAttrs for m::Meter {
        meter_value(value): f64;
        meter_min(min): f64;
        meter_max(max): f64;
        low(low): f64;
        high(high): f64;
        optimum(optimum): f64
    }
}

// ── Link / navigation elements ────────────────────────────────────────────

element_attrs! {
    /// Attribute setters for `<a>` (anchor) elements.
    AnchorAttrs for m::A {
        href(href): string;
        target(target): enum(LinkTarget);
        download(download): string;
        rel(rel): string;
        hreflang(hreflang): string;
        link_type(r#type): string;
        ping(ping): string;
        referrerpolicy(referrerpolicy): enum(ReferrerPolicy)
    }
}

element_attrs! {
    /// Attribute setters for `<link>` elements.
    LinkAttrs for m::Link {
        href(href): string;
        rel(rel): string;
        link_type(r#type): string;
        media(media): string;
        sizes(sizes): string;
        hreflang(hreflang): string;
        link_as(r#as): enum(LinkAs);
        crossorigin(crossorigin): enum(CrossOrigin);
        integrity(integrity): string;
        referrerpolicy(referrerpolicy): enum(ReferrerPolicy)
    }
}

// ── Media elements ────────────────────────────────────────────────────────

element_attrs! {
    /// Attribute setters for `<img>` elements.
    ImgAttrs for m::Img {
        src(src): string;
        alt(alt): string;
        width(width): u32;
        height(height): u32;
        srcset(srcset): string;
        sizes(sizes): string;
        loading(loading): enum(Loading);
        decoding(decoding): enum(ImageDecoding);
        crossorigin(crossorigin): enum(CrossOrigin);
        usemap(usemap): string;
        ismap(ismap): bool;
        referrerpolicy(referrerpolicy): enum(ReferrerPolicy)
    }
}

element_attrs! {
    /// Attribute setters for `<video>` elements.
    VideoAttrs for m::Video {
        src(src): string;
        controls(controls): bool;
        autoplay(autoplay): bool;
        loop_attr(r#loop): bool;
        muted(muted): bool;
        poster(poster): string;
        preload(preload): enum(Preload);
        width(width): u32;
        height(height): u32;
        playsinline(playsinline): bool;
        crossorigin(crossorigin): enum(CrossOrigin)
    }
}

element_attrs! {
    /// Attribute setters for `<audio>` elements.
    AudioAttrs for m::Audio {
        src(src): string;
        controls(controls): bool;
        autoplay(autoplay): bool;
        loop_attr(r#loop): bool;
        muted(muted): bool;
        preload(preload): enum(Preload);
        crossorigin(crossorigin): enum(CrossOrigin)
    }
}

element_attrs! {
    /// Attribute setters for `<source>` elements.
    SourceAttrs for m::Source {
        src(src): string;
        srcset(srcset): string;
        sizes(sizes): string;
        media(media): string;
        source_type(r#type): string;
        width(width): u32;
        height(height): u32
    }
}

element_attrs! {
    /// Attribute setters for `<track>` elements.
    TrackAttrs for m::Track {
        src(src): string;
        kind(kind): enum(TrackKind);
        srclang(srclang): string;
        label(label): string;
        default(default): bool
    }
}

element_attrs! {
    /// Attribute setters for `<iframe>` elements.
    IframeAttrs for m::Iframe {
        src(src): string;
        srcdoc(srcdoc): string;
        name(name): string;
        width(width): u32;
        height(height): u32;
        allow(allow): string;
        allowfullscreen(allowfullscreen): bool;
        loading(loading): enum(Loading);
        referrerpolicy(referrerpolicy): enum(ReferrerPolicy);
        sandbox(sandbox): string
    }
}

element_attrs! {
    /// Attribute setters for `<canvas>` elements.
    CanvasAttrs for m::Canvas {
        width(width): u32;
        height(height): u32
    }
}

// ── Table elements ────────────────────────────────────────────────────────

element_attrs! {
    /// Attribute setters for `<td>` elements.
    TdAttrs for m::Td {
        colspan(colspan): u32;
        rowspan(rowspan): u32;
        headers(headers): string
    }
}

element_attrs! {
    /// Attribute setters for `<th>` elements.
    ThAttrs for m::Th {
        colspan(colspan): u32;
        rowspan(rowspan): u32;
        headers(headers): string;
        scope(scope): enum(TableHeaderScope);
        abbr(abbr): string
    }
}

element_attrs! {
    /// Attribute setters for `<col>` elements.
    ColAttrs for m::Col {
        span(span): u32
    }
}

element_attrs! {
    /// Attribute setters for `<colgroup>` elements.
    ColgroupAttrs for m::Colgroup {
        span(span): u32
    }
}

// ── Metadata / head elements ──────────────────────────────────────────────

element_attrs! {
    /// Attribute setters for `<meta>` elements.
    MetaAttrs for m::Meta {
        name(name): string;
        content(content): string;
        charset(charset): string;
        http_equiv(http_equiv): string
    }
}

element_attrs! {
    /// Attribute setters for `<script>` elements.
    ScriptAttrs for m::Script {
        src(src): string;
        script_type(r#type): string;
        async_attr(r#async): bool;
        defer(defer): bool;
        crossorigin(crossorigin): enum(CrossOrigin);
        integrity(integrity): string;
        nomodule(nomodule): bool;
        nonce(nonce): string;
        referrerpolicy(referrerpolicy): enum(ReferrerPolicy)
    }
}

// ── Misc elements ─────────────────────────────────────────────────────────

element_attrs! {
    /// Attribute setters for `<details>` elements.
    DetailsAttrs for m::Details {
        open(open): bool;
        name(name): string
    }
}

element_attrs! {
    /// Attribute setters for `<dialog>` elements.
    DialogAttrs for m::Dialog {
        open(open): bool
    }
}

element_attrs! {
    /// Attribute setters for `<time>` elements.
    TimeAttrs for m::Time {
        datetime(datetime): string
    }
}

element_attrs! {
    /// Attribute setters for `<ol>` elements.
    OlAttrs for m::Ol {
        reversed(reversed): bool;
        start(start): i32;
        ol_type(r#type): enum(OlType)
    }
}

element_attrs! {
    /// Attribute setters for `<blockquote>` elements.
    BlockquoteAttrs for m::Blockquote {
        cite(cite): string
    }
}

element_attrs! {
    /// Attribute setters for `<del>` elements.
    DelAttrs for m::Del {
        cite(cite): string;
        datetime(datetime): string
    }
}

element_attrs! {
    /// Attribute setters for `<ins>` elements.
    InsAttrs for m::Ins {
        cite(cite): string;
        datetime(datetime): string
    }
}

element_attrs! {
    /// Attribute setters for `<data>` elements.
    DataElAttrs for m::Data {
        data_value(value): string
    }
}

element_attrs! {
    /// Attribute setters for `<svg>` elements.
    SvgAttrs for m::Svg {
        width(width): enum(SvgLength);
        height(height): enum(SvgLength);
        view_box(view_box): string;
        xmlns(xmlns): string;
        fill(fill): string;
        stroke(stroke): string
    }
}

element_attrs! {
    /// Attribute setters for `<path>` (SVG) elements.
    SvgPathAttrs for m::SvgPath {
        d(d): string;
        fill(fill): string;
        stroke(stroke): string;
        stroke_width(stroke_width): string;
        fill_rule(fill_rule): string;
        opacity(opacity): string;
        transform(transform): string
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

/// Create a custom element with the given tag name.
///
/// The tag name should contain a hyphen (e.g. `"my-component"`).
#[inline]
pub fn custom(tag_name: impl Into<ArcStr>) -> El {
  El {
    node: Node::new(m::CustomElement::new(tag_name)),
  }
}

/// Create a text node.
#[inline]
pub fn text(t: impl Into<ArcStr>) -> El {
  El {
    node: Node::new(t.into()),
  }
}

/// Create an empty node that renders nothing.
#[inline]
pub fn empty() -> El {
  El { node: Node::new("") }
}

// ── Custom properties ───────────────────────────────────────────────────────

impl El {
  /// Set a CSS custom property on this node.
  pub fn custom_property(mut self, name: impl Into<ArcStr>, value: impl Into<ArcStr>) -> Self {
    self.node.custom_properties.insert(name.into(), value.into());
    self
  }
}

#[cfg(test)]
mod tests {
  use std::cell::Cell;

  use lui_events as ev;

  use super::*;

  fn input_event(value: Option<&str>, checked: Option<bool>) -> HtmlEvent {
    HtmlEvent::Input(ev::events::InputEvent {
      base: ev::events::UIEvent {
        base: ev::events::Event {
          event_type: ev::HtmlEventType::from(ev::HtmlEventType::INPUT),
          bubbles: true,
          cancelable: true,
          composed: true,
          target: Some(Vec::new()),
          current_target: Some(Vec::new()),
          event_phase: ev::EventPhase::AtTarget,
          default_prevented: Cell::new(false),
          propagation_stopped: Cell::new(false),
          immediate_propagation_stopped: Cell::new(false),
          is_trusted: true,
          time_stamp: 0.0,
        },
        detail: 0,
      },
      data: None,
      input_type: ev::InputType::InsertText,
      value: value.map(str::to_string),
      checked,
      is_composing: false,
    })
  }

  #[test]
  fn bind_value_sets_initial_value_and_updates_observable() {
    let value = Observable::new("initial");
    let node = input().bind(value.clone()).into_node();
    match &node.element {
      Element::Input(input) => assert_eq!(input.value.as_deref(), Some("initial")),
      _ => panic!("expected input"),
    }

    let ev = input_event(Some("next"), None);
    for cb in &node.on_input {
      cb(&ev);
    }

    assert_eq!(&*value.get(), "next");
  }

  #[test]
  fn bind_checked_sets_initial_state_and_updates_observable() {
    let checked = Observable::new(false);
    let node = input().bind_checked(checked.clone()).into_node();
    match &node.element {
      Element::Input(input) => assert_eq!(input.checked, Some(false)),
      _ => panic!("expected input"),
    }

    let ev = input_event(None, Some(true));
    for cb in &node.on_input {
      cb(&ev);
    }

    assert!(checked.get());
  }

  #[test]
  fn attribute_reflects_input_type() {
    let node = input().attribute("type", "text").into_node();
    match &node.element {
      Element::Input(input) => assert!(matches!(input.r#type, Some(InputType::Text))),
      _ => panic!("expected input"),
    }
  }
}
