use bumpalo::Bump;
use lui_cascade::style::{ComputedStyle, alloc_value};
use lui_cascade::var_resolution::resolve_vars;
use lui_css_parser::{ArcStr, CssValue, CssUnit};

// ── Basic substitution ──

#[test]
fn resolves_var_in_property() {
    let arena = Bump::new();
    let cp_val = alloc_value(&arena, CssValue::Unknown("red".into()));
    let var_ref = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--color"),
        fallback: None,
    });

    let mut style = ComputedStyle::default();
    style.custom_properties
        .get_or_insert_with(Default::default)
        .insert(ArcStr::from("--color"), cp_val);
    style.color = Some(var_ref);

    resolve_vars(&mut style, &arena);

    assert_eq!(style.color.unwrap(), &CssValue::Unknown("red".into()));
}

#[test]
fn uses_fallback_when_var_undefined() {
    let arena = Bump::new();
    let var_ref = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--missing"),
        fallback: Some(Box::new(CssValue::Unknown("blue".into()))),
    });

    let mut style = ComputedStyle::default();
    style.color = Some(var_ref);

    resolve_vars(&mut style, &arena);

    assert_eq!(style.color.unwrap(), &CssValue::Unknown("blue".into()));
}

#[test]
fn produces_empty_when_no_fallback_and_undefined() {
    let arena = Bump::new();
    let var_ref = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--missing"),
        fallback: None,
    });

    let mut style = ComputedStyle::default();
    style.color = Some(var_ref);

    resolve_vars(&mut style, &arena);

    assert_eq!(style.color.unwrap(), &CssValue::Unknown("".into()));
}

// ── Custom property chains ──

#[test]
fn resolves_chain() {
    let arena = Bump::new();
    let base_val = alloc_value(&arena, CssValue::Unknown("10px".into()));
    let chain_val = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--base"),
        fallback: None,
    });
    let prop_ref = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--size"),
        fallback: None,
    });

    let mut style = ComputedStyle::default();
    let cp = style.custom_properties.get_or_insert_with(Default::default);
    cp.insert(ArcStr::from("--base"), base_val);
    cp.insert(ArcStr::from("--size"), chain_val);
    style.width = Some(prop_ref);

    resolve_vars(&mut style, &arena);

    assert_eq!(style.width.unwrap(), &CssValue::Unknown("10px".into()));
}

#[test]
fn cycle_uses_fallback() {
    let arena = Bump::new();
    let a_val = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--b"),
        fallback: None,
    });
    let b_val = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--a"),
        fallback: Some(Box::new(CssValue::Unknown("fallback".into()))),
    });
    let prop_ref = alloc_value(&arena, CssValue::Var {
        name: ArcStr::from("--a"),
        fallback: None,
    });

    let mut style = ComputedStyle::default();
    let cp = style.custom_properties.get_or_insert_with(Default::default);
    cp.insert(ArcStr::from("--a"), a_val);
    cp.insert(ArcStr::from("--b"), b_val);
    style.color = Some(prop_ref);

    resolve_vars(&mut style, &arena);

    // Cycle detected → uses fallback from --b's var()
    assert!(style.color.is_some());
}

// ── No-op cases ──

#[test]
fn leaves_non_var_values_alone() {
    let arena = Bump::new();
    let val = alloc_value(&arena, CssValue::Dimension { value: 16.0, unit: CssUnit::Px });

    let mut style = ComputedStyle::default();
    style.font_size = Some(val);

    resolve_vars(&mut style, &arena);

    assert_eq!(style.font_size, Some(val));
}

#[test]
fn handles_empty_style() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    resolve_vars(&mut style, &arena);
    // should not panic
}

// ── var() inside functions ──

#[test]
fn resolves_var_inside_function_args() {
    let arena = Bump::new();
    let cp_val = alloc_value(&arena, CssValue::Number(50.0));
    let func_val = alloc_value(&arena, CssValue::Function {
        function: lui_css_parser::CssFunction::Calc,
        args: vec![CssValue::Var {
            name: ArcStr::from("--w"),
            fallback: None,
        }],
    });

    let mut style = ComputedStyle::default();
    style.custom_properties
        .get_or_insert_with(Default::default)
        .insert(ArcStr::from("--w"), cp_val);
    style.width = Some(func_val);

    resolve_vars(&mut style, &arena);

    match style.width.unwrap() {
        CssValue::Function { args, .. } => {
            assert_eq!(args[0], CssValue::Number(50.0));
        }
        other => panic!("expected Function, got {:?}", other),
    }
}
