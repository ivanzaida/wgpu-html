use wgpu_html_models as html;
use wgpu_html_models::common::html_enums::*;
use wgpu_html_tree::Element;

/// Parse a tag name and raw attribute list into a typed `Element`.
///
/// Returns `None` for unrecognized tags; callers should drop the subtree.
pub fn parse_element(tag: &str, attrs: &[(String, String)]) -> Option<Element> {
    Some(match tag {
        "html" => {
            let mut el = html::Html::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "xmlns" => el.xmlns = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Html(el)
        }
        "head" => {
            let mut el = html::Head::default();
            set_global!(el, attrs);
            Element::Head(el)
        }
        "body" => {
            let mut el = html::Body::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "onload" => el.onload = Some(v.clone()),
                    "onunload" => el.onunload = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Body(el)
        }
        "title" => {
            let mut el = html::Title::default();
            set_global!(el, attrs);
            Element::Title(el)
        }
        "meta" => {
            let mut el = html::Meta::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "name" => el.name = Some(v.clone()),
                    "content" => el.content = Some(v.clone()),
                    "charset" => el.charset = Some(v.clone()),
                    "http-equiv" => el.http_equiv = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Meta(el)
        }
        "link" => {
            let mut el = html::Link::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "href" => el.href = Some(v.clone()),
                    "rel" => el.rel = Some(v.clone()),
                    "type" => el.r#type = Some(v.clone()),
                    "media" => el.media = Some(v.clone()),
                    "sizes" => el.sizes = Some(v.clone()),
                    "hreflang" => el.hreflang = Some(v.clone()),
                    "as" => el.r#as = parse_link_as(v),
                    "crossorigin" => el.crossorigin = parse_crossorigin(v),
                    "integrity" => el.integrity = Some(v.clone()),
                    "referrerpolicy" => el.referrerpolicy = parse_referrer_policy(v),
                    _ => {}
                }
            }
            Element::Link(el)
        }
        "style" => {
            let mut el = html::StyleElement::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "type" => el.r#type = Some(v.clone()),
                    "media" => el.media = Some(v.clone()),
                    "nonce" => el.nonce = Some(v.clone()),
                    _ => {}
                }
            }
            Element::StyleElement(el)
        }
        "script" => {
            let mut el = html::Script::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "type" => el.r#type = Some(v.clone()),
                    "async" => el.r#async = Some(parse_bool_attr(v)),
                    "defer" => el.defer = Some(parse_bool_attr(v)),
                    "crossorigin" => el.crossorigin = parse_crossorigin(v),
                    "integrity" => el.integrity = Some(v.clone()),
                    "nomodule" => el.nomodule = Some(parse_bool_attr(v)),
                    "nonce" => el.nonce = Some(v.clone()),
                    "referrerpolicy" => el.referrerpolicy = parse_referrer_policy(v),
                    _ => {}
                }
            }
            Element::Script(el)
        }
        "noscript" => {
            let mut el = html::Noscript::default();
            set_global!(el, attrs);
            Element::Noscript(el)
        }
        "h1" => {
            let mut el = html::H1::default();
            set_global!(el, attrs);
            Element::H1(el)
        }
        "h2" => {
            let mut el = html::H2::default();
            set_global!(el, attrs);
            Element::H2(el)
        }
        "h3" => {
            let mut el = html::H3::default();
            set_global!(el, attrs);
            Element::H3(el)
        }
        "h4" => {
            let mut el = html::H4::default();
            set_global!(el, attrs);
            Element::H4(el)
        }
        "h5" => {
            let mut el = html::H5::default();
            set_global!(el, attrs);
            Element::H5(el)
        }
        "h6" => {
            let mut el = html::H6::default();
            set_global!(el, attrs);
            Element::H6(el)
        }
        "p" => {
            let mut el = html::P::default();
            set_global!(el, attrs);
            Element::P(el)
        }
        "br" => {
            let mut el = html::Br::default();
            set_global!(el, attrs);
            Element::Br(el)
        }
        "hr" => {
            let mut el = html::Hr::default();
            set_global!(el, attrs);
            Element::Hr(el)
        }
        "pre" => {
            let mut el = html::Pre::default();
            set_global!(el, attrs);
            Element::Pre(el)
        }
        "blockquote" => {
            let mut el = html::Blockquote::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "cite" => el.cite = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Blockquote(el)
        }
        "address" => {
            let mut el = html::Address::default();
            set_global!(el, attrs);
            Element::Address(el)
        }
        "span" => {
            let mut el = html::Span::default();
            set_global!(el, attrs);
            Element::Span(el)
        }
        "a" => {
            let mut el = html::A::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "href" => el.href = Some(v.clone()),
                    "target" => el.target = parse_link_target(v),
                    "download" => el.download = Some(v.clone()),
                    "rel" => el.rel = Some(v.clone()),
                    "hreflang" => el.hreflang = Some(v.clone()),
                    "type" => el.r#type = Some(v.clone()),
                    "ping" => el.ping = Some(v.clone()),
                    "referrerpolicy" => el.referrerpolicy = parse_referrer_policy(v),
                    _ => {}
                }
            }
            Element::A(el)
        }
        "strong" => {
            let mut el = html::Strong::default();
            set_global!(el, attrs);
            Element::Strong(el)
        }
        "b" => {
            let mut el = html::B::default();
            set_global!(el, attrs);
            Element::B(el)
        }
        "em" => {
            let mut el = html::Em::default();
            set_global!(el, attrs);
            Element::Em(el)
        }
        "i" => {
            let mut el = html::I::default();
            set_global!(el, attrs);
            Element::I(el)
        }
        "u" => {
            let mut el = html::U::default();
            set_global!(el, attrs);
            Element::U(el)
        }
        "s" => {
            let mut el = html::S::default();
            set_global!(el, attrs);
            Element::S(el)
        }
        "small" => {
            let mut el = html::Small::default();
            set_global!(el, attrs);
            Element::Small(el)
        }
        "mark" => {
            let mut el = html::Mark::default();
            set_global!(el, attrs);
            Element::Mark(el)
        }
        "code" => {
            let mut el = html::Code::default();
            set_global!(el, attrs);
            Element::Code(el)
        }
        "kbd" => {
            let mut el = html::Kbd::default();
            set_global!(el, attrs);
            Element::Kbd(el)
        }
        "samp" => {
            let mut el = html::Samp::default();
            set_global!(el, attrs);
            Element::Samp(el)
        }
        "var" => {
            let mut el = html::Var::default();
            set_global!(el, attrs);
            Element::Var(el)
        }
        "abbr" => {
            let mut el = html::Abbr::default();
            set_global!(el, attrs);
            Element::Abbr(el)
        }
        "cite" => {
            let mut el = html::Cite::default();
            set_global!(el, attrs);
            Element::Cite(el)
        }
        "dfn" => {
            let mut el = html::Dfn::default();
            set_global!(el, attrs);
            Element::Dfn(el)
        }
        "sub" => {
            let mut el = html::Sub::default();
            set_global!(el, attrs);
            Element::Sub(el)
        }
        "sup" => {
            let mut el = html::Sup::default();
            set_global!(el, attrs);
            Element::Sup(el)
        }
        "time" => {
            let mut el = html::Time::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "datetime" => el.datetime = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Time(el)
        }
        "ul" => {
            let mut el = html::Ul::default();
            set_global!(el, attrs);
            Element::Ul(el)
        }
        "ol" => {
            let mut el = html::Ol::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "reversed" => el.reversed = Some(parse_bool_attr(v)),
                    "start" => el.start = v.parse().ok(),
                    "type" => el.r#type = parse_ol_type(v),
                    _ => {}
                }
            }
            Element::Ol(el)
        }
        "li" => {
            let mut el = html::Li::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "value" => el.value = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Li(el)
        }
        "dl" => {
            let mut el = html::Dl::default();
            set_global!(el, attrs);
            Element::Dl(el)
        }
        "dt" => {
            let mut el = html::Dt::default();
            set_global!(el, attrs);
            Element::Dt(el)
        }
        "dd" => {
            let mut el = html::Dd::default();
            set_global!(el, attrs);
            Element::Dd(el)
        }
        "header" => {
            let mut el = html::Header::default();
            set_global!(el, attrs);
            Element::Header(el)
        }
        "nav" => {
            let mut el = html::Nav::default();
            set_global!(el, attrs);
            Element::Nav(el)
        }
        "main" => {
            let mut el = html::Main::default();
            set_global!(el, attrs);
            Element::Main(el)
        }
        "section" => {
            let mut el = html::Section::default();
            set_global!(el, attrs);
            Element::Section(el)
        }
        "article" => {
            let mut el = html::Article::default();
            set_global!(el, attrs);
            Element::Article(el)
        }
        "aside" => {
            let mut el = html::Aside::default();
            set_global!(el, attrs);
            Element::Aside(el)
        }
        "footer" => {
            let mut el = html::Footer::default();
            set_global!(el, attrs);
            Element::Footer(el)
        }
        "div" => {
            let mut el = html::Div::default();
            set_global!(el, attrs);
            Element::Div(el)
        }
        "img" => {
            let mut el = html::Img::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "alt" => el.alt = Some(v.clone()),
                    "width" => el.width = v.parse().ok(),
                    "height" => el.height = v.parse().ok(),
                    "srcset" => el.srcset = Some(v.clone()),
                    "sizes" => el.sizes = Some(v.clone()),
                    "loading" => el.loading = parse_loading(v),
                    "decoding" => el.decoding = parse_image_decoding(v),
                    "crossorigin" => el.crossorigin = parse_crossorigin(v),
                    "usemap" => el.usemap = Some(v.clone()),
                    "ismap" => el.ismap = Some(parse_bool_attr(v)),
                    "referrerpolicy" => el.referrerpolicy = parse_referrer_policy(v),
                    _ => {}
                }
            }
            Element::Img(el)
        }
        "picture" => {
            let mut el = html::Picture::default();
            set_global!(el, attrs);
            Element::Picture(el)
        }
        "source" => {
            let mut el = html::Source::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "srcset" => el.srcset = Some(v.clone()),
                    "sizes" => el.sizes = Some(v.clone()),
                    "media" => el.media = Some(v.clone()),
                    "type" => el.r#type = Some(v.clone()),
                    "width" => el.width = v.parse().ok(),
                    "height" => el.height = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Source(el)
        }
        "video" => {
            let mut el = html::Video::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "controls" => el.controls = Some(parse_bool_attr(v)),
                    "autoplay" => el.autoplay = Some(parse_bool_attr(v)),
                    "loop" => el.r#loop = Some(parse_bool_attr(v)),
                    "muted" => el.muted = Some(parse_bool_attr(v)),
                    "poster" => el.poster = Some(v.clone()),
                    "preload" => el.preload = parse_preload(v),
                    "width" => el.width = v.parse().ok(),
                    "height" => el.height = v.parse().ok(),
                    "playsinline" => el.playsinline = Some(parse_bool_attr(v)),
                    "crossorigin" => el.crossorigin = parse_crossorigin(v),
                    _ => {}
                }
            }
            Element::Video(el)
        }
        "audio" => {
            let mut el = html::Audio::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "controls" => el.controls = Some(parse_bool_attr(v)),
                    "autoplay" => el.autoplay = Some(parse_bool_attr(v)),
                    "loop" => el.r#loop = Some(parse_bool_attr(v)),
                    "muted" => el.muted = Some(parse_bool_attr(v)),
                    "preload" => el.preload = parse_preload(v),
                    "crossorigin" => el.crossorigin = parse_crossorigin(v),
                    _ => {}
                }
            }
            Element::Audio(el)
        }
        "track" => {
            let mut el = html::Track::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "kind" => el.kind = parse_track_kind(v),
                    "srclang" => el.srclang = Some(v.clone()),
                    "label" => el.label = Some(v.clone()),
                    "default" => el.default = Some(parse_bool_attr(v)),
                    _ => {}
                }
            }
            Element::Track(el)
        }
        "iframe" => {
            let mut el = html::Iframe::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "src" => el.src = Some(v.clone()),
                    "srcdoc" => el.srcdoc = Some(v.clone()),
                    "name" => el.name = Some(v.clone()),
                    "width" => el.width = v.parse().ok(),
                    "height" => el.height = v.parse().ok(),
                    "allow" => el.allow = Some(v.clone()),
                    "allowfullscreen" => el.allowfullscreen = Some(parse_bool_attr(v)),
                    "loading" => el.loading = parse_loading(v),
                    "referrerpolicy" => el.referrerpolicy = parse_referrer_policy(v),
                    "sandbox" => el.sandbox = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Iframe(el)
        }
        "canvas" => {
            let mut el = html::Canvas::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "width" => el.width = v.parse().ok(),
                    "height" => el.height = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Canvas(el)
        }
        "svg" => {
            let mut el = html::Svg::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "width" => el.width = parse_svg_length(v),
                    "height" => el.height = parse_svg_length(v),
                    "viewbox" => el.view_box = Some(v.clone()),
                    "xmlns" => el.xmlns = Some(v.clone()),
                    "fill" => el.fill = Some(v.clone()),
                    "stroke" => el.stroke = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Svg(el)
        }
        "table" => {
            let mut el = html::Table::default();
            set_global!(el, attrs);
            Element::Table(el)
        }
        "caption" => {
            let mut el = html::Caption::default();
            set_global!(el, attrs);
            Element::Caption(el)
        }
        "thead" => {
            let mut el = html::Thead::default();
            set_global!(el, attrs);
            Element::Thead(el)
        }
        "tbody" => {
            let mut el = html::Tbody::default();
            set_global!(el, attrs);
            Element::Tbody(el)
        }
        "tfoot" => {
            let mut el = html::Tfoot::default();
            set_global!(el, attrs);
            Element::Tfoot(el)
        }
        "tr" => {
            let mut el = html::Tr::default();
            set_global!(el, attrs);
            Element::Tr(el)
        }
        "th" => {
            let mut el = html::Th::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "colspan" => el.colspan = v.parse().ok(),
                    "rowspan" => el.rowspan = v.parse().ok(),
                    "headers" => el.headers = Some(v.clone()),
                    "scope" => el.scope = parse_table_header_scope(v),
                    "abbr" => el.abbr = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Th(el)
        }
        "td" => {
            let mut el = html::Td::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "colspan" => el.colspan = v.parse().ok(),
                    "rowspan" => el.rowspan = v.parse().ok(),
                    "headers" => el.headers = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Td(el)
        }
        "colgroup" => {
            let mut el = html::Colgroup::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "span" => el.span = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Colgroup(el)
        }
        "col" => {
            let mut el = html::Col::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "span" => el.span = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Col(el)
        }
        "form" => {
            let mut el = html::Form::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "action" => el.action = Some(v.clone()),
                    "method" => el.method = parse_form_method(v),
                    "enctype" => el.enctype = parse_form_encoding(v),
                    "target" => el.target = parse_link_target(v),
                    "autocomplete" => el.autocomplete = parse_autocomplete(v),
                    "novalidate" => el.novalidate = Some(parse_bool_attr(v)),
                    "name" => el.name = Some(v.clone()),
                    "rel" => el.rel = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Form(el)
        }
        "label" => {
            let mut el = html::Label::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "for" => el.r#for = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Label(el)
        }
        "input" => {
            let mut el = html::Input::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "type" => el.r#type = parse_input_type(v),
                    "name" => el.name = Some(v.clone()),
                    "value" => el.value = Some(v.clone()),
                    "placeholder" => el.placeholder = Some(v.clone()),
                    "required" => el.required = Some(parse_bool_attr(v)),
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    "readonly" => el.readonly = Some(parse_bool_attr(v)),
                    "checked" => el.checked = Some(parse_bool_attr(v)),
                    "min" => el.min = Some(v.clone()),
                    "max" => el.max = Some(v.clone()),
                    "step" => el.step = Some(v.clone()),
                    "minlength" => el.minlength = v.parse().ok(),
                    "maxlength" => el.maxlength = v.parse().ok(),
                    "pattern" => el.pattern = Some(v.clone()),
                    "autocomplete" => el.autocomplete = Some(v.clone()),
                    "autofocus" => el.autofocus = Some(parse_bool_attr(v)),
                    "multiple" => el.multiple = Some(parse_bool_attr(v)),
                    "accept" => el.accept = Some(v.clone()),
                    "capture" => el.capture = parse_capture_mode(v),
                    "size" => el.size = v.parse().ok(),
                    "list" => el.list = Some(v.clone()),
                    "form" => el.form = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Input(el)
        }
        "textarea" => {
            let mut el = html::Textarea::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "name" => el.name = Some(v.clone()),
                    "placeholder" => el.placeholder = Some(v.clone()),
                    "required" => el.required = Some(parse_bool_attr(v)),
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    "readonly" => el.readonly = Some(parse_bool_attr(v)),
                    "rows" => el.rows = v.parse().ok(),
                    "cols" => el.cols = v.parse().ok(),
                    "minlength" => el.minlength = v.parse().ok(),
                    "maxlength" => el.maxlength = v.parse().ok(),
                    "wrap" => el.wrap = parse_textarea_wrap(v),
                    "autocomplete" => el.autocomplete = Some(v.clone()),
                    "autofocus" => el.autofocus = Some(parse_bool_attr(v)),
                    "form" => el.form = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Textarea(el)
        }
        "button" => {
            let mut el = html::Button::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "type" => el.r#type = parse_button_type(v),
                    "name" => el.name = Some(v.clone()),
                    "value" => el.value = Some(v.clone()),
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    "autofocus" => el.autofocus = Some(parse_bool_attr(v)),
                    "form" => el.form = Some(v.clone()),
                    "formaction" => el.formaction = Some(v.clone()),
                    "formenctype" => el.formenctype = parse_form_encoding(v),
                    "formmethod" => el.formmethod = parse_form_method(v),
                    "formnovalidate" => el.formnovalidate = Some(parse_bool_attr(v)),
                    "formtarget" => el.formtarget = parse_link_target(v),
                    _ => {}
                }
            }
            Element::Button(el)
        }
        "select" => {
            let mut el = html::Select::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "name" => el.name = Some(v.clone()),
                    "required" => el.required = Some(parse_bool_attr(v)),
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    "multiple" => el.multiple = Some(parse_bool_attr(v)),
                    "size" => el.size = v.parse().ok(),
                    "autofocus" => el.autofocus = Some(parse_bool_attr(v)),
                    "form" => el.form = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Select(el)
        }
        "option" => {
            let mut el = html::OptionElement::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "value" => el.value = Some(v.clone()),
                    "label" => el.label = Some(v.clone()),
                    "selected" => el.selected = Some(parse_bool_attr(v)),
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    _ => {}
                }
            }
            Element::OptionElement(el)
        }
        "optgroup" => {
            let mut el = html::Optgroup::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "label" => el.label = Some(v.clone()),
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    _ => {}
                }
            }
            Element::Optgroup(el)
        }
        "fieldset" => {
            let mut el = html::Fieldset::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "disabled" => el.disabled = Some(parse_bool_attr(v)),
                    "form" => el.form = Some(v.clone()),
                    "name" => el.name = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Fieldset(el)
        }
        "legend" => {
            let mut el = html::Legend::default();
            set_global!(el, attrs);
            Element::Legend(el)
        }
        "datalist" => {
            let mut el = html::Datalist::default();
            set_global!(el, attrs);
            Element::Datalist(el)
        }
        "output" => {
            let mut el = html::Output::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "for" => el.r#for = Some(v.split_whitespace().map(String::from).collect()),
                    "form" => el.form = Some(v.clone()),
                    "name" => el.name = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Output(el)
        }
        "progress" => {
            let mut el = html::Progress::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "value" => el.value = v.parse().ok(),
                    "max" => el.max = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Progress(el)
        }
        "meter" => {
            let mut el = html::Meter::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "value" => el.value = v.parse().ok(),
                    "min" => el.min = v.parse().ok(),
                    "max" => el.max = v.parse().ok(),
                    "low" => el.low = v.parse().ok(),
                    "high" => el.high = v.parse().ok(),
                    "optimum" => el.optimum = v.parse().ok(),
                    _ => {}
                }
            }
            Element::Meter(el)
        }
        "details" => {
            let mut el = html::Details::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "open" => el.open = Some(parse_bool_attr(v)),
                    "name" => el.name = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Details(el)
        }
        "summary" => {
            let mut el = html::Summary::default();
            set_global!(el, attrs);
            Element::Summary(el)
        }
        "dialog" => {
            let mut el = html::Dialog::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "open" => el.open = Some(parse_bool_attr(v)),
                    _ => {}
                }
            }
            Element::Dialog(el)
        }
        "template" => {
            let mut el = html::Template::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "shadowrootmode" => el.shadowrootmode = parse_shadow_root_mode(v),
                    _ => {}
                }
            }
            Element::Template(el)
        }
        "slot" => {
            let mut el = html::Slot::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "name" => el.name = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Slot(el)
        }
        "del" => {
            let mut el = html::Del::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "cite" => el.cite = Some(v.clone()),
                    "datetime" => el.datetime = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Del(el)
        }
        "ins" => {
            let mut el = html::Ins::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "cite" => el.cite = Some(v.clone()),
                    "datetime" => el.datetime = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Ins(el)
        }
        "bdi" => {
            let mut el = html::Bdi::default();
            set_global!(el, attrs);
            Element::Bdi(el)
        }
        "bdo" => {
            let mut el = html::Bdo::default();
            set_global!(el, attrs);
            Element::Bdo(el)
        }
        "wbr" => {
            let mut el = html::Wbr::default();
            set_global!(el, attrs);
            Element::Wbr(el)
        }
        "data" => {
            let mut el = html::Data::default();
            set_global!(el, attrs);
            for (n, v) in attrs {
                match n.as_str() {
                    "value" => el.value = Some(v.clone()),
                    _ => {}
                }
            }
            Element::Data(el)
        }
        "ruby" => {
            let mut el = html::Ruby::default();
            set_global!(el, attrs);
            Element::Ruby(el)
        }
        "rt" => {
            let mut el = html::Rt::default();
            set_global!(el, attrs);
            Element::Rt(el)
        }
        "rp" => {
            let mut el = html::Rp::default();
            set_global!(el, attrs);
            Element::Rp(el)
        }
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Macro for setting global attributes on any element struct
// ---------------------------------------------------------------------------

