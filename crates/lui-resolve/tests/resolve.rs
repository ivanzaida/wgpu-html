use lui_resolve::{resolve, ResolverContext, ResolutionContext};
use lui_css_parser::{CssValue, CssUnit, CssFunction, ArcStr};
use bumpalo::Bump;

// ── helpers ────────────────────────────────────────────────────────────

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}

/// Extract the numeric value from a CssValue, panicking if it's not a Number.
fn as_num(v: &CssValue) -> f64 {
    match v {
        CssValue::Number(n) => *n,
        other => panic!("expected Number, got {:?}", other),
    }
}

fn resolve_fn(func: CssFunction, args: Vec<CssValue>) -> CssValue {
    let ctx = ResolverContext::default();
    resolve(&CssValue::Function { function: func, args }, &ctx)
}

// ═══════════════════════════════════════════════════════════════════════
//  Existing tests (unchanged)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn number_passthrough() {
    let ctx = ResolverContext::default();
    let v = CssValue::Number(42.0);
    assert_eq!(resolve(&v, &ctx), CssValue::Number(42.0));
}

#[test]
fn px_passthrough() {
    let ctx = ResolverContext::default();
    let v = CssValue::Dimension { value: 10.0, unit: CssUnit::Px };
    assert_eq!(resolve(&v, &ctx), v);
}

#[test]
fn em_resolves_to_px() {
    let ctx = ResolverContext { parent_font_size: 20.0, ..Default::default() };
    let v = CssValue::Dimension { value: 2.0, unit: CssUnit::Em };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 40.0, unit: CssUnit::Px });
}

#[test]
fn rem_resolves_to_px() {
    let ctx = ResolverContext { root_font_size: 18.0, ..Default::default() };
    let v = CssValue::Dimension { value: 2.0, unit: CssUnit::Rem };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 36.0, unit: CssUnit::Px });
}

#[test]
fn vw_resolves_to_px() {
    let ctx = ResolverContext { viewport_width: 800.0, ..Default::default() };
    let v = CssValue::Dimension { value: 50.0, unit: CssUnit::Vw };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 400.0, unit: CssUnit::Px });
}

#[test]
fn vh_resolves_to_px() {
    let ctx = ResolverContext { viewport_height: 600.0, ..Default::default() };
    let v = CssValue::Dimension { value: 50.0, unit: CssUnit::Vh };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 300.0, unit: CssUnit::Px });
}

#[test]
fn calc_simple_addition() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![
            CssValue::Number(2.0),
            CssValue::Unknown("+".into()),
            CssValue::Number(3.0),
        ],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Number(5.0));
}

#[test]
fn calc_with_px() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![
            CssValue::Dimension { value: 10.0, unit: CssUnit::Px },
            CssValue::Unknown("+".into()),
            CssValue::Dimension { value: 5.0, unit: CssUnit::Px },
        ],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 15.0, unit: CssUnit::Px });
}

#[test]
fn calc_with_subtraction() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![
            CssValue::Dimension { value: 20.0, unit: CssUnit::Px },
            CssValue::Unknown("-".into()),
            CssValue::Dimension { value: 8.0, unit: CssUnit::Px },
        ],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 12.0, unit: CssUnit::Px });
}

#[test]
fn calc_with_em_and_px() {
    let ctx = ResolverContext { parent_font_size: 16.0, ..Default::default() };
    let v = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![
            CssValue::Dimension { value: 1.0, unit: CssUnit::Em },
            CssValue::Unknown("+".into()),
            CssValue::Dimension { value: 8.0, unit: CssUnit::Px },
        ],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 24.0, unit: CssUnit::Px });
}

#[test]
fn min_function() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Min,
        args: vec![CssValue::Number(5.0), CssValue::Number(3.0), CssValue::Number(8.0)],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Number(3.0));
}

#[test]
fn max_function() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Max,
        args: vec![CssValue::Number(5.0), CssValue::Number(3.0), CssValue::Number(8.0)],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Number(8.0));
}

