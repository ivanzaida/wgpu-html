use std::collections::{HashMap, HashSet};

use super::apply_css_property;
use crate::{style::Style, values::ArcStr};

/// Returns `true` if the value string contains a `var(` token that
/// isn't inside a quoted string. This is a conservative check.
pub(crate) fn value_contains_var(value: &str) -> bool {
  let bytes = value.as_bytes();
  let mut i = 0;
  let mut quote: Option<u8> = None;
  while i < bytes.len() {
    match quote {
      Some(q) => {
        if bytes[i] == q {
          quote = None;
        }
      }
      None => match bytes[i] {
        b'"' | b'\'' => quote = Some(bytes[i]),
        b'v' | b'V' => {
          if i + 4 <= bytes.len()
            && bytes[i + 1].to_ascii_lowercase() == b'a'
            && bytes[i + 2].to_ascii_lowercase() == b'r'
            && bytes[i + 3] == b'('
          {
            return true;
          }
        }
        _ => {}
      },
    }
    i += 1;
  }
  false
}

/// Resolve all `var()` references in a fully-cascaded, inherited style.
///
/// Phase 1: resolve `var()` inside custom-property values so that
/// `--a: var(--b)` chains collapse.
/// Phase 2: for every entry in `var_properties`, substitute variables
/// and re-parse the resolved value through `apply_css_property`.
pub fn resolve_var_references(style: &mut Style) {
  // Phase 1 -- resolve var() inside custom property values.
  let keys: Vec<ArcStr> = style.custom_properties.keys().cloned().collect();
  let mut resolved_cp = style.custom_properties.clone();
  for key in &keys {
    let mut resolving = HashSet::new();
    if let Some(val) = resolved_cp.get(key.as_ref()).cloned() {
      if value_contains_var(&val) {
        resolving.insert(key.to_string());
        let substituted = substitute_vars(&val, &resolved_cp, &mut resolving);
        resolved_cp.insert(key.clone(), ArcStr::from(substituted.as_str()));
      }
    }
  }
  style.custom_properties = resolved_cp;

  // Phase 2 -- resolve var() in regular property declarations.
  let pending: Vec<(ArcStr, ArcStr)> = style.var_properties.drain().collect();
  for (prop, raw_value) in pending {
    let mut resolving = HashSet::new();
    let resolved = substitute_vars(&raw_value, &style.custom_properties, &mut resolving);
    if !resolved.is_empty() {
      apply_css_property(style, &prop, &resolved);
    }
  }
}

/// Replace all `var(--name)` and `var(--name, fallback)` occurrences
/// in `value` with the corresponding custom-property value. Detects
/// cycles via `resolving` and falls back gracefully.
fn substitute_vars(value: &str, custom_props: &HashMap<ArcStr, ArcStr>, resolving: &mut HashSet<String>) -> String {
  let bytes = value.as_bytes();
  let mut out = String::with_capacity(value.len());
  let mut i = 0;
  let mut quote: Option<u8> = None;

  while i < bytes.len() {
    match quote {
      Some(q) => {
        out.push(bytes[i] as char);
        if bytes[i] == q {
          quote = None;
        }
        i += 1;
      }
      None => {
        if (bytes[i] == b'v' || bytes[i] == b'V')
          && i + 4 <= bytes.len()
          && bytes[i + 1].to_ascii_lowercase() == b'a'
          && bytes[i + 2].to_ascii_lowercase() == b'r'
          && bytes[i + 3] == b'('
        {
          // Found `var(` -- parse the contents.
          let start = i;
          i += 4; // skip "var("
                  // Find matching `)`.
          let mut depth = 1i32;
          let inner_start = i;
          while i < bytes.len() && depth > 0 {
            match bytes[i] {
              b'(' => depth += 1,
              b')' => depth -= 1,
              b'"' | b'\'' => {
                let q = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != q {
                  i += 1;
                }
              }
              _ => {}
            }
            if depth > 0 {
              i += 1;
            }
          }
          if depth != 0 {
            // Unbalanced -- emit raw.
            out.push_str(&value[start..]);
            break;
          }
          let inner = value[inner_start..i].trim();
          i += 1; // skip ')'

          // Split inner into name and optional fallback.
          let (name, fallback) = split_var_args(inner);
          let name = name.trim();

          if resolving.contains(name) {
            // Cycle -- use fallback.
            if let Some(fb) = fallback {
              out.push_str(&substitute_vars(fb.trim(), custom_props, resolving));
            }
          } else if let Some(cp_val) = custom_props.get(name) {
            let mut resolved = cp_val.to_string();
            if value_contains_var(&resolved) {
              resolving.insert(name.to_owned());
              resolved = substitute_vars(&resolved, custom_props, resolving);
              resolving.remove(name);
            }
            out.push_str(&resolved);
          } else if let Some(fb) = fallback {
            out.push_str(&substitute_vars(fb.trim(), custom_props, resolving));
          }
          // If no value and no fallback, nothing is appended
          // (guaranteed-invalid per CSS spec).
        } else {
          if bytes[i] == b'"' || bytes[i] == b'\'' {
            quote = Some(bytes[i]);
          }
          out.push(bytes[i] as char);
          i += 1;
        }
      }
    }
  }
  out
}

/// Split the argument inside `var(...)` into (name, optional_fallback).
/// The fallback is everything after the first top-level `,`.
fn split_var_args(s: &str) -> (&str, Option<&str>) {
  let bytes = s.as_bytes();
  let mut depth = 0i32;
  let mut i = 0;
  while i < bytes.len() {
    match bytes[i] {
      b'(' => depth += 1,
      b')' => depth -= 1,
      b',' if depth == 0 => {
        return (&s[..i], Some(&s[i + 1..]));
      }
      _ => {}
    }
    i += 1;
  }
  (s, None)
}