macro_rules! set_global {
    ($el:expr, $attrs:expr) => {
        for (name, value) in $attrs {
            match name.as_str() {
                "id" => $el.id = Some(value.clone()),
                "class" => $el.class = Some(value.clone()),
                "style" => $el.style = Some(value.clone()),
                "title" => $el.title = Some(value.clone()),
                "lang" => $el.lang = Some(value.clone()),
                "dir" => $el.dir = parse_html_direction(value),
                "hidden" => $el.hidden = Some(true),
                "tabindex" => $el.tabindex = value.parse().ok(),
                "accesskey" => $el.accesskey = Some(value.clone()),
                "contenteditable" => $el.contenteditable = Some(parse_bool_attr(value)),
                "draggable" => $el.draggable = Some(parse_bool_attr(value)),
                "spellcheck" => $el.spellcheck = Some(parse_bool_attr(value)),
                "translate" => $el.translate = Some(parse_bool_attr(value)),
                "role" => $el.role = parse_aria_role(value),
                _ => {
                    if let Some(suffix) = name.strip_prefix("aria-") {
                        $el.aria_attrs.insert(suffix.to_string(), value.clone());
                    } else if let Some(suffix) = name.strip_prefix("data-") {
                        $el.data_attrs.insert(suffix.to_string(), value.clone());
                    }
                }
            }
        }
    };
}
use set_global;

