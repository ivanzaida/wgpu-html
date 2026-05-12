//! Math function resolution: `calc()`, `min()`, `max()`, `clamp()`.
//! Does NOT resolve relative units — use `units::resolve_units` for that.

use lui_css_parser::{CssFunction, CssValue};

use crate::math_helpers::{MathValue, to_f64, to_math_value};

pub fn resolve_math(value: &CssValue) -> CssValue {
    match value {
        CssValue::Function { function, args } => {
            match function {
                CssFunction::Calc => resolve_calc(args),
                CssFunction::Min => resolve_min_max(args, true),
                CssFunction::Max => resolve_min_max(args, false),
                CssFunction::Clamp => resolve_clamp(args),
                _ => {
                    let resolved_args: Vec<CssValue> = args.iter().map(resolve_math).collect();
                    CssValue::Function { function: function.clone(), args: resolved_args }
                }
            }
        }
        _ => value.clone(),
    }
}

// ---------------------------------------------------------------------------
// calc()
// ---------------------------------------------------------------------------

fn resolve_calc(args: &[CssValue]) -> CssValue {
    if args.is_empty() { return CssValue::Number(0.0); }
    evaluate_calc_sum(args)
}

fn evaluate_calc_sum(terms: &[CssValue]) -> CssValue {
    let mut values: Vec<MathValue> = Vec::new();
    let mut ops: Vec<char> = Vec::new();

    for term in terms {
        match &term {
            CssValue::Unknown(op) if is_operator(op) => {
                ops.push(op.chars().next().unwrap_or('+'));
            }
            _ => {
                let val = resolve_calc_value(term);
                match ops.last() {
                    Some('-') => { values.push(val.negate()); ops.pop(); }
                    _ => values.push(val),
                }
            }
        }
    }

    if values.is_empty() { return CssValue::Number(0.0); }
    if values.len() == 1 { return values[0].to_css_value(); }

    let result = values.into_iter().reduce(|a, b| a.add(b)).unwrap();
    result.to_css_value()
}

fn resolve_calc_value(value: &CssValue) -> MathValue {
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
            resolve_calc_value(&v)
        }
        _ => MathValue::Number(0.0),
    }
}

// ---------------------------------------------------------------------------
// min() / max()
// ---------------------------------------------------------------------------

fn resolve_min_max(args: &[CssValue], is_min: bool) -> CssValue {
    let mut resolved: Vec<CssValue> = args.iter().map(resolve_math).collect();
    let comparable: Vec<MathValue> = resolved.iter().map(to_math_value).collect();
    let best_idx = if is_min {
        comparable.iter().enumerate().min_by(|(_, a), (_, b)| f64::total_cmp(&a.as_f64(), &b.as_f64()))
    } else {
        comparable.iter().enumerate().max_by(|(_, a), (_, b)| f64::total_cmp(&a.as_f64(), &b.as_f64()))
    };
    match best_idx { Some((idx, _)) => resolved.swap_remove(idx), None => CssValue::Number(0.0) }
}

// ---------------------------------------------------------------------------
// clamp()
// ---------------------------------------------------------------------------

fn resolve_clamp(args: &[CssValue]) -> CssValue {
    if args.len() < 3 { return CssValue::Number(0.0); }
    let min = resolve_math(&args[0]);
    let val = resolve_math(&args[1]);
    let max = resolve_math(&args[2]);
    let mn = to_f64(&min);
    let vl = to_f64(&val);
    let mx = to_f64(&max);
    if vl < mn { min } else if vl > mx { max } else { val }
}

fn is_operator(s: &str) -> bool {
    s == "+" || s == "-" || s == "*" || s == "/"
}
