//! Self-contained HTML element type.
//!
//! One variant per HTML element from the WHATWG spec,
//! plus `Unknown(ArcStr)` for unrecognized tags.
//!
//! All element knowledge is baked in at compile time — just like
//! [`CssProperty`] in `lui-css-parser`.

use std::fmt;

use crate::ArcStr;

/// Every HTML element recognized by this parser.
///
/// Source: <https://github.com/w3c/webref/blob/curated/ed/elements/html.json>
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HtmlElement {
    /// Raw text content (not an HTML element).
    Text(ArcStr),
    /// HTML comment.
    Comment(ArcStr),
    // ── Document structure ──
    Html,
    Head,
    Body,
    Title,
    Base,
    Link,
    Meta,
    Style,
    // ── Sections ──
    Article,
    Section,
    Nav,
    Aside,
    H1, H2, H3, H4, H5, H6,
    Hgroup,
    Header,
    Footer,
    Address,
    // ── Grouping content ──
    P,
    Hr,
    Pre,
    Blockquote,
    Ol,
    Ul,
    Menu,
    Li,
    Dl,
    Dt,
    Dd,
    Figure,
    Figcaption,
    Main,
    Search,
    Div,
    // ── Text-level semantics ──
    A,
    Em,
    Strong,
    Small,
    S,
    Cite,
    Q,
    Dfn,
    Abbr,
    Ruby,
    Rt,
    Rp,
    Data,
    Time,
    Code,
    Var,
    Samp,
    Kbd,
    Sub,
    Sup,
    I,
    B,
    U,
    Mark,
    Bdi,
    Bdo,
    Span,
    Br,
    Wbr,
    Ins,
    Del,
    // ── Embedded content ──
    Picture,
    Source,
    Img,
    Iframe,
    Embed,
    Object,
    Video,
    Audio,
    Track,
    // ── Image maps ──
    Map,
    Area,
    // ── Tables ──
    Table,
    Caption,
    Colgroup,
    Col,
    Tbody,
    Thead,
    Tfoot,
    Tr,
    Td,
    Th,
    // ── Forms ──
    Form,
    Label,
    Input,
    Button,
    Select,
    Datalist,
    Optgroup,
    OptionElement,
    Textarea,
    Output,
    Progress,
    Meter,
    Fieldset,
    Legend,
    Selectedcontent,
    // ── Interactive ──
    Details,
    Summary,
    Dialog,
    // ── Scripting ──
    Script,
    Noscript,
    Template,
    Slot,
    // ── Canvas ──
    Canvas,
    // ── Obsolete ──
    Applet,
    Acronym,
    Bgsound,
    Dir,
    Frame,
    Frameset,
    Noframes,
    Isindex,
    Keygen,
    Listing,
    Menuitem,
    Nextid,
    Noembed,
    Param,
    Plaintext,
    Rb,
    Rtc,
    Strike,
    Xmp,
    Basefont,
    Big,
    Blink,
    Center,
    Font,
    Marquee,
    Multicol,
    Nobr,
    Spacer,
    Tt,
    // ── SVG (recognized by the parser) ──
    Svg,
    SvgPath,
    /// Catch-all for unknown tags. The contained string is the lowercased tag name.
    Unknown(ArcStr),
}