// ---------------------------------------------------------------------------
// Enum parsers for HTML attribute values
// ---------------------------------------------------------------------------

fn parse_bool_attr(value: &str) -> bool {
    // Boolean attributes: presence means true. Empty string or attribute name itself = true.
    match value.to_ascii_lowercase().as_str() {
        "" | "true" | "yes" | "1" => true,
        "false" | "no" | "0" => false,
        // Attribute name as value (e.g. disabled="disabled") is true
        _ => true,
    }
}

fn parse_html_direction(value: &str) -> Option<HtmlDirection> {
    match value.to_ascii_lowercase().as_str() {
        "ltr" => Some(HtmlDirection::Ltr),
        "rtl" => Some(HtmlDirection::Rtl),
        "auto" => Some(HtmlDirection::Auto),
        _ => None,
    }
}

fn parse_aria_role(value: &str) -> Option<AriaRole> {
    match value.to_ascii_lowercase().as_str() {
        "button" => Some(AriaRole::Button),
        "checkbox" => Some(AriaRole::Checkbox),
        "dialog" => Some(AriaRole::Dialog),
        "link" => Some(AriaRole::Link),
        "listbox" => Some(AriaRole::Listbox),
        "menu" => Some(AriaRole::Menu),
        "menuitem" => Some(AriaRole::Menuitem),
        "navigation" => Some(AriaRole::Navigation),
        "option" => Some(AriaRole::Option),
        "progressbar" => Some(AriaRole::Progressbar),
        "radio" => Some(AriaRole::Radio),
        "search" => Some(AriaRole::Search),
        "slider" => Some(AriaRole::Slider),
        "status" => Some(AriaRole::Status),
        "tab" => Some(AriaRole::Tab),
        "tablist" => Some(AriaRole::Tablist),
        "textbox" => Some(AriaRole::Textbox),
        "tooltip" => Some(AriaRole::Tooltip),
        "tree" => Some(AriaRole::Tree),
        "treeitem" => Some(AriaRole::Treeitem),
        _ => None,
    }
}

