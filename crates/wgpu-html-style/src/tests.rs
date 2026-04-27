use super::*;
use wgpu_html_models::common::css_enums::{CssColor, CssLength};
use wgpu_html_parser::{Selector, parse_stylesheet};

fn elem_div() -> Element {
    Element::Div(wgpu_html_models::Div::default())
}

fn elem_div_with(id: Option<&str>, class: Option<&str>) -> Element {
    let mut d = wgpu_html_models::Div::default();
    d.id = id.map(str::to_string);
    d.class = class.map(str::to_string);
    Element::Div(d)
}

fn elem_p() -> Element {
    Element::P(wgpu_html_models::P::default())
}

// --------------------------------------------------------------------------
// Selector matching
// --------------------------------------------------------------------------

#[test]
fn matches_tag_only() {
    let sel = Selector {
        tag: Some("div".into()),
        ..Default::default()
    };
    assert!(matches_selector(&sel, &elem_div()));
    assert!(!matches_selector(&sel, &elem_p()));
}

#[test]
fn matches_id() {
    let sel = Selector {
        id: Some("hero".into()),
        ..Default::default()
    };
    assert!(matches_selector(&sel, &elem_div_with(Some("hero"), None)));
    assert!(!matches_selector(&sel, &elem_div_with(Some("other"), None)));
    assert!(!matches_selector(&sel, &elem_div_with(None, None)));
}

#[test]
fn matches_class_one_of_many() {
    let sel = Selector {
        classes: vec!["card".into()],
        ..Default::default()
    };
    assert!(matches_selector(
        &sel,
        &elem_div_with(None, Some("big card primary"))
    ));
    assert!(!matches_selector(
        &sel,
        &elem_div_with(None, Some("big primary"))
    ));
}

#[test]
fn matches_compound_all_required() {
    let sel = Selector {
        tag: Some("div".into()),
        id: Some("hero".into()),
        classes: vec!["card".into(), "big".into()],
        ..Default::default()
    };
    assert!(matches_selector(
        &sel,
        &elem_div_with(Some("hero"), Some("card big primary"))
    ));
    // missing one class → fails
    assert!(!matches_selector(
        &sel,
        &elem_div_with(Some("hero"), Some("card primary"))
    ));
}

#[test]
fn universal_matches_any() {
    let sel = Selector {
        universal: true,
        ..Default::default()
    };
    assert!(matches_selector(&sel, &elem_div()));
    assert!(matches_selector(&sel, &elem_p()));
}

// --------------------------------------------------------------------------
// computed_style: cascade order
// --------------------------------------------------------------------------

#[test]
fn id_beats_class() {
    let sheet = parse_stylesheet(
        "
        .card { background-color: blue; }
        #hero { background-color: red; }
        ",
    );
    let el = elem_div_with(Some("hero"), Some("card"));
    let style = computed_style(&el, &sheet);
    let bg = style.background_color.expect("set");
    // The id rule has higher specificity → red wins.
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}

#[test]
fn class_beats_tag() {
    let sheet = parse_stylesheet(
        "
        div { background-color: blue; }
        .card { background-color: red; }
        ",
    );
    let el = elem_div_with(None, Some("card"));
    let style = computed_style(&el, &sheet);
    let bg = style.background_color.expect("set");
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}

#[test]
fn inline_beats_id() {
    let sheet = parse_stylesheet("#hero { background-color: blue; }");
    let mut div = wgpu_html_models::Div::default();
    div.id = Some("hero".into());
    div.style = Some("background-color: red;".into());
    let style = computed_style(&Element::Div(div), &sheet);
    let bg = style.background_color.expect("set");
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}

#[test]
fn rules_at_same_specificity_apply_in_source_order() {
    let sheet = parse_stylesheet(
        "
        .card { background-color: blue; }
        .card { background-color: red; }
        ",
    );
    let el = elem_div_with(None, Some("card"));
    let style = computed_style(&el, &sheet);
    let bg = style.background_color.expect("set");
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}

#[test]
fn unrelated_rules_do_not_apply() {
    let sheet = parse_stylesheet(".other { width: 999px; }");
    let el = elem_div_with(None, Some("card"));
    let style = computed_style(&el, &sheet);
    assert!(style.width.is_none());
}

#[test]
fn comma_lists_all_match() {
    let sheet = parse_stylesheet("h1, h2, .big { color: red; }");
    let el = elem_div_with(None, Some("big"));
    let style = computed_style(&el, &sheet);
    assert!(style.color.is_some());
}

// --------------------------------------------------------------------------
// End-to-end cascade()
// --------------------------------------------------------------------------

#[test]
fn cascade_extracts_style_block_and_applies() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            #parent { width: 100px; padding: 10px; }
            .child { width: 30px; height: 30px; }
        </style>
        <div id="parent">
            <div class="child"></div>
        </div>
        "#,
    );
    let cascaded = cascade(&tree);
    let body = cascaded.root.as_ref().expect("synthetic body");
    // root is a synthetic <body> wrapping <style> + <div id=parent>
    let parent = body
        .children
        .iter()
        .find(|c| matches!(c.element, Element::Div(_)))
        .expect("parent div");
    assert!(matches!(parent.style.width, Some(CssLength::Px(v)) if v == 100.0));
    assert!(parent.style.padding.is_some());
    let child = &parent.children[0];
    assert!(matches!(child.style.width, Some(CssLength::Px(v)) if v == 30.0));
    assert!(matches!(child.style.height, Some(CssLength::Px(v)) if v == 30.0));
}

#[test]
fn cascade_inline_style_takes_precedence_over_block() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            #x { background-color: blue; }
        </style>
        <div id="x" style="background-color: red;"></div>
        "#,
    );
    let cascaded = cascade(&tree);
    let body = cascaded.root.as_ref().unwrap();
    let div = body
        .children
        .iter()
        .find(|c| matches!(c.element, Element::Div(_)))
        .unwrap();
    let bg = div.style.background_color.as_ref().unwrap();
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}
