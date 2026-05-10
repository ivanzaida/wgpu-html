use super::helpers::*;
use crate::*;
// --------------------------------------------------------------------------
// End-to-end cascade()
// --------------------------------------------------------------------------

#[test]
fn cascade_extracts_style_block_and_applies() {
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
  assert!(matches!(bg, CssColor::Named(s) if &**s == "red"));
}

#[test]
fn ua_stylesheet_applies_browser_display_defaults() {
  let tree = lui_parser::parse(
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
    find_style(root, &|el| matches!(el, Element::Li(_))).unwrap().display,
    Some(Display::ListItem)
  ));
  assert!(matches!(
    find_style(root, &|el| matches!(el, Element::Table(_))).unwrap().display,
    Some(Display::Table)
  ));
  assert!(matches!(
    find_style(root, &|el| matches!(el, Element::Tr(_))).unwrap().display,
    Some(Display::TableRow)
  ));
  assert!(matches!(
    find_style(root, &|el| matches!(el, Element::Td(_))).unwrap().display,
    Some(Display::TableCell)
  ));
  assert!(matches!(
    find_style(root, &|el| matches!(el, Element::Rt(_))).unwrap().display,
    Some(Display::RubyText)
  ));
  assert!(matches!(
    find_style(root, &|el| matches!(el, Element::Rp(_))).unwrap().display,
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
  let tree = lui_parser::parse(
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
  assert!(matches!(bg, CssColor::Named(s) if &**s == "blue"));
}

#[test]
fn linked_stylesheet_applies_when_host_registered() {
  let mut tree = lui_parser::parse(
    r#"
        <link rel="stylesheet" href="devtools.css">
        <div id="box"></div>
        "#,
  );
  tree.register_linked_stylesheet("devtools.css", "#box { width: 42px; }");

  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let div = find_style(
    root,
    &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
  )
  .unwrap();
  assert!(matches!(div.width, Some(CssLength::Px(v)) if (v - 42.0).abs() < 0.01));
}

#[test]
fn linked_stylesheet_media_attribute_gates_source() {
  let mut tree = lui_parser::parse(
    r#"
        <link rel="stylesheet" href="narrow.css" media="(max-width: 500px)">
        <div id="box"></div>
        "#,
  );
  tree.register_linked_stylesheet("narrow.css", "#box { width: 64px; }");

  let small = cascade_with_media(&tree, &MediaContext::screen(400.0, 800.0, 1.0));
  let root = small.root.as_ref().unwrap();
  let div = find_style(
    root,
    &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
  )
  .unwrap();
  assert!(matches!(div.width, Some(CssLength::Px(v)) if (v - 64.0).abs() < 0.01));

  let large = cascade_with_media(&tree, &MediaContext::screen(900.0, 800.0, 1.0));
  let root = large.root.as_ref().unwrap();
  let div = find_style(
    root,
    &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("box")),
  )
  .unwrap();
  assert!(div.width.is_none());
}

#[test]
fn inserted_template_content_participates_in_cascade() {
  let mut tree = lui_parser::parse(
    r#"
        <style>.card { width: 77px; }</style>
        <template id="tpl"><div class="card"></div></template>
        <div id="host"></div>
        "#,
  );
  tree
    .append_template_content_to_id("tpl", "host")
    .expect("template inserted");

  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let host = root
    .children
    .iter()
    .find(|node| matches!(&node.element, Element::Div(d) if d.id.as_deref() == Some("host")))
    .expect("host");
  assert_eq!(host.children.len(), 1);
  assert!(matches!(host.children[0].style.width, Some(CssLength::Px(v)) if (v - 77.0).abs() < 0.01));
}

#[test]
fn media_query_rules_match_viewport_context() {
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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

  let hidden_div = find_style(root, &|el| matches!(el, Element::Div(d) if d.hidden == Some(true))).unwrap();
  assert!(matches!(hidden_div.display, Some(Display::None)));

  let closed_dialog = find_style(root, &|el| matches!(el, Element::Dialog(d) if d.open != Some(true))).unwrap();
  assert!(matches!(closed_dialog.display, Some(Display::None)));

  let open_dialog = find_style(root, &|el| matches!(el, Element::Dialog(d) if d.open == Some(true))).unwrap();
  assert!(matches!(open_dialog.display, Some(Display::Block)));

  let hidden_input = find_style(root, &|el| {
    matches!(
        el,
        Element::Input(i)
            if matches!(
                i.r#type,
                Some(lui_models::common::html_enums::InputType::Hidden)
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
                Some(lui_models::common::html_enums::InputType::Submit)
            )
    )
  })
  .unwrap();
  assert!(matches!(submit_input.display, Some(Display::InlineBlock)));
  assert!(matches!(submit_input.cursor, Some(Cursor::Default)));
  assert!(matches!(submit_input.box_sizing, Some(BoxSizing::BorderBox)));

  let abbr = find_style(root, &|el| matches!(el, Element::Abbr(_))).unwrap();
  assert_eq!(abbr.text_decoration.as_deref(), Some("underline dotted"));

  let rtl = find_style(root, &|el| matches!(el, Element::Div(d) if d.dir.is_some())).unwrap();
  assert_eq!(rtl.deferred_longhands.get("direction").map(|s| &**s), Some("rtl"));
}

#[test]
fn ua_form_font_initial_resets_inherited_text_styles() {
  let tree = lui_parser::parse(
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
  let input = find_style(cascaded.root.as_ref().unwrap(), &|el| matches!(el, Element::Input(_))).unwrap();
  assert!(input.font_size.is_some(), "UA should set explicit font-size on form controls");
  assert!(matches!(input.color, Some(CssColor::Named(ref v)) if &**v == "fieldtext"));
  assert!(input.font_weight.is_some(), "UA should reset font-weight on form controls");
  assert!(input.font_style.is_some(), "UA should reset font-style on form controls");
  assert!(matches!(input.line_height, Some(CssLength::Raw(ref v)) if &**v == "normal"));
  assert!(matches!(
      input.letter_spacing,
      Some(CssLength::Raw(ref v)) if &**v == "normal"
  ));
  assert!(matches!(input.text_align, Some(TextAlign::Start)));
}

#[test]
fn cascade_range_input_overrides_generic_input_styles() {
  use lui_models::common::html_enums::InputType;
  let tree = lui_parser::parse(r#"<input type="range" />"#);
  let cascaded = cascade(&tree);
  let style = find_style(cascaded.root.as_ref().unwrap(), &|el| {
    matches!(el, Element::Input(inp) if matches!(inp.r#type, Some(InputType::Range)))
  })
  .expect("should find range input");

  eprintln!("range padding_left: {:?}", style.padding_left);
  eprintln!("range padding_right: {:?}", style.padding_right);
  eprintln!("range border_left_width: {:?}", style.border_left_width);
  eprintln!("range border_right_width: {:?}", style.border_right_width);
  eprintln!("range border_left_style: {:?}", style.border_left_style);
  eprintln!("range background_color: {:?}", style.background_color);

  // Range should have no padding (overriding input's padding: 1px 2px)
  assert!(
    style.padding_left.is_none()
      || matches!(&style.padding_left, Some(CssLength::Px(v)) if *v == 0.0)
      || matches!(&style.padding_left, Some(CssLength::Zero)),
    "range padding_left should be 0, got {:?}",
    style.padding_left
  );

  // Range should have no border (overriding input's border: 2px inset)
  assert!(
    style.border_left_width.is_none()
      || matches!(&style.border_left_width, Some(CssLength::Px(v)) if *v == 0.0)
      || matches!(&style.border_left_width, Some(CssLength::Zero)),
    "range border_left_width should be 0/none, got {:?}",
    style.border_left_width
  );
}

// ── Advanced selector tests ───────────────────────────────────────────────
// These verify that the cascade delegates to query.rs's full CSS4 matching.

#[test]
fn cascade_child_combinator() {
  let tree = lui_parser::parse(
    r#"
    <style>
      .parent > .child { width: 42px; }
    </style>
    <div class="parent">
      <div class="child" id="direct"></div>
      <div>
        <div class="child" id="nested"></div>
      </div>
    </div>
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let direct = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("direct"))).unwrap();
  let nested = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("nested"))).unwrap();
  assert!(matches!(direct.width, Some(CssLength::Px(v)) if (v - 42.0).abs() < 0.01));
  assert!(nested.width.is_none());
}

#[test]
fn cascade_adjacent_sibling_combinator() {
  let tree = lui_parser::parse(
    r#"
    <style>
      .a + .b { height: 10px; }
    </style>
    <div>
      <div class="a"></div>
      <div class="b" id="adjacent"></div>
      <div class="b" id="nonadjacent"></div>
    </div>
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let adj = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("adjacent"))).unwrap();
  let nonadj = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("nonadjacent"))).unwrap();
  assert!(matches!(adj.height, Some(CssLength::Px(v)) if (v - 10.0).abs() < 0.01));
  assert!(nonadj.height.is_none());
}