fn parse_link_target(value: &str) -> Option<LinkTarget> {
    match value.to_ascii_lowercase().as_str() {
        "_blank" => Some(LinkTarget::Blank),
        "_self" => Some(LinkTarget::SelfTarget),
        "_parent" => Some(LinkTarget::Parent),
        "_top" => Some(LinkTarget::Top),
        "" => None,
        _ => Some(LinkTarget::Named(value.to_string())),
    }
}

fn parse_link_as(value: &str) -> Option<LinkAs> {
    match value.to_ascii_lowercase().as_str() {
        "audio" => Some(LinkAs::Audio),
        "document" => Some(LinkAs::Document),
        "embed" => Some(LinkAs::Embed),
        "fetch" => Some(LinkAs::Fetch),
        "font" => Some(LinkAs::Font),
        "image" => Some(LinkAs::Image),
        "object" => Some(LinkAs::Object),
        "script" => Some(LinkAs::Script),
        "style" => Some(LinkAs::Style),
        "track" => Some(LinkAs::Track),
        "video" => Some(LinkAs::Video),
        "worker" => Some(LinkAs::Worker),
        _ => None,
    }
}

fn parse_crossorigin(value: &str) -> Option<CrossOrigin> {
    match value.to_ascii_lowercase().as_str() {
        "anonymous" | "" => Some(CrossOrigin::Anonymous),
        "use-credentials" => Some(CrossOrigin::UseCredentials),
        _ => None,
    }
}

