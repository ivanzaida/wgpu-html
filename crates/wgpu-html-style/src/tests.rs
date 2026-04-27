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

#[test]
fn matches_selector_rejects_descendant_without_ancestors() {
    // `.row .item` against an `.item` element with no ancestor
    // context must NOT match — the simple wrapper has no chain.
    let sel = Selector {
        classes: vec!["item".into()],
        ancestors: vec![Selector {
            classes: vec!["row".into()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let item = elem_div_with(None, Some("item"));
    assert!(!matches_selector(&sel, &item));
}

#[test]
fn matches_selector_in_tree_walks_ancestors() {
    let sel = Selector {
        classes: vec!["item".into()],
        ancestors: vec![Selector {
            classes: vec!["row".into()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let row = elem_div_with(None, Some("row"));
    let item = elem_div_with(None, Some("item"));
    // Direct parent matches → fires.
    assert!(matches_selector_in_tree(&sel, &item, &[&row]));
    // No ancestor `.row` → fails.
    let neutral = elem_div_with(None, Some("box"));
    assert!(!matches_selector_in_tree(&sel, &item, &[&neutral]));
    // Deeper ancestor `.row` (with an unrelated parent in between) →
    // descendant combinator is non-adjacent, still fires.
    assert!(matches_selector_in_tree(
        &sel,
        &item,
        &[&neutral, &row]
    ));
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

// --------------------------------------------------------------------------
// !important — cascade-3 §6.4 ordering
// --------------------------------------------------------------------------

fn first_div(tree: &Tree) -> CascadedNode {
    let cascaded = cascade(tree);
    let body = cascaded.root.expect("expected a root");
    body.children
        .into_iter()
        .find(|c| matches!(c.element, Element::Div(_)))
        .expect("expected a div under root")
}

#[test]
fn important_in_lower_specificity_beats_normal_in_higher() {
    // `div` (specificity 1) marked !important wins over `#x` (256)
    // marked normal — important rules form a separate cascade band
    // applied above all normal rules regardless of specificity.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { background-color: green !important; }
            #x  { background-color: blue; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    let bg = div.style.background_color.as_ref().unwrap();
    assert!(matches!(bg, CssColor::Named(s) if s == "green"));
}

#[test]
fn important_specificity_still_orders_within_band() {
    // Two !important rules: the more specific one (id > class) wins.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            .c { background-color: green !important; }
            #x { background-color: blue !important; }
        </style>
        <div id="x" class="c"></div>
        "#,
    );
    let div = first_div(&tree);
    let bg = div.style.background_color.as_ref().unwrap();
    assert!(matches!(bg, CssColor::Named(s) if s == "blue"));
}

#[test]
fn important_author_overrides_inline_normal() {
    // Inline `style="…"` is at the inline-normal layer and loses to
    // any author !important rule, no matter how low the selector's
    // specificity (here `div`, specificity 1).
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { color: green !important; }
        </style>
        <div style="color: red;"></div>
        "#,
    );
    let div = first_div(&tree);
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "green"));
}

#[test]
fn inline_important_beats_author_important() {
    // Inline !important sits above author !important.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { color: green !important; }
        </style>
        <div style="color: red !important;"></div>
        "#,
    );
    let div = first_div(&tree);
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "red"));
}

#[test]
fn important_does_not_leak_across_properties() {
    // `color !important` doesn't affect `background-color` cascade —
    // each property is cascaded independently. Here the normal-band
    // `#x` rule wins for `background-color`, while the !important
    // band wins for `color`.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { color: green !important; background-color: green; }
            #x  { background-color: blue; color: blue; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    // color: !important from div wins.
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "green"));
    // background-color: #x (normal, more specific) wins.
    let bg = div.style.background_color.as_ref().unwrap();
    assert!(matches!(bg, CssColor::Named(s) if s == "blue"));
}

#[test]
fn important_whitespace_variants_are_recognised() {
    // CSS allows whitespace between `!` and `important`, and the
    // keyword is case-insensitive. The parser must accept both.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { color: red !  IMPORTANT ; }
            #x  { color: blue; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "red"));
}

// --------------------------------------------------------------------------
// CSS-wide keywords — `inherit` / `initial` / `unset`
// --------------------------------------------------------------------------

#[test]
fn inherit_keyword_takes_parent_value_for_non_inherited_property() {
    // `background-color` is *not* normally inherited. With `inherit`
    // on the child it must take the parent's value anyway.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { background-color: orange; }
            div  { background-color: inherit; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    let bg = div.style.background_color.as_ref().unwrap();
    assert!(matches!(bg, CssColor::Named(s) if s == "orange"));
}

#[test]
fn initial_keyword_blocks_implicit_inheritance() {
    // `color` is inherited. With `color: initial`, the child must
    // *not* take the parent's color — even though implicit
    // inheritance would otherwise fill it in.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { color: red; }
            div  { color: initial; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    // `initial` collapses to None when there's no UA default tracked.
    assert!(div.style.color.is_none());
}

#[test]
fn unset_acts_as_inherit_for_inherited_property() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { color: red; }
            div  { color: unset; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    // `color` is inherited → `unset` === `inherit` → red.
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "red"));
}

#[test]
fn unset_acts_as_initial_for_non_inherited_property() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { background-color: red; }
            div  { background-color: unset; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    // `background-color` isn't inherited → `unset` === `initial` → None.
    assert!(div.style.background_color.is_none());
}

#[test]
fn inherit_within_a_block_displaces_an_earlier_value() {
    // Within one rule, source-order resolves: `color: inherit;`
    // declared after `color: red;` must win.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { color: green; }
            div  { color: red; color: inherit; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "green"));
}

