use wgpu_html_tree::Element;

/// Inline `style="..."` string for an element, if any. `Element::Text`
/// has no styles.
pub(crate) fn element_style_attr(el: &Element) -> Option<&str> {
    macro_rules! arms {
        ($($v:ident),* $(,)?) => {
            match el {
                Element::Text(_) => None,
                $(Element::$v(e) => e.style.as_deref(),)*
            }
        };
    }
    arms!(
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
}
