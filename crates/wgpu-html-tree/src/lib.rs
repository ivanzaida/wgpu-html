//! Element tree.
//!
//! `Tree` is the root container. It holds a single `Node`. Each `Node` pairs
//! an `Element` (one of the HTML element model structs from
//! `wgpu-html-models`, or raw text) with its child nodes.
//!
//! Models stay pure data. Composition lives here.

use std::time::Duration;
use wgpu_html_models as m;

mod events;
mod fonts;

pub use events::{InteractionState, Modifiers, MouseButton, MouseCallback, MouseEvent};
pub use fonts::{FontFace, FontHandle, FontRegistry, FontStyleAxis};

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub root: Option<Node>,
    /// Fonts available to this document. Populated by the host before
    /// layout / paint; consulted by the cascade and the text crate.
    /// See `docs/text.md` §3.
    pub fonts: FontRegistry,
    /// Live interaction state (hover / active / pointer position).
    /// Mutated by the dispatcher in `wgpu_html::interactivity`; the
    /// cascade and paint passes read it but never write.
    pub interaction: InteractionState,
    /// How long to keep decoded images in memory after their last use
    /// before reclaiming. `None` leaves the engine default in place
    /// (5 minutes); `Some(d)` overrides it on the next layout pass.
    /// The setting is applied process-wide — successive trees with
    /// different values "win" in last-applied order. Set to a small
    /// value to aggressively free memory on memory-constrained hosts;
    /// raise it to keep images warm across navigations.
    pub asset_cache_ttl: Option<Duration>,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            root: Some(root),
            fonts: FontRegistry::new(),
            interaction: InteractionState::default(),
            asset_cache_ttl: None,
        }
    }

    /// Register a font face with this document and return its handle.
    /// Re-registering a face with the same `(family, weight, style)`
    /// overrides the previous one (later registration wins on ties
    /// during matching).
    pub fn register_font(&mut self, face: FontFace) -> FontHandle {
        self.fonts.register(face)
    }

    /// Find the first descendant whose `id` attribute equals `id`,
    /// document-order. Returns `None` if no element matches or the
    /// tree is empty.
    ///
    /// ```ignore
    /// if let Some(el) = tree.get_element_by_id("submit") {
    ///     el.on_click = Some(std::sync::Arc::new(|ev| {
    ///         eprintln!("clicked at {:?}", ev.pos);
    ///     }));
    /// }
    /// ```
    pub fn get_element_by_id(&mut self, id: &str) -> Option<&mut Node> {
        self.root.as_mut()?.find_by_id_mut(id)
    }
}

