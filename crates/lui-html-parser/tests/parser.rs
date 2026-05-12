use lui_html_parser::{ArcStr, HtmlElement, HtmlNode, parse};

// ── Helpers ────────────────────────────────────────────────────────────

#[track_caller]
fn text_of(node: &HtmlNode) -> &str {
    match &node.element {
        HtmlElement::Text(s) => s,
        other => panic!("expected HtmlElement::Text, got {other:?}"),
    }
}

// ── From tree_builder.rs ───────────────────────────────────────────────

#[test]
fn simple_tree() {
    let doc = parse("<div><p>hello</p></div>");
    assert_eq!(doc.roots.len(), 1);
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    assert_eq!(div.children.len(), 1);
    let p = &div.children[0];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(p.children.len(), 1);
    assert!(matches!(p.children[0].element, HtmlElement::Text(_)));
}

#[test]
fn void_elements_are_leaves() {
    let doc = parse("<div><br><hr><img></div>");
    let div = &doc.roots[0];
    assert_eq!(div.children.len(), 3);
    assert_eq!(div.children[0].element, HtmlElement::Br);
    assert_eq!(div.children[1].element, HtmlElement::Hr);
    assert_eq!(div.children[2].element, HtmlElement::Img);
    for c in &div.children {
        assert!(c.children.is_empty());
    }
}

#[test]
fn auto_close_p_before_p() {
    let doc = parse("<p>one<p>two");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::P);
    assert_eq!(doc.roots[0].children.len(), 1);
    assert_eq!(text_of(&doc.roots[0].children[0]), "one");
    assert_eq!(doc.roots[1].element, HtmlElement::P);
    assert_eq!(doc.roots[1].children.len(), 1);
    assert_eq!(text_of(&doc.roots[1].children[0]), "two");
}

#[test]
fn unknown_tag_preserved() {
    let doc = parse("<div><frobnicate>x</frobnicate><p>y</p></div>");
    assert_eq!(doc.roots.len(), 1);
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    assert_eq!(div.children.len(), 2);
    assert_eq!(div.children[0].element, HtmlElement::Unknown("frobnicate".into()));
    assert_eq!(div.children[0].children.len(), 1);
    assert_eq!(text_of(&div.children[0].children[0]), "x");
    assert_eq!(div.children[1].element, HtmlElement::P);
    assert_eq!(text_of(&div.children[1].children[0]), "y");
}

#[test]
fn template_contents_are_retained() {
    let doc = parse(
        "<template id=\"tpl\"><div>hidden</div></template><p>shown</p>",
    );
    assert_eq!(doc.roots.len(), 2);
    let template = &doc.roots[0];
    assert_eq!(template.element, HtmlElement::Template);
    assert_eq!(template.attrs.get("id").map(|s| &**s), Some("tpl"));
    assert_eq!(template.children.len(), 1);
    assert_eq!(template.children[0].element, HtmlElement::Div);
    assert_eq!(text_of(&template.children[0].children[0]), "hidden");

    let p = &doc.roots[1];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(text_of(&p.children[0]), "shown");
}

