//! Element tree.
//!
//! `Tree` is the root container. It holds a single `Node`. Each `Node` pairs
//! an `Element` (one of the HTML element model structs from
//! `wgpu-html-models`, or raw text) with its child nodes.
//!
//! Models stay pure data. Composition lives here.

use std::collections::HashMap;
use std::time::Duration;
use wgpu_html_models as m;

mod dispatch;
mod events;
mod focus;
mod fonts;
mod query;
pub mod text_edit;
pub mod tree_hook;

pub use query::{Combinator, ComplexSelector, CompoundSelector, SelectorList};

pub use dispatch::{
    blur, dispatch_mouse_down, dispatch_mouse_up, dispatch_pointer_leave, dispatch_pointer_move,
    focus, focus_next, key_down, key_up, text_input,
};
pub use events::{
    EditCursor, EventCallback, HtmlEvent, HtmlEventType, InteractionSnapshot, InteractionState,
    Modifier, Modifiers, MouseButton, MouseCallback, MouseEvent, SelectionColors, TextCursor,
    TextSelection,
};
pub use focus::{
    focusable_paths, is_focusable, is_keyboard_focusable, keyboard_focusable_paths, next_in_order,
    prev_in_order,
};
pub use fonts::{FontFace, FontHandle, FontRegistry, FontStyleAxis};
pub use tree_hook::{
    TreeHook, TreeHookResponse, TreeHookHandle, TreeLifecycleEvent, TreeLifecyclePhase,
    TreeLifecycleStage, TreeRenderEvent, TreeRenderViewport,
};

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub root: Option<Node>,
    /// Fonts available to this document. Populated by the host before
    /// layout / paint; consulted by the cascade and the text crate.
    /// See `docs/text.md` §3.
    pub fonts: FontRegistry,
    /// Live interaction state (hover / active / focus / pointer
    /// position / text selection / scroll offsets). Mutated by the
    /// dispatchers in `crate::dispatch` (re-exported as `tree.focus(…)`,
    /// `tree.key_down(…)`, `tree.dispatch_mouse_down(…)`, etc.); the
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
    /// URLs (or local file paths) the host wants pre-fetched into the
    /// image cache before they're referenced from the DOM. The layout
    /// pass walks this list once per pass and dispatches any
    /// not-yet-known URL to the worker pool. Calls are idempotent —
    /// already-cached URLs are skipped — so it's safe to populate
    /// once at startup via [`Tree::preload_asset`] and forget about
    /// it.
    pub preload_queue: Vec<String>,
    /// Host hooks registered on this document. Integration crates emit through
    /// `Tree::emit_*` methods so hook dispatch stays owned by this crate.
    pub hooks: Vec<TreeHookHandle>,
    /// Monotonically increasing counter, bumped whenever the DOM
    /// structure or content changes (custom properties, form control
    /// values, etc.). The pipeline cache compares this against its
    /// stored value to detect mutations that require re-cascade + relayout.
    pub generation: u64,
}

impl Tree {
    pub fn new(root: Node) -> Self {
        Self {
            root: Some(root),
            fonts: FontRegistry::new(),
            interaction: InteractionState::default(),
            asset_cache_ttl: None,
            preload_queue: Vec::new(),
            hooks: Vec::new(),
            generation: 0,
        }
    }

