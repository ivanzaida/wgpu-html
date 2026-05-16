use std::sync::Arc;

use lui_core::{
  ArcStr, EventHandler, HtmlElement, HtmlNode,
  events::{DocumentEvent, MouseEventInit},
};
use lui_parse::parse_declaration_block;

use super::super::{signal::Signal, app::NodeRef};

#[derive(Clone)]
pub struct El {
  pub(crate) node: HtmlNode,
}

impl El {
  #[inline]
  pub fn into_node(self) -> HtmlNode {
    self.node
  }
}

impl From<El> for HtmlNode {
  #[inline]
  fn from(el: El) -> HtmlNode {
    el.node
  }
}

#[derive(Clone, Default)]
pub struct Children(Vec<El>);

impl Children {
  pub fn empty() -> Self {
    Self(Vec::new())
  }

  pub fn from(iter: impl IntoIterator<Item = El>) -> Self {
    Self(iter.into_iter().collect())
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

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

impl El {
  pub fn id(mut self, value: impl AsRef<str>) -> Self {
    self.node.set_id(value.as_ref());
    self
  }

  pub fn ref_node(self, node_ref: NodeRef) -> Self {
    self.id(node_ref.id())
  }

  pub fn class(mut self, value: impl AsRef<str>) -> Self {
    self.node.class_list_mut().set(value.as_ref());
    self
  }

  pub fn style(mut self, value: impl AsRef<str>) -> Self {
    self
      .node
      .set_styles(parse_declaration_block(value.as_ref()).unwrap_or_default());
    self
  }

  pub fn attr_title(mut self, value: impl AsRef<str>) -> Self {
    self.node.set_attribute("title", value.as_ref());
    self
  }

  pub fn hidden(mut self, value: bool) -> Self {
    self.node.set_attribute("hidden", bool_attr(value));
    self
  }

  pub fn tabindex(mut self, value: i32) -> Self {
    self.node.set_attribute("tabindex", &value.to_string());
    self
  }

  pub fn data(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
    self
      .node
      .set_attribute(&format!("data-{}", key.as_ref()), value.as_ref());
    self
  }

  pub fn aria(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
    self
      .node
      .set_attribute(&format!("aria-{}", key.as_ref()), value.as_ref());
    self
  }

  pub fn attribute(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
    match name.as_ref() {
      "class" => self.node.class_list_mut().set(value.as_ref()),
      "style" => self
        .node
        .set_styles(parse_declaration_block(value.as_ref()).unwrap_or_default()),
      name => self.node.set_attribute(name, value.as_ref()),
    }
    self
  }

  pub fn custom_property(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
    let name = name.as_ref();
    let prop = if name.starts_with("--") {
      name.to_string()
    } else {
      format!("--{name}")
    };
    let css = format!("{prop}: {}", value.as_ref());
    let mut styles = self.node.styles().to_vec();
    styles.extend(parse_declaration_block(&css).unwrap_or_default());
    self.node.set_styles(styles);
    self
  }

  pub fn text(mut self, text: impl AsRef<str>) -> Self {
    self.node.append_child(HtmlNode::text(text.as_ref()));
    self
  }

  pub fn children(mut self, children: impl IntoIterator<Item = El>) -> Self {
    for child in children {
      self.node.append_child(child.node);
    }
    self
  }

  pub fn child(mut self, child: El) -> Self {
    self.node.append_child(child.node);
    self
  }
}

pub type MouseCallback = Arc<dyn Fn(&MouseEventInit) + Send + Sync>;
pub type EventCallback = Arc<dyn Fn(&DocumentEvent) + Send + Sync>;

impl El {
  pub fn bind(self, value: Signal<ArcStr>) -> Self {
    self.bind_value(value)
  }

  pub fn bind_value(mut self, value: Signal<ArcStr>) -> Self {
    let current = value.get();
    self.node.set_attribute("value", &current);
    self.on_input(move |event| {
      if let Some(next) = input_value(event) {
        value.set(ArcStr::from(next));
      }
    })
  }

  pub fn bind_checked(mut self, checked: Signal<bool>) -> Self {
    self.node.set_attribute("checked", bool_attr(checked.get()));
    self.on_input(move |event| {
      if let Some(next) = input_checked(event) {
        checked.set(next);
      }
    })
  }

  pub fn on_click(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self {
    self.on_mouse_event("click", Arc::new(f))
  }
  pub fn on_click_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("click", cb) }
  pub fn on_mouse_down(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("mousedown", Arc::new(f)) }
  pub fn on_mouse_down_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("mousedown", cb) }
  pub fn on_mouse_up(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("mouseup", Arc::new(f)) }
  pub fn on_mouse_up_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("mouseup", cb) }
  pub fn on_mouse_move(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("mousemove", Arc::new(f)) }
  pub fn on_mouse_move_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("mousemove", cb) }
  pub fn on_mouse_enter(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("mouseenter", Arc::new(f)) }
  pub fn on_mouse_enter_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("mouseenter", cb) }
  pub fn on_mouse_leave(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("mouseleave", Arc::new(f)) }
  pub fn on_mouse_leave_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("mouseleave", cb) }
  pub fn on_dblclick(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("dblclick", Arc::new(f)) }
  pub fn on_dblclick_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("dblclick", cb) }
  pub fn on_contextmenu(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("contextmenu", Arc::new(f)) }
  pub fn on_contextmenu_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("contextmenu", cb) }
  pub fn on_auxclick(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("auxclick", Arc::new(f)) }
  pub fn on_auxclick_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("auxclick", cb) }
  pub fn on_dragstart(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("dragstart", Arc::new(f)) }
  pub fn on_dragstart_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("dragstart", cb) }
  pub fn on_dragend(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("dragend", Arc::new(f)) }
  pub fn on_dragend_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("dragend", cb) }
  pub fn on_drag(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("drag", Arc::new(f)) }
  pub fn on_drag_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("drag", cb) }
  pub fn on_dragover(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("dragover", Arc::new(f)) }
  pub fn on_dragover_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("dragover", cb) }
  pub fn on_dragenter(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("dragenter", Arc::new(f)) }
  pub fn on_dragenter_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("dragenter", cb) }
  pub fn on_dragleave(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("dragleave", Arc::new(f)) }
  pub fn on_dragleave_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("dragleave", cb) }
  pub fn on_drop(self, f: impl Fn(&MouseEventInit) + Send + Sync + 'static) -> Self { self.on_mouse_event("drop", Arc::new(f)) }
  pub fn on_drop_cb(self, cb: MouseCallback) -> Self { self.on_mouse_event("drop", cb) }

  pub fn on_event(mut self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self {
    let cb = Arc::new(f);
    self.node.add_event_listener("*", Arc::new(move |_node, event| cb(event)));
    self
  }
  pub fn on_event_cb(mut self, cb: EventCallback) -> Self {
    self.node.add_event_listener("*", Arc::new(move |_node, event| cb(event)));
    self
  }

  pub fn on_keydown(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("keydown", Arc::new(f)) }
  pub fn on_keydown_cb(self, cb: EventCallback) -> Self { self.on_document_event("keydown", cb) }
  pub fn on_keyup(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("keyup", Arc::new(f)) }
  pub fn on_keyup_cb(self, cb: EventCallback) -> Self { self.on_document_event("keyup", cb) }
  pub fn on_focus(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("focus", Arc::new(f)) }
  pub fn on_focus_cb(self, cb: EventCallback) -> Self { self.on_document_event("focus", cb) }
  pub fn on_blur(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("blur", Arc::new(f)) }
  pub fn on_blur_cb(self, cb: EventCallback) -> Self { self.on_document_event("blur", cb) }
  pub fn on_focusin(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("focusin", Arc::new(f)) }
  pub fn on_focusin_cb(self, cb: EventCallback) -> Self { self.on_document_event("focusin", cb) }
  pub fn on_focusout(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("focusout", Arc::new(f)) }
  pub fn on_focusout_cb(self, cb: EventCallback) -> Self { self.on_document_event("focusout", cb) }
  pub fn on_input(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("input", Arc::new(f)) }
  pub fn on_input_cb(self, cb: EventCallback) -> Self { self.on_document_event("input", cb) }
  pub fn on_beforeinput(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("beforeinput", Arc::new(f)) }
  pub fn on_beforeinput_cb(self, cb: EventCallback) -> Self { self.on_document_event("beforeinput", cb) }
  pub fn on_change(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("change", Arc::new(f)) }
  pub fn on_change_cb(self, cb: EventCallback) -> Self { self.on_document_event("change", cb) }
  pub fn on_wheel(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("wheel", Arc::new(f)) }
  pub fn on_wheel_cb(self, cb: EventCallback) -> Self { self.on_document_event("wheel", cb) }
  pub fn on_copy(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("copy", Arc::new(f)) }
  pub fn on_copy_cb(self, cb: EventCallback) -> Self { self.on_document_event("copy", cb) }
  pub fn on_cut(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("cut", Arc::new(f)) }
  pub fn on_cut_cb(self, cb: EventCallback) -> Self { self.on_document_event("cut", cb) }
  pub fn on_paste(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("paste", Arc::new(f)) }
  pub fn on_paste_cb(self, cb: EventCallback) -> Self { self.on_document_event("paste", cb) }
  pub fn on_scroll(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("scroll", Arc::new(f)) }
  pub fn on_scroll_cb(self, cb: EventCallback) -> Self { self.on_document_event("scroll", cb) }
  pub fn on_select(self, f: impl Fn(&DocumentEvent) + Send + Sync + 'static) -> Self { self.on_document_event("select", Arc::new(f)) }
  pub fn on_select_cb(self, cb: EventCallback) -> Self { self.on_document_event("select", cb) }

  fn on_mouse_event(mut self, event_type: &'static str, cb: MouseCallback) -> Self {
    self.node.add_event_listener(event_type, mouse_handler(cb));
    self
  }

  fn on_document_event(mut self, event_type: &'static str, cb: EventCallback) -> Self {
    self.node.add_event_listener(event_type, Arc::new(move |_node, event| cb(event)));
    self
  }
}

fn mouse_handler(cb: MouseCallback) -> EventHandler {
  Arc::new(move |_node, event| {
    if let Some(mouse) = mouse_event(event) {
      cb(mouse);
    }
  })
}

fn mouse_event(event: &DocumentEvent) -> Option<&MouseEventInit> {
  match event {
    DocumentEvent::MouseEvent(event) => Some(event),
    DocumentEvent::PointerEvent(event) => Some(&event.mouse),
    DocumentEvent::WheelEvent(event) => Some(&event.mouse),
    DocumentEvent::DragEvent(event) => Some(&event.mouse),
    _ => None,
  }
}

fn input_value(event: &DocumentEvent) -> Option<&str> {
  match event {
    DocumentEvent::Value(event) => event.value.as_deref(),
    _ => None,
  }
}

fn input_checked(event: &DocumentEvent) -> Option<bool> {
  match event {
    DocumentEvent::Value(event) => event
      .value
      .as_deref()
      .map(|value| value == "true" || value == "1" || value == "on"),
    _ => None,
  }
}

fn bool_attr(value: bool) -> &'static str {
  if value { "true" } else { "false" }
}

macro_rules! el_constructors {
  ($($fn_name:ident => $element:expr),* $(,)?) => {
    $(
      #[allow(dead_code)]
      #[inline]
      pub fn $fn_name() -> El {
        El { node: HtmlNode::new($element) }
      }
    )*
  };
}

el_constructors! {
  html => HtmlElement::Html, head => HtmlElement::Head, body => HtmlElement::Body,
  title => HtmlElement::Title, meta => HtmlElement::Meta, link => HtmlElement::Link,
  style_el => HtmlElement::Style, script => HtmlElement::Script, noscript => HtmlElement::Noscript,
  h1 => HtmlElement::H1, h2 => HtmlElement::H2, h3 => HtmlElement::H3,
  h4 => HtmlElement::H4, h5 => HtmlElement::H5, h6 => HtmlElement::H6,
  p => HtmlElement::P, br => HtmlElement::Br, hr => HtmlElement::Hr,
  pre => HtmlElement::Pre, blockquote => HtmlElement::Blockquote, address => HtmlElement::Address,
  span => HtmlElement::Span, a => HtmlElement::A, strong => HtmlElement::Strong,
  b => HtmlElement::B, em => HtmlElement::Em, i => HtmlElement::I,
  u => HtmlElement::U, s => HtmlElement::S, small => HtmlElement::Small,
  mark => HtmlElement::Mark, code => HtmlElement::Code, kbd => HtmlElement::Kbd,
  samp => HtmlElement::Samp, var => HtmlElement::Var, abbr => HtmlElement::Abbr,
  cite => HtmlElement::Cite, dfn => HtmlElement::Dfn, sub => HtmlElement::Sub,
  sup => HtmlElement::Sup, time => HtmlElement::Time,
  ul => HtmlElement::Ul, ol => HtmlElement::Ol, li => HtmlElement::Li,
  dl => HtmlElement::Dl, dt => HtmlElement::Dt, dd => HtmlElement::Dd,
  header => HtmlElement::Header, nav => HtmlElement::Nav, main_el => HtmlElement::Main,
  section => HtmlElement::Section, article => HtmlElement::Article, aside => HtmlElement::Aside,
  footer => HtmlElement::Footer, div => HtmlElement::Div,
  img => HtmlElement::Img, picture => HtmlElement::Picture, source => HtmlElement::Source,
  video => HtmlElement::Video, audio => HtmlElement::Audio, track => HtmlElement::Track,
  iframe => HtmlElement::Iframe, canvas => HtmlElement::Canvas, svg => HtmlElement::Svg,
  svg_path => HtmlElement::SvgPath,
  table => HtmlElement::Table, caption => HtmlElement::Caption, thead => HtmlElement::Thead,
  tbody => HtmlElement::Tbody, tfoot => HtmlElement::Tfoot, tr => HtmlElement::Tr,
  th => HtmlElement::Th, td => HtmlElement::Td, colgroup => HtmlElement::Colgroup,
  col => HtmlElement::Col,
  form => HtmlElement::Form, label => HtmlElement::Label, input => HtmlElement::Input,
  textarea => HtmlElement::Textarea, button => HtmlElement::Button, select => HtmlElement::Select,
  option_element => HtmlElement::OptionElement, optgroup => HtmlElement::Optgroup,
  fieldset => HtmlElement::Fieldset, legend => HtmlElement::Legend, datalist => HtmlElement::Datalist,
  output => HtmlElement::Output, progress => HtmlElement::Progress, meter => HtmlElement::Meter,
  details => HtmlElement::Details, summary => HtmlElement::Summary, dialog => HtmlElement::Dialog,
  template => HtmlElement::Template, slot => HtmlElement::Slot,
  del => HtmlElement::Del, ins => HtmlElement::Ins, bdi => HtmlElement::Bdi,
  bdo => HtmlElement::Bdo, wbr => HtmlElement::Wbr, data => HtmlElement::Data,
  ruby => HtmlElement::Ruby, rt => HtmlElement::Rt, rp => HtmlElement::Rp,
}

#[inline]
pub fn custom(tag_name: impl AsRef<str>) -> El {
  El {
    node: HtmlNode::new(HtmlElement::Unknown(ArcStr::from(tag_name.as_ref()))),
  }
}

#[inline]
pub fn text(text: impl AsRef<str>) -> El {
  El {
    node: HtmlNode::text(text.as_ref()),
  }
}

#[inline]
pub fn empty() -> El {
  text("")
}

#[inline]
pub fn show(condition: bool, el: impl FnOnce() -> El) -> El {
  if condition { el() } else { empty() }
}