#[test]
fn clamp_function() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Clamp,
        args: vec![CssValue::Number(1.0), CssValue::Number(10.0), CssValue::Number(5.0)],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Number(5.0)); // 1..5 : 10 clamped to 5
}

#[test]
fn clamp_below_min() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Clamp,
        args: vec![CssValue::Number(10.0), CssValue::Number(5.0), CssValue::Number(20.0)],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Number(10.0)); // clamp(10, 5, 20) → max(10, min(5, 20)) = 10
}

#[test]
fn clamp_above_max() {
    let ctx = ResolverContext::default();
    let v = CssValue::Function {
        function: CssFunction::Clamp,
        args: vec![CssValue::Number(1.0), CssValue::Number(25.0), CssValue::Number(10.0)],
    };
    assert_eq!(resolve(&v, &ctx), CssValue::Number(10.0)); // clamp(1, 25, 10) → max(1, min(25, 10)) = 10
}

#[test]
fn percentage_passthrough() {
    let ctx = ResolverContext::default();
    let v = CssValue::Percentage(75.0);
    assert_eq!(resolve(&v, &ctx), CssValue::Percentage(75.0));
}

#[test]
fn absolute_units_converted() {
    let ctx = ResolverContext::default();
    let v = CssValue::Dimension { value: 1.0, unit: CssUnit::In };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 96.0, unit: CssUnit::Px });
}

#[test]
fn non_length_units_preserved() {
    let ctx = ResolverContext::default();
    let v = CssValue::Dimension { value: 90.0, unit: CssUnit::Deg };
    assert_eq!(resolve(&v, &ctx), v);
}

#[test]
fn nested_calc() {
    let ctx = ResolverContext::default();
    let inner = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![CssValue::Number(2.0), CssValue::Unknown("+".into()), CssValue::Number(3.0)],
    };
    let outer = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![CssValue::Number(10.0), CssValue::Unknown("+".into()), inner],
    };
    assert_eq!(resolve(&outer, &ctx), CssValue::Number(15.0));
}

// ═══════════════════════════════════════════════════════════════════════
//  Unary math functions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn abs_negative_returns_positive() {
    assert_eq!(resolve_fn(CssFunction::Abs, vec![CssValue::Number(-5.0)]), CssValue::Number(5.0));
}

#[test]
fn abs_positive_returns_same() {
    assert_eq!(resolve_fn(CssFunction::Abs, vec![CssValue::Number(3.14)]), CssValue::Number(3.14));
}

#[test]
fn abs_zero() {
    assert_eq!(resolve_fn(CssFunction::Abs, vec![CssValue::Number(0.0)]), CssValue::Number(0.0));
}

#[test]
fn abs_empty_args_defaults_to_zero() {
    // empty args → args.first() = None → math1 falls back to Number(0.0) → abs(0) = 0
    assert_eq!(resolve_fn(CssFunction::Abs, vec![]), CssValue::Number(0.0));
}

#[test]
fn sign_negative() {
    assert_eq!(resolve_fn(CssFunction::Sign, vec![CssValue::Number(-5.0)]), CssValue::Number(-1.0));
}

#[test]
fn sign_positive() {
    assert_eq!(resolve_fn(CssFunction::Sign, vec![CssValue::Number(3.14)]), CssValue::Number(1.0));
}

#[test]
fn sign_zero() {
    // Rust's f64::signum(0.0) returns 1.0 (matching the IEEE behavior for +0)
    assert_eq!(resolve_fn(CssFunction::Sign, vec![CssValue::Number(0.0)]), CssValue::Number(1.0));
}

#[test]
fn sqrt_perfect_square() {
    assert_eq!(resolve_fn(CssFunction::Sqrt, vec![CssValue::Number(16.0)]), CssValue::Number(4.0));
}

#[test]
fn sqrt_irrational() {
    let r = resolve_fn(CssFunction::Sqrt, vec![CssValue::Number(2.0)]);
    assert!(approx_eq(as_num(&r), 1.4142135623730951));
}

