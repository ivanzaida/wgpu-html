//! Generic accessors over the typed element variants.

use lui_models::{
  ArcStr,
  common::html_enums::{ButtonType, HtmlDirection, InputType},
};
use lui_tree::{Element, Node};

/// Expand `$cb!(...)` with the same comma-separated list of `Element`
/// variants used everywhere in this module. Defined first so the
/// functions below can refer to it.
macro_rules! arms_list {
  ($cb:ident) => {
    $cb!(
      Html,
      Head,
      Body,
      Title,
      Meta,
      Link,
      StyleElement,
      Script,
      Noscript,
      H1,
      H2,
      H3,
      H4,
      H5,
      H6,
      P,
      Br,
      Hr,
      Pre,
      Blockquote,
      Address,
      Span,
      A,
      Strong,
      B,
      Em,
      I,
      U,
      S,
      Small,
      Mark,
      Code,
      Kbd,
      Samp,
      Var,
      Abbr,
      Cite,
      Dfn,
      Sub,
      Sup,
      Time,
      Ul,
      Ol,
      Li,
      Dl,
      Dt,
      Dd,
      Header,
      Nav,
      Main,
      Section,
      Article,
      Aside,
      Footer,
      Div,
      Img,
      Picture,
      Source,
      Video,
      Audio,
      Track,
      Iframe,
      Canvas,
      Svg,
      SvgPath,
      Table,
      Caption,
      Thead,
      Tbody,
      Tfoot,
      Tr,
      Th,
      Td,
      Colgroup,
      Col,
      Form,
      Label,
      Input,
      Textarea,
      Button,
      Select,
      OptionElement,
      Optgroup,
      Fieldset,
      Legend,
      Datalist,
      Output,
      Progress,
      Meter,
      Details,
      Summary,
      Dialog,
      Template,
      Slot,
      Del,
      Ins,
      Bdi,
      Bdo,
      Wbr,
      Data,
      Ruby,
      Rt,
      Rp,
    )
  };
}

/// Tag name (lowercase) for an `Element`. `Text` returns `None`.
pub fn element_tag(el: &Element) -> Option<&str> {
  macro_rules! arms {
        ($($v:ident => $tag:literal),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                Element::CustomElement(e) => Some(&e.tag_name),
                Element::SvgElement(e) => Some(&e.tag),
                $(Element::$v(_) => Some($tag),)*
            }
        };
    }
  arms!(
      Html => "html", Head => "head", Body => "body", Title => "title",
      Meta => "meta", Link => "link", StyleElement => "style", Script => "script",
      Noscript => "noscript",
      H1 => "h1", H2 => "h2", H3 => "h3", H4 => "h4", H5 => "h5", H6 => "h6",
      P => "p", Br => "br", Hr => "hr", Pre => "pre",
      Blockquote => "blockquote", Address => "address",
      Span => "span", A => "a", Strong => "strong", B => "b", Em => "em",
      I => "i", U => "u", S => "s", Small => "small", Mark => "mark",
      Code => "code", Kbd => "kbd", Samp => "samp", Var => "var",
      Abbr => "abbr", Cite => "cite", Dfn => "dfn", Sub => "sub", Sup => "sup",
      Time => "time",
      Ul => "ul", Ol => "ol", Li => "li", Dl => "dl", Dt => "dt", Dd => "dd",
      Header => "header", Nav => "nav", Main => "main", Section => "section",
      Article => "article", Aside => "aside", Footer => "footer", Div => "div",
      Img => "img", Picture => "picture", Source => "source", Video => "video",
      Audio => "audio", Track => "track", Iframe => "iframe", Canvas => "canvas",
      Svg => "svg",
      SvgPath => "path",
      Table => "table", Caption => "caption", Thead => "thead", Tbody => "tbody",
      Tfoot => "tfoot", Tr => "tr", Th => "th", Td => "td",
      Colgroup => "colgroup", Col => "col",
      Form => "form", Label => "label", Input => "input", Textarea => "textarea",
      Button => "button", Select => "select", OptionElement => "option",
      Optgroup => "optgroup", Fieldset => "fieldset", Legend => "legend",
      Datalist => "datalist", Output => "output", Progress => "progress",
      Meter => "meter",
      Details => "details", Summary => "summary", Dialog => "dialog",
      Template => "template", Slot => "slot",
      Del => "del", Ins => "ins", Bdi => "bdi", Bdo => "bdo", Wbr => "wbr",
      Data => "data", Ruby => "ruby", Rt => "rt", Rp => "rp",
  )
}

pub fn element_id(el: &Element) -> Option<&str> {
  macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                Element::CustomElement(e) => e.id.as_deref(),
                Element::SvgElement(e) => e.id.as_deref(),
                $(Element::$v(e) => e.id.as_deref(),)*
            }
        };
    }
  arms_list!(arms)
}