    /// Set a CSS custom property on the document root. Shorthand for
    /// `tree.root.set_custom_property(name, value)`.
    pub fn set_custom_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        if let Some(root) = &mut self.root {
            root.set_custom_property(name, value);
            self.generation += 1;
        }
    }

    /// Remove a programmatic custom property from the document root.
    pub fn remove_custom_property(&mut self, name: &str) -> Option<String> {
        let v = self.root.as_mut()?.remove_custom_property(name);
        if v.is_some() {
            self.generation += 1;
        }
        v
    }

    /// Register a font face with this document and return its handle.
    /// Re-registering a face with the same `(family, weight, style)`
    /// overrides the previous one (later registration wins on ties
    /// during matching).
    pub fn register_font(&mut self, face: FontFace) -> FontHandle {
        self.fonts.register(face)
    }

    /// Queue an image URL (or local filesystem path) for pre-loading.
    /// The next call to `paint_tree*` / `compute_layout` will dispatch
    /// the URL to the image-fetch worker pool if it's not already in
    /// the cache, so the first frame that actually needs the image
    /// doesn't wait. Duplicates are de-duped — calling this with the
    /// same URL twice is a no-op the second time.
    ///
    /// Typical usage at startup:
    /// ```ignore
    /// tree.preload_asset("https://example.com/hero.png");
    /// tree.preload_asset("assets/icons/menu.png");
    /// ```
    pub fn preload_asset(&mut self, src: impl Into<String>) {
        let s = src.into();
        if s.is_empty() || self.preload_queue.iter().any(|u| u == &s) {
            return;
        }
        self.preload_queue.push(s);
    }

    /// Return an immutable reference to the currently focused element,
    /// or `None` if nothing is focused or the focus path is stale.
    ///
    /// Useful for reading the focused form control's value without
    /// walking the path manually.
    pub fn active_element(&self) -> Option<&Node> {
        let path = self.interaction.focus_path.as_deref()?;
        self.root.as_ref()?.at_path(path)
    }

    /// Return a mutable reference to the currently focused element,
    /// or `None` if nothing is focused or the focus path is stale.
    pub fn active_element_mut(&mut self) -> Option<&mut Node> {
        let path = self.interaction.focus_path.clone()?;
        self.root.as_mut()?.at_path_mut(&path)
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

    /// Override the colors used when painting selected text.
    pub fn set_selection_colors(&mut self, background: [f32; 4], foreground: [f32; 4]) {
        self.interaction.selection_colors = SelectionColors {
            background,
            foreground,
        };
    }

    /// Clear any active text selection and exit selection-drag mode.
    pub fn clear_selection(&mut self) {
        self.interaction.selection = None;
        self.interaction.selecting_text = false;
    }
}

