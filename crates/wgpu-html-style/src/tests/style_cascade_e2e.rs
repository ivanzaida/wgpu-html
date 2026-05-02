use crate::*;
use super::helpers::*;
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
fn linked_stylesheet_applies_when_host_registered() {
  let mut tree = wgpu_html_parser::parse(
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
  let mut tree = wgpu_html_parser::parse(
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
  let mut tree = wgpu_html_parser::parse(
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
  assert!(matches!(submit_input.box_sizing, Some(BoxSizing::BorderBox)));

  let abbr = find_style(root, &|el| matches!(el, Element::Abbr(_))).unwrap();
  assert_eq!(abbr.text_decoration.as_deref(), Some("underline dotted"));

  let rtl = find_style(root, &|el| matches!(el, Element::Div(d) if d.dir.is_some())).unwrap();
  assert_eq!(rtl.deferred_longhands.get("direction").map(String::as_str), Some("rtl"));
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
  let input = find_style(cascaded.root.as_ref().unwrap(), &|el| matches!(el, Element::Input(_))).unwrap();
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