#[test]
fn cascade_general_sibling_combinator() {
  let tree = lui_parser::parse(
    r#"
    <style>
      .a ~ .c { width: 5px; }
    </style>
    <div>
      <div class="a"></div>
      <div class="b"></div>
      <div class="c" id="sib"></div>
    </div>
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let sib = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("sib"))).unwrap();
  assert!(matches!(sib.width, Some(CssLength::Px(v)) if (v - 5.0).abs() < 0.01));
}

#[test]
fn cascade_nth_child_pseudo_class() {
  let tree = lui_parser::parse(
    r#"
    <style>
      .list > div:nth-child(2) { width: 99px; }
    </style>
    <div class="list">
      <div id="first"></div>
      <div id="second"></div>
      <div id="third"></div>
    </div>
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let first = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("first"))).unwrap();
  let second = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("second"))).unwrap();
  let third = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("third"))).unwrap();
  assert!(first.width.is_none());
  assert!(matches!(second.width, Some(CssLength::Px(v)) if (v - 99.0).abs() < 0.01));
  assert!(third.width.is_none());
}

#[test]
fn cascade_not_pseudo_class() {
  let tree = lui_parser::parse(
    r#"
    <style>
      div:not(.skip) { height: 77px; }
    </style>
    <div id="yes"></div>
    <div id="no" class="skip"></div>
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let yes = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("yes"))).unwrap();
  let no = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("no"))).unwrap();
  assert!(matches!(yes.height, Some(CssLength::Px(v)) if (v - 77.0).abs() < 0.01));
  assert!(no.height.is_none());
}

