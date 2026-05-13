//! Math-only function resolution (`calc`, `min`, `max`, `clamp`).
//! Does NOT resolve relative units — use the full [`crate::ResolutionContext`]
//! for unit-aware resolution.

use lui_core::{CssFunction, CssValue};

use crate::math_helpers::MathValue;

/// Resolve math functions without unit conversion.
/// Used by the cascade after `var()` resolution but before unit resolution.
pub fn resolve_math(value: &CssValue) -> CssValue {
    match value {
        CssValue::Function { function, args } => match function {
            CssFunction::Calc => resolve_calc(args),
            CssFunction::Min => resolve_min_max(args, true),
            CssFunction::Max => resolve_min_max(args, false),
            CssFunction::Clamp => resolve_clamp(args),
            _ => {
                let resolved_args: Vec<CssValue> = args.iter().map(resolve_math).collect();
                CssValue::Function { function: function.clone(), args: resolved_args }
            }
        },
        _ => value.clone(),
    }
}

fn resolve_calc(args: &[CssValue]) -> CssValue {
    if args.is_empty() { return CssValue::Number(0.0); }
    let mut values: Vec<MathValue> = Vec::new();
    let mut ops: Vec<char> = Vec::new();
    for term in args {
        match &term {
            CssValue::Unknown(op) if is_operator(op) => ops.push(op.chars().next().unwrap_or('+')),
            _ => {
                let val = to_math_value(term);
                match ops.last() {
                    Some('-') => { values.push(val.negate()); ops.pop(); }
                    _ => values.push(val),
                }
            }
        }
    }
    if values.is_empty() { return CssValue::Number(0.0); }
    if values.len() == 1 { return values[0].to_css_value(); }
    values.into_iter().reduce(|a, b| a.add(b)).unwrap().to_css_value()
}

fn to_math_value(value: &CssValue) -> MathValue {
    match value {
        CssValue::Number(n) => MathValue::Number(*n),
        CssValue::Percentage(n) => MathValue::Percentage(*n),
        CssValue::Dimension { value, unit } => MathValue::Dimension(*value, unit.clone()),
        CssValue::Function { function, args } => {
            let v = match function {
                CssFunction::Calc => resolve_calc(args),
                CssFunction::Min => resolve_min_max(args, true),
                CssFunction::Max => resolve_min_max(args, false),
                CssFunction::Clamp => resolve_clamp(args),
                _ => CssValue::Function { function: function.clone(), args: args.iter().map(resolve_math).collect() },
            };
            to_math_value(&v)
        }
        _ => MathValue::Number(0.0),
    }
}

fn resolve_min_max(args: &[CssValue], is_min: bool) -> CssValue {
    let mut resolved: Vec<CssValue> = args.iter().map(resolve_math).collect();
    let comparable: Vec<MathValue> = resolved.iter().map(|v| to_math_value_simple(v)).collect();
    let best_idx = if is_min {
        comparable.iter().enumerate().min_by(|(_, a), (_, b)| f64::total_cmp(&a.as_f64(), &b.as_f64()))
    } else {
        comparable.iter().enumerate().max_by(|(_, a), (_, b)| f64::total_cmp(&a.as_f64(), &b.as_f64()))
    };
    match best_idx { Some((idx, _)) => resolved.swap_remove(idx), None => CssValue::Number(0.0) }
}

fn to_math_value_simple(v: &CssValue) -> MathValue {
    match v {
        CssValue::Number(n) => MathValue::Number(*n),
        CssValue::Percentage(n) => MathValue::Percentage(*n),
        CssValue::Dimension { value, unit } => MathValue::Dimension(*value, unit.clone()),
        _ => MathValue::Number(0.0),
    }
}

fn resolve_clamp(args: &[CssValue]) -> CssValue {
    if args.len() < 3 { return CssValue::Number(0.0); }
    let min = resolve_math(&args[0]);
    let val = resolve_math(&args[1]);
    let max = resolve_math(&args[2]);
    let mn = simple_f64(&min);
    let vl = simple_f64(&val);
    let mx = simple_f64(&max);
    if vl < mn { min } else if vl > mx { max } else { val }
}

fn simple_f64(v: &CssValue) -> f64 {
    match v {
        CssValue::Number(n) | CssValue::Percentage(n) => *n,
        CssValue::Dimension { value, .. } => *value,
        _ => 0.0,
    }
}

fn is_operator(s: &str) -> bool {
    s == "+" || s == "-" || s == "*" || s == "/"
}