impl HtmlElement {
    /// Resolve a lowercased tag name to its `HtmlElement` variant.
    ///
    /// SVG element names and custom elements (containing `-`) are matched
    /// to `Svg`/`SvgPath` or `Unknown`.
    pub fn from_name(name: &str) -> Self {
        const ENTRIES: &[(&str, HtmlElement)] = &[
            ("a", HtmlElement::A),
            ("abbr", HtmlElement::Abbr),
            ("acronym", HtmlElement::Acronym),
            ("address", HtmlElement::Address),
            ("applet", HtmlElement::Applet),
            ("area", HtmlElement::Area),
            ("article", HtmlElement::Article),
            ("aside", HtmlElement::Aside),
            ("audio", HtmlElement::Audio),
            ("b", HtmlElement::B),
            ("base", HtmlElement::Base),
            ("basefont", HtmlElement::Basefont),
            ("bdi", HtmlElement::Bdi),
            ("bdo", HtmlElement::Bdo),
            ("bgsound", HtmlElement::Bgsound),
            ("big", HtmlElement::Big),
            ("blink", HtmlElement::Blink),
            ("blockquote", HtmlElement::Blockquote),
            ("body", HtmlElement::Body),
            ("br", HtmlElement::Br),
            ("button", HtmlElement::Button),
            ("canvas", HtmlElement::Canvas),
            ("caption", HtmlElement::Caption),
            ("center", HtmlElement::Center),
            ("cite", HtmlElement::Cite),
            ("code", HtmlElement::Code),
            ("col", HtmlElement::Col),
            ("colgroup", HtmlElement::Colgroup),
            ("data", HtmlElement::Data),
            ("datalist", HtmlElement::Datalist),
            ("dd", HtmlElement::Dd),
            ("del", HtmlElement::Del),
            ("details", HtmlElement::Details),
            ("dfn", HtmlElement::Dfn),
            ("dialog", HtmlElement::Dialog),
            ("dir", HtmlElement::Dir),
            ("div", HtmlElement::Div),
            ("dl", HtmlElement::Dl),
            ("dt", HtmlElement::Dt),
            ("em", HtmlElement::Em),
            ("embed", HtmlElement::Embed),
            ("fieldset", HtmlElement::Fieldset),
            ("figcaption", HtmlElement::Figcaption),
            ("figure", HtmlElement::Figure),
            ("font", HtmlElement::Font),
            ("footer", HtmlElement::Footer),
            ("form", HtmlElement::Form),
            ("frame", HtmlElement::Frame),
            ("frameset", HtmlElement::Frameset),
            ("h1", HtmlElement::H1),
            ("h2", HtmlElement::H2),
            ("h3", HtmlElement::H3),
            ("h4", HtmlElement::H4),
            ("h5", HtmlElement::H5),
            ("h6", HtmlElement::H6),
            ("head", HtmlElement::Head),
            ("header", HtmlElement::Header),
            ("hgroup", HtmlElement::Hgroup),
            ("hr", HtmlElement::Hr),
            ("html", HtmlElement::Html),
            ("i", HtmlElement::I),
            ("iframe", HtmlElement::Iframe),
            ("img", HtmlElement::Img),
            ("input", HtmlElement::Input),
            ("ins", HtmlElement::Ins),
            ("isindex", HtmlElement::Isindex),
            ("kbd", HtmlElement::Kbd),
            ("keygen", HtmlElement::Keygen),
            ("label", HtmlElement::Label),
            ("legend", HtmlElement::Legend),
            ("li", HtmlElement::Li),
            ("link", HtmlElement::Link),
            ("listing", HtmlElement::Listing),
            ("main", HtmlElement::Main),
            ("map", HtmlElement::Map),
            ("mark", HtmlElement::Mark),
            ("marquee", HtmlElement::Marquee),
            ("menu", HtmlElement::Menu),
            ("menuitem", HtmlElement::Menuitem),
            ("meta", HtmlElement::Meta),
            ("meter", HtmlElement::Meter),
            ("multicol", HtmlElement::Multicol),
            ("nav", HtmlElement::Nav),
            ("nextid", HtmlElement::Nextid),
            ("nobr", HtmlElement::Nobr),
            ("noembed", HtmlElement::Noembed),
            ("noframes", HtmlElement::Noframes),
            ("noscript", HtmlElement::Noscript),
            ("object", HtmlElement::Object),
            ("ol", HtmlElement::Ol),
            ("optgroup", HtmlElement::Optgroup),
            ("option", HtmlElement::OptionElement),
            ("output", HtmlElement::Output),
            ("p", HtmlElement::P),
            ("param", HtmlElement::Param),
            ("picture", HtmlElement::Picture),
            ("plaintext", HtmlElement::Plaintext),
            ("pre", HtmlElement::Pre),
            ("progress", HtmlElement::Progress),
            ("q", HtmlElement::Q),
            ("rb", HtmlElement::Rb),
            ("rp", HtmlElement::Rp),
            ("rt", HtmlElement::Rt),
            ("rtc", HtmlElement::Rtc),
            ("ruby", HtmlElement::Ruby),
            ("s", HtmlElement::S),
            ("samp", HtmlElement::Samp),
            ("script", HtmlElement::Script),
            ("search", HtmlElement::Search),
            ("section", HtmlElement::Section),
            ("select", HtmlElement::Select),
            ("selectedcontent", HtmlElement::Selectedcontent),
            ("slot", HtmlElement::Slot),
            ("small", HtmlElement::Small),
            ("source", HtmlElement::Source),
            ("spacer", HtmlElement::Spacer),
            ("span", HtmlElement::Span),
            ("strike", HtmlElement::Strike),
            ("strong", HtmlElement::Strong),
            ("style", HtmlElement::Style),
            ("sub", HtmlElement::Sub),
            ("summary", HtmlElement::Summary),
            ("sup", HtmlElement::Sup),
            ("table", HtmlElement::Table),
            ("tbody", HtmlElement::Tbody),
            ("td", HtmlElement::Td),
            ("template", HtmlElement::Template),
            ("textarea", HtmlElement::Textarea),
            ("tfoot", HtmlElement::Tfoot),
            ("th", HtmlElement::Th),
            ("thead", HtmlElement::Thead),
            ("time", HtmlElement::Time),
            ("title", HtmlElement::Title),
            ("tr", HtmlElement::Tr),
            ("track", HtmlElement::Track),
            ("tt", HtmlElement::Tt),
            ("u", HtmlElement::U),
            ("ul", HtmlElement::Ul),
            ("var", HtmlElement::Var),
            ("video", HtmlElement::Video),
            ("wbr", HtmlElement::Wbr),
            ("xmp", HtmlElement::Xmp),
        ];

        if let Some(idx) = ENTRIES.iter().position(|(s, _)| *s == name) {
            return ENTRIES[idx].1.clone();
        }

        // SVG elements
        if name == "svg" { return HtmlElement::Svg; }
        if name == "path" { return HtmlElement::SvgPath; }
        if SVG_ELEMENTS.contains(&name) { return HtmlElement::Svg; }

        // Everything else (including custom elements with `-`)
        HtmlElement::Unknown(ArcStr::from(name))
    }

