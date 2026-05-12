use lui_html_parser::{HtmlElement, parse};

/// Helper: parse `html`, expect exactly one root, and run the predicate on its element.
#[track_caller]
fn assert_root_element(html: &str, expected: HtmlElement) {
    let doc = parse(html);
    assert_eq!(
        doc.roots.len(),
        1,
        "expected exactly 1 root for `{html}`, got {}: {doc:?}",
        doc.roots.len(),
    );
    assert_eq!(
        doc.roots[0].element, expected,
        "unexpected root element for `{html}`",
    );
}

// ── Document structure ────────────────────────────────────────────────

#[test]
fn parses_html() { assert_root_element("<html></html>", HtmlElement::Html); }
#[test]
fn parses_head() { assert_root_element("<head></head>", HtmlElement::Head); }
#[test]
fn parses_body() { assert_root_element("<body></body>", HtmlElement::Body); }
#[test]
fn parses_title() { assert_root_element("<title></title>", HtmlElement::Title); }
#[test]
fn parses_style() { assert_root_element("<style></style>", HtmlElement::Style); }
#[test]
fn parses_script() { assert_root_element("<script></script>", HtmlElement::Script); }
#[test]
fn parses_noscript() { assert_root_element("<noscript></noscript>", HtmlElement::Noscript); }

// ── Headings & grouping ───────────────────────────────────────────────

#[test]
fn parses_h1() { assert_root_element("<h1></h1>", HtmlElement::H1); }
#[test]
fn parses_h2() { assert_root_element("<h2></h2>", HtmlElement::H2); }
#[test]
fn parses_h3() { assert_root_element("<h3></h3>", HtmlElement::H3); }
#[test]
fn parses_h4() { assert_root_element("<h4></h4>", HtmlElement::H4); }
#[test]
fn parses_h5() { assert_root_element("<h5></h5>", HtmlElement::H5); }
#[test]
fn parses_h6() { assert_root_element("<h6></h6>", HtmlElement::H6); }
#[test]
fn parses_p() { assert_root_element("<p></p>", HtmlElement::P); }
#[test]
fn parses_pre() { assert_root_element("<pre></pre>", HtmlElement::Pre); }
#[test]
fn parses_blockquote() { assert_root_element("<blockquote></blockquote>", HtmlElement::Blockquote); }
#[test]
fn parses_address() { assert_root_element("<address></address>", HtmlElement::Address); }

// ── Inline text semantics ─────────────────────────────────────────────

#[test]
fn parses_span() { assert_root_element("<span></span>", HtmlElement::Span); }
#[test]
fn parses_a() { assert_root_element("<a></a>", HtmlElement::A); }
#[test]
fn parses_strong() { assert_root_element("<strong></strong>", HtmlElement::Strong); }
#[test]
fn parses_b() { assert_root_element("<b></b>", HtmlElement::B); }
#[test]
fn parses_em() { assert_root_element("<em></em>", HtmlElement::Em); }
#[test]
fn parses_i() { assert_root_element("<i></i>", HtmlElement::I); }
#[test]
fn parses_u() { assert_root_element("<u></u>", HtmlElement::U); }
#[test]
fn parses_s() { assert_root_element("<s></s>", HtmlElement::S); }
#[test]
fn parses_small() { assert_root_element("<small></small>", HtmlElement::Small); }
#[test]
fn parses_mark() { assert_root_element("<mark></mark>", HtmlElement::Mark); }
#[test]
fn parses_code() { assert_root_element("<code></code>", HtmlElement::Code); }
#[test]
fn parses_kbd() { assert_root_element("<kbd></kbd>", HtmlElement::Kbd); }
#[test]
fn parses_samp() { assert_root_element("<samp></samp>", HtmlElement::Samp); }
#[test]
fn parses_var() { assert_root_element("<var></var>", HtmlElement::Var); }
#[test]
fn parses_abbr() { assert_root_element("<abbr></abbr>", HtmlElement::Abbr); }
#[test]
fn parses_cite() { assert_root_element("<cite></cite>", HtmlElement::Cite); }
#[test]
fn parses_dfn() { assert_root_element("<dfn></dfn>", HtmlElement::Dfn); }
#[test]
fn parses_sub() { assert_root_element("<sub></sub>", HtmlElement::Sub); }
#[test]
fn parses_sup() { assert_root_element("<sup></sup>", HtmlElement::Sup); }
#[test]
fn parses_time() { assert_root_element("<time></time>", HtmlElement::Time); }
#[test]
fn parses_del() { assert_root_element("<del></del>", HtmlElement::Del); }
#[test]
fn parses_ins() { assert_root_element("<ins></ins>", HtmlElement::Ins); }
#[test]
fn parses_bdi() { assert_root_element("<bdi></bdi>", HtmlElement::Bdi); }
#[test]
fn parses_bdo() { assert_root_element("<bdo></bdo>", HtmlElement::Bdo); }
#[test]
fn parses_data() { assert_root_element("<data></data>", HtmlElement::Data); }
#[test]
fn parses_ruby() { assert_root_element("<ruby></ruby>", HtmlElement::Ruby); }
#[test]
fn parses_rt() { assert_root_element("<rt></rt>", HtmlElement::Rt); }
#[test]
fn parses_rp() { assert_root_element("<rp></rp>", HtmlElement::Rp); }

