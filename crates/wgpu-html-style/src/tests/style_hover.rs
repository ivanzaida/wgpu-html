use crate::*;
use super::helpers::*;
// --------------------------------------------------------------------------

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
  let style = find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("b")).expect("found");
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
  let style = find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("b")).expect("found");
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
  let outer_style = find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("outer")).expect("found");
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
  let style = find_style(&cascaded.root.unwrap(), &|el| matches!(el, Element::Div(_))).expect("found");
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
  let inner_style = find_style(&cascaded.root.as_ref().unwrap(), &|el| element_id(el) == Some("inner")).expect("found");
  let inner_bg = inner_style.background_color.expect("set");
  assert!(
    matches!(inner_bg.clone(), CssColor::Named(s) if s == "red"),
    "inner background expected red, got {inner_bg:?}"
  );

  // Outer is NOT focused (only its descendant is). Plain rule wins.
  let outer_style = find_style(&cascaded.root.as_ref().unwrap(), &|el| element_id(el) == Some("outer")).expect("found");
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
  let style = find_style(&cascaded.root.unwrap(), &|el| element_id(el) == Some("x")).expect("found");
  let bg = style.background_color.expect("set");
  assert!(matches!(bg, CssColor::Named(s) if s == "white"));
}
