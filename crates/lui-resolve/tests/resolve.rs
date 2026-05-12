use lui_resolve::{resolve, ResolverContext};
use lui_css_parser::{CssValue, CssUnit, CssFunction};

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