#[test]
fn sqrt_zero() {
    assert_eq!(resolve_fn(CssFunction::Sqrt, vec![CssValue::Number(0.0)]), CssValue::Number(0.0));
}

#[test]
fn exp_zero_returns_one() {
    assert_eq!(resolve_fn(CssFunction::Exp, vec![CssValue::Number(0.0)]), CssValue::Number(1.0));
}

#[test]
fn exp_one_returns_e() {
    let r = resolve_fn(CssFunction::Exp, vec![CssValue::Number(1.0)]);
    assert!(approx_eq(as_num(&r), std::f64::consts::E));
}

#[test]
fn sin_zero() {
    let r = resolve_fn(CssFunction::Sin, vec![CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), 0.0));
}

#[test]
fn sin_pi_half_returns_one() {
    let r = resolve_fn(CssFunction::Sin, vec![CssValue::Number(std::f64::consts::PI / 2.0)]);
    assert!(approx_eq(as_num(&r), 1.0));
}

#[test]
fn cos_zero_returns_one() {
    let r = resolve_fn(CssFunction::Cos, vec![CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), 1.0));
}

#[test]
fn cos_pi_returns_negative_one() {
    let r = resolve_fn(CssFunction::Cos, vec![CssValue::Number(std::f64::consts::PI)]);
    assert!(approx_eq(as_num(&r), -1.0));
}

#[test]
fn tan_zero() {
    let r = resolve_fn(CssFunction::Tan, vec![CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), 0.0));
}

#[test]
fn tan_pi_quarter_returns_one() {
    let r = resolve_fn(CssFunction::Tan, vec![CssValue::Number(std::f64::consts::PI / 4.0)]);
    assert!(approx_eq(as_num(&r), 1.0));
}

#[test]
fn asin_zero() {
    let r = resolve_fn(CssFunction::Asin, vec![CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), 0.0));
}

#[test]
fn asin_one_returns_pi_half() {
    let r = resolve_fn(CssFunction::Asin, vec![CssValue::Number(1.0)]);
    assert!(approx_eq(as_num(&r), std::f64::consts::PI / 2.0));
}

#[test]
fn acos_one_returns_zero() {
    let r = resolve_fn(CssFunction::Acos, vec![CssValue::Number(1.0)]);
    assert!(approx_eq(as_num(&r), 0.0));
}

#[test]
fn acos_zero_returns_pi_half() {
    let r = resolve_fn(CssFunction::Acos, vec![CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), std::f64::consts::PI / 2.0));
}

#[test]
fn atan_zero() {
    let r = resolve_fn(CssFunction::Atan, vec![CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), 0.0));
}

#[test]
fn atan_one_returns_pi_quarter() {
    let r = resolve_fn(CssFunction::Atan, vec![CssValue::Number(1.0)]);
    assert!(approx_eq(as_num(&r), std::f64::consts::PI / 4.0));
}

// ═══════════════════════════════════════════════════════════════════════
//  Binary math functions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn atan2_one_one_returns_pi_quarter() {
    let r = resolve_fn(CssFunction::Atan2, vec![CssValue::Number(1.0), CssValue::Number(1.0)]);
    assert!(approx_eq(as_num(&r), std::f64::consts::PI / 4.0));
}

#[test]
fn atan2_one_zero_returns_pi_half() {
    let r = resolve_fn(CssFunction::Atan2, vec![CssValue::Number(1.0), CssValue::Number(0.0)]);
    assert!(approx_eq(as_num(&r), std::f64::consts::PI / 2.0));
}

#[test]
fn pow_two_cubed() {
    assert_eq!(resolve_fn(CssFunction::Pow, vec![CssValue::Number(2.0), CssValue::Number(3.0)]), CssValue::Number(8.0));
}

#[test]
fn pow_four_to_half_returns_two() {
    assert_eq!(resolve_fn(CssFunction::Pow, vec![CssValue::Number(4.0), CssValue::Number(0.5)]), CssValue::Number(2.0));
}

