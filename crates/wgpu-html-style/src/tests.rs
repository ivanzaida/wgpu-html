use super::*;
use wgpu_html_models::common::css_enums::{
    BoxSizing, CssColor, CssLength, Cursor, Display, TextAlign,
};
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
    assert!(matches_selector_in_tree(&sel, &item, &[&neutral, &row]));
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
fn background_shorthand_higher_priority_clears_lower_priority_image() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { background-image: url('assets/bg.png'); }
            #x  { background: #1b1d22; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(matches!(
        div.style.background_color,
        Some(CssColor::Hex(ref s)) if s == "#1b1d22"
    ));
    assert!(div.style.background_image.is_none());
}

#[test]
fn font_shorthand_and_longhand_obey_source_order_in_one_rule() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { font: 15px Arial, sans-serif; font-size: 16px; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(matches!(div.style.font_size, Some(CssLength::Px(v)) if v == 16.0));
    assert_eq!(div.style.font_family.as_deref(), Some("Arial, sans-serif"));
    assert!(matches!(
        div.style.font_style,
        Some(wgpu_html_models::common::css_enums::FontStyle::Normal)
    ));
    assert!(matches!(
        div.style.font_weight,
        Some(wgpu_html_models::common::css_enums::FontWeight::Normal)
    ));
    assert!(matches!(div.style.line_height, Some(CssLength::Raw(ref v)) if v == "normal"));
    assert_eq!(
        div.style
            .deferred_longhands
            .get("font-variant")
            .map(String::as_str),
        Some("normal")
    );
    assert_eq!(
        div.style
            .deferred_longhands
            .get("font-stretch")
            .map(String::as_str),
        Some("normal")
    );
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

#[test]
fn ua_stylesheet_applies_browser_display_defaults() {
    let tree = wgpu_html_parser::parse(
        r#"
        <section></section>
        <template><div></div></template>
        <ul><li></li></ul>
        <table><tr><td></td><th></th></tr></table>
        <ruby><rt></rt><rp></rp></ruby>
        "#,
    );
    let cascaded = cascade(&tree);
    let root = cascaded.root.as_ref().unwrap();
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Section(_)))
            .unwrap()
            .display,
        Some(Display::Block)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Li(_)))
            .unwrap()
            .display,
        Some(Display::ListItem)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Table(_)))
            .unwrap()
            .display,
        Some(Display::Table)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Tr(_)))
            .unwrap()
            .display,
        Some(Display::TableRow)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Td(_)))
            .unwrap()
            .display,
        Some(Display::TableCell)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Rt(_)))
            .unwrap()
            .display,
        Some(Display::RubyText)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Rp(_)))
            .unwrap()
            .display,
        Some(Display::None)
    ));
    assert!(matches!(
        find_style(root, &|el| matches!(el, Element::Template(_)))
            .unwrap()
            .display,
        Some(Display::None)
    ));
}

#[test]
fn stylesheet_collection_skips_template_contents() {
    let tree = wgpu_html_parser::parse(
        r#"
        <template>
            <style>#outside { background-color: red; }</style>
        </template>
        <style>#outside { background-color: blue; }</style>
        <div id="outside"></div>
        "#,
    );

    let sheet = collect_stylesheet(&tree);
    assert_eq!(sheet.rules.len(), 1);

    let cascaded = cascade(&tree);
    let root = cascaded.root.as_ref().unwrap();
    let div = find_style(
        root,
        &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("outside")),
    )
    .unwrap();
    let bg = div.background_color.as_ref().unwrap();
    assert!(matches!(bg, CssColor::Named(s) if s == "blue"));
}

#[test]
fn media_query_rules_match_viewport_context() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            #box { width: 100px; }
            @media screen and (max-width: 500px) {
                #box { width: 200px; }
            }
        </style>
        <div id="box"></div>
        "#,
    );

    let small = cascade_with_media(&tree, &MediaContext::screen(400.0, 800.0, 1.0));
    let root = small.root.as_ref().unwrap();
    let div = find_style(
        root,
        &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
    )
    .unwrap();
    assert!(matches!(div.width, Some(CssLength::Px(v)) if (v - 200.0).abs() < 0.01));

    let large = cascade_with_media(&tree, &MediaContext::screen(900.0, 800.0, 1.0));
    let root = large.root.as_ref().unwrap();
    let div = find_style(
        root,
        &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
    )
    .unwrap();
    assert!(matches!(div.width, Some(CssLength::Px(v)) if (v - 100.0).abs() < 0.01));
}

