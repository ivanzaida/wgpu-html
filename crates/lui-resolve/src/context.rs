//! Resolution context with built-in CSS function support and a custom
//! function registry.
//!
//! Built-in functions (`calc`, `min`, `max`, `abs`, `sin`, …) are resolved
//! via a match on [`CssFunction`] and cannot be overridden.
//! Custom functions use a string-keyed registry.
use bumpalo::Bump;
use lui_core::{ArcStr, CssFunction, CssUnit, CssValue};
use rustc_hash::FxHashMap;

use crate::ResolverContext;
use crate::vars::{resolve_custom_properties, resolve_var_value};

type FnHandler = Box<dyn Fn(&[CssValue], &ResolverContext, &Bump) -> CssValue>;

/// Resolution context: environment + custom function registry.
pub struct ResolutionContext {
    pub env: ResolverContext,
    custom: FxHashMap<String, FnHandler>,
    custom_properties: FxHashMap<ArcStr, CssValue>,
}

impl ResolutionContext {
    pub fn new(env: ResolverContext) -> Self {
        Self { env, custom: FxHashMap::default(), custom_properties: FxHashMap::default() }
    }

    pub fn set_custom_properties(&mut self, raw: &FxHashMap<ArcStr, CssValue>) {
        self.custom_properties = raw.clone();
    }

    pub fn register(
        &mut self,
        name: &str,
        handler: impl Fn(&[CssValue], &ResolverContext, &Bump) -> CssValue + 'static,
    ) {
        self.custom.insert(name.to_ascii_lowercase(), Box::new(handler));
    }

    /// Resolve a CSS value, allocating results in `arena`.
    /// Returns the input reference for unchanged passthrough values.
    pub fn resolve_value<'a>(&self, value: &'a CssValue, arena: &'a Bump) -> &'a CssValue {
        match value {
            CssValue::Var { .. } => self.resolve_var(value, arena),

            CssValue::Function { function, args } => {
                // Resolve var() / nested functions in args first
                let resolved_args: Vec<CssValue> = args.iter().map(|a| self.resolve_value(a, arena).clone()).collect();

                let result = match function {
                    // ── Math: calc / min / max / clamp ──
                    CssFunction::Calc => evaluate_calc(&resolved_args, &self.env),
                    CssFunction::Min  => evaluate_min_max(&resolved_args, &self.env, true),
                    CssFunction::Max  => evaluate_min_max(&resolved_args, &self.env, false),
                    CssFunction::Clamp => evaluate_clamp(&resolved_args, &self.env),

                    // ── Unary math ──
                    CssFunction::Abs  => math1(args, f64::abs),
                    CssFunction::Sign => math1(args, |x| x.signum()),
                    CssFunction::Sqrt => math1(args, f64::sqrt),
                    CssFunction::Exp  => math1(args, f64::exp),
                    CssFunction::Sin  => math1(args, f64::sin),
                    CssFunction::Cos  => math1(args, f64::cos),
                    CssFunction::Tan  => math1(args, f64::tan),
                    CssFunction::Asin => math1(args, f64::asin),
                    CssFunction::Acos => math1(args, f64::acos),
                    CssFunction::Atan => math1(args, f64::atan),

                    // ── Binary math ──
                    CssFunction::Atan2 => math2(args, |y, x| f64::atan2(y, x)),
                    CssFunction::Pow   => math2(args, |b, e| b.powf(e)),
                    CssFunction::Mod   => math2(args, |a, b| a % b),
                    CssFunction::Rem   => math2(args, |a, b| a % b),

                    CssFunction::Log => {
                        if args.len() >= 2 {
                            math2(args, |v, base| v.log(base))
                        } else {
                            math1(args, |x| x.ln())
                        }
                    }

                    CssFunction::Hypot => {
                        let vs: Vec<f64> = args.iter().filter_map(to_f64).collect();
                        CssValue::Number(vs.iter().map(|x| x * x).sum::<f64>().sqrt())
                    }

                    CssFunction::Round => {
                        match args.last() {
                            Some(CssValue::Dimension { value, unit }) => CssValue::Dimension { value: value.round(), unit: unit.clone() },
                            Some(CssValue::Number(n)) => CssValue::Number(n.round()),
                            _ => CssValue::Number(0.0),
                        }
                    }

                    CssFunction::Progress => {
                        if args.len() < 3 { return arena.alloc(CssValue::Number(0.0)); }
                        let start = to_f64(&args[0]).unwrap_or(0.0);
                        let end   = to_f64(&args[1]).unwrap_or(0.0);
                        let val   = to_f64(&args[2]).unwrap_or(0.0);
                        if (end - start).abs() < f64::EPSILON { return arena.alloc(CssValue::Number(0.0)); }
                        CssValue::Number(((val - start) / (end - start)).clamp(0.0, 1.0))
                    }

                    // ── Custom or unknown ──
                    other => {
                        let name = other.name();
                        if let Some(h) = self.custom.get(name) {
                            let rargs: Vec<CssValue> = args.iter().map(|v| self.resolve_value(v, arena).clone()).collect();
                            let result = h(&rargs, &self.env, arena);
                            return arena.alloc(result);
                        } else {
                            return value; // passthrough
                        }
                    }
                };

                arena.alloc(result)
            }

            CssValue::Dimension { value: dim_val, unit } => {
                let v = *dim_val;
                let px = crate::units::resolve_to_px(v, unit, &self.env);
                if let Some(px) = px {
                    if px != v || *unit != CssUnit::Px {
                        return arena.alloc(CssValue::Dimension { value: px, unit: CssUnit::Px });
                    }
                }
                value
            }

            _ => value, // passthrough: Number, String, Color, Url, Unknown, Percentage
        }
    }