pub fn element_class(node: &Node) -> Option<ArcStr> {
  if node.class_list.is_empty() {
    None
  } else {
    Some(ArcStr::from(
      node.class_list.iter().map(|c| c.as_ref()).collect::<Vec<_>>().join(" "),
    ))
  }
}

pub fn element_style_attr(el: &Element) -> Option<&str> {
  macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                Element::CustomElement(e) => e.style.as_deref(),
                Element::SvgElement(e) => e.style.as_deref(),
                $(Element::$v(e) => e.style.as_deref(),)*
            }
        };
    }
  arms_list!(arms)
}

pub fn element_attr(node: &Node, name: &str) -> Option<String> {
  let el = &node.element;
  match name {
    "id" => element_id(el).map(str::to_string),
    "class" => element_class(node).map(|s| s.to_string()),
    "style" => element_style_attr(el).map(str::to_string),
    "title" => global_attr_string(el, |e| e.title().map(|s| s.to_string())),
    "lang" => global_attr_string(el, |e| e.lang().map(|s| s.to_string())),
    "dir" => global_attr_string(el, |e| e.dir().map(direction_attr_value).map(str::to_string)),
    "hidden" => global_bool_attr(el, |e| e.hidden()),
    "tabindex" => global_attr_string(el, |e| e.tabindex().map(|v| v.to_string())),
    "accesskey" => global_attr_string(el, |e| e.accesskey().map(|s| s.to_string())),
    "contenteditable" => global_bool_attr(el, |e| e.contenteditable()),
    "draggable" => global_bool_attr(el, |e| e.draggable()),
    "spellcheck" => global_bool_attr(el, |e| e.spellcheck()),
    "translate" => global_bool_attr(el, |e| e.translate()),
    "type" => element_type_attr(el),
    "open" => match el {
      Element::Dialog(e) => bool_attr(e.open),
      _ => None,
    },
    _ => {
      if let Some(suffix) = name.strip_prefix("data-") {
        if let Some(v) = node.data_attr(suffix) {
          return Some(v.to_string());
        }
      }
      if let Some(suffix) = name.strip_prefix("aria-") {
        if let Some(v) = node.aria_attr(suffix) {
          return Some(v.to_string());
        }
      }
      if let Element::CustomElement(e) = el {
        return e.custom_attrs.get(name).map(|s| s.to_string());
      }
      None
    }
  }
}

fn global_attr_string<F>(el: &Element, get: F) -> Option<String>
where
  F: Fn(&dyn GlobalAttrs) -> Option<String>,
{
  with_global_attrs(el, |e| get(e)).flatten()
}

fn global_bool_attr<F>(el: &Element, get: F) -> Option<String>
where
  F: Fn(&dyn GlobalAttrs) -> Option<bool>,
{
  with_global_attrs(el, |e| bool_attr(get(e))).flatten()
}

trait GlobalAttrs {
  fn title(&self) -> Option<&ArcStr>;
  fn lang(&self) -> Option<&ArcStr>;
  fn dir(&self) -> Option<&HtmlDirection>;
  fn hidden(&self) -> Option<bool>;
  fn tabindex(&self) -> Option<i32>;
  fn accesskey(&self) -> Option<&ArcStr>;
  fn contenteditable(&self) -> Option<bool>;
  fn draggable(&self) -> Option<bool>;
  fn spellcheck(&self) -> Option<bool>;
  fn translate(&self) -> Option<bool>;
}

macro_rules! impl_global_attrs {
    ($($ty:path),* $(,)?) => {
        $(
            impl GlobalAttrs for $ty {
                fn title(&self) -> Option<&ArcStr> { self.title.as_ref() }
                fn lang(&self) -> Option<&ArcStr> { self.lang.as_ref() }
                fn dir(&self) -> Option<&HtmlDirection> { self.dir.as_ref() }
                fn hidden(&self) -> Option<bool> { self.hidden }
                fn tabindex(&self) -> Option<i32> { self.tabindex }
                fn accesskey(&self) -> Option<&ArcStr> { self.accesskey.as_ref() }
                fn contenteditable(&self) -> Option<bool> { self.contenteditable }
                fn draggable(&self) -> Option<bool> { self.draggable }
                fn spellcheck(&self) -> Option<bool> { self.spellcheck }
                fn translate(&self) -> Option<bool> { self.translate }
            }
        )*
    };
}