// ── Lists ─────────────────────────────────────────────────────────────

#[test]
fn parses_ul() { assert_root_element("<ul></ul>", HtmlElement::Ul); }
#[test]
fn parses_ol() { assert_root_element("<ol></ol>", HtmlElement::Ol); }
#[test]
fn parses_li() { assert_root_element("<li></li>", HtmlElement::Li); }
#[test]
fn parses_dl() { assert_root_element("<dl></dl>", HtmlElement::Dl); }
#[test]
fn parses_dt() { assert_root_element("<dt></dt>", HtmlElement::Dt); }
#[test]
fn parses_dd() { assert_root_element("<dd></dd>", HtmlElement::Dd); }

// ── Sections ──────────────────────────────────────────────────────────

#[test]
fn parses_header() { assert_root_element("<header></header>", HtmlElement::Header); }
#[test]
fn parses_nav() { assert_root_element("<nav></nav>", HtmlElement::Nav); }
#[test]
fn parses_main() { assert_root_element("<main></main>", HtmlElement::Main); }
#[test]
fn parses_section() { assert_root_element("<section></section>", HtmlElement::Section); }
#[test]
fn parses_article() { assert_root_element("<article></article>", HtmlElement::Article); }
#[test]
fn parses_aside() { assert_root_element("<aside></aside>", HtmlElement::Aside); }
#[test]
fn parses_footer() { assert_root_element("<footer></footer>", HtmlElement::Footer); }
#[test]
fn parses_div() { assert_root_element("<div></div>", HtmlElement::Div); }

// ── Embedded content ──────────────────────────────────────────────────

#[test]
fn parses_picture() { assert_root_element("<picture></picture>", HtmlElement::Picture); }
#[test]
fn parses_video() { assert_root_element("<video></video>", HtmlElement::Video); }
#[test]
fn parses_audio() { assert_root_element("<audio></audio>", HtmlElement::Audio); }
#[test]
fn parses_iframe() { assert_root_element("<iframe></iframe>", HtmlElement::Iframe); }
#[test]
fn parses_canvas() { assert_root_element("<canvas></canvas>", HtmlElement::Canvas); }
#[test]
fn parses_svg() { assert_root_element("<svg></svg>", HtmlElement::Svg); }

// ── Tables ────────────────────────────────────────────────────────────

