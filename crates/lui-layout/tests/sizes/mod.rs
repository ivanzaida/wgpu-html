use bumpalo::Bump;
use lui_cascade::ComputedStyle;
use lui_layout::sizes;
use crate::helpers::*;

// ============================================================================
// 3. sizes.rs tests
// ============================================================================

#[test]
fn resolve_length_px_returns_dimension() {
    let result = sizes::resolve_length(Some(&px(10.0)), 100.0);
    assert_eq!(result, Some(10.0), "10px should resolve to Some(10.0)");
}

#[test]
fn resolve_length_percentage_resolves_against_containing() {
    let result = sizes::resolve_length(Some(&pct(50.0)), 200.0);
    assert_eq!(result, Some(100.0), "50% of 200 should be 100");
}

#[test]
fn resolve_length_auto_returns_none() {
    let result = sizes::resolve_length(Some(&auto()), 100.0);
    assert_eq!(result, None, "auto should resolve to None");
}

#[test]
fn resolve_length_none_returns_none() {
    let result = sizes::resolve_length(None, 100.0);
    assert_eq!(result, None, "no value should resolve to None");
}

#[test]
fn resolve_length_number_zero_returns_zero() {
    let result = sizes::resolve_length(Some(&num(0.0)), 100.0);
    assert_eq!(result, Some(0.0), "Number(0) should resolve to Some(0.0)");
}

#[test]
fn resolve_length_percentage_zero_returns_zero() {
    let result = sizes::resolve_length(Some(&pct(0.0)), 200.0);
    assert_eq!(result, Some(0.0), "0% should resolve to 0.0");
}

#[test]
fn resolve_box_sizes_with_all_properties_set() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(px(100.0)));
    style.height = Some(arena.alloc(px(50.0)));
    style.min_width = Some(arena.alloc(px(0.0)));
    style.min_height = Some(arena.alloc(px(0.0)));
    style.max_width = Some(arena.alloc(px(500.0)));
    style.max_height = Some(arena.alloc(px(300.0)));

    let bs = sizes::resolve_box_sizes(&style, 800.0, 600.0);

    assert_eq!(bs.width, Some(100.0));
    assert_eq!(bs.height, Some(50.0));
    assert_eq!(bs.min_width, Some(0.0));
    assert_eq!(bs.min_height, Some(0.0));
    assert_eq!(bs.max_width, Some(500.0));
    assert_eq!(bs.max_height, Some(300.0));
}

#[test]
fn resolve_box_sizes_with_auto_width_height() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(auto()));
    style.height = Some(arena.alloc(auto()));

    let bs = sizes::resolve_box_sizes(&style, 800.0, 600.0);
    assert_eq!(bs.width, None, "auto width should be None");
    assert_eq!(bs.height, None, "auto height should be None");
}

#[test]
fn resolve_box_sizes_percentage_against_containing() {
    let arena = Bump::new();
    let mut style = ComputedStyle::default();
    style.width = Some(arena.alloc(pct(50.0)));
    style.height = Some(arena.alloc(pct(25.0)));

    let bs = sizes::resolve_box_sizes(&style, 800.0, 600.0);
    assert_eq!(bs.width, Some(400.0), "50% of 800 = 400");
    assert_eq!(bs.height, Some(150.0), "25% of 600 = 150");
}

// ============================================================================
// 17. sizes.rs clamp_with_minmax tests
// ============================================================================

#[test]
fn clamp_with_minmax_no_constraints() {
    assert_eq!(sizes::clamp_with_minmax(100.0, None, None), 100.0);
}

#[test]
fn clamp_with_minmax_min_only() {
    assert_eq!(sizes::clamp_with_minmax(50.0, Some(80.0), None), 80.0);
    assert_eq!(sizes::clamp_with_minmax(100.0, Some(80.0), None), 100.0);
}

#[test]
fn clamp_with_minmax_max_only() {
    assert_eq!(sizes::clamp_with_minmax(200.0, None, Some(150.0)), 150.0);
    assert_eq!(sizes::clamp_with_minmax(100.0, None, Some(150.0)), 100.0);
}

#[test]
fn clamp_with_minmax_both() {
    assert_eq!(sizes::clamp_with_minmax(50.0, Some(80.0), Some(200.0)), 80.0);
    assert_eq!(sizes::clamp_with_minmax(150.0, Some(80.0), Some(200.0)), 150.0);
    assert_eq!(sizes::clamp_with_minmax(250.0, Some(80.0), Some(200.0)), 200.0);
}