#[test]
fn comments_and_doctype_dropped() {
    let doc = parse("<!DOCTYPE html><!--c--><p>hi</p>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::Comment(ArcStr::from("c")));
    let p = &doc.roots[1];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(p.children.len(), 1);
    assert!(matches!(p.children[0].element, HtmlElement::Text(_)));
}

#[test]
fn consecutive_body_elements() {
    let doc = parse("<body><p>a</p></body><body><p>b</p></body>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::Body);
    assert_eq!(doc.roots[0].children.len(), 1);
    assert_eq!(doc.roots[0].children[0].element, HtmlElement::P);
    assert_eq!(doc.roots[1].element, HtmlElement::Body);
    assert_eq!(doc.roots[1].children.len(), 1);
    assert_eq!(doc.roots[1].children[0].element, HtmlElement::P);
}

#[test]
fn style_and_body_are_siblings() {
    let doc = parse("<style>h1{color:red}</style><body><p>hi</p></body>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::Style);
    assert_eq!(doc.roots[0].children.len(), 1);
    assert!(matches!(doc.roots[0].children[0].element, HtmlElement::Text(_)));
    assert_eq!(doc.roots[1].element, HtmlElement::Body);
    assert_eq!(doc.roots[1].children[0].element, HtmlElement::P);
}

#[test]
fn consecutive_html_elements() {
    let doc = parse("<html><body><p>ok</p></body></html><html></html>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::Html);
    assert_eq!(doc.roots[0].children.len(), 1);
    assert_eq!(doc.roots[0].children[0].element, HtmlElement::Body);
    assert_eq!(doc.roots[1].element, HtmlElement::Html);
    assert!(doc.roots[1].children.is_empty());
}

// ── From html/tree.rs ──────────────────────────────────────────────────

#[test]
fn empty_input_has_no_root() {
    let doc = parse("");
    assert!(doc.roots.is_empty());
}

#[test]
fn whitespace_only_input_has_no_root() {
    let doc = parse("   \n\t  ");
    assert!(doc.roots.is_empty());
}

#[test]
fn pure_text_input_becomes_text_root() {
    let doc = parse("hello");
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(text_of(&doc.roots[0]), "hello");
    assert!(doc.roots[0].children.is_empty());
}

#[test]
fn doctype_only_has_no_root() {
    let doc = parse("<!DOCTYPE html>");
    assert!(doc.roots.is_empty());
}

#[test]
fn comment_only_has_no_root() {
    let doc = parse("<!-- nothing here -->");
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(doc.roots[0].element, HtmlElement::Comment(ArcStr::from(" nothing here ")));
}

#[test]
fn paragraph_with_text() {
    let doc = parse("<p>hello</p>");
    assert_eq!(doc.roots.len(), 1);
    let p = &doc.roots[0];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(p.children.len(), 1);
    assert_eq!(text_of(&p.children[0]), "hello");
}

#[test]
fn span_with_text() {
    let doc = parse("<span>x</span>");
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(doc.roots[0].element, HtmlElement::Span);
    assert_eq!(doc.roots[0].children.len(), 1);
}

#[test]
fn div_p_text_nesting() {
    let doc = parse("<div><p>hi</p></div>");
    assert_eq!(doc.roots.len(), 1);
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    let p = &div.children[0];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(text_of(&p.children[0]), "hi");
}

#[test]
fn three_level_nesting() {
    let doc = parse("<section><article><p>x</p></article></section>");
    assert_eq!(doc.roots.len(), 1);
    let r = &doc.roots[0];
    let a = &r.children[0];
    let p = &a.children[0];
    assert_eq!(r.element, HtmlElement::Section);
    assert_eq!(a.element, HtmlElement::Article);
    assert_eq!(p.element, HtmlElement::P);
}

#[test]
fn mixed_inline_in_paragraph() {
    let doc = parse("<p>hello <strong>bold</strong> and <em>italic</em>!</p>");
    assert_eq!(doc.roots.len(), 1);
    let p = &doc.roots[0];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(p.children.len(), 5);
    assert_eq!(text_of(&p.children[0]), "hello ");
    assert_eq!(p.children[1].element, HtmlElement::Strong);
    assert_eq!(text_of(&p.children[2]), " and ");
    assert_eq!(p.children[3].element, HtmlElement::Em);
    assert_eq!(text_of(&p.children[4]), "!");
}

#[test]
fn deep_inline_nesting() {
    let doc = parse("<p>a<b>b<i>i<u>u</u></i></b></p>");
    assert_eq!(doc.roots.len(), 1);
    let p = &doc.roots[0];
    assert_eq!(p.element, HtmlElement::P);
    let b = &p.children[1];
    let i = &b.children[1];
    let u = &i.children[1];
    assert_eq!(b.element, HtmlElement::B);
    assert_eq!(i.element, HtmlElement::I);
    assert_eq!(u.element, HtmlElement::U);
    assert_eq!(text_of(&u.children[0]), "u");
}

#[test]
fn full_document_skeleton() {
    let html = "<!DOCTYPE html>\
        <html>\
            <head><title>T</title></head>\
            <body><h1>Hi</h1><p>p</p></body>\
        </html>";
    let doc = parse(html);
    assert_eq!(doc.roots.len(), 1);
    let r = &doc.roots[0];
    assert_eq!(r.element, HtmlElement::Html);
    assert_eq!(r.children.len(), 2);
    let head = &r.children[0];
    let body = &r.children[1];
    assert_eq!(head.element, HtmlElement::Head);
    assert_eq!(body.element, HtmlElement::Body);
    let title = &head.children[0];
    assert_eq!(title.element, HtmlElement::Title);
    assert_eq!(text_of(&title.children[0]), "T");
    assert_eq!(body.children[0].element, HtmlElement::H1);
    assert_eq!(body.children[1].element, HtmlElement::P);
}

#[test]
fn multiple_top_level_are_roots() {
    let doc = parse("<p>one</p><p>two</p>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::P);
    assert_eq!(doc.roots[1].element, HtmlElement::P);
}

#[test]
fn single_top_level_is_root() {
    let doc = parse("<p>only</p>");
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(doc.roots[0].element, HtmlElement::P);
}

#[test]
fn div_has_id_class() {
    let doc = parse(r#"<div id="hero" class="card big"></div>"#);
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    assert_eq!(div.attrs.get("id").map(|s| &**s), Some("hero"));
    assert_eq!(div.attrs.get("class").map(|s| &**s), Some("card big"));
}

#[test]
fn p_has_inline_style_string() {
    let doc = parse(r#"<p style="color: red; padding: 8px;">x</p>"#);
    let p = &doc.roots[0];
    assert_eq!(p.element, HtmlElement::P);
    assert_eq!(p.attrs.get("style").map(|s| &**s), Some("color: red; padding: 8px;"));
}

#[test]
fn anchor_href() {
    let doc = parse(r#"<a href="https://example.com">link</a>"#);
    let a = &doc.roots[0];
    assert_eq!(a.element, HtmlElement::A);
    assert_eq!(a.attrs.get("href").map(|s| &**s), Some("https://example.com"));
}

#[test]
fn img_src_alt() {
    let doc = parse(r#"<img src="/x.png" alt="x">"#);
    let img = &doc.roots[0];
    assert_eq!(img.element, HtmlElement::Img);
    assert_eq!(img.attrs.get("src").map(|s| &**s), Some("/x.png"));
    assert_eq!(img.attrs.get("alt").map(|s| &**s), Some("x"));
}

#[test]
fn input_type_value() {
    let doc = parse(r#"<input type="email" value="a@b.com">"#);
    let inp = &doc.roots[0];
    assert_eq!(inp.element, HtmlElement::Input);
    assert_eq!(inp.attrs.get("value").map(|s| &**s), Some("a@b.com"));
    assert!(inp.attrs.contains_key("type"));
}

#[test]
fn unordered_list_with_items() {
    let doc = parse("<ul><li>a</li><li>b</li><li>c</li></ul>");
    assert_eq!(doc.roots.len(), 1);
    let ul = &doc.roots[0];
    assert_eq!(ul.element, HtmlElement::Ul);
    assert_eq!(ul.children.len(), 3);
    for li in &ul.children {
        assert_eq!(li.element, HtmlElement::Li);
    }
}

#[test]
fn definition_list() {
    let doc = parse("<dl><dt>k</dt><dd>v</dd></dl>");
    let dl = &doc.roots[0];
    assert_eq!(dl.element, HtmlElement::Dl);
    assert_eq!(dl.children[0].element, HtmlElement::Dt);
    assert_eq!(dl.children[1].element, HtmlElement::Dd);
}

#[test]
fn table_with_thead_tbody() {
    let doc = parse(
        "<table>\
           <thead><tr><th>h1</th><th>h2</th></tr></thead>\
           <tbody><tr><td>a</td><td>b</td></tr></tbody>\
         </table>",
    );
    let table = &doc.roots[0];
    assert_eq!(table.element, HtmlElement::Table);
    let thead = &table.children[0];
    let tbody = &table.children[1];
    assert_eq!(thead.element, HtmlElement::Thead);
    assert_eq!(tbody.element, HtmlElement::Tbody);
    let head_row = &thead.children[0];
    assert_eq!(head_row.element, HtmlElement::Tr);
    assert_eq!(head_row.children.len(), 2);
    assert_eq!(head_row.children[0].element, HtmlElement::Th);
    let body_row = &tbody.children[0];
    assert_eq!(body_row.children.len(), 2);
    assert_eq!(body_row.children[0].element, HtmlElement::Td);
}

#[test]
fn form_with_inputs_and_button() {
    let doc = parse(
        r#"<form>
            <label for="name">Name</label>
            <input type="text" name="name">
            <button type="submit">Send</button>
        </form>"#,
    );
    let form = &doc.roots[0];
    assert_eq!(form.element, HtmlElement::Form);
    assert!(form.children.iter().any(|c| c.element == HtmlElement::Label));
    assert!(form.children.iter().any(|c| c.element == HtmlElement::Input));
    assert!(form.children.iter().any(|c| c.element == HtmlElement::Button));
}

#[test]
fn select_with_options() {
    let doc = parse("<select><option>a</option><option>b</option><optgroup></optgroup></select>");
    let select = &doc.roots[0];
    assert_eq!(select.element, HtmlElement::Select);
    assert_eq!(select.children[0].element, HtmlElement::OptionElement);
    assert_eq!(select.children[1].element, HtmlElement::OptionElement);
    assert_eq!(select.children[2].element, HtmlElement::Optgroup);
}

#[test]
fn p_auto_closes_when_div_opens() {
    let doc = parse("<p>a<div>b</div>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::P);
    assert_eq!(doc.roots[0].children.len(), 1);
    assert_eq!(text_of(&doc.roots[0].children[0]), "a");
    assert_eq!(doc.roots[1].element, HtmlElement::Div);
    assert_eq!(text_of(&doc.roots[1].children[0]), "b");
}

#[test]
fn li_auto_closes_when_next_li_opens() {
    let doc = parse("<ul><li>a<li>b<li>c</ul>");
    let ul = &doc.roots[0];
    assert_eq!(ul.element, HtmlElement::Ul);
    assert_eq!(ul.children.len(), 3);
}

#[test]
fn dt_auto_closes_dd() {
    let doc = parse("<dl><dt>k1<dd>v1<dt>k2<dd>v2</dl>");
    let dl = &doc.roots[0];
    assert_eq!(dl.element, HtmlElement::Dl);
    assert_eq!(dl.children.len(), 4);
    assert_eq!(dl.children[0].element, HtmlElement::Dt);
    assert_eq!(dl.children[1].element, HtmlElement::Dd);
    assert_eq!(dl.children[2].element, HtmlElement::Dt);
    assert_eq!(dl.children[3].element, HtmlElement::Dd);
}

#[test]
fn tr_auto_closes_tr() {
    let doc = parse("<table><tbody><tr><td>a</td><tr><td>b</td></tbody></table>");
    let table = &doc.roots[0];
    let tbody = &table.children[0];
    assert_eq!(tbody.children.len(), 2);
    for tr in &tbody.children {
        assert_eq!(tr.element, HtmlElement::Tr);
    }
}

#[test]
fn self_closing_syntax_works_for_known_tags() {
    let doc = parse("<div><br /><hr /></div>");
    let div = &doc.roots[0];
    assert_eq!(div.children.len(), 2);
}

#[test]
fn unclosed_tags_at_eof_are_closed_implicitly() {
    let doc = parse("<div><span>hello");
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    let span = &div.children[0];
    assert_eq!(span.element, HtmlElement::Span);
    assert_eq!(text_of(&span.children[0]), "hello");
}

#[test]
fn unknown_tag_subtree_is_preserved() {
    let doc = parse("<div>before<frob>nested<p>inside</p></frob>after</div>");
    assert_eq!(doc.roots.len(), 1);
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    assert_eq!(div.children.len(), 3);
    assert_eq!(text_of(&div.children[0]), "before");

    let frob = &div.children[1];
    assert_eq!(frob.element, HtmlElement::Unknown("frob".into()));
    assert_eq!(frob.children.len(), 2);
    assert_eq!(text_of(&frob.children[0]), "nested");
    assert_eq!(frob.children[1].element, HtmlElement::P);
    assert_eq!(text_of(&frob.children[1].children[0]), "inside");

    assert_eq!(text_of(&div.children[2]), "after");
}

#[test]
fn doctype_and_comments_are_stripped() {
    let doc = parse("<!DOCTYPE html><!--top--><div><!--inside-->ok<!--end--></div>");
    // Comment before div is preserved; doctype is dropped
    assert_eq!(doc.roots[0].element, HtmlElement::Comment(ArcStr::from("top")));
    let div = &doc.roots[1];
    assert_eq!(div.element, HtmlElement::Div);
    assert_eq!(div.children.len(), 3); // comment, text, comment
    assert_eq!(div.children[0].element, HtmlElement::Comment(ArcStr::from("inside")));
    assert_eq!(text_of(&div.children[1]), "ok");
    assert_eq!(div.children[2].element, HtmlElement::Comment(ArcStr::from("end")));
}

// Note: `parse_inline_style` tests from the old `html/tree.rs` are not migrated because
// `lui-html-parser` does not contain inline style parsing — that lives in `lui-style`.

#[test]
fn multiple_data_attrs_are_all_preserved() {
    let doc = parse(r#"<div style="display: block" data-id="42" data-name="hello" data-foo="bar"></div>"#);
    let div = &doc.roots[0];
    assert_eq!(div.element, HtmlElement::Div);
    assert_eq!(div.attrs.get("data-id").map(|s| &**s), Some("42"));
    assert_eq!(div.attrs.get("data-name").map(|s| &**s), Some("hello"));
    assert_eq!(div.attrs.get("data-foo").map(|s| &**s), Some("bar"));
}

#[test]
fn multiple_aria_attrs_are_all_preserved() {
    let doc = parse(r#"<button aria-label="Close" aria-pressed="false" aria-controls="menu"></button>"#);
    let btn = &doc.roots[0];
    assert_eq!(btn.element, HtmlElement::Button);
    assert_eq!(btn.attrs.get("aria-label").map(|s| &**s), Some("Close"));
    assert_eq!(btn.attrs.get("aria-pressed").map(|s| &**s), Some("false"));
    assert_eq!(btn.attrs.get("aria-controls").map(|s| &**s), Some("menu"));
}

#[test]
fn whitespace_text_dropped() {
    let doc = parse("<div>   </div>");
    assert_eq!(doc.roots.len(), 1);
    assert!(doc.roots[0].children.is_empty());
}

#[test]
fn comment_preserved_as_root_with_div() {
    let doc = parse("<!-- hello --><div></div>");
    assert_eq!(doc.roots.len(), 2);
    assert_eq!(doc.roots[0].element, HtmlElement::Comment(" hello ".into()));
    assert_eq!(doc.roots[1].element, HtmlElement::Div);
}

#[test]
fn custom_element_with_dash() {
    let doc = parse(r#"<my-widget foo="bar">content</my-widget>"#);
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(doc.roots[0].element, HtmlElement::Unknown("my-widget".into()));
    assert_eq!(doc.roots[0].attrs.get("foo").map(|s| &**s), Some("bar"));
    assert_eq!(doc.roots[0].children.len(), 1);
    assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("content".into()));
}

#[test]
fn raw_text_script_parses_content_as_text() {
    let doc = parse("<script>var x = '<div>';</script>");
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(doc.roots[0].element, HtmlElement::Script);
    assert_eq!(doc.roots[0].children.len(), 1);
    assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("var x = '<div>';".into()));
}

#[test]
fn raw_text_style_parses_content_as_text() {
    let doc = parse("<style>div { color: red; }</style>");
    assert_eq!(doc.roots.len(), 1);
    assert_eq!(doc.roots[0].element, HtmlElement::Style);
    assert_eq!(doc.roots[0].children[0].element, HtmlElement::Text("div { color: red; }".into()));
}

#[test]
fn all_known_elements_parse() {
    for tag in &["html", "head", "body", "title", "base", "link", "meta", "style",
        "article", "section", "nav", "aside", "h1", "h2", "h3", "h4", "h5", "h6",
        "hgroup", "header", "footer", "address",
        "p", "hr", "pre", "blockquote", "ol", "ul", "menu", "li", "dl", "dt", "dd",
        "figure", "figcaption", "main", "search", "div",
        "a", "em", "strong", "small", "s", "cite", "q", "dfn", "abbr",
        "ruby", "rt", "rp", "data", "time", "code", "var", "samp", "kbd",
        "sub", "sup", "i", "b", "u", "mark", "bdi", "bdo", "span", "wbr",
        "ins", "del",
        "picture", "source", "img", "iframe", "embed", "object", "video", "audio", "track",
        "map", "area",
        "table", "caption", "colgroup", "col", "tbody", "thead", "tfoot", "tr", "td", "th",
        "form", "label", "input", "button", "select", "datalist", "optgroup", "option",
        "textarea", "output", "progress", "meter", "fieldset", "legend", "selectedcontent",
        "details", "summary", "dialog",
        "noscript", "template", "slot", "canvas",
        // Obsolete
        "marquee", "blink", "font", "center", "big", "small", "strike", "tt",
        "applet", "acronym", "bgsound", "dir", "frame", "frameset", "noframes",
        "isindex", "keygen", "listing", "menuitem", "nextid", "noembed", "param",
        "plaintext", "rb", "rtc", "xmp", "basefont", "multicol", "nobr", "spacer",
    ] {
        let html = format!("<{}></{}>", tag, tag);
        let doc = parse(&html);
        if doc.roots.is_empty() && HtmlElement::from_name(tag).is_void() {
            // Void elements have no children so <tag></tag> is still ok
            let doc2 = parse(&format!("<{}>", tag));
            assert!(!doc2.roots.is_empty(), "void element <{}> should parse", tag);
        } else {
            assert!(!doc.roots.is_empty(), "<{}> should parse", tag);
        }
    }
}