#[test]
fn pow_zero_exp_returns_one() {
    assert_eq!(resolve_fn(CssFunction::Pow, vec![CssValue::Number(42.0), CssValue::Number(0.0)]), CssValue::Number(1.0));
}

#[test]
fn modulo_ten_three() {
    assert_eq!(resolve_fn(CssFunction::Mod, vec![CssValue::Number(10.0), CssValue::Number(3.0)]), CssValue::Number(1.0));
}

#[test]
fn modulo_negative() {
    assert_eq!(resolve_fn(CssFunction::Mod, vec![CssValue::Number(-10.0), CssValue::Number(3.0)]), CssValue::Number(-1.0));
}

#[test]
fn rem_ten_three() {
    assert_eq!(resolve_fn(CssFunction::Rem, vec![CssValue::Number(10.0), CssValue::Number(3.0)]), CssValue::Number(1.0));
}

#[test]
fn rem_negative() {
    assert_eq!(resolve_fn(CssFunction::Rem, vec![CssValue::Number(-10.0), CssValue::Number(3.0)]), CssValue::Number(-1.0));
}

#[test]
fn log_with_base_two_of_eight() {
    let r = resolve_fn(CssFunction::Log, vec![CssValue::Number(8.0), CssValue::Number(2.0)]);
    assert!(approx_eq(as_num(&r), 3.0));
}

#[test]
fn log_natural_of_e() {
    let r = resolve_fn(CssFunction::Log, vec![CssValue::Number(std::f64::consts::E)]);
    assert!(approx_eq(as_num(&r), 1.0));
}

#[test]
fn log_natural_of_one() {
    let r = resolve_fn(CssFunction::Log, vec![CssValue::Number(1.0)]);
    assert!(approx_eq(as_num(&r), 0.0));
}

#[test]
fn log_empty_args_is_natural_log_of_zero() {
    // math1 processes first arg if present, else defaults to 0.0; ln(0) = -inf
    let r = resolve_fn(CssFunction::Log, vec![]);
    let n = as_num(&r);
    assert!(n.is_infinite() && n < 0.0);
}

// ═══════════════════════════════════════════════════════════════════════
//  Variadic math functions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn hypot_three_four_returns_five() {
    assert_eq!(resolve_fn(CssFunction::Hypot, vec![CssValue::Number(3.0), CssValue::Number(4.0)]), CssValue::Number(5.0));
}

#[test]
fn hypot_single_arg() {
    assert_eq!(resolve_fn(CssFunction::Hypot, vec![CssValue::Number(5.0)]), CssValue::Number(5.0));
}

#[test]
fn hypot_empty_is_zero() {
    assert_eq!(resolve_fn(CssFunction::Hypot, vec![]), CssValue::Number(0.0));
}

#[test]
fn round_up() {
    assert_eq!(resolve_fn(CssFunction::Round, vec![CssValue::Number(3.6)]), CssValue::Number(4.0));
}

#[test]
fn round_down() {
    assert_eq!(resolve_fn(CssFunction::Round, vec![CssValue::Number(3.4)]), CssValue::Number(3.0));
}

#[test]
fn round_exact_integer() {
    assert_eq!(resolve_fn(CssFunction::Round, vec![CssValue::Number(10.0)]), CssValue::Number(10.0));
}

#[test]
fn round_px_already_integer() {
    let r = resolve_fn(CssFunction::Round, vec![
        CssValue::Dimension { value: 10.0, unit: CssUnit::Px },
    ]);
    assert_eq!(r, CssValue::Dimension { value: 10.0, unit: CssUnit::Px });
}

#[test]
fn round_px_up() {
    let r = resolve_fn(CssFunction::Round, vec![
        CssValue::Dimension { value: 10.6, unit: CssUnit::Px },
    ]);
    assert_eq!(r, CssValue::Dimension { value: 11.0, unit: CssUnit::Px });
}