#[test]
fn style_media_attribute_gates_whole_style_block() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style media="(orientation: landscape)">
            #box { height: 40px; }
        </style>
        <div id="box"></div>
        "#,
    );

    let landscape = cascade_with_media(&tree, &MediaContext::screen(800.0, 400.0, 1.0));
    let root = landscape.root.as_ref().unwrap();
    let div = find_style(
        root,
        &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
    )
    .unwrap();
    assert!(matches!(div.height, Some(CssLength::Px(v)) if (v - 40.0).abs() < 0.01));

    let portrait = cascade_with_media(&tree, &MediaContext::screen(400.0, 800.0, 1.0));
    let root = portrait.root.as_ref().unwrap();
    let div = find_style(
        root,
        &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
    )
    .unwrap();
    assert!(div.height.is_none());
}

#[test]
fn ua_attribute_selectors_apply() {
    let tree = wgpu_html_parser::parse(
        r#"
        <div hidden></div>
        <dialog></dialog>
        <dialog open></dialog>
        <input type="hidden">
        <input type="submit">
        <abbr title="expanded"></abbr>
        <div dir="rtl"></div>
        "#,
    );
    let cascaded = cascade(&tree);
    let root = cascaded.root.as_ref().unwrap();

    let hidden_div = find_style(
        root,
        &|el| matches!(el, Element::Div(d) if d.hidden == Some(true)),
    )
    .unwrap();
    assert!(matches!(hidden_div.display, Some(Display::None)));

    let closed_dialog = find_style(
        root,
        &|el| matches!(el, Element::Dialog(d) if d.open != Some(true)),
    )
    .unwrap();
    assert!(matches!(closed_dialog.display, Some(Display::None)));

    let open_dialog = find_style(
        root,
        &|el| matches!(el, Element::Dialog(d) if d.open == Some(true)),
    )
    .unwrap();
    assert!(matches!(open_dialog.display, Some(Display::Block)));

    let hidden_input = find_style(root, &|el| {
        matches!(
            el,
            Element::Input(i)
                if matches!(
                    i.r#type,
                    Some(wgpu_html_models::common::html_enums::InputType::Hidden)
                )
        )
    })
    .unwrap();
    assert!(matches!(hidden_input.display, Some(Display::None)));

    let submit_input = find_style(root, &|el| {
        matches!(
            el,
            Element::Input(i)
                if matches!(
                    i.r#type,
                    Some(wgpu_html_models::common::html_enums::InputType::Submit)
                )
        )
    })
    .unwrap();
    assert!(matches!(submit_input.display, Some(Display::InlineBlock)));
    assert!(matches!(submit_input.cursor, Some(Cursor::Default)));
    assert!(matches!(
        submit_input.box_sizing,
        Some(BoxSizing::BorderBox)
    ));

    let abbr = find_style(root, &|el| matches!(el, Element::Abbr(_))).unwrap();
    assert_eq!(abbr.text_decoration.as_deref(), Some("underline dotted"));

    let rtl = find_style(root, &|el| matches!(el, Element::Div(d) if d.dir.is_some())).unwrap();
    assert_eq!(
        rtl.deferred_longhands.get("direction").map(String::as_str),
        Some("rtl")
    );
}

#[test]
fn ua_form_font_initial_resets_inherited_text_styles() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body {
                font-size: 20px;
                font-weight: bold;
                font-style: italic;
                color: red;
                letter-spacing: 2px;
            }
        </style>
        <input>
        "#,
    );
    let cascaded = cascade(&tree);
    let input = find_style(cascaded.root.as_ref().unwrap(), &|el| {
        matches!(el, Element::Input(_))
    })
    .unwrap();
    assert!(input.font_size.is_none());
    assert!(matches!(input.color, Some(CssColor::Named(ref v)) if v == "fieldtext"));
    assert!(input.font_weight.is_none());
    assert!(input.font_style.is_none());
    assert!(matches!(input.line_height, Some(CssLength::Raw(ref v)) if v == "normal"));
    assert!(matches!(
        input.letter_spacing,
        Some(CssLength::Raw(ref v)) if v == "normal"
    ));
    assert!(matches!(input.text_align, Some(TextAlign::Start)));
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
fn background_keyword_initial_clears_prior_longhands() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { background-color: red; background-image: url('assets/bg.png'); }
            #x  { background: initial; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(div.style.background.is_none());
    assert!(div.style.background_color.is_none());
    assert!(div.style.background_image.is_none());
}

#[test]
fn generic_shorthand_initial_clears_typed_member_longhands() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { margin-top: 12px; margin-right: 13px; padding-left: 9px; }
            #x  { margin: initial; padding: initial; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(div.style.margin_top.is_none());
    assert!(div.style.margin_right.is_none());
    assert!(div.style.margin_bottom.is_none());
    assert!(div.style.margin_left.is_none());
    assert!(div.style.padding_left.is_none());
}