    fn resolve_var<'a>(&self, value: &'a CssValue, arena: &'a Bump) -> &'a CssValue {
        let raw: FxHashMap<ArcStr, &CssValue> = self.custom_properties.iter()
            .map(|(k, v)| (k.clone(), v))
            .collect();
        let resolved = resolve_custom_properties(&raw, arena);
        let result = resolve_var_value(value, &resolved, &FxHashMap::default(), arena);
        if matches!(&result, CssValue::Var { .. }) {
            value // unresolved var → passthrough
        } else {
            arena.alloc(result)
        }
    }
}

// ---------------------------------------------------------------------------
// Built-in calc/min/max/clamp evaluators (unit-aware)
// ---------------------------------------------------------------------------

use crate::math_helpers::ResolvedNumber;

fn to_number(value: &CssValue, env: &ResolverContext) -> ResolvedNumber {
    match value {
        CssValue::Number(n) => ResolvedNumber::Number(*n),
        CssValue::Percentage(n) => ResolvedNumber::Percentage(*n),
        CssValue::Dimension { value, unit } => match crate::units::resolve_to_px(*value, unit, env) {
            Some(px) => ResolvedNumber::Px(px),
            None => ResolvedNumber::Number(*value),
        },
        CssValue::Function { function, args } => match function {
            CssFunction::Calc => to_number(&evaluate_calc(args, env), env),
            CssFunction::Min => to_number(&evaluate_min_max(args, env, true), env),
            CssFunction::Max => to_number(&evaluate_min_max(args, env, false), env),
            CssFunction::Clamp => to_number(&evaluate_clamp(args, env), env),
            _ => ResolvedNumber::Number(0.0),
        },
        _ => ResolvedNumber::Number(0.0),
    }
}

fn evaluate_calc(args: &[CssValue], env: &ResolverContext) -> CssValue {
    if args.is_empty() { return CssValue::Number(0.0); }
    let mut values: Vec<ResolvedNumber> = Vec::new();
    let mut ops: Vec<char> = Vec::new();
    for term in args {
        match &term {
            CssValue::Unknown(op) if is_operator(op) => ops.push(op.chars().next().unwrap_or('+')),
            _ => {
                let val = to_number(term, env);
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

fn evaluate_min_max(args: &[CssValue], env: &ResolverContext, is_min: bool) -> CssValue {
    let numbers: Vec<ResolvedNumber> = args.iter().map(|a| to_number(a, env)).collect();
    let best_idx = if is_min {
        numbers.iter().enumerate().min_by(|(_, a), (_, b)| ResolvedNumber::partial_cmp_rn(a, b))
    } else {
        numbers.iter().enumerate().max_by(|(_, a), (_, b)| ResolvedNumber::partial_cmp_rn(a, b))
    };
    match best_idx { Some((idx, _)) => numbers[idx].to_css_value(), None => CssValue::Number(0.0) }
}

fn evaluate_clamp(args: &[CssValue], env: &ResolverContext) -> CssValue {
    if args.len() < 3 { return CssValue::Number(0.0); }
    let min = to_number(&args[0], env);
    let val = to_number(&args[1], env);
    let max = to_number(&args[2], env);
    if val.as_f64() <= min.as_f64() { return args[0].clone(); }
    if val.as_f64() >= max.as_f64() { return args[2].clone(); }
    args[1].clone()
}

// ---------------------------------------------------------------------------
// Pure-math helpers
// ---------------------------------------------------------------------------

fn math1(args: &[CssValue], f: fn(f64) -> f64) -> CssValue {
    match to_f64(args.first().unwrap_or(&CssValue::Number(0.0))) {
        Some(x) => CssValue::Number(f(x)),
        None => CssValue::Number(f64::NAN),
    }
}

fn math2(args: &[CssValue], f: fn(f64, f64) -> f64) -> CssValue {
    let a = to_f64(args.first().unwrap_or(&CssValue::Number(0.0))).unwrap_or(0.0);
    let b = to_f64(args.get(1).unwrap_or(&CssValue::Number(0.0))).unwrap_or(0.0);
    CssValue::Number(f(a, b))
}

fn to_f64(v: &CssValue) -> Option<f64> {
    match v {
        CssValue::Number(n) | CssValue::Percentage(n) => Some(*n),
        CssValue::Dimension { value, .. } => Some(*value),
        _ => None,
    }
}

fn is_operator(s: &str) -> bool {
    s == "+" || s == "-" || s == "*" || s == "/"
}