#[test]
fn round_px_down() {
    let r = resolve_fn(CssFunction::Round, vec![
        CssValue::Dimension { value: 10.3, unit: CssUnit::Px },
    ]);
    assert_eq!(r, CssValue::Dimension { value: 10.0, unit: CssUnit::Px });
}

#[test]
fn round_empty_args_returns_zero() {
    assert_eq!(resolve_fn(CssFunction::Round, vec![]), CssValue::Number(0.0));
}

#[test]
fn progress_midpoint() {
    assert_eq!(
        resolve_fn(CssFunction::Progress, vec![
            CssValue::Number(0.0),
            CssValue::Number(100.0),
            CssValue::Number(50.0),
        ]),
        CssValue::Number(0.5),
    );
}

#[test]
fn progress_below_start_clamped_to_zero() {
    assert_eq!(
        resolve_fn(CssFunction::Progress, vec![
            CssValue::Number(0.0),
            CssValue::Number(100.0),
            CssValue::Number(-10.0),
        ]),
        CssValue::Number(0.0),
    );
}

#[test]
fn progress_above_end_clamped_to_one() {
    assert_eq!(
        resolve_fn(CssFunction::Progress, vec![
            CssValue::Number(0.0),
            CssValue::Number(100.0),
            CssValue::Number(200.0),
        ]),
        CssValue::Number(1.0),
    );
}

#[test]
fn progress_start_equal_end_returns_zero() {
    let ctx = ResolverContext::default();
    let arena = Bump::new();
    let rctx = ResolutionContext::new(ctx);
    let v = CssValue::Function {
        function: CssFunction::Progress,
        args: vec![
            CssValue::Number(50.0),
            CssValue::Number(50.0),
            CssValue::Number(50.0),
        ],
    };
    let result = rctx.resolve_value(&v, &arena);
    assert_eq!(*result, CssValue::Number(0.0));
}

#[test]
fn progress_insufficient_args_returns_zero() {
    // less than 3 args → early return 0
    let ctx = ResolverContext::default();
    let arena = Bump::new();
    let rctx = ResolutionContext::new(ctx);
    let v = CssValue::Function {
        function: CssFunction::Progress,
        args: vec![CssValue::Number(0.0), CssValue::Number(100.0)],
    };
    let result = rctx.resolve_value(&v, &arena);
    assert_eq!(*result, CssValue::Number(0.0));
}

// ═══════════════════════════════════════════════════════════════════════
//  Custom function registration
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn custom_function_is_called() {
    // Register a handler for a known non-builtin function name.
    // CssFunction::Unknown always has name() → "", so we use a real variant
    // whose name() matches the registered key.
    let mut rctx = ResolutionContext::new(ResolverContext::default());
    rctx.register("blur", |args, _env, _arena| {
        let n = match args.first() {
            Some(CssValue::Number(v)) => *v,
            _ => 0.0,
        };
        CssValue::Number(n * 2.0)
    });
    let arena = Bump::new();
    let v = CssValue::Function {
        function: CssFunction::Blur,   // name() → "blur"
        args: vec![CssValue::Number(21.0)],
    };
    let result = rctx.resolve_value(&v, &arena);
    assert_eq!(*result, CssValue::Number(42.0));
}

#[test]
fn builtin_cannot_be_overridden_by_custom() {
    // Registering "abs" should NOT override the built-in abs.
    let mut rctx = ResolutionContext::new(ResolverContext::default());
    rctx.register("abs", |_args, _env, _arena| {
        CssValue::Number(999.0) // attempt to override
    });
    let arena = Bump::new();
    let v = CssValue::Function {
        function: CssFunction::Abs,
        args: vec![CssValue::Number(-5.0)],
    };
    let result = rctx.resolve_value(&v, &arena);
    // Built-in wins: abs(-5) = 5
    assert_eq!(*result, CssValue::Number(5.0));
}