#[derive(Clone)]
pub struct Node {
    pub element: Element,
    pub children: Vec<Node>,
    /// CSS custom properties set programmatically on this node.
    /// Behaves as if declared in an inline `style` attribute — the
    /// cascade sees them after author/inline layers, and they
    /// inherit to descendants just like CSS-declared custom
    /// properties. Keys include the `--` prefix (e.g. `"--color"`).
    pub custom_properties: HashMap<String, String>,
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
    /// General-purpose handler that receives the full [`HtmlEvent`] for any
    /// event dispatched to this node, fired *after* the type-specific slot
    /// (e.g. `on_click`). Use this for keyboard, focus, wheel, or any event
    /// without a dedicated slot.
    pub on_event: Option<EventCallback>,
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The callback slots can't be Debug-printed; just note whether
        // each is wired so the tree's structure stays inspectable.
        f.debug_struct("Node")
            .field("element", &self.element)
            .field("children", &self.children)
            .field("custom_properties", &self.custom_properties)
            .field("on_click", &self.on_click.as_ref().map(|_| "<fn>"))
            .field(
                "on_mouse_down",
                &self.on_mouse_down.as_ref().map(|_| "<fn>"),
            )
            .field("on_mouse_up", &self.on_mouse_up.as_ref().map(|_| "<fn>"))
            .field(
                "on_mouse_enter",
                &self.on_mouse_enter.as_ref().map(|_| "<fn>"),
            )
            .field(
                "on_mouse_leave",
                &self.on_mouse_leave.as_ref().map(|_| "<fn>"),
            )
            .field("on_event", &self.on_event.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Node {
    pub fn new(element: impl Into<Element>) -> Self {
        Self {
            element: element.into(),
            children: Vec::new(),
            custom_properties: HashMap::new(),
            on_click: None,
            on_mouse_down: None,
            on_mouse_up: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_event: None,
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

    /// Set a CSS custom property on this node. Behaves as if declared
    /// in an inline `style` attribute — inherits to descendants and
    /// is available via `var(--name)` in CSS values.
    ///
    /// `name` must include the `--` prefix (e.g. `"--theme-color"`).
    pub fn set_custom_property(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.custom_properties.insert(name.into(), value.into());
    }

    /// Remove a previously set programmatic custom property.
    pub fn remove_custom_property(&mut self, name: &str) -> Option<String> {
        self.custom_properties.remove(name)
    }

    /// Read a programmatic custom property set on this node (does NOT
    /// walk the cascade or ancestors).
    pub fn custom_property(&self, name: &str) -> Option<&str> {
        self.custom_properties.get(name).map(|s| s.as_str())
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
    /// Walk a child-index path and return an immutable reference to
    /// the node at the end. Returns `None` for out-of-bounds indices.
    pub fn at_path(&self, path: &[usize]) -> Option<&Node> {
        let mut cursor: &Node = self;
        for &i in path {
            cursor = cursor.children.get(i)?;
        }
        Some(cursor)
    }

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

    /// `tabindex` HTML attribute on this element, if set. `Text`
    /// returns `None`.
    pub fn tabindex(&self) -> Option<i32> {
        macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.tabindex,)*
                }
            };
        }
        all_element_variants!(arms)
    }

    /// `class` HTML attribute on this element, if set. `Text`
    /// returns `None`. The returned string is the raw attribute
    /// value — split on ASCII whitespace to enumerate classes.
    pub fn class(&self) -> Option<&str> {
        macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.class.as_deref(),)*
                }
            };
        }
        all_element_variants!(arms)
    }

    /// Look up an HTML attribute by name (case-insensitive).
    /// Returns `None` if the element doesn't carry that attribute
    /// or the slot is empty.
    ///
    /// Coverage is the subset that actually shows up in selectors:
    /// the global attributes (`id`, `class`, `title`, `lang`,
    /// `tabindex`, `hidden`, `style`), `data-*` / `aria-*` entries,
    /// and the most common per-element attributes (`type`, `name`,
    /// `value`, `placeholder`, `href`, `src`, `alt`, `for`,
    /// `content`, plus boolean form attributes `disabled`,
    /// `readonly`, `required`, `checked`, `selected`, `multiple`,
    /// `autofocus`). Boolean attributes return `Some(String::new())`
    /// when present so they participate in `[attr]` presence
    /// filters, and `[attr=""]` matches them — same shape as the
    /// browser's reflection of HTML boolean attributes.
    ///
    /// `Text` nodes have no attributes and always return `None`.
    pub fn attr(&self, name: &str) -> Option<String> {
        let lname = name.to_ascii_lowercase();

        if let Some(v) = self.global_attr(&lname) {
            return Some(v);
        }
        if let Some(suffix) = lname.strip_prefix("data-") {
            return self.data_attr(suffix);
        }
        if let Some(suffix) = lname.strip_prefix("aria-") {
            return self.aria_attr(suffix);
        }
        self.specific_attr(&lname)
    }

    fn global_attr(&self, name: &str) -> Option<String> {
        macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => match name {
                        "id" => e.id.clone(),
                        "class" => e.class.clone(),
                        "title" => e.title.clone(),
                        "lang" => e.lang.clone(),
                        "dir" => e.dir.as_ref().map(|d| {
                            use wgpu_html_models::common::html_enums::HtmlDirection;
                            match d {
                                HtmlDirection::Ltr => "ltr",
                                HtmlDirection::Rtl => "rtl",
                                HtmlDirection::Auto => "auto",
                            }.to_owned()
                        }),
                        "tabindex" => e.tabindex.map(|t| t.to_string()),
                        "hidden" => match e.hidden { Some(true) => Some(String::new()), _ => None },
                        "style" => e.style.clone(),
                        _ => None,
                    },)*
                }
            };
        }
        all_element_variants!(arms)
    }

    fn data_attr(&self, suffix: &str) -> Option<String> {
        macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.data_attrs.get(suffix).cloned(),)*
                }
            };
        }
        all_element_variants!(arms)
    }

    fn aria_attr(&self, suffix: &str) -> Option<String> {
        macro_rules! arms {
            ($($v:ident),* $(,)?) => {
                match self {
                    Element::Text(_) => None,
                    $(Element::$v(e) => e.aria_attrs.get(suffix).cloned(),)*
                }
            };
        }
        all_element_variants!(arms)
    }

    fn specific_attr(&self, name: &str) -> Option<String> {
        // Boolean attribute helper: `Some(true)` becomes the empty
        // string (`[attr]` presence test), anything else `None`.
        fn flag(b: Option<bool>) -> Option<String> {
            match b {
                Some(true) => Some(String::new()),
                _ => None,
            }
        }
        match (name, self) {
            // type
            ("type", Element::Input(e)) => e.r#type.as_ref().map(|t| {
                use m::common::html_enums::InputType::*;
                match t {
                    Button => "button",
                    Checkbox => "checkbox",
                    Color => "color",
                    Date => "date",
                    DatetimeLocal => "datetime-local",
                    Email => "email",
                    File => "file",
                    Hidden => "hidden",
                    Image => "image",
                    Month => "month",
                    Number => "number",
                    Password => "password",
                    Radio => "radio",
                    Range => "range",
                    Reset => "reset",
                    Search => "search",
                    Submit => "submit",
                    Tel => "tel",
                    Text => "text",
                    Time => "time",
                    Url => "url",
                    Week => "week",
                }
                .to_owned()
            }),
            ("type", Element::Button(e)) => e.r#type.as_ref().map(|t| {
                use m::common::html_enums::ButtonType::*;
                match t {
                    Button => "button",
                    Submit => "submit",
                    Reset => "reset",
                }
                .to_owned()
            }),
            ("type", Element::Source(e)) => e.r#type.clone(),
            ("type", Element::Script(e)) => e.r#type.clone(),
            ("type", Element::StyleElement(e)) => e.r#type.clone(),
            ("type", Element::Link(e)) => e.r#type.clone(),
            ("type", Element::A(e)) => e.r#type.clone(),

            // name
            ("name", Element::Input(e)) => e.name.clone(),
            ("name", Element::Textarea(e)) => e.name.clone(),
            ("name", Element::Select(e)) => e.name.clone(),
            ("name", Element::Button(e)) => e.name.clone(),
            ("name", Element::Output(e)) => e.name.clone(),
            ("name", Element::Form(e)) => e.name.clone(),
            ("name", Element::Iframe(e)) => e.name.clone(),
            ("name", Element::Slot(e)) => e.name.clone(),
            ("name", Element::Meta(e)) => e.name.clone(),
            ("name", Element::Fieldset(e)) => e.name.clone(),
            ("name", Element::Details(e)) => e.name.clone(),

            // value
            ("value", Element::Input(e)) => e.value.clone(),
            ("value", Element::Button(e)) => e.value.clone(),
            ("value", Element::OptionElement(e)) => e.value.clone(),
            ("value", Element::Data(e)) => e.value.clone(),
            ("value", Element::Progress(e)) => e.value.map(|v| v.to_string()),
            ("value", Element::Meter(e)) => e.value.map(|v| v.to_string()),
            ("value", Element::Li(e)) => e.value.map(|v| v.to_string()),

            // content (meta)
            ("content", Element::Meta(e)) => e.content.clone(),

            // href
            ("href", Element::A(e)) => e.href.clone(),
            ("href", Element::Link(e)) => e.href.clone(),

            // src
            ("src", Element::Img(e)) => e.src.clone(),
            ("src", Element::Iframe(e)) => e.src.clone(),
            ("src", Element::Source(e)) => e.src.clone(),
            ("src", Element::Video(e)) => e.src.clone(),
            ("src", Element::Audio(e)) => e.src.clone(),
            ("src", Element::Track(e)) => e.src.clone(),
            ("src", Element::Script(e)) => e.src.clone(),

            // alt
            ("alt", Element::Img(e)) => e.alt.clone(),

            // for
            ("for", Element::Label(e)) => e.r#for.clone(),
            ("for", Element::Output(e)) => e.r#for.as_ref().map(|v| v.join(" ")),

            // placeholder
            ("placeholder", Element::Input(e)) => e.placeholder.clone(),
            ("placeholder", Element::Textarea(e)) => e.placeholder.clone(),

            // booleans
            ("disabled", Element::Input(e)) => flag(e.disabled),
            ("disabled", Element::Textarea(e)) => flag(e.disabled),
            ("disabled", Element::Select(e)) => flag(e.disabled),
            ("disabled", Element::Button(e)) => flag(e.disabled),
            ("disabled", Element::Optgroup(e)) => flag(e.disabled),
            ("disabled", Element::OptionElement(e)) => flag(e.disabled),
            ("disabled", Element::Fieldset(e)) => flag(e.disabled),

            ("readonly", Element::Input(e)) => flag(e.readonly),
            ("readonly", Element::Textarea(e)) => flag(e.readonly),

            ("required", Element::Input(e)) => flag(e.required),
            ("required", Element::Textarea(e)) => flag(e.required),
            ("required", Element::Select(e)) => flag(e.required),

            ("checked", Element::Input(e)) => flag(e.checked),
            ("selected", Element::OptionElement(e)) => flag(e.selected),

            ("multiple", Element::Input(e)) => flag(e.multiple),
            ("multiple", Element::Select(e)) => flag(e.multiple),

            ("autofocus", Element::Input(e)) => flag(e.autofocus),
            ("autofocus", Element::Textarea(e)) => flag(e.autofocus),
            ("autofocus", Element::Select(e)) => flag(e.autofocus),
            ("autofocus", Element::Button(e)) => flag(e.autofocus),

            _ => None,
        }
    }

    /// Lowercase HTML tag name for this element (e.g. `"div"`,
    /// `"option"`, `"style"`). `Text` returns `"#text"`.
    pub fn tag_name(&self) -> &'static str {
        match self {
            Element::Text(_) => "#text",
            Element::StyleElement(_) => "style",
            Element::OptionElement(_) => "option",
            Element::Html(_) => "html",
            Element::Head(_) => "head",
            Element::Body(_) => "body",
            Element::Title(_) => "title",
            Element::Meta(_) => "meta",
            Element::Link(_) => "link",
            Element::Script(_) => "script",
            Element::Noscript(_) => "noscript",
            Element::H1(_) => "h1",
            Element::H2(_) => "h2",
            Element::H3(_) => "h3",
            Element::H4(_) => "h4",
            Element::H5(_) => "h5",
            Element::H6(_) => "h6",
            Element::P(_) => "p",
            Element::Br(_) => "br",
            Element::Hr(_) => "hr",
            Element::Pre(_) => "pre",
            Element::Blockquote(_) => "blockquote",
            Element::Address(_) => "address",
            Element::Span(_) => "span",
            Element::A(_) => "a",
            Element::Strong(_) => "strong",
            Element::B(_) => "b",
            Element::Em(_) => "em",
            Element::I(_) => "i",
            Element::U(_) => "u",
            Element::S(_) => "s",
            Element::Small(_) => "small",
            Element::Mark(_) => "mark",
            Element::Code(_) => "code",
            Element::Kbd(_) => "kbd",
            Element::Samp(_) => "samp",
            Element::Var(_) => "var",
            Element::Abbr(_) => "abbr",
            Element::Cite(_) => "cite",
            Element::Dfn(_) => "dfn",
            Element::Sub(_) => "sub",
            Element::Sup(_) => "sup",
            Element::Time(_) => "time",
            Element::Ul(_) => "ul",
            Element::Ol(_) => "ol",
            Element::Li(_) => "li",
            Element::Dl(_) => "dl",
            Element::Dt(_) => "dt",
            Element::Dd(_) => "dd",
            Element::Header(_) => "header",
            Element::Nav(_) => "nav",
            Element::Main(_) => "main",
            Element::Section(_) => "section",
            Element::Article(_) => "article",
            Element::Aside(_) => "aside",
            Element::Footer(_) => "footer",
            Element::Div(_) => "div",
            Element::Img(_) => "img",
            Element::Picture(_) => "picture",
            Element::Source(_) => "source",
            Element::Video(_) => "video",
            Element::Audio(_) => "audio",
            Element::Track(_) => "track",
            Element::Iframe(_) => "iframe",
            Element::Canvas(_) => "canvas",
            Element::Svg(_) => "svg",
            Element::Table(_) => "table",
            Element::Caption(_) => "caption",
            Element::Thead(_) => "thead",
            Element::Tbody(_) => "tbody",
            Element::Tfoot(_) => "tfoot",
            Element::Tr(_) => "tr",
            Element::Th(_) => "th",
            Element::Td(_) => "td",
            Element::Colgroup(_) => "colgroup",
            Element::Col(_) => "col",
            Element::Form(_) => "form",
            Element::Label(_) => "label",
            Element::Input(_) => "input",
            Element::Textarea(_) => "textarea",
            Element::Button(_) => "button",
            Element::Select(_) => "select",
            Element::Optgroup(_) => "optgroup",
            Element::Fieldset(_) => "fieldset",
            Element::Legend(_) => "legend",
            Element::Datalist(_) => "datalist",
            Element::Output(_) => "output",
            Element::Progress(_) => "progress",
            Element::Meter(_) => "meter",
            Element::Details(_) => "details",
            Element::Summary(_) => "summary",
            Element::Dialog(_) => "dialog",
            Element::Template(_) => "template",
            Element::Slot(_) => "slot",
            Element::Del(_) => "del",
            Element::Ins(_) => "ins",
            Element::Bdi(_) => "bdi",
            Element::Bdo(_) => "bdo",
            Element::Wbr(_) => "wbr",
            Element::Data(_) => "data",
            Element::Ruby(_) => "ruby",
            Element::Rt(_) => "rt",
            Element::Rp(_) => "rp",
        }
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
        tree.get_element_by_id("target").unwrap().on_click = Some(Arc::new(move |_ev| {
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
