use super::helpers::*;
// --------------------------------------------------------------------------

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