fn parse_referrer_policy(value: &str) -> Option<ReferrerPolicy> {
    match value.to_ascii_lowercase().as_str() {
        "no-referrer" => Some(ReferrerPolicy::NoReferrer),
        "no-referrer-when-downgrade" => Some(ReferrerPolicy::NoReferrerWhenDowngrade),
        "origin" => Some(ReferrerPolicy::Origin),
        "origin-when-cross-origin" => Some(ReferrerPolicy::OriginWhenCrossOrigin),
        "same-origin" => Some(ReferrerPolicy::SameOrigin),
        "strict-origin" => Some(ReferrerPolicy::StrictOrigin),
        "strict-origin-when-cross-origin" => Some(ReferrerPolicy::StrictOriginWhenCrossOrigin),
        "unsafe-url" => Some(ReferrerPolicy::UnsafeUrl),
        _ => None,
    }
}

fn parse_ol_type(value: &str) -> Option<OlType> {
    match value {
        "1" => Some(OlType::Decimal),
        "a" => Some(OlType::LowerAlpha),
        "A" => Some(OlType::UpperAlpha),
        "i" => Some(OlType::LowerRoman),
        "I" => Some(OlType::UpperRoman),
        _ => None,
    }
}

fn parse_loading(value: &str) -> Option<Loading> {
    match value.to_ascii_lowercase().as_str() {
        "eager" => Some(Loading::Eager),
        "lazy" => Some(Loading::Lazy),
        _ => None,
    }
}

