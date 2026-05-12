use lui_cascade::matching::{Dir, MatchContext, any_selector_matches};
use lui_css_parser::parse_selector_list;
use lui_html_parser::parse;
use crate::helpers::root_ctx;

// ── :target ──

#[test]
fn target_matches_when_id_equals_fragment() {
    let doc = parse(r#"<div id="section1"></div>"#);
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":target").unwrap();

    let ctx = MatchContext { target_id: Some("section1"), ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn target_rejects_when_id_differs() {
    let doc = parse(r#"<div id="section1"></div>"#);
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":target").unwrap();

    let ctx = MatchContext { target_id: Some("other"), ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_none());
}

#[test]
fn target_rejects_when_no_fragment() {
    let doc = parse(r#"<div id="section1"></div>"#);
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":target").unwrap();

    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_none());
}

#[test]
fn target_rejects_element_without_id() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":target").unwrap();

    let ctx = MatchContext { target_id: Some("anything"), ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_none());
}

// ── :lang() ──

#[test]
fn lang_matches_exact() {
    let doc = parse(r#"<p lang="en">text</p>"#);
    let p = &doc.root.children[0];
    let sel = parse_selector_list(":lang(en)").unwrap();

    let ctx = MatchContext { lang: Some("en"), ..root_ctx() };
    assert!(any_selector_matches(&sel, p, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn lang_matches_subtag() {
    let doc = parse(r#"<p lang="en-US">text</p>"#);
    let p = &doc.root.children[0];
    let sel = parse_selector_list(":lang(en)").unwrap();

    let ctx = MatchContext { lang: Some("en-US"), ..root_ctx() };
    assert!(any_selector_matches(&sel, p, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn lang_rejects_wrong_language() {
    let doc = parse(r#"<p lang="fr">text</p>"#);
    let p = &doc.root.children[0];
    let sel = parse_selector_list(":lang(en)").unwrap();

    let ctx = MatchContext { lang: Some("fr"), ..root_ctx() };
    assert!(any_selector_matches(&sel, p, &ctx, &[], Some(&doc.root)).is_none());
}

#[test]
fn lang_rejects_when_no_lang() {
    let doc = parse("<p>text</p>");
    let p = &doc.root.children[0];
    let sel = parse_selector_list(":lang(en)").unwrap();

    assert!(any_selector_matches(&sel, p, &root_ctx(), &[], Some(&doc.root)).is_none());
}

// ── :dir() ──

#[test]
fn dir_ltr_matches() {
    let doc = parse(r#"<div dir="ltr"></div>"#);
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":dir(ltr)").unwrap();

    let ctx = MatchContext { dir: Some(Dir::Ltr), ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn dir_rtl_matches() {
    let doc = parse(r#"<div dir="rtl"></div>"#);
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":dir(rtl)").unwrap();

    let ctx = MatchContext { dir: Some(Dir::Rtl), ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn dir_rejects_mismatch() {
    let doc = parse(r#"<div dir="ltr"></div>"#);
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":dir(rtl)").unwrap();

    let ctx = MatchContext { dir: Some(Dir::Ltr), ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_none());
}

// ── :defined ──

#[test]
fn defined_matches_known_element() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":defined").unwrap();

    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_some());
}

#[test]
fn defined_rejects_unknown_element() {
    let doc = parse("<my-widget></my-widget>");
    let widget = &doc.root.children[0];
    let sel = parse_selector_list(":defined").unwrap();

    assert!(any_selector_matches(&sel, widget, &root_ctx(), &[], Some(&doc.root)).is_none());
}

// ── :scope ──

#[test]
fn scope_matches_root() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":scope").unwrap();

    let ctx = MatchContext { is_root: true, ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn scope_rejects_non_root() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":scope").unwrap();

    let ctx = MatchContext { is_root: false, ..Default::default() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_none());
}

// ── :fullscreen / :modal ──

#[test]
fn fullscreen_matches_when_set() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":fullscreen").unwrap();

    let ctx = MatchContext { is_fullscreen: true, ..root_ctx() };
    assert!(any_selector_matches(&sel, div, &ctx, &[], Some(&doc.root)).is_some());
}

#[test]
fn fullscreen_rejects_by_default() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":fullscreen").unwrap();

    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_none());
}

#[test]
fn modal_matches_when_set() {
    let doc = parse("<dialog></dialog>");
    let dialog = &doc.root.children[0];
    let sel = parse_selector_list(":modal").unwrap();

    let ctx = MatchContext { is_modal: true, ..root_ctx() };
    assert!(any_selector_matches(&sel, dialog, &ctx, &[], Some(&doc.root)).is_some());
}

// ── Unknown pseudo-classes reject ──

#[test]
fn unknown_pseudo_rejects() {
    let doc = parse("<div></div>");
    let div = &doc.root.children[0];
    let sel = parse_selector_list(":nonsense").unwrap();

    assert!(any_selector_matches(&sel, div, &root_ctx(), &[], Some(&doc.root)).is_none());
}
