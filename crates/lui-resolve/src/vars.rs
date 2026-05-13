//! CSS custom property (`var()`) resolution.
//!
//! Resolves `var(--name)` references to their computed values using a
//! two-pass approach:
//! 1. Copy values without `var()` directly
//! 2. Resolve chains (--a: var(--b) where --b: 10px) with cycle detection

use bumpalo::Bump;
use lui_core::{ArcStr, CssValue};
use rustc_hash::{FxHashMap, FxHashSet};

/// Resolve `var()` references in a CSS value using resolved and raw
/// custom property maps. Recursively resolves chains and `var()` inside
/// function arguments. Allocated values live in `arena`.
pub fn resolve_var_value(
    value: &CssValue,
    resolved: &FxHashMap<ArcStr, &CssValue>,
    raw: &FxHashMap<ArcStr, &CssValue>,
    arena: &Bump,
) -> CssValue {
    resolve_var_impl(value, resolved, raw, arena, &mut FxHashSet::default())
}

fn resolve_var_impl<'a>(
    value: &CssValue,
    resolved: &FxHashMap<ArcStr, &'a CssValue>,
    raw: &FxHashMap<ArcStr, &'a CssValue>,
    arena: &'a Bump,
    resolving: &mut FxHashSet<ArcStr>,
) -> CssValue {
    let lookup = |name: &ArcStr| -> Option<&'a CssValue> {
        resolved.get(name).copied().or_else(|| raw.get(name).copied())
    };

    match value {
        CssValue::Var { name, fallback } => {
            if resolving.contains(name) {
                return match fallback {
                    Some(fb) => resolve_var_impl(fb, resolved, raw, arena, resolving),
                    None => CssValue::Unknown(ArcStr::default()),
                };
            }
            if let Some(cp_val) = lookup(name) {
                if !contains_var(cp_val) {
                    return cp_val.clone();
                }
                resolving.insert(name.clone());
                let result = resolve_var_impl(cp_val, resolved, raw, arena, resolving);
                resolving.remove(name);
                result
            } else {
                match fallback {
                    Some(fb) => resolve_var_impl(fb, resolved, raw, arena, resolving),
                    None => CssValue::Unknown(ArcStr::default()),
                }
            }
        }
        CssValue::Function { function, args } => {
            if args.iter().any(contains_var) {
                let new_args: Vec<CssValue> = args
                    .iter()
                    .map(|a| resolve_var_impl(a, resolved, raw, arena, resolving))
                    .collect();
                CssValue::Function { function: function.clone(), args: new_args }
            } else {
                value.clone()
            }
        }
        _ => value.clone(),
    }
}

/// Resolve custom property chains (--a: var(--b) where --b: 10px).
/// Returns a new map with all chains flattened.
pub fn resolve_custom_properties<'a>(
    raw: &FxHashMap<ArcStr, &'a CssValue>,
    arena: &'a Bump,
) -> FxHashMap<ArcStr, &'a CssValue> {
    let mut resolved: FxHashMap<ArcStr, &'a CssValue> = FxHashMap::default();

    for (name, value) in raw.iter() {
        if !contains_var(value) {
            resolved.insert(name.clone(), *value);
        }
    }

    for (name, value) in raw.iter() {
        if contains_var(value) {
            let substituted = resolve_var_value(value, &resolved, raw, arena);
            resolved.insert(name.clone(), arena.alloc(substituted));
        }
    }

    resolved
}

pub fn contains_var(value: &CssValue) -> bool {
    match value {
        CssValue::Var { .. } => true,
        CssValue::Function { args, .. } => args.iter().any(contains_var),
        _ => false,
    }
}