#[test]
fn calc_cannot_be_overridden_by_custom() {
    let mut rctx = ResolutionContext::new(ResolverContext::default());
    rctx.register("calc", |_args, _env, _arena| {
        CssValue::Number(999.0)
    });
    let arena = Bump::new();
    let v = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![
            CssValue::Number(2.0),
            CssValue::Unknown("+".into()),
            CssValue::Number(3.0),
        ],
    };
    let result = rctx.resolve_value(&v, &arena);
    assert_eq!(*result, CssValue::Number(5.0));
}

#[test]
fn unknown_function_passthrough_when_no_handler() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Function {
        function: CssFunction::Unknown("no-such-func".into()),
        args: vec![CssValue::Number(1.0)],
    };
    let result = rctx.resolve_value(&v, &arena);
    // Should pass through unchanged (same identity)
    assert!(std::ptr::eq(result, &v));
}

// ═══════════════════════════════════════════════════════════════════════
//  Edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn calc_empty_args_returns_zero() {
    assert_eq!(resolve_fn(CssFunction::Calc, vec![]), CssValue::Number(0.0));
}

#[test]
fn min_empty_args_returns_zero() {
    assert_eq!(resolve_fn(CssFunction::Min, vec![]), CssValue::Number(0.0));
}

#[test]
fn max_empty_args_returns_zero() {
    assert_eq!(resolve_fn(CssFunction::Max, vec![]), CssValue::Number(0.0));
}

#[test]
fn clamp_insufficient_args_returns_zero() {
    assert_eq!(
        resolve_fn(CssFunction::Clamp, vec![CssValue::Number(1.0), CssValue::Number(10.0)]),
        CssValue::Number(0.0),
    );
}

#[test]
fn non_numeric_arg_in_math1_returns_nan() {
    // abs(String("hello")) → to_f64 returns None → math1 returns NaN
    let r = resolve_fn(CssFunction::Abs, vec![CssValue::String("hello".into())]);
    assert!(as_num(&r).is_nan());
}

#[test]
fn non_numeric_arg_in_math2_defaults_to_zero() {
    // pow(String("x"), Number(3)) → first arg to_f64 returns None → defaults to 0.0
    assert_eq!(
        resolve_fn(CssFunction::Pow, vec![
            CssValue::String("x".into()),
            CssValue::Number(3.0),
        ]),
        CssValue::Number(0.0), // 0^3 = 0
    );
}

#[test]
fn number_passthrough_keeps_same_reference() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Number(42.0);
    let result = rctx.resolve_value(&v, &arena);
    assert!(std::ptr::eq(result, &v));
}

#[test]
fn percentage_passthrough_keeps_same_reference() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Percentage(75.0);
    let result = rctx.resolve_value(&v, &arena);
    assert!(std::ptr::eq(result, &v));
}

#[test]
fn string_passthrough_keeps_same_reference() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::String("hello".into());
    let result = rctx.resolve_value(&v, &arena);
    assert!(std::ptr::eq(result, &v));
}

#[test]
fn unknown_passthrough_keeps_same_reference() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Unknown("anything".into());
    let result = rctx.resolve_value(&v, &arena);
    assert!(std::ptr::eq(result, &v));
}

#[test]
fn dimension_px_unchanged_passthrough_same_reference() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Dimension { value: 10.0, unit: CssUnit::Px };
    let result = rctx.resolve_value(&v, &arena);
    // Already Px with same value → passthrough
    assert!(std::ptr::eq(result, &v));
}

#[test]
fn dimension_in_converted_not_same_reference() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Dimension { value: 1.0, unit: CssUnit::In };
    let result = rctx.resolve_value(&v, &arena);
    // Converted to Px → new allocation, not same reference
    assert!(!std::ptr::eq(result, &v));
    assert_eq!(*result, CssValue::Dimension { value: 96.0, unit: CssUnit::Px });
}

#[test]
fn custom_property_var_resolution() {
    let mut rctx = ResolutionContext::new(ResolverContext::default());
    use rustc_hash::FxHashMap;
    let mut map = FxHashMap::default();
    map.insert(ArcStr::from("--my-color"), CssValue::String("red".into()));
    rctx.set_custom_properties(&map);

    let arena = Bump::new();
    let v = CssValue::Var {
        name: ArcStr::from("--my-color"),
        fallback: None,
    };
    let result = rctx.resolve_value(&v, &arena);
    assert_eq!(*result, CssValue::String("red".into()));
}