fn parse_image_decoding(value: &str) -> Option<ImageDecoding> {
    match value.to_ascii_lowercase().as_str() {
        "sync" => Some(ImageDecoding::Sync),
        "async" => Some(ImageDecoding::Async),
        "auto" => Some(ImageDecoding::Auto),
        _ => None,
    }
}

fn parse_preload(value: &str) -> Option<Preload> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(Preload::None),
        "metadata" => Some(Preload::Metadata),
        "auto" | "" => Some(Preload::Auto),
        _ => None,
    }
}

fn parse_track_kind(value: &str) -> Option<TrackKind> {
    match value.to_ascii_lowercase().as_str() {
        "subtitles" => Some(TrackKind::Subtitles),
        "captions" => Some(TrackKind::Captions),
        "descriptions" => Some(TrackKind::Descriptions),
        "chapters" => Some(TrackKind::Chapters),
        "metadata" => Some(TrackKind::Metadata),
        _ => None,
    }
}

fn parse_svg_length(value: &str) -> Option<SvgLength> {
    let v = value.trim();
    if v == "auto" {
        return Some(SvgLength::Auto);
    }
    if let Some(s) = v.strip_suffix("px") {
        return s.trim().parse::<f32>().ok().map(SvgLength::Px);
    }
    if let Some(s) = v.strip_suffix('%') {
        return s.trim().parse::<f32>().ok().map(SvgLength::Percent);
    }
    if let Some(s) = v.strip_suffix("em") {
        return s.trim().parse::<f32>().ok().map(SvgLength::Em);
    }
    if let Some(s) = v.strip_suffix("rem") {
        return s.trim().parse::<f32>().ok().map(SvgLength::Rem);
    }
    // Try as raw number (pixels implied)
    if let Ok(n) = v.parse::<f32>() {
        return Some(SvgLength::Px(n));
    }
    Some(SvgLength::Raw(value.to_string()))
}