#[test]
fn parses_table() { assert_root_element("<table></table>", HtmlElement::Table); }
#[test]
fn parses_caption() { assert_root_element("<caption></caption>", HtmlElement::Caption); }
#[test]
fn parses_thead() { assert_root_element("<thead></thead>", HtmlElement::Thead); }
#[test]
fn parses_tbody() { assert_root_element("<tbody></tbody>", HtmlElement::Tbody); }
#[test]
fn parses_tfoot() { assert_root_element("<tfoot></tfoot>", HtmlElement::Tfoot); }
#[test]
fn parses_tr() { assert_root_element("<tr></tr>", HtmlElement::Tr); }
#[test]
fn parses_th() { assert_root_element("<th></th>", HtmlElement::Th); }
#[test]
fn parses_td() { assert_root_element("<td></td>", HtmlElement::Td); }
#[test]
fn parses_colgroup() { assert_root_element("<colgroup></colgroup>", HtmlElement::Colgroup); }

// ── Forms ─────────────────────────────────────────────────────────────

#[test]
fn parses_form() { assert_root_element("<form></form>", HtmlElement::Form); }
#[test]
fn parses_label() { assert_root_element("<label></label>", HtmlElement::Label); }
#[test]
fn parses_textarea() { assert_root_element("<textarea></textarea>", HtmlElement::Textarea); }
#[test]
fn parses_button() { assert_root_element("<button></button>", HtmlElement::Button); }
#[test]
fn parses_select() { assert_root_element("<select></select>", HtmlElement::Select); }
#[test]
fn parses_option() { assert_root_element("<option></option>", HtmlElement::OptionElement); }
#[test]
fn parses_optgroup() { assert_root_element("<optgroup></optgroup>", HtmlElement::Optgroup); }
#[test]
fn parses_fieldset() { assert_root_element("<fieldset></fieldset>", HtmlElement::Fieldset); }
#[test]
fn parses_legend() { assert_root_element("<legend></legend>", HtmlElement::Legend); }
#[test]
fn parses_datalist() { assert_root_element("<datalist></datalist>", HtmlElement::Datalist); }
#[test]
fn parses_output() { assert_root_element("<output></output>", HtmlElement::Output); }
#[test]
fn parses_progress() { assert_root_element("<progress></progress>", HtmlElement::Progress); }
#[test]
fn parses_meter() { assert_root_element("<meter></meter>", HtmlElement::Meter); }

// ── Interactive ───────────────────────────────────────────────────────

#[test]
fn parses_details() { assert_root_element("<details></details>", HtmlElement::Details); }
#[test]
fn parses_summary() { assert_root_element("<summary></summary>", HtmlElement::Summary); }
#[test]
fn parses_dialog() { assert_root_element("<dialog></dialog>", HtmlElement::Dialog); }
#[test]
fn parses_template() { assert_root_element("<template></template>", HtmlElement::Template); }
#[test]
fn parses_slot() { assert_root_element("<slot></slot>", HtmlElement::Slot); }

// ── Void elements ─────────────────────────────────────────────────────

#[test]
fn parses_meta() {
    let doc = parse("<meta>");
    assert_eq!(doc.roots[0].element, HtmlElement::Meta);
}
#[test]
fn parses_link() { assert_root_element("<link>", HtmlElement::Link); }
#[test]
fn parses_br() { assert_root_element("<br>", HtmlElement::Br); }
#[test]
fn parses_hr() { assert_root_element("<hr>", HtmlElement::Hr); }
#[test]
fn parses_img() { assert_root_element("<img>", HtmlElement::Img); }
#[test]
fn parses_source() { assert_root_element("<source>", HtmlElement::Source); }
#[test]
fn parses_track() { assert_root_element("<track>", HtmlElement::Track); }
#[test]
fn parses_col() { assert_root_element("<col>", HtmlElement::Col); }
#[test]
fn parses_input() { assert_root_element("<input>", HtmlElement::Input); }
#[test]
fn parses_wbr() { assert_root_element("<wbr>", HtmlElement::Wbr); }

// ── HtmlElement::from_name / tag_name / is_void / is_raw_text ─────────

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
