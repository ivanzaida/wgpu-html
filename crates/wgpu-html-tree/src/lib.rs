//! Element tree.
//!
//! `Tree` is the root container. It holds a single `Node`. Each `Node` pairs
//! an `Element` (one of the HTML element model structs from
//! `wgpu-html-models`, or raw text) with its child nodes.
//!
//! Models stay pure data. Composition lives here.

use wgpu_html_models as m;

mod fonts;

pub use fonts::{FontFace, FontHandle, FontRegistry, FontStyleAxis};

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub root: Option<Node>,
    /// Fonts available to this document. Populated by the host before
    /// layout / paint; consulted by the cascade and the text crate.
    /// See `docs/text.md` §3.
    pub fonts: FontRegistry,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            root: Some(root),
            fonts: FontRegistry::new(),
        }
    }

    /// Register a font face with this document and return its handle.
    /// Re-registering a face with the same `(family, weight, style)`
    /// overrides the previous one (later registration wins on ties
    /// during matching).
    pub fn register_font(&mut self, face: FontFace) -> FontHandle {
        self.fonts.register(face)
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

    /// Walk a child-index path from this node to a descendant. An empty
    /// path returns `Some(self)`. Returns `None` if any index is out of
    /// bounds.
    pub fn at_path_mut(&mut self, path: &[usize]) -> Option<&mut Node> {
        let mut cursor: &mut Node = self;
        for &i in path {
            cursor = cursor.children.get_mut(i)?;
        }
        Some(cursor)
    }

    /// Walk a child-index path and collect every node visited, ordered
    /// deepest descendant → root. The first element is the deepest hit;
    /// the last is `self`.
    ///
    /// Soundness: the returned `&mut` references all alias into nested
    /// subtrees of the same borrow — each is an ancestor of the next.
    /// Two of them must never be dereferenced concurrently. Walking the
    /// chain one step at a time (event bubbling, etc.) is fine.
    pub fn ancestry_at_path_mut(&mut self, path: &[usize]) -> Vec<&mut Node> {
        let mut out: Vec<&mut Node> = Vec::with_capacity(path.len() + 1);
        // SAFETY: every pointer is derived from `self`'s exclusive
        // borrow and points at a strict subtree of the previous one.
        // We rely on the documented contract that callers do not
        // access two of the returned references simultaneously.
        unsafe {
            let mut cursor: *mut Node = self as *mut Node;
            out.push(&mut *cursor);
            for &i in path {
                let children: *mut Vec<Node> = &raw mut (*cursor).children;
                if i >= (*children).len() {
                    break;
                }
                cursor = (*children).as_mut_ptr().add(i);
                out.push(&mut *cursor);
            }
        }
        out.reverse();
        out
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