#[test]
fn value_after_keyword_within_a_block_displaces_the_keyword() {
    // Reverse of the previous case: a normal value after a keyword
    // (same property, same block) wins.
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { color: green; }
            div  { color: inherit; color: red; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "red"));
}

#[test]
fn keyword_loses_to_more_specific_important_value() {
    // `inherit !important` at id-specificity beats inline normal,
    // but a class-specificity `!important red` is at lower
    // specificity, so the inline normal of red would be... let's
    // wire a clearer case. Here: `inherit !important` at
    // .child specificity is in the !important band; an inline
    // normal at the body sub-band is below it; final color is
    // inherited from body (parent).
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body  { color: green; }
            .card { color: inherit !important; }
        </style>
        <div class="card" style="color: blue;"></div>
        "#,
    );
    let div = first_div(&tree);
    // Layer 1 (author normal) sets nothing for color on .card.
    // Layer 2 (inline normal) sets color = blue.
    // Layer 3 (author !important) sets color keyword = inherit
    //   → clears value, records keyword.
    // Layer 4 (inline !important) — empty.
    // Resolution: keyword Inherit → parent.color = green.
    let c = div.style.color.as_ref().unwrap();
    assert!(matches!(c, CssColor::Named(s) if s == "green"));
}

#[test]
fn root_inherit_keyword_resolves_to_initial() {
    // The cascaded root has no parent, so `inherit` collapses to
    // initial (None in our impl). The synthetic body wrapping in the
    // parser places the styled element under a body that does have a
    // parent — so write it on the root html instead.
    let tree = wgpu_html_parser::parse(
        r#"<div style="color: inherit;"></div>"#,
    );
    let cascaded = cascade(&tree);
    let root = cascaded.root.expect("root");
    // Whether `root` is the div directly or a body wrapper depends on
    // the tree builder; either way, the `color: inherit` declaration
    // sits without a parent to inherit from and resolves to None.
    let walk = |n: &CascadedNode| -> bool {
        if matches!(n.element, Element::Div(_)) {
            return n.style.color.is_none();
        }
        n.children
            .iter()
            .any(|c| matches!(c.element, Element::Div(_)) && c.style.color.is_none())
    };
    assert!(walk(&root));
}

// --------------------------------------------------------------------------
// :hover / :active pseudo-classes
// --------------------------------------------------------------------------

/// Find the first descendant matching `pred` and return its style.
fn find_style<F: Fn(&Element) -> bool>(node: &CascadedNode, pred: &F) -> Option<Style> {
    if pred(&node.element) {
        return Some(node.style.clone());
    }
    for c in &node.children {
        if let Some(s) = find_style(c, pred) {
            return Some(s);
        }
    }
    None
}

#[test]
fn hover_rule_does_not_match_without_state() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            #b { background-color: blue; }
            #b:hover { background-color: red; }
        </style>
        <div id="b"></div>
        "#,
    );
    let cascaded = cascade(&tree);
    let style = find_style(&cascaded.root.unwrap(), &|el| {
        element_id(el) == Some("b")
    })
    .expect("found");
    let bg = style.background_color.expect("set");
    assert!(matches!(bg, CssColor::Named(s) if s == "blue"));
}

#[test]
fn hover_rule_applies_when_path_in_hover_chain() {
    let mut tree = wgpu_html_parser::parse(
        r#"
        <style>
            #b { background-color: blue; }
            #b:hover { background-color: red; }
        </style>
        <div id="b"></div>
        "#,
    );
    // The tree builder wraps the children in a synthetic body when
    // there are multiple top-level nodes (the <style> + the <div>).
    // Path from root walks past <style> at index 0 to reach the div
    // at index 1.
    tree.interaction.hover_path = Some(vec![1]);
    let cascaded = cascade(&tree);
    let style = find_style(&cascaded.root.unwrap(), &|el| {
        element_id(el) == Some("b")
    })
    .expect("found");
    let bg = style.background_color.expect("set");
    // :hover rule wins now (same specificity as #b alone, source order
    // says the :hover rule comes second).
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}

#[test]
fn ancestor_in_hover_chain_also_hovers() {
    // Per CSS, hovering a descendant marks every ancestor as :hover.
    // Setting hover_path to the deeper descendant's path should make
    // the ancestor's `:hover` rule fire.
    let mut tree = wgpu_html_parser::parse(
        r#"
        <style>
            #outer { background-color: white; }
            #outer:hover { background-color: yellow; }
        </style>
        <div id="outer"><span id="inner">hi</span></div>
        "#,
    );
    // Path: <body> → [0]=<style>, [1]=<div id=outer>, [1, 0]=<span>.
    tree.interaction.hover_path = Some(vec![1, 0]);
    let cascaded = cascade(&tree);
    let outer_style = find_style(&cascaded.root.unwrap(), &|el| {
        element_id(el) == Some("outer")
    })
    .expect("found");
    let bg = outer_style.background_color.expect("set");
    assert!(matches!(bg, CssColor::Named(s) if s == "yellow"));
}

#[test]
fn hover_specificity_beats_plain_class() {
    // `.x:hover` (1 class + 1 pseudo = 2 classes) should beat `.x.y`
    // (2 classes) only on tie-break — same specificity, source order.
    // Here we test that `:hover` does add specificity vs plain `.x`.
    let mut tree = wgpu_html_parser::parse(
        r#"
        <style>
            div:hover { background-color: red; }
            div { background-color: blue; }
        </style>
        <div></div>
        "#,
    );
    tree.interaction.hover_path = Some(vec![1]);
    let cascaded = cascade(&tree);
    let style = find_style(&cascaded.root.unwrap(), &|el| matches!(el, Element::Div(_)))
        .expect("found");
    let bg = style.background_color.expect("set");
    // div:hover (tag + pseudo = 1 tag + 1 class) beats plain div (1 tag).
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}
