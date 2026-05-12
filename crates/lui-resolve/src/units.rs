//! Unit resolution: converts relative units (`em`, `rem`, `vw`, `vh`, etc.)
//! and absolute units (`cm`, `mm`, `in`, `pt`, `pc`, `Q`) into `px`.
//! Non-length units (angles, time, frequency, resolution) are left as-is.

use lui_css_parser::{CssFunction, CssUnit, CssValue};

use crate::ResolverContext;
use crate::math::resolve_math;
use crate::math_helpers::{ResolvedNumber, to_f64};

/// Resolve relative and absolute length units to `px`.
/// Handles math functions with unit-aware evaluation.
pub fn resolve_units(value: &CssValue, ctx: &ResolverContext) -> CssValue {
    match value {
        CssValue::Number(_) | CssValue::String(_) | CssValue::Color(_)
        | CssValue::Url(_) | CssValue::Var { .. } => value.clone(),

        CssValue::Percentage(_) | CssValue::Unknown(_) => value.clone(),

        CssValue::Dimension { value, unit } => resolve_dimension(*value, unit, ctx),

        CssValue::Function { function, args } => {
            match function {
                CssFunction::Calc => resolve_calc_units(args, ctx),
                CssFunction::Min => resolve_min_max_units(args, ctx, true),
                CssFunction::Max => resolve_min_max_units(args, ctx, false),
                CssFunction::Clamp => resolve_clamp_units(args, ctx),
                _ => {
                    // Non-math functions: resolve math in their arguments only
                    let v = resolve_math(value);
                    v.clone()
                }
            }
        }
    }
}

pub fn resolve_dimension(value: f64, unit: &CssUnit, ctx: &ResolverContext) -> CssValue {
    match resolve_to_px(value, unit, ctx) {
        Some(px) => CssValue::Dimension { value: px, unit: CssUnit::Px },
        None => CssValue::Dimension { value, unit: unit.clone() },
    }
}

pub fn resolve_to_px(value: f64, unit: &CssUnit, ctx: &ResolverContext) -> Option<f64> {
    match unit {
        CssUnit::Px => Some(value),
        CssUnit::Cm => Some(value * 37.79527559055118),
        CssUnit::Mm => Some(value * 3.779527559055118),
        CssUnit::In => Some(value * 96.0),
        CssUnit::Pt => Some(value * 1.3333333333333333),
        CssUnit::Pc => Some(value * 16.0),
        CssUnit::Q  => Some(value * 2.362204724409449),
        CssUnit::Em  => Some(value * ctx.parent_font_size as f64),
        CssUnit::Rem => Some(value * ctx.root_font_size as f64),
        CssUnit::Ex  => Some(value * ctx.parent_font_size as f64 * 0.5),
        CssUnit::Ch  => Some(value * ctx.parent_font_size as f64 * 0.5),
        CssUnit::Vw   => Some(value * (ctx.viewport_width as f64) / 100.0),
        CssUnit::Vh   => Some(value * (ctx.viewport_height as f64) / 100.0),
        CssUnit::Vmin => Some(value * (ctx.viewport_width.min(ctx.viewport_height) as f64) / 100.0),
        CssUnit::Vmax => Some(value * (ctx.viewport_width.max(ctx.viewport_height) as f64) / 100.0),
        CssUnit::Vi | CssUnit::Vb => Some(value * (ctx.viewport_width as f64) / 100.0),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// calc() with unit-aware resolution
// ---------------------------------------------------------------------------

fn resolve_calc_units(args: &[CssValue], ctx: &ResolverContext) -> CssValue {
    if args.is_empty() { return CssValue::Number(0.0); }
    evaluate_calc_sum_units(args, ctx)
}

fn evaluate_calc_sum_units(terms: &[CssValue], ctx: &ResolverContext) -> CssValue {
    let mut values: Vec<ResolvedNumber> = Vec::new();
    let mut ops: Vec<char> = Vec::new();
    for term in terms {
        match &term {
            CssValue::Unknown(op) if is_op(op) => ops.push(op.chars().next().unwrap_or('+')),
            _ => {
                let val = resolve_calc_value_units(term, ctx);
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

fn resolve_calc_value_units(value: &CssValue, ctx: &ResolverContext) -> ResolvedNumber {
    match value {
        CssValue::Number(n) => ResolvedNumber::Number(*n),
        CssValue::Percentage(n) => ResolvedNumber::Percentage(*n),
        CssValue::Dimension { value, unit } => {
            match resolve_to_px(*value, unit, ctx) {
                Some(px) => ResolvedNumber::Px(px),
                None => ResolvedNumber::Number(*value),
            }
        }
        CssValue::Function { function, args } => {
            match function {
                CssFunction::Calc => resolve_calc_value_units(&resolve_calc_units(args, ctx), ctx),
                CssFunction::Min => resolve_calc_value_units(&resolve_min_max_units(args, ctx, true), ctx),
                CssFunction::Max => resolve_calc_value_units(&resolve_min_max_units(args, ctx, false), ctx),
                _ => ResolvedNumber::Number(0.0),
            }
        }
        _ => ResolvedNumber::Number(0.0),
    }
}

fn resolve_min_max_units(args: &[CssValue], ctx: &ResolverContext, is_min: bool) -> CssValue {
    let mut resolved: Vec<CssValue> = args.iter().map(|a| resolve_units(a, ctx)).collect();
    let comparable: Vec<ResolvedNumber> = resolved.iter().map(|v| to_resolved(v, ctx)).collect();
    let best_idx = if is_min {
        comparable.iter().enumerate().min_by(|(_, a), (_, b)| ResolvedNumber::partial_cmp_rn(a, b))
    } else {
        comparable.iter().enumerate().max_by(|(_, a), (_, b)| ResolvedNumber::partial_cmp_rn(a, b))
    };
    match best_idx { Some((idx, _)) => resolved.swap_remove(idx), None => CssValue::Number(0.0) }
}

fn resolve_clamp_units(args: &[CssValue], ctx: &ResolverContext) -> CssValue {
    if args.len() < 3 { return CssValue::Number(0.0); }
    let min = resolve_units(&args[0], ctx);
    let val = resolve_units(&args[1], ctx);
    let max = resolve_units(&args[2], ctx);
    let mn = to_f64(&min);
    let vl = to_f64(&val);
    let mx = to_f64(&max);
    if vl < mn { min } else if vl > mx { max } else { val }
}

fn to_resolved(value: &CssValue, ctx: &ResolverContext) -> ResolvedNumber {
    match value {
        CssValue::Number(n) => ResolvedNumber::Number(*n),
        CssValue::Percentage(n) => ResolvedNumber::Percentage(*n),
        CssValue::Dimension { value, unit } => {
            match resolve_to_px(*value, unit, ctx) {
                Some(px) => ResolvedNumber::Px(px),
                None => ResolvedNumber::Number(*value),
            }
        }
        _ => ResolvedNumber::Number(0.0),
    }
}

fn is_op(s: &str) -> bool {
    s == "+" || s == "-" || s == "*" || s == "/"
}
