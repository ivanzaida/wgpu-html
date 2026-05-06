use super::helpers::*;
use crate::*;
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
  assert!(matches!(bg, CssColor::Named(ref s) if &**s == "red"));
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
  assert!(matches!(bg, CssColor::Named(ref s) if &**s == "red"));
}

#[test]
fn inline_beats_id() {
  let sheet = parse_stylesheet("#hero { background-color: blue; }");
  let mut div = wgpu_html_models::Div::default();
  div.id = Some("hero".into());
  div.style = Some("background-color: red;".into());
  let style = computed_style(&Element::Div(div), &sheet);
  let bg = style.background_color.expect("set");
  assert!(matches!(bg, CssColor::Named(ref s) if &**s == "red"));
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
      Some(CssColor::Hex(ref s)) if &**s == "#1b1d22"
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
  assert!(matches!(div.style.line_height, Some(CssLength::Raw(ref v)) if &**v == "normal"));
  assert_eq!(
    div.style.deferred_longhands.get("font-variant").map(|s| &**s),
    Some("normal")
  );
  assert_eq!(
    div.style.deferred_longhands.get("font-stretch").map(|s| &**s),
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
  assert!(matches!(bg, CssColor::Named(ref s) if &**s == "red"));
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
