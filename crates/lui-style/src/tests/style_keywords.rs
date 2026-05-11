use super::helpers::*;
use crate::*;
// --------------------------------------------------------------------------
// CSS-wide keywords — `inherit` / `initial` / `unset`
// --------------------------------------------------------------------------

#[test]
fn inherit_keyword_takes_parent_value_for_non_inherited_property() {
  // `background-color` is *not* normally inherited. With `inherit`
  // on the child it must take the parent's value anyway.
  let tree = lui_parser::parse(
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
  assert!(matches!(bg, CssColor::Named(s) if &**s == "orange"));
}

#[test]
fn initial_keyword_blocks_implicit_inheritance() {
  // `color` is inherited. With `color: initial`, the child must
  // *not* take the parent's color — even though implicit
  // inheritance would otherwise fill it in.
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
  assert!(matches!(c, CssColor::Named(s) if &**s == "red"));
}

#[test]
fn unset_acts_as_initial_for_non_inherited_property() {
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
    r#"
        <style>
            div { transition-property: opacity; transition-duration: 200ms; }
            #x  { transition: initial; }
        </style>
        <div id="x"></div>
        "#,
  );
  let div = first_div(&tree);
  assert!(!div.style.deferred_longhands.contains_key("transition-property"));
  assert!(!div.style.deferred_longhands.contains_key("transition-duration"));
}

#[test]
fn shorthand_keyword_after_member_keyword_clears_member_keyword() {
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
    r#"
        <style>
            div { transition: initial; transition-duration: 200ms; }
        </style>
        <div></div>
        "#,
  );
  let div = first_div(&tree);
  assert_eq!(
    div.style.deferred_longhands.get("transition-duration").map(|s| &**s),
    Some("200ms")
  );
  assert!(!div.style.deferred_longhands.contains_key("transition-property"));
}

#[test]
fn all_initial_clears_typed_and_deferred_properties() {
  let tree = lui_parser::parse(
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
  let tree = lui_parser::parse(
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
    div.style.deferred_longhands.get("white-space-collapse").map(|s| &**s),
    Some("preserve")
  );
  assert_eq!(
    div.style.deferred_longhands.get("text-wrap-mode").map(|s| &**s),
    Some("wrap")
  );
}

#[test]
fn inherit_within_a_block_displaces_an_earlier_value() {
  // Within one rule, source-order resolves: `color: inherit;`
  // declared after `color: red;` must win.
  let tree = lui_parser::parse(
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
  assert!(matches!(c, CssColor::Named(s) if &**s == "green"));
}

#[test]
fn value_after_keyword_within_a_block_displaces_the_keyword() {
  // Reverse of the previous case: a normal value after a keyword
  // (same property, same block) wins.
  let tree = lui_parser::parse(
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
  assert!(matches!(c, CssColor::Named(s) if &**s == "red"));
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
  let tree = lui_parser::parse(
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
  assert!(matches!(c, CssColor::Named(s) if &**s == "green"));
}

#[test]
fn root_inherit_keyword_resolves_to_initial() {
  // The cascaded root has no parent, so `inherit` collapses to
  // initial (None in our impl). The synthetic body wrapping in the
  // parser places the styled element under a body that does have a
  // parent — so write it on the root html instead.
  let tree = lui_parser::parse(r#"<div style="color: inherit;"></div>"#);
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