#[test]
fn resolve_convenience_uses_internal_context() {
    let ctx = ResolverContext { parent_font_size: 20.0, ..Default::default() };
    let v = CssValue::Dimension { value: 1.0, unit: CssUnit::In };
    assert_eq!(resolve(&v, &ctx), CssValue::Dimension { value: 96.0, unit: CssUnit::Px });
}

// ═══════════════════════════════════════════════════════════════════════
//  Nested function calls in args — current behavior
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn abs_of_nested_calc_is_nan() {
    // abs(calc(-2 + -3)) — math1 reads raw `args`, to_f64 returns None for
    // the calc Function variant, so the result is NaN.
    let inner = CssValue::Function {
        function: CssFunction::Calc,
        args: vec![CssValue::Number(-2.0), CssValue::Unknown("+".into()), CssValue::Number(-3.0)],
    };
    let r = resolve_fn(CssFunction::Abs, vec![inner]);
    assert!(as_num(&r).is_nan());
}

#[test]
fn sin_of_dimension_uses_raw_value() {
    // sin(… rad) → to_f64 extracts the numeric value from the Dimension
    let r = resolve_fn(CssFunction::Sin, vec![
        CssValue::Dimension { value: std::f64::consts::PI / 2.0, unit: CssUnit::Rad },
    ]);
    assert!(approx_eq(as_num(&r), 1.0));
}

// ═══════════════════════════════════════════════════════════════════════
//  Percentages in math functions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn abs_of_percentage() {
    let r = resolve_fn(CssFunction::Abs, vec![CssValue::Percentage(-42.0)]);
    assert_eq!(as_num(&r), 42.0);
}

#[test]
fn sign_of_percentage() {
    assert_eq!(
        resolve_fn(CssFunction::Sign, vec![CssValue::Percentage(-99.0)]),
        CssValue::Number(-1.0),
    );
}

#[test]
fn progress_with_percentages() {
    assert_eq!(
        resolve_fn(CssFunction::Progress, vec![
            CssValue::Percentage(0.0),
            CssValue::Percentage(100.0),
            CssValue::Percentage(50.0),
        ]),
        CssValue::Number(0.5),
    );
}

#[test]
fn hypot_with_percentages() {
    let r = resolve_fn(CssFunction::Hypot, vec![
        CssValue::Percentage(3.0),
        CssValue::Percentage(4.0),
    ]);
    assert_eq!(as_num(&r), 5.0);
}

// ═══════════════════════════════════════════════════════════════════════
//  ResolutionContext::resolve_value — further exercised
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn resolve_value_allocates_result_in_arena() {
    let rctx = ResolutionContext::new(ResolverContext::default());
    let arena = Bump::new();
    let v = CssValue::Function {
        function: CssFunction::Abs,
        args: vec![CssValue::Number(-10.0)],
    };
    let result = rctx.resolve_value(&v, &arena);
    // The result is allocated in the arena, not the original stack value
    assert!(!std::ptr::eq(result, &v));
    assert_eq!(*result, CssValue::Number(10.0));
}

#[test]
fn log_with_two_args_uses_base() {
    // log(100, 10) = 2
    let r = resolve_fn(CssFunction::Log, vec![CssValue::Number(100.0), CssValue::Number(10.0)]);
    assert!(approx_eq(as_num(&r), 2.0));
}

#[test]
fn log_with_three_args_still_uses_first_two() {
    // log(100, 10, extra) — math2 takes first two → 2.0
    let r = resolve_fn(CssFunction::Log, vec![
        CssValue::Number(100.0),
        CssValue::Number(10.0),
        CssValue::Number(999.0),
    ]);
    assert!(approx_eq(as_num(&r), 2.0));
}
