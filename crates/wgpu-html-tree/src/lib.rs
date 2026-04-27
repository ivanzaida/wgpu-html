//! Element tree.
//!
//! `Tree` is the root container. It holds a single `Node`. Each `Node` pairs
//! an `Element` (one of the HTML element model structs from
//! `wgpu-html-models`, or raw text) with its child nodes.
//!
//! Models stay pure data. Composition lives here.

use wgpu_html_models as m;

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub root: Option<Node>,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self { root: Some(root) }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub element: Element,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(element: impl Into<Element>) -> Self {
        Self {
            element: element.into(),
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    pub fn push(&mut self, child: Node) -> &mut Self {
        self.children.push(child);
        self
    }
}

/// One variant per HTML element kind, plus raw text.
#[derive(Debug, Clone)]
pub enum Element {
    Text(String),

    Html(m::Html),
    Head(m::Head),
    Body(m::Body),
    Title(m::Title),
    Meta(m::Meta),
    Link(m::Link),
    StyleElement(m::StyleElement),
    Script(m::Script),
    Noscript(m::Noscript),

    H1(m::H1),
    H2(m::H2),
    H3(m::H3),
    H4(m::H4),
    H5(m::H5),
    H6(m::H6),
    P(m::P),
    Br(m::Br),
    Hr(m::Hr),
    Pre(m::Pre),
    Blockquote(m::Blockquote),
    Address(m::Address),

    Span(m::Span),
    A(m::A),
    Strong(m::Strong),
    B(m::B),
    Em(m::Em),
    I(m::I),
    U(m::U),
    S(m::S),
    Small(m::Small),
    Mark(m::Mark),
    Code(m::Code),
    Kbd(m::Kbd),
    Samp(m::Samp),
    Var(m::Var),
    Abbr(m::Abbr),
    Cite(m::Cite),
    Dfn(m::Dfn),
    Sub(m::Sub),
    Sup(m::Sup),
    Time(m::Time),

    Ul(m::Ul),
    Ol(m::Ol),
    Li(m::Li),
    Dl(m::Dl),
    Dt(m::Dt),
    Dd(m::Dd),

    Header(m::Header),
    Nav(m::Nav),
    Main(m::Main),
    Section(m::Section),
    Article(m::Article),
    Aside(m::Aside),
    Footer(m::Footer),
    Div(m::Div),

    Img(m::Img),
    Picture(m::Picture),
    Source(m::Source),
    Video(m::Video),
    Audio(m::Audio),
    Track(m::Track),
    Iframe(m::Iframe),
    Canvas(m::Canvas),
    Svg(m::Svg),

    Table(m::Table),
    Caption(m::Caption),
    Thead(m::Thead),
    Tbody(m::Tbody),
    Tfoot(m::Tfoot),
    Tr(m::Tr),
    Th(m::Th),
    Td(m::Td),
    Colgroup(m::Colgroup),
    Col(m::Col),

    Form(m::Form),
    Label(m::Label),
    Input(m::Input),
    Textarea(m::Textarea),
    Button(m::Button),
    Select(m::Select),
    OptionElement(m::OptionElement),
    Optgroup(m::Optgroup),
    Fieldset(m::Fieldset),
    Legend(m::Legend),
    Datalist(m::Datalist),
    Output(m::Output),
    Progress(m::Progress),
    Meter(m::Meter),

    Details(m::Details),
    Summary(m::Summary),
    Dialog(m::Dialog),
    Template(m::Template),
    Slot(m::Slot),
    Del(m::Del),
    Ins(m::Ins),
    Bdi(m::Bdi),
    Bdo(m::Bdo),
    Wbr(m::Wbr),
    Data(m::Data),
    Ruby(m::Ruby),
    Rt(m::Rt),
    Rp(m::Rp),
}

/// Generate `From<T> for Element` impls so `Node::new(Div::default())` works.
macro_rules! element_from {
    ($($variant:ident => $ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Element {
                #[inline]
                fn from(v: $ty) -> Self { Element::$variant(v) }
            }
        )*
    };
}

impl From<String> for Element {
    fn from(s: String) -> Self {
        Element::Text(s)
    }
}
impl From<&str> for Element {
    fn from(s: &str) -> Self {
        Element::Text(s.to_owned())
    }
}

element_from! {
    Html => m::Html, Head => m::Head, Body => m::Body, Title => m::Title,
    Meta => m::Meta, Link => m::Link, StyleElement => m::StyleElement,
    Script => m::Script, Noscript => m::Noscript,

    H1 => m::H1, H2 => m::H2, H3 => m::H3, H4 => m::H4, H5 => m::H5, H6 => m::H6,
    P => m::P, Br => m::Br, Hr => m::Hr, Pre => m::Pre,
    Blockquote => m::Blockquote, Address => m::Address,

    Span => m::Span, A => m::A, Strong => m::Strong, B => m::B, Em => m::Em,
    I => m::I, U => m::U, S => m::S, Small => m::Small, Mark => m::Mark,
    Code => m::Code, Kbd => m::Kbd, Samp => m::Samp, Var => m::Var,
    Abbr => m::Abbr, Cite => m::Cite, Dfn => m::Dfn, Sub => m::Sub, Sup => m::Sup,
    Time => m::Time,

    Ul => m::Ul, Ol => m::Ol, Li => m::Li, Dl => m::Dl, Dt => m::Dt, Dd => m::Dd,

    Header => m::Header, Nav => m::Nav, Main => m::Main, Section => m::Section,
    Article => m::Article, Aside => m::Aside, Footer => m::Footer, Div => m::Div,

    Img => m::Img, Picture => m::Picture, Source => m::Source, Video => m::Video,
    Audio => m::Audio, Track => m::Track, Iframe => m::Iframe, Canvas => m::Canvas,
    Svg => m::Svg,

    Table => m::Table, Caption => m::Caption, Thead => m::Thead, Tbody => m::Tbody,
    Tfoot => m::Tfoot, Tr => m::Tr, Th => m::Th, Td => m::Td,
    Colgroup => m::Colgroup, Col => m::Col,

    Form => m::Form, Label => m::Label, Input => m::Input, Textarea => m::Textarea,
    Button => m::Button, Select => m::Select, OptionElement => m::OptionElement,
    Optgroup => m::Optgroup, Fieldset => m::Fieldset, Legend => m::Legend,
    Datalist => m::Datalist, Output => m::Output, Progress => m::Progress,
    Meter => m::Meter,

    Details => m::Details, Summary => m::Summary, Dialog => m::Dialog,
    Template => m::Template, Slot => m::Slot, Del => m::Del, Ins => m::Ins,
    Bdi => m::Bdi, Bdo => m::Bdo, Wbr => m::Wbr, Data => m::Data,
    Ruby => m::Ruby, Rt => m::Rt, Rp => m::Rp,
}
