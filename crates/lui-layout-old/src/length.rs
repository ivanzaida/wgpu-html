use lui_models::common::css_enums::{CssLength, CssMathExpr, CssNumericFunction};

use crate::Ctx;

const DEFAULT_FONT_PX: f32 = 16.0;

/// Resolve a CSS length to physical pixels.
///
/// - `Px` is multiplied by the current CSS-px -> physical-px scale.
/// - `Percent` is resolved against `parent_size_px`.
/// - `Vw` / `Vh` / `Vmin` / `Vmax` against the viewport.
/// - `Em` / `Rem` against `DEFAULT_FONT_PX` (real font metrics come later).
/// - `Auto` and `Raw(_)` return `None` (the caller picks a default).
pub(crate) fn resolve(len: Option<&CssLength>, parent_size_px: f32, ctx: &Ctx) -> Option<f32> {
  match len? {
    CssLength::Px(v) => Some(*v * ctx.scale),
    CssLength::Percent(v) => Some(parent_size_px * v / 100.0),
    CssLength::Em(v) | CssLength::Rem(v) => Some(*v * DEFAULT_FONT_PX * ctx.scale),
    CssLength::Vw(v) => Some(ctx.viewport_w * v / 100.0),
    CssLength::Vh(v) => Some(ctx.viewport_h * v / 100.0),
    CssLength::Vmin(v) => Some(ctx.viewport_w.min(ctx.viewport_h) * v / 100.0),
    CssLength::Vmax(v) => Some(ctx.viewport_w.max(ctx.viewport_h) * v / 100.0),
    CssLength::Zero => Some(0.0),
    CssLength::Calc(expr) => eval_math(expr, parent_size_px, ctx).map(|v| v.into_px()),
    CssLength::Min(items) => items
      .iter()
      .filter_map(|l| resolve(Some(l), parent_size_px, ctx))
      .reduce(f32::min),
    CssLength::Max(items) => items
      .iter()
      .filter_map(|l| resolve(Some(l), parent_size_px, ctx))
      .reduce(f32::max),
    CssLength::Clamp { min, preferred, max } => {
      let min = resolve(Some(min), parent_size_px, ctx)?;
      let preferred = resolve(Some(preferred), parent_size_px, ctx)?;
      let max = resolve(Some(max), parent_size_px, ctx)?;
      Some(preferred.clamp(min, max))
    }
    CssLength::Auto | CssLength::Raw(_) => None,
  }
}

#[derive(Debug, Clone, Copy)]
enum MathValue {
  Number(f32),
  LengthPx(f32),
}

impl MathValue {
  fn into_px(self) -> f32 {
    match self {
      MathValue::Number(v) | MathValue::LengthPx(v) => v,
    }
  }

  fn number(self) -> f32 {
    match self {
      MathValue::Number(v) | MathValue::LengthPx(v) => v,
    }
  }
}

fn eval_math(expr: &CssMathExpr, parent_size_px: f32, ctx: &Ctx) -> Option<MathValue> {
  match expr {
    CssMathExpr::Length(len) => resolve(Some(len), parent_size_px, ctx).map(MathValue::LengthPx),
    CssMathExpr::Number(v) => Some(MathValue::Number(*v)),
    CssMathExpr::Add(a, b) => binary_same(a, b, parent_size_px, ctx, |x, y| x + y),
    CssMathExpr::Sub(a, b) => binary_same(a, b, parent_size_px, ctx, |x, y| x - y),
    CssMathExpr::Mul(a, b) => {
      let a = eval_math(a, parent_size_px, ctx)?;
      let b = eval_math(b, parent_size_px, ctx)?;
      match (a, b) {
        (MathValue::LengthPx(x), MathValue::Number(y)) | (MathValue::Number(y), MathValue::LengthPx(x)) => {
          Some(MathValue::LengthPx(x * y))
        }
        (MathValue::Number(x), MathValue::Number(y)) => Some(MathValue::Number(x * y)),
        (MathValue::LengthPx(_), MathValue::LengthPx(_)) => None,
      }
    }
    CssMathExpr::Div(a, b) => {
      let a = eval_math(a, parent_size_px, ctx)?;
      let b = eval_math(b, parent_size_px, ctx)?;
      let denom = b.number();
      if denom == 0.0 {
        return None;
      }
      match (a, b) {
        (MathValue::LengthPx(x), MathValue::Number(_)) => Some(MathValue::LengthPx(x / denom)),
        (MathValue::Number(x), MathValue::Number(_)) => Some(MathValue::Number(x / denom)),
        (MathValue::LengthPx(x), MathValue::LengthPx(_)) => Some(MathValue::Number(x / denom)),
        (MathValue::Number(_), MathValue::LengthPx(_)) => None,
      }
    }
    CssMathExpr::Function(kind, args) => eval_numeric_function(*kind, args, parent_size_px, ctx),
  }
}

fn binary_same(
  a: &CssMathExpr,
  b: &CssMathExpr,
  parent_size_px: f32,
  ctx: &Ctx,
  op: impl FnOnce(f32, f32) -> f32,
) -> Option<MathValue> {
  let a = eval_math(a, parent_size_px, ctx)?;
  let b = eval_math(b, parent_size_px, ctx)?;
  match (a, b) {
    (MathValue::LengthPx(x), MathValue::LengthPx(y)) => Some(MathValue::LengthPx(op(x, y))),
    (MathValue::Number(x), MathValue::Number(y)) => Some(MathValue::Number(op(x, y))),
    _ => None,
  }
}

fn eval_numeric_function(
  kind: CssNumericFunction,
  args: &[CssMathExpr],
  parent_size_px: f32,
  ctx: &Ctx,
) -> Option<MathValue> {
  let nums: Vec<f32> = args
    .iter()
    .map(|arg| eval_math(arg, parent_size_px, ctx).map(MathValue::number))
    .collect::<Option<Vec<_>>>()?;
  let v = match kind {
    CssNumericFunction::Sin => nums.first()?.sin(),
    CssNumericFunction::Cos => nums.first()?.cos(),
    CssNumericFunction::Tan => nums.first()?.tan(),
    CssNumericFunction::Asin => nums.first()?.asin(),
    CssNumericFunction::Acos => nums.first()?.acos(),
    CssNumericFunction::Atan => nums.first()?.atan(),
    CssNumericFunction::Atan2 => nums.first()?.atan2(*nums.get(1)?),
    CssNumericFunction::Pow => nums.first()?.powf(*nums.get(1)?),
    CssNumericFunction::Sqrt => nums.first()?.sqrt(),
    CssNumericFunction::Hypot => nums.iter().map(|n| n * n).sum::<f32>().sqrt(),
    CssNumericFunction::Log => {
      let x = *nums.first()?;
      if let Some(base) = nums.get(1) {
        x.log(*base)
      } else {
        x.ln()
      }
    }
    CssNumericFunction::Exp => nums.first()?.exp(),
    CssNumericFunction::Abs => nums.first()?.abs(),
    CssNumericFunction::Sign => nums.first()?.signum(),
    CssNumericFunction::Mod | CssNumericFunction::Rem => nums.first()?.rem_euclid(*nums.get(1)?),
    CssNumericFunction::Round => nums.first()?.round(),
  };
  v.is_finite().then_some(MathValue::Number(v))
}
