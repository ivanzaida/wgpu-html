use bumpalo::Bump;
use lui_cascade::{
  style::{ComputedStyle, alloc_value},
  var_resolution::resolve_vars,
};
use lui_core::{ArcStr, CssValue};

#[test]
fn var_resolves_from_inherited_custom_property() {
  let arena = Bump::new();

  // Parent defines --color: red
  let red = alloc_value(&arena, CssValue::Unknown("red".into()));
  let mut parent = ComputedStyle::default();
  parent
    .custom_properties
    .get_or_insert_with(Default::default)
    .insert(ArcStr::from("--color"), red);

  // Child uses color: var(--color) but doesn't define --color itself
  let var_ref = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--color"),
      fallback: None,
    },
  );
  let mut child = ComputedStyle::default();
  child.color = Some(var_ref);

  // Inherit, then resolve
  child.inherit_from(&parent);
  resolve_vars(&mut child, &arena);

  assert_eq!(child.color.unwrap(), &CssValue::Unknown("red".into()));
}

#[test]
fn child_custom_property_overrides_inherited() {
  let arena = Bump::new();

  // Parent defines --size: 10px
  let parent_val = alloc_value(&arena, CssValue::Unknown("10px".into()));
  let mut parent = ComputedStyle::default();
  parent
    .custom_properties
    .get_or_insert_with(Default::default)
    .insert(ArcStr::from("--size"), parent_val);

  // Child redefines --size: 20px and uses width: var(--size)
  let child_val = alloc_value(&arena, CssValue::Unknown("20px".into()));
  let var_ref = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--size"),
      fallback: None,
    },
  );
  let mut child = ComputedStyle::default();
  child
    .custom_properties
    .get_or_insert_with(Default::default)
    .insert(ArcStr::from("--size"), child_val);
  child.width = Some(var_ref);

  // Inherit (child's --size should win), then resolve
  child.inherit_from(&parent);
  resolve_vars(&mut child, &arena);

  assert_eq!(child.width.unwrap(), &CssValue::Unknown("20px".into()));
}

#[test]
fn var_chain_across_inheritance() {
  let arena = Bump::new();

  // Grandparent defines --base: 8px
  let base = alloc_value(&arena, CssValue::Unknown("8px".into()));
  let mut grandparent = ComputedStyle::default();
  grandparent
    .custom_properties
    .get_or_insert_with(Default::default)
    .insert(ArcStr::from("--base"), base);

  // Parent defines --gap: var(--base), inherits --base
  let gap_var = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--base"),
      fallback: None,
    },
  );
  let mut parent = ComputedStyle::default();
  parent
    .custom_properties
    .get_or_insert_with(Default::default)
    .insert(ArcStr::from("--gap"), gap_var);
  parent.inherit_from(&grandparent);

  // Child uses row-gap: var(--gap), inherits both --base and --gap
  let row_gap_var = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--gap"),
      fallback: None,
    },
  );
  let mut child = ComputedStyle::default();
  child.row_gap = Some(row_gap_var);

  child.inherit_from(&parent);
  resolve_vars(&mut child, &arena);

  assert_eq!(child.row_gap.unwrap(), &CssValue::Unknown("8px".into()));
}

#[test]
fn inherited_var_with_fallback_when_parent_missing() {
  let arena = Bump::new();

  let parent = ComputedStyle::default();

  let var_ref = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--undefined"),
      fallback: Some(Box::new(CssValue::Unknown("16px".into()))),
    },
  );
  let mut child = ComputedStyle::default();
  child.font_size = Some(var_ref);

  child.inherit_from(&parent);
  resolve_vars(&mut child, &arena);

  assert_eq!(child.font_size.unwrap(), &CssValue::Unknown("16px".into()));
}

#[test]
fn multiple_vars_from_same_inherited_source() {
  let arena = Bump::new();

  let theme = alloc_value(&arena, CssValue::Unknown("blue".into()));
  let mut parent = ComputedStyle::default();
  parent
    .custom_properties
    .get_or_insert_with(Default::default)
    .insert(ArcStr::from("--theme"), theme);

  let color_var = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--theme"),
      fallback: None,
    },
  );
  let bg_var = alloc_value(
    &arena,
    CssValue::Var {
      name: ArcStr::from("--theme"),
      fallback: None,
    },
  );

  let mut child = ComputedStyle::default();
  child.color = Some(color_var);
  child.background_color = Some(bg_var);

  child.inherit_from(&parent);
  resolve_vars(&mut child, &arena);

  assert_eq!(child.color.unwrap(), &CssValue::Unknown("blue".into()));
  assert_eq!(child.background_color.unwrap(), &CssValue::Unknown("blue".into()));
}