#[test]
fn generic_shorthand_initial_clears_deferred_member_longhands() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { transition-property: opacity; transition-duration: 200ms; }
            #x  { transition: initial; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(
        !div.style
            .deferred_longhands
            .contains_key("transition-property")
    );
    assert!(
        !div.style
            .deferred_longhands
            .contains_key("transition-duration")
    );
}

#[test]
fn shorthand_keyword_after_member_keyword_clears_member_keyword() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { margin-top: 20px; }
            div  { margin-top: inherit; margin: initial; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(div.style.margin_top.is_none());
}

#[test]
fn longhand_value_after_shorthand_keyword_clears_covering_keyword() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { margin: initial; margin-top: 12px; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(matches!(div.style.margin_top, Some(CssLength::Px(v)) if v == 12.0));
    assert!(div.style.margin_right.is_none());
}

#[test]
fn deferred_longhand_value_after_shorthand_keyword_clears_covering_keyword() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { transition: initial; transition-duration: 200ms; }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    assert_eq!(
        div.style
            .deferred_longhands
            .get("transition-duration")
            .map(String::as_str),
        Some("200ms")
    );
    assert!(
        !div.style
            .deferred_longhands
            .contains_key("transition-property")
    );
}

#[test]
fn all_initial_clears_typed_and_deferred_properties() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            div { color: red; animation: fade 1s linear; }
            #x  { all: initial; }
        </style>
        <div id="x"></div>
        "#,
    );
    let div = first_div(&tree);
    assert!(div.style.color.is_none());
    assert!(div.style.animation.is_none());
    assert!(!div.style.deferred_longhands.contains_key("animation-name"));
}

#[test]
fn deferred_inherited_longhand_flows_through_cascade() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            body { white-space: pre-wrap; }
            div  { }
        </style>
        <div></div>
        "#,
    );
    let div = first_div(&tree);
    assert_eq!(
        div.style
            .deferred_longhands
            .get("white-space-collapse")
            .map(String::as_str),
        Some("preserve")
    );
    assert_eq!(
        div.style
            .deferred_longhands
            .get("text-wrap-mode")
            .map(String::as_str),
        Some("wrap")
    );
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
    let tree = wgpu_html_parser::parse(r#"<div style="color: inherit;"></div>"#);
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
    let style =
        find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("b")).expect("found");
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
    let style =
        find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("b")).expect("found");
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
    let style =
        find_style(&cascaded.root.unwrap(), &|el| matches!(el, Element::Div(_))).expect("found");
    let bg = style.background_color.expect("set");
    // div:hover (tag + pseudo = 1 tag + 1 class) beats plain div (1 tag).
    assert!(matches!(bg, CssColor::Named(s) if s == "red"));
}

#[test]
fn focus_rule_applies_only_to_focused_element() {
    // `:focus` matches only the exact focused element, not its
    // ancestors (unlike `:hover`).
    let mut tree = wgpu_html_parser::parse(
        r#"
        <style>
            #outer { background-color: white; }
            #outer:focus { background-color: yellow; }
            #inner { background-color: white; }
            #inner:focus { background-color: red; }
        </style>
        <div id="outer"><span id="inner">hi</span></div>
        "#,
    );
    // Path: <body> → [0]=<style>, [1]=<div id=outer>, [1, 0]=<span>.
    // Focus the inner span only.
    tree.interaction.focus_path = Some(vec![1, 0]);
    let cascaded = cascade(&tree);

    // Inner is focused → its :focus rule applies.
    let inner_style = find_style(&cascaded.root.as_ref().unwrap(), &|el| {
        element_id(el) == Some("inner")
    })
    .expect("found");
    let inner_bg = inner_style.background_color.expect("set");
    assert!(
        matches!(inner_bg.clone(), CssColor::Named(s) if s == "red"),
        "inner background expected red, got {inner_bg:?}"
    );

    // Outer is NOT focused (only its descendant is). Plain rule wins.
    let outer_style = find_style(&cascaded.root.as_ref().unwrap(), &|el| {
        element_id(el) == Some("outer")
    })
    .expect("found");
    let outer_bg = outer_style.background_color.expect("set");
    assert!(
        matches!(outer_bg.clone(), CssColor::Named(s) if s == "white"),
        "outer background expected white (focus does not propagate), got {outer_bg:?}"
    );
}

#[test]
fn focus_rule_does_not_apply_when_focus_path_is_none() {
    let tree = wgpu_html_parser::parse(
        r#"
        <style>
            input { background-color: white; }
            input:focus { background-color: yellow; }
        </style>
        <input id="x">
        "#,
    );
    // No focus set.
    assert!(tree.interaction.focus_path.is_none());
    let cascaded = cascade(&tree);
    let style =
        find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("x")).expect("found");
    let bg = style.background_color.expect("set");
    assert!(matches!(bg, CssColor::Named(s) if s == "white"));
}
