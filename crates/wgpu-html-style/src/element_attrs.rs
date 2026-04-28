//! Generic accessors over the typed element variants.

use wgpu_html_models::common::html_enums::{ButtonType, HtmlDirection, InputType};
use wgpu_html_tree::Element;

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
pub fn element_tag(el: &Element) -> Option<&'static str> {
    macro_rules! arms {
        ($($v:ident => $tag:literal),* $(,)?) => {
            match el {
                Element::Text(_) => None,
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
                $(Element::$v(e) => e.id.as_deref(),)*
            }
        };
    }
    arms_list!(arms)
}

pub fn element_class(el: &Element) -> Option<&str> {
    macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                $(Element::$v(e) => e.class.as_deref(),)*
            }
        };
    }
    arms_list!(arms)
}

pub fn element_style_attr(el: &Element) -> Option<&str> {
    macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                $(Element::$v(e) => e.style.as_deref(),)*
            }
        };
    }
    arms_list!(arms)
}

pub fn element_attr(el: &Element, name: &str) -> Option<String> {
    match name {
        "id" => element_id(el).map(str::to_string),
        "class" => element_class(el).map(str::to_string),
        "style" => element_style_attr(el).map(str::to_string),
        "title" => global_attr_string(el, |e| e.title().cloned()),
        "lang" => global_attr_string(el, |e| e.lang().cloned()),
        "dir" => global_attr_string(el, |e| {
            e.dir().map(direction_attr_value).map(str::to_string)
        }),
        "hidden" => global_bool_attr(el, |e| e.hidden()),
        "tabindex" => global_attr_string(el, |e| e.tabindex().map(|v| v.to_string())),
        "accesskey" => global_attr_string(el, |e| e.accesskey().cloned()),
        "contenteditable" => global_bool_attr(el, |e| e.contenteditable()),
        "draggable" => global_bool_attr(el, |e| e.draggable()),
        "spellcheck" => global_bool_attr(el, |e| e.spellcheck()),
        "translate" => global_bool_attr(el, |e| e.translate()),
        "type" => element_type_attr(el),
        "open" => match el {
            Element::Dialog(e) => bool_attr(e.open),
            _ => None,
        },
        _ => data_or_aria_attr(el, name),
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
    fn title(&self) -> Option<&String>;
    fn lang(&self) -> Option<&String>;
    fn dir(&self) -> Option<&HtmlDirection>;
    fn hidden(&self) -> Option<bool>;
    fn tabindex(&self) -> Option<i32>;
    fn accesskey(&self) -> Option<&String>;
    fn contenteditable(&self) -> Option<bool>;
    fn draggable(&self) -> Option<bool>;
    fn spellcheck(&self) -> Option<bool>;
    fn translate(&self) -> Option<bool>;
    fn aria_attr(&self, name: &str) -> Option<&String>;
    fn data_attr(&self, name: &str) -> Option<&String>;
}

macro_rules! impl_global_attrs {
    ($($ty:path),* $(,)?) => {
        $(
            impl GlobalAttrs for $ty {
                fn title(&self) -> Option<&String> { self.title.as_ref() }
                fn lang(&self) -> Option<&String> { self.lang.as_ref() }
                fn dir(&self) -> Option<&HtmlDirection> { self.dir.as_ref() }
                fn hidden(&self) -> Option<bool> { self.hidden }
                fn tabindex(&self) -> Option<i32> { self.tabindex }
                fn accesskey(&self) -> Option<&String> { self.accesskey.as_ref() }
                fn contenteditable(&self) -> Option<bool> { self.contenteditable }
                fn draggable(&self) -> Option<bool> { self.draggable }
                fn spellcheck(&self) -> Option<bool> { self.spellcheck }
                fn translate(&self) -> Option<bool> { self.translate }
                fn aria_attr(&self, name: &str) -> Option<&String> {
                    self.aria_attrs.get(name)
                }
                fn data_attr(&self, name: &str) -> Option<&String> {
                    self.data_attrs.get(name)
                }
            }
        )*
    };
}

impl_global_attrs!(
    wgpu_html_models::Html,
    wgpu_html_models::Head,
    wgpu_html_models::Body,
    wgpu_html_models::Title,
    wgpu_html_models::Meta,
    wgpu_html_models::Link,
    wgpu_html_models::StyleElement,
    wgpu_html_models::Script,
    wgpu_html_models::Noscript,
    wgpu_html_models::H1,
    wgpu_html_models::H2,
    wgpu_html_models::H3,
    wgpu_html_models::H4,
    wgpu_html_models::H5,
    wgpu_html_models::H6,
    wgpu_html_models::P,
    wgpu_html_models::Br,
    wgpu_html_models::Hr,
    wgpu_html_models::Pre,
    wgpu_html_models::Blockquote,
    wgpu_html_models::Address,
    wgpu_html_models::Span,
    wgpu_html_models::A,
    wgpu_html_models::Strong,
    wgpu_html_models::B,
    wgpu_html_models::Em,
    wgpu_html_models::I,
    wgpu_html_models::U,
    wgpu_html_models::S,
    wgpu_html_models::Small,
    wgpu_html_models::Mark,
    wgpu_html_models::Code,
    wgpu_html_models::Kbd,
    wgpu_html_models::Samp,
    wgpu_html_models::Var,
    wgpu_html_models::Abbr,
    wgpu_html_models::Cite,
    wgpu_html_models::Dfn,
    wgpu_html_models::Sub,
    wgpu_html_models::Sup,
    wgpu_html_models::Time,
    wgpu_html_models::Ul,
    wgpu_html_models::Ol,
    wgpu_html_models::Li,
    wgpu_html_models::Dl,
    wgpu_html_models::Dt,
    wgpu_html_models::Dd,
    wgpu_html_models::Header,
    wgpu_html_models::Nav,
    wgpu_html_models::Main,
    wgpu_html_models::Section,
    wgpu_html_models::Article,
    wgpu_html_models::Aside,
    wgpu_html_models::Footer,
    wgpu_html_models::Div,
    wgpu_html_models::Img,
    wgpu_html_models::Picture,
    wgpu_html_models::Source,
    wgpu_html_models::Video,
    wgpu_html_models::Audio,
    wgpu_html_models::Track,
    wgpu_html_models::Iframe,
    wgpu_html_models::Canvas,
    wgpu_html_models::Svg,
    wgpu_html_models::Table,
    wgpu_html_models::Caption,
    wgpu_html_models::Thead,
    wgpu_html_models::Tbody,
    wgpu_html_models::Tfoot,
    wgpu_html_models::Tr,
    wgpu_html_models::Th,
    wgpu_html_models::Td,
    wgpu_html_models::Colgroup,
    wgpu_html_models::Col,
    wgpu_html_models::Form,
    wgpu_html_models::Label,
    wgpu_html_models::Input,
    wgpu_html_models::Textarea,
    wgpu_html_models::Button,
    wgpu_html_models::Select,
    wgpu_html_models::OptionElement,
    wgpu_html_models::Optgroup,
    wgpu_html_models::Fieldset,
    wgpu_html_models::Legend,
    wgpu_html_models::Datalist,
    wgpu_html_models::Output,
    wgpu_html_models::Progress,
    wgpu_html_models::Meter,
    wgpu_html_models::Details,
    wgpu_html_models::Summary,
    wgpu_html_models::Dialog,
    wgpu_html_models::Template,
    wgpu_html_models::Slot,
    wgpu_html_models::Del,
    wgpu_html_models::Ins,
    wgpu_html_models::Bdi,
    wgpu_html_models::Bdo,
    wgpu_html_models::Wbr,
    wgpu_html_models::Data,
    wgpu_html_models::Ruby,
    wgpu_html_models::Rt,
    wgpu_html_models::Rp,
);

fn with_global_attrs<R>(el: &Element, f: impl FnOnce(&dyn GlobalAttrs) -> R) -> Option<R> {
    macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                $(Element::$v(e) => Some(f(e)),)*
            }
        };
    }
    arms_list!(arms)
}

fn data_or_aria_attr(el: &Element, name: &str) -> Option<String> {
    if let Some(suffix) = name.strip_prefix("data-") {
        return with_global_attrs(el, |e| e.data_attr(suffix).cloned()).flatten();
    }
    if let Some(suffix) = name.strip_prefix("aria-") {
        return with_global_attrs(el, |e| e.aria_attr(suffix).cloned()).flatten();
    }
    None
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
