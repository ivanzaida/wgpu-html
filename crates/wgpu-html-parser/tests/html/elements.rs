use wgpu_html_parser::parse;
use wgpu_html_tree::Element;

#[track_caller]
fn assert_root(html: &str, ok: impl FnOnce(&Element) -> bool) {
  let tree = parse(html);
  let root = tree.root.as_ref().expect("expected a root node");
  assert!(
    ok(&root.element),
    "unexpected root element for `{html}`: {:?}",
    root.element
  );
}

macro_rules! tag_tests {
    ( normal: $( ($name:ident, $tag:literal, $variant:ident) ),* $(,)? ) => {
        $(
            #[test]
            fn $name() {
                let html = concat!("<", $tag, "></", $tag, ">");
                assert_root(html, |el| matches!(el, Element::$variant(_)));
            }
        )*
    };
    ( void: $( ($name:ident, $tag:literal, $variant:ident) ),* $(,)? ) => {
        $(
            #[test]
            fn $name() {
                let html = concat!("<", $tag, ">");
                assert_root(html, |el| matches!(el, Element::$variant(_)));
            }
        )*
    };
}

tag_tests!(normal:
    (parses_html, "html", Html),
    (parses_head, "head", Head),
    (parses_body, "body", Body),
    (parses_title, "title", Title),
    (parses_style, "style", StyleElement),
    (parses_script, "script", Script),
    (parses_noscript, "noscript", Noscript),
);

tag_tests!(normal:
    (parses_h1, "h1", H1),
    (parses_h2, "h2", H2),
    (parses_h3, "h3", H3),
    (parses_h4, "h4", H4),
    (parses_h5, "h5", H5),
    (parses_h6, "h6", H6),
    (parses_p, "p", P),
    (parses_pre, "pre", Pre),
    (parses_blockquote, "blockquote", Blockquote),
    (parses_address, "address", Address),
);

tag_tests!(normal:
    (parses_span, "span", Span),
    (parses_a, "a", A),
    (parses_strong, "strong", Strong),
    (parses_b, "b", B),
    (parses_em, "em", Em),
    (parses_i, "i", I),
    (parses_u, "u", U),
    (parses_s, "s", S),
    (parses_small, "small", Small),
    (parses_mark, "mark", Mark),
    (parses_code, "code", Code),
    (parses_kbd, "kbd", Kbd),
    (parses_samp, "samp", Samp),
    (parses_var, "var", Var),
    (parses_abbr, "abbr", Abbr),
    (parses_cite, "cite", Cite),
    (parses_dfn, "dfn", Dfn),
    (parses_sub, "sub", Sub),
    (parses_sup, "sup", Sup),
    (parses_time, "time", Time),
    (parses_del, "del", Del),
    (parses_ins, "ins", Ins),
    (parses_bdi, "bdi", Bdi),
    (parses_bdo, "bdo", Bdo),
    (parses_data, "data", Data),
    (parses_ruby, "ruby", Ruby),
    (parses_rt, "rt", Rt),
    (parses_rp, "rp", Rp),
);

tag_tests!(normal:
    (parses_ul, "ul", Ul),
    (parses_ol, "ol", Ol),
    (parses_li, "li", Li),
    (parses_dl, "dl", Dl),
    (parses_dt, "dt", Dt),
    (parses_dd, "dd", Dd),
);

tag_tests!(normal:
    (parses_header, "header", Header),
    (parses_nav, "nav", Nav),
    (parses_main, "main", Main),
    (parses_section, "section", Section),
    (parses_article, "article", Article),
    (parses_aside, "aside", Aside),
    (parses_footer, "footer", Footer),
    (parses_div, "div", Div),
);

tag_tests!(normal:
    (parses_picture, "picture", Picture),
    (parses_video, "video", Video),
    (parses_audio, "audio", Audio),
    (parses_iframe, "iframe", Iframe),
    (parses_canvas, "canvas", Canvas),
    (parses_svg, "svg", Svg),
);

tag_tests!(normal:
    (parses_table, "table", Table),
    (parses_caption, "caption", Caption),
    (parses_thead, "thead", Thead),
    (parses_tbody, "tbody", Tbody),
    (parses_tfoot, "tfoot", Tfoot),
    (parses_tr, "tr", Tr),
    (parses_th, "th", Th),
    (parses_td, "td", Td),
    (parses_colgroup, "colgroup", Colgroup),
);

tag_tests!(normal:
    (parses_form, "form", Form),
    (parses_label, "label", Label),
    (parses_textarea, "textarea", Textarea),
    (parses_button, "button", Button),
    (parses_select, "select", Select),
    (parses_option, "option", OptionElement),
    (parses_optgroup, "optgroup", Optgroup),
    (parses_fieldset, "fieldset", Fieldset),
    (parses_legend, "legend", Legend),
    (parses_datalist, "datalist", Datalist),
    (parses_output, "output", Output),
    (parses_progress, "progress", Progress),
    (parses_meter, "meter", Meter),
);

tag_tests!(normal:
    (parses_details, "details", Details),
    (parses_summary, "summary", Summary),
    (parses_dialog, "dialog", Dialog),
    (parses_template, "template", Template),
    (parses_slot, "slot", Slot),
);

tag_tests!(void:
    (parses_meta, "meta", Meta),
    (parses_link, "link", Link),
    (parses_br, "br", Br),
    (parses_hr, "hr", Hr),
    (parses_img, "img", Img),
    (parses_source, "source", Source),
    (parses_track, "track", Track),
    (parses_col, "col", Col),
    (parses_input, "input", Input),
    (parses_wbr, "wbr", Wbr),
);