    /// Returns the lowercased HTML tag name.
    pub fn tag_name(&self) -> &str {
        match self {
            HtmlElement::Text(_) => "#text",
            HtmlElement::Comment(_) => "#comment",
            HtmlElement::Unknown(name) => name,
            _ => HtmlElement::tag_name_static(self),
        }
    }

    fn tag_name_static(&self) -> &'static str {
        match self {
            HtmlElement::Html => "html",
            HtmlElement::Head => "head",
            HtmlElement::Body => "body",
            HtmlElement::Title => "title",
            HtmlElement::Base => "base",
            HtmlElement::Link => "link",
            HtmlElement::Meta => "meta",
            HtmlElement::Style => "style",
            HtmlElement::Article => "article",
            HtmlElement::Section => "section",
            HtmlElement::Nav => "nav",
            HtmlElement::Aside => "aside",
            HtmlElement::H1 => "h1",
            HtmlElement::H2 => "h2",
            HtmlElement::H3 => "h3",
            HtmlElement::H4 => "h4",
            HtmlElement::H5 => "h5",
            HtmlElement::H6 => "h6",
            HtmlElement::Hgroup => "hgroup",
            HtmlElement::Header => "header",
            HtmlElement::Footer => "footer",
            HtmlElement::Address => "address",
            HtmlElement::P => "p",
            HtmlElement::Hr => "hr",
            HtmlElement::Pre => "pre",
            HtmlElement::Blockquote => "blockquote",
            HtmlElement::Ol => "ol",
            HtmlElement::Ul => "ul",
            HtmlElement::Menu => "menu",
            HtmlElement::Li => "li",
            HtmlElement::Dl => "dl",
            HtmlElement::Dt => "dt",
            HtmlElement::Dd => "dd",
            HtmlElement::Figure => "figure",
            HtmlElement::Figcaption => "figcaption",
            HtmlElement::Main => "main",
            HtmlElement::Search => "search",
            HtmlElement::Div => "div",
            HtmlElement::A => "a",
            HtmlElement::Em => "em",
            HtmlElement::Strong => "strong",
            HtmlElement::Small => "small",
            HtmlElement::S => "s",
            HtmlElement::Cite => "cite",
            HtmlElement::Q => "q",
            HtmlElement::Dfn => "dfn",
            HtmlElement::Abbr => "abbr",
            HtmlElement::Ruby => "ruby",
            HtmlElement::Rt => "rt",
            HtmlElement::Rp => "rp",
            HtmlElement::Data => "data",
            HtmlElement::Time => "time",
            HtmlElement::Code => "code",
            HtmlElement::Var => "var",
            HtmlElement::Samp => "samp",
            HtmlElement::Kbd => "kbd",
            HtmlElement::Sub => "sub",
            HtmlElement::Sup => "sup",
            HtmlElement::I => "i",
            HtmlElement::B => "b",
            HtmlElement::U => "u",
            HtmlElement::Mark => "mark",
            HtmlElement::Bdi => "bdi",
            HtmlElement::Bdo => "bdo",
            HtmlElement::Span => "span",
            HtmlElement::Br => "br",
            HtmlElement::Wbr => "wbr",
            HtmlElement::Ins => "ins",
            HtmlElement::Del => "del",
            HtmlElement::Picture => "picture",
            HtmlElement::Source => "source",
            HtmlElement::Img => "img",
            HtmlElement::Iframe => "iframe",
            HtmlElement::Embed => "embed",
            HtmlElement::Object => "object",
            HtmlElement::Video => "video",
            HtmlElement::Audio => "audio",
            HtmlElement::Track => "track",
            HtmlElement::Map => "map",
            HtmlElement::Area => "area",
            HtmlElement::Table => "table",
            HtmlElement::Caption => "caption",
            HtmlElement::Colgroup => "colgroup",
            HtmlElement::Col => "col",
            HtmlElement::Tbody => "tbody",
            HtmlElement::Thead => "thead",
            HtmlElement::Tfoot => "tfoot",
            HtmlElement::Tr => "tr",
            HtmlElement::Td => "td",
            HtmlElement::Th => "th",
            HtmlElement::Form => "form",
            HtmlElement::Label => "label",
            HtmlElement::Input => "input",
            HtmlElement::Button => "button",
            HtmlElement::Select => "select",
            HtmlElement::Datalist => "datalist",
            HtmlElement::Optgroup => "optgroup",
            HtmlElement::OptionElement => "option",
            HtmlElement::Textarea => "textarea",
            HtmlElement::Output => "output",
            HtmlElement::Progress => "progress",
            HtmlElement::Meter => "meter",
            HtmlElement::Fieldset => "fieldset",
            HtmlElement::Legend => "legend",
            HtmlElement::Selectedcontent => "selectedcontent",
            HtmlElement::Details => "details",
            HtmlElement::Summary => "summary",
            HtmlElement::Dialog => "dialog",
            HtmlElement::Script => "script",
            HtmlElement::Noscript => "noscript",
            HtmlElement::Template => "template",
            HtmlElement::Slot => "slot",
            HtmlElement::Canvas => "canvas",
            HtmlElement::Applet => "applet",
            HtmlElement::Acronym => "acronym",
            HtmlElement::Bgsound => "bgsound",
            HtmlElement::Dir => "dir",
            HtmlElement::Frame => "frame",
            HtmlElement::Frameset => "frameset",
            HtmlElement::Noframes => "noframes",
            HtmlElement::Isindex => "isindex",
            HtmlElement::Keygen => "keygen",
            HtmlElement::Listing => "listing",
            HtmlElement::Menuitem => "menuitem",
            HtmlElement::Nextid => "nextid",
            HtmlElement::Noembed => "noembed",
            HtmlElement::Param => "param",
            HtmlElement::Plaintext => "plaintext",
            HtmlElement::Rb => "rb",
            HtmlElement::Rtc => "rtc",
            HtmlElement::Strike => "strike",
            HtmlElement::Xmp => "xmp",
            HtmlElement::Basefont => "basefont",
            HtmlElement::Big => "big",
            HtmlElement::Blink => "blink",
            HtmlElement::Center => "center",
            HtmlElement::Font => "font",
            HtmlElement::Marquee => "marquee",
            HtmlElement::Multicol => "multicol",
            HtmlElement::Nobr => "nobr",
            HtmlElement::Spacer => "spacer",
            HtmlElement::Tt => "tt",
            HtmlElement::Svg => "svg",
            HtmlElement::SvgPath => "path",
            HtmlElement::Text(_) => "#text",
            HtmlElement::Comment(_) => "#comment",
            HtmlElement::Unknown(_) => unreachable!(),
        }
    }

    /// True for void elements that cannot have children.
    pub fn is_void(&self) -> bool {
        matches!(
            self.tag_name(),
            "area" | "base" | "br" | "col" | "embed" | "hr" | "img"
                | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr"
        )
    }

    /// True for elements parsed as raw text (no child HTML).
    pub fn is_raw_text(&self) -> bool {
        matches!(self.tag_name(), "script" | "style" | "textarea" | "title")
    }

    /// True for recognized elements (not `Unknown` or `Text` or `Comment`).
    pub fn is_known(&self) -> bool {
        !matches!(self, HtmlElement::Unknown(_) | HtmlElement::Text(_) | HtmlElement::Comment(_))
    }

    /// True for text nodes.
    pub fn is_text(&self) -> bool {
        matches!(self, HtmlElement::Text(_))
    }

    /// True for comment nodes.
    pub fn is_comment(&self) -> bool {
        matches!(self, HtmlElement::Comment(_))
    }
}