#[test]
fn cascade_is_pseudo_class() {
  let tree = lui_parser::parse(
    r#"
    <style>
      :is(.a, .b) { width: 11px; }
    </style>
    <div class="a" id="a"></div>
    <div class="b" id="b"></div>
    <div class="c" id="c"></div>
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let a = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("a"))).unwrap();
  let b = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("b"))).unwrap();
  let c = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("c"))).unwrap();
  assert!(matches!(a.width, Some(CssLength::Px(v)) if (v - 11.0).abs() < 0.01));
  assert!(matches!(b.width, Some(CssLength::Px(v)) if (v - 11.0).abs() < 0.01));
  assert!(c.width.is_none());
}

#[test]
fn cascade_attribute_selector() {
  let tree = lui_parser::parse(
    r#"
    <style>
      input[type="hidden"] { display: none; }
    </style>
    <input type="hidden" id="h">
    <input type="text" id="t">
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let hidden = find_style(root, &|el| matches!(el, Element::Input(i) if i.id.as_deref() == Some("h"))).unwrap();
  let text = find_style(root, &|el| matches!(el, Element::Input(i) if i.id.as_deref() == Some("t"))).unwrap();
  assert!(matches!(hidden.display, Some(Display::None)));
  assert!(!matches!(text.display, Some(Display::None)));
}

#[test]
fn cascade_disabled_pseudo_class() {
  let tree = lui_parser::parse(
    r#"
    <style>
      input:disabled { width: 50px; }
    </style>
    <input disabled id="dis">
    <input id="en">
    "#,
  );
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let dis = find_style(root, &|el| matches!(el, Element::Input(i) if i.id.as_deref() == Some("dis"))).unwrap();
  let en = find_style(root, &|el| matches!(el, Element::Input(i) if i.id.as_deref() == Some("en"))).unwrap();
  assert!(matches!(dis.width, Some(CssLength::Px(v)) if (v - 50.0).abs() < 0.01));
  assert!(en.width.is_none());
}

#[test]
fn cascade_focus_within_pseudo_class() {
  let mut tree = lui_parser::parse(
    r#"
    <style>
      .container:focus-within { width: 200px; }
    </style>
    <div class="container" id="c">
      <input id="inp">
    </div>
    "#,
  );
  tree.interaction.focus_path = Some(vec![0, 0]);
  let cascaded = cascade(&tree);
  let root = cascaded.root.as_ref().unwrap();
  let container = find_style(root, &|el| matches!(el, Element::Div(d) if d.id.as_deref() == Some("c"))).unwrap();
  assert!(matches!(container.width, Some(CssLength::Px(v)) if (v - 200.0).abs() < 0.01));
}