#[derive(Clone)]
pub struct Node {
    pub element: Element,
    pub children: Vec<Node>,
    /// Fires when a primary-button press *and* the matching release
    /// both land inside this node's subtree. Bubbles target → root.
    pub on_click: Option<MouseCallback>,
    /// Fires on every primary-button press, target → root.
    pub on_mouse_down: Option<MouseCallback>,
    /// Fires on every primary-button release, target → root.
    pub on_mouse_up: Option<MouseCallback>,
    /// Fires when the pointer enters this node's subtree (root-first
    /// across the entered chain). No bubbling beyond the entered set.
    pub on_mouse_enter: Option<MouseCallback>,
    /// Fires when the pointer leaves this node's subtree
    /// (deepest-first across the left chain).
    pub on_mouse_leave: Option<MouseCallback>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The callback slots can't be Debug-printed; just note whether
        // each is wired so the tree's structure stays inspectable.
        f.debug_struct("Node")
            .field("element", &self.element)
            .field("children", &self.children)
            .field("on_click", &self.on_click.as_ref().map(|_| "<fn>"))
            .field("on_mouse_down", &self.on_mouse_down.as_ref().map(|_| "<fn>"))
            .field("on_mouse_up", &self.on_mouse_up.as_ref().map(|_| "<fn>"))
            .field("on_mouse_enter", &self.on_mouse_enter.as_ref().map(|_| "<fn>"))
            .field("on_mouse_leave", &self.on_mouse_leave.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Node {
    pub fn new(element: impl Into<Element>) -> Self {
        Self {
            element: element.into(),
            children: Vec::new(),
            on_click: None,
            on_mouse_down: None,
            on_mouse_up: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
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

    /// Depth-first search for a descendant (or `self`) whose `id`
    /// attribute equals `id`. Document order; first match wins.
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut Node> {
        if self.element.id() == Some(id) {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_by_id_mut(id) {
                return Some(found);
            }
        }
        None
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

/// Same variant list used for any "do this for every element" dispatch.
/// `Text` is excluded — it has no attributes.
macro_rules! all_element_variants {
    ($cb:ident) => {
        $cb!(
            Html, Head, Body, Title, Meta, Link, StyleElement, Script, Noscript,
            H1, H2, H3, H4, H5, H6, P, Br, Hr, Pre, Blockquote, Address,
            Span, A, Strong, B, Em, I, U, S, Small, Mark, Code, Kbd, Samp, Var,
            Abbr, Cite, Dfn, Sub, Sup, Time,
            Ul, Ol, Li, Dl, Dt, Dd,
            Header, Nav, Main, Section, Article, Aside, Footer, Div,
            Img, Picture, Source, Video, Audio, Track, Iframe, Canvas, Svg,
            Table, Caption, Thead, Tbody, Tfoot, Tr, Th, Td, Colgroup, Col,
            Form, Label, Input, Textarea, Button, Select, OptionElement, Optgroup,
            Fieldset, Legend, Datalist, Output, Progress, Meter,
            Details, Summary, Dialog, Template, Slot,
            Del, Ins, Bdi, Bdo, Wbr, Data, Ruby, Rt, Rp,
        )
    };
}

impl Element {
    /// `id` HTML attribute on this element, if set. `Text` has no
    /// attributes and returns `None`.
    pub fn id(&self) -> Option<&str> {
        macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.id.as_deref(),)*
                }
            };
        }
        all_element_variants!(arms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn div_with_id(id: &str) -> m::Div {
        m::Div {
            id: Some(id.to_string()),
            ..m::Div::default()
        }
    }

    #[test]
    fn element_id_reads_global_attribute() {
        let div = Element::Div(div_with_id("hero"));
        assert_eq!(div.id(), Some("hero"));
        let txt = Element::Text("hi".into());
        assert_eq!(txt.id(), None);
    }

    #[test]
    fn get_element_by_id_finds_descendant() {
        let body = Node::new(m::Body::default()).with_children(vec![
            Node::new(div_with_id("outer")).with_children(vec![Node::new(div_with_id("inner"))]),
        ]);
        let mut tree = Tree::new(body);
        assert!(tree.get_element_by_id("outer").is_some());
        assert!(tree.get_element_by_id("inner").is_some());
        assert!(tree.get_element_by_id("missing").is_none());
    }

    #[test]
    fn on_click_field_is_assignable_and_invokable() {
        let mut tree = Tree::new(Node::new(div_with_id("target")));
        let counter = Arc::new(AtomicUsize::new(0));
        let c2 = counter.clone();

        // Direct field assignment in the friendly style:
        // `tree.get_element_by_id(id).on_click = cb`.
        tree.get_element_by_id("target").unwrap().on_click =
            Some(Arc::new(move |_ev| {
                c2.fetch_add(1, Ordering::Relaxed);
            }));

        // The callback isn't fired by storage alone — invoke it.
        let cb = tree
            .get_element_by_id("target")
            .unwrap()
            .on_click
            .clone()
            .unwrap();
        let ev = MouseEvent {
            pos: (0.0, 0.0),
            button: Some(MouseButton::Primary),
            modifiers: Modifiers::default(),
            target_path: vec![],
            current_path: vec![],
        };
        cb(&ev);
        cb(&ev);
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn first_match_wins_in_document_order() {
        let body = Node::new(m::Body::default()).with_children(vec![
            Node::new(div_with_id("dup")),
            Node::new(div_with_id("dup")),
        ]);
        let mut tree = Tree::new(body);
        let first = tree.get_element_by_id("dup").unwrap();
        // Mutate so we can identify which one we got back without
        // depending on pointer identity.
        first.on_click = Some(Arc::new(|_| {}));
        let body_node = tree.root.as_ref().unwrap();
        assert!(body_node.children[0].on_click.is_some());
        assert!(body_node.children[1].on_click.is_none());
    }
}