impl fmt::Display for HtmlElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.tag_name())
    }
}

/// Auto-close rules: when opening `child`, close any open `parent`.
///
/// Returns `true` if the open element should be implicitly closed when
/// a new element is being opened inside it.
pub fn should_auto_close(open_tag: &str, opening_tag: &str) -> bool {
    match open_tag {
        "p" => matches!(
            opening_tag,
            "address" | "article" | "aside" | "blockquote" | "details" | "div"
                | "dl" | "fieldset" | "figcaption" | "figure" | "footer" | "form"
                | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                | "header" | "hgroup" | "hr" | "main" | "menu" | "nav"
                | "ol" | "p" | "pre" | "section" | "table" | "ul"
        ),
        "li" => opening_tag == "li",
        "dt" => matches!(opening_tag, "dt" | "dd"),
        "dd" => matches!(opening_tag, "dt" | "dd"),
        "thead" => matches!(opening_tag, "tbody" | "tfoot"),
        "tbody" => matches!(opening_tag, "tbody" | "tfoot"),
        "tr" => opening_tag == "tr",
        "th" => matches!(opening_tag, "td" | "th" | "tr"),
        "td" => matches!(opening_tag, "td" | "th" | "tr"),
        "option" => matches!(opening_tag, "option" | "optgroup"),
        "optgroup" => opening_tag == "optgroup",
        "rt" => matches!(opening_tag, "rt" | "rp"),
        "rp" => matches!(opening_tag, "rt" | "rp"),
        _ => false,
    }
}