fn parse_table_header_scope(value: &str) -> Option<TableHeaderScope> {
    match value.to_ascii_lowercase().as_str() {
        "row" => Some(TableHeaderScope::Row),
        "col" => Some(TableHeaderScope::Col),
        "rowgroup" => Some(TableHeaderScope::RowGroup),
        "colgroup" => Some(TableHeaderScope::ColGroup),
        "auto" => Some(TableHeaderScope::Auto),
        _ => None,
    }
}

fn parse_form_method(value: &str) -> Option<FormMethod> {
    match value.to_ascii_lowercase().as_str() {
        "get" => Some(FormMethod::Get),
        "post" => Some(FormMethod::Post),
        "dialog" => Some(FormMethod::Dialog),
        _ => None,
    }
}

fn parse_form_encoding(value: &str) -> Option<FormEncoding> {
    match value.to_ascii_lowercase().as_str() {
        "application/x-www-form-urlencoded" => Some(FormEncoding::UrlEncoded),
        "multipart/form-data" => Some(FormEncoding::MultipartFormData),
        "text/plain" => Some(FormEncoding::TextPlain),
        _ => None,
    }
}

fn parse_autocomplete(value: &str) -> Option<AutoComplete> {
    match value.to_ascii_lowercase().as_str() {
        "on" => Some(AutoComplete::On),
        "off" => Some(AutoComplete::Off),
        _ => None,
    }
}

