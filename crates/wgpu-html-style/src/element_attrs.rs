//! Generic accessors over the typed element variants.

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