/// SVG element names that produce `HtmlElement::Svg`.
pub const SVG_ELEMENTS: &[&str] = &[
    "circle", "rect", "ellipse", "line", "polygon", "polyline", "g", "defs", "symbol", "use", "marker",
    "clippath", "mask", "pattern", "image", "stop", "text", "tspan", "textpath",
    "lineargradient", "radialgradient",
    "filter", "fegaussianblur", "feblend", "fecolormatrix",
    "fecomponenttransfer", "fecomposite", "feconvolvematrix",
    "fediffuselighting", "fedisplacementmap", "fedropshadow",
    "feflood", "feimage", "femerge", "femergenode",
    "femorphology", "feoffset", "fespecularlighting",
    "fetile", "feturbulence",
    "fedistantlight", "fepointlight", "fespotlight",
    "animate", "animatemotion", "animatetransform", "set",
    "desc", "title", "metadata", "switch", "view", "foreignobject",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_tag_roundtrips() {
        for (tag, expected) in &[
            ("div", HtmlElement::Div),
            ("span", HtmlElement::Span),
            ("a", HtmlElement::A),
            ("p", HtmlElement::P),
            ("html", HtmlElement::Html),
            ("body", HtmlElement::Body),
            ("input", HtmlElement::Input),
            ("textarea", HtmlElement::Textarea),
            ("script", HtmlElement::Script),
            ("style", HtmlElement::Style),
            ("br", HtmlElement::Br),
            ("img", HtmlElement::Img),
            ("svg", HtmlElement::Svg),
            ("path", HtmlElement::SvgPath),
        ] {
            assert_eq!(HtmlElement::from_name(tag), *expected);
            assert_eq!(expected.tag_name(), *tag);
        }
    }

    #[test]
    fn unknown_tag_is_unknown() {
        assert_eq!(HtmlElement::from_name("foo"), HtmlElement::Unknown("foo".into()));
    }

    #[test]
    fn custom_elements_are_unknown() {
        assert_eq!(HtmlElement::from_name("my-widget"), HtmlElement::Unknown("my-widget".into()));
    }

    #[test]
    fn void_elements_are_void() {
        for tag in &["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"] {
            assert!(HtmlElement::from_name(tag).is_void(), "{} should be void", tag);
        }
        assert!(!HtmlElement::from_name("div").is_void());
    }

    #[test]
    fn raw_text_elements() {
        for tag in &["script", "style", "textarea", "title"] {
            assert!(HtmlElement::from_name(tag).is_raw_text(), "{} should be raw text", tag);
        }
        assert!(!HtmlElement::from_name("div").is_raw_text());
    }

    #[test]
    fn obsolete_elements_recognized() {
        assert_eq!(HtmlElement::from_name("marquee"), HtmlElement::Marquee);
        assert_eq!(HtmlElement::from_name("blink"), HtmlElement::Blink);
        assert_eq!(HtmlElement::from_name("font"), HtmlElement::Font);
    }
}