impl_global_attrs!(
  lui_models::Html,
  lui_models::Head,
  lui_models::Body,
  lui_models::Title,
  lui_models::Meta,
  lui_models::Link,
  lui_models::StyleElement,
  lui_models::Script,
  lui_models::Noscript,
  lui_models::H1,
  lui_models::H2,
  lui_models::H3,
  lui_models::H4,
  lui_models::H5,
  lui_models::H6,
  lui_models::P,
  lui_models::Br,
  lui_models::Hr,
  lui_models::Pre,
  lui_models::Blockquote,
  lui_models::Address,
  lui_models::Span,
  lui_models::A,
  lui_models::Strong,
  lui_models::B,
  lui_models::Em,
  lui_models::I,
  lui_models::U,
  lui_models::S,
  lui_models::Small,
  lui_models::Mark,
  lui_models::Code,
  lui_models::Kbd,
  lui_models::Samp,
  lui_models::Var,
  lui_models::Abbr,
  lui_models::Cite,
  lui_models::Dfn,
  lui_models::Sub,
  lui_models::Sup,
  lui_models::Time,
  lui_models::Ul,
  lui_models::Ol,
  lui_models::Li,
  lui_models::Dl,
  lui_models::Dt,
  lui_models::Dd,
  lui_models::Header,
  lui_models::Nav,
  lui_models::Main,
  lui_models::Section,
  lui_models::Article,
  lui_models::Aside,
  lui_models::Footer,
  lui_models::Div,
  lui_models::Img,
  lui_models::Picture,
  lui_models::Source,
  lui_models::Video,
  lui_models::Audio,
  lui_models::Track,
  lui_models::Iframe,
  lui_models::Canvas,
  lui_models::Svg,
  lui_models::SvgPath,
  lui_models::Table,
  lui_models::Caption,
  lui_models::Thead,
  lui_models::Tbody,
  lui_models::Tfoot,
  lui_models::Tr,
  lui_models::Th,
  lui_models::Td,
  lui_models::Colgroup,
  lui_models::Col,
  lui_models::Form,
  lui_models::Label,
  lui_models::Input,
  lui_models::Textarea,
  lui_models::Button,
  lui_models::Select,
  lui_models::OptionElement,
  lui_models::Optgroup,
  lui_models::Fieldset,
  lui_models::Legend,
  lui_models::Datalist,
  lui_models::Output,
  lui_models::Progress,
  lui_models::Meter,
  lui_models::Details,
  lui_models::Summary,
  lui_models::Dialog,
  lui_models::Template,
  lui_models::Slot,
  lui_models::Del,
  lui_models::Ins,
  lui_models::Bdi,
  lui_models::Bdo,
  lui_models::Wbr,
  lui_models::Data,
  lui_models::Ruby,
  lui_models::Rt,
  lui_models::Rp,
  lui_models::CustomElement,
  lui_models::SvgElement,
);

fn with_global_attrs<R>(el: &Element, f: impl FnOnce(&dyn GlobalAttrs) -> R) -> Option<R> {
  macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                Element::CustomElement(e) => Some(f(e)),
                Element::SvgElement(e) => Some(f(e)),
                $(Element::$v(e) => Some(f(e)),)*
            }
        };
    }
  arms_list!(arms)
}

fn bool_attr(value: Option<bool>) -> Option<String> {
  value.and_then(|v| if v { Some(String::new()) } else { None })
}

fn direction_attr_value(dir: &HtmlDirection) -> &'static str {
  match dir {
    HtmlDirection::Ltr => "ltr",
    HtmlDirection::Rtl => "rtl",
    HtmlDirection::Auto => "auto",
  }
}

fn element_type_attr(el: &Element) -> Option<String> {
  match el {
    Element::Input(e) => e.r#type.as_ref().map(input_type_attr_value),
    Element::Button(e) => e.r#type.as_ref().map(button_type_attr_value),
    _ => None,
  }
}

fn input_type_attr_value(value: &InputType) -> String {
  let s = match value {
    InputType::Button => "button",
    InputType::Checkbox => "checkbox",
    InputType::Color => "color",
    InputType::Date => "date",
    InputType::DatetimeLocal => "datetime-local",
    InputType::Email => "email",
    InputType::File => "file",
    InputType::Hidden => "hidden",
    InputType::Image => "image",
    InputType::Month => "month",
    InputType::Number => "number",
    InputType::Password => "password",
    InputType::Radio => "radio",
    InputType::Range => "range",
    InputType::Reset => "reset",
    InputType::Search => "search",
    InputType::Submit => "submit",
    InputType::Tel => "tel",
    InputType::Text => "text",
    InputType::Time => "time",
    InputType::Url => "url",
    InputType::Week => "week",
  };
  s.to_string()
}

fn button_type_attr_value(value: &ButtonType) -> String {
  let s = match value {
    ButtonType::Button => "button",
    ButtonType::Submit => "submit",
    ButtonType::Reset => "reset",
  };
  s.to_string()
}