fn parse_input_type(value: &str) -> Option<InputType> {
    match value.to_ascii_lowercase().as_str() {
        "button" => Some(InputType::Button),
        "checkbox" => Some(InputType::Checkbox),
        "color" => Some(InputType::Color),
        "date" => Some(InputType::Date),
        "datetime-local" => Some(InputType::DatetimeLocal),
        "email" => Some(InputType::Email),
        "file" => Some(InputType::File),
        "hidden" => Some(InputType::Hidden),
        "image" => Some(InputType::Image),
        "month" => Some(InputType::Month),
        "number" => Some(InputType::Number),
        "password" => Some(InputType::Password),
        "radio" => Some(InputType::Radio),
        "range" => Some(InputType::Range),
        "reset" => Some(InputType::Reset),
        "search" => Some(InputType::Search),
        "submit" => Some(InputType::Submit),
        "tel" => Some(InputType::Tel),
        "text" => Some(InputType::Text),
        "time" => Some(InputType::Time),
        "url" => Some(InputType::Url),
        "week" => Some(InputType::Week),
        _ => None,
    }
}

fn parse_capture_mode(value: &str) -> Option<CaptureMode> {
    match value.to_ascii_lowercase().as_str() {
        "user" => Some(CaptureMode::User),
        "environment" => Some(CaptureMode::Environment),
        _ => None,
    }
}

fn parse_textarea_wrap(value: &str) -> Option<TextareaWrap> {
    match value.to_ascii_lowercase().as_str() {
        "hard" => Some(TextareaWrap::Hard),
        "soft" => Some(TextareaWrap::Soft),
        "off" => Some(TextareaWrap::Off),
        _ => None,
    }
}

fn parse_button_type(value: &str) -> Option<ButtonType> {
    match value.to_ascii_lowercase().as_str() {
        "button" => Some(ButtonType::Button),
        "submit" => Some(ButtonType::Submit),
        "reset" => Some(ButtonType::Reset),
        _ => None,
    }
}

fn parse_shadow_root_mode(value: &str) -> Option<ShadowRootMode> {
    match value.to_ascii_lowercase().as_str() {
        "open" => Some(ShadowRootMode::Open),
        "closed" => Some(ShadowRootMode::Closed),
        _ => None,
    }
}
