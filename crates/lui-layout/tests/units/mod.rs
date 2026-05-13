use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_core::{CssUnit, CssValue};
use lui_layout::{LayoutContext, engine::layout_tree, sizes};
use crate::helpers::*;

// ============================================================================
// 22. em/rem unit resolution tests
// ============================================================================

#[test]
fn em_unit_resolved_at_cascade_time() {
    // The cascade resolves em→px using parent_font_size (default 16px).
    // width:10em → 10*16=160px (cascade resolves before layout)
    let (doc, ctx) = flex_lt(r#"
        <div style="width:10em; height:50px">em-sized</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 160.0).abs() < 1.0,
        "10em at default 16px should be 160px, got {}", el.content.width);
}

#[test]
fn rem_unit_resolved_at_cascade_time() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:10rem; height:50px">rem-sized</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 160.0).abs() < 1.0,
        "10rem at root 16px should be 160px, got {}", el.content.width);
}

#[test]
fn em_height_resolved_at_cascade_time() {
    // 3em height at default 16px = 48px
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px; height:3em">tall</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.height - 48.0).abs() < 1.0,
        "3em at default 16px should be 48px, got {}", el.content.height);
}

#[test]
fn em_font_size_doubles_parent() {
    // font-size: 2em → 2*16=32px. width: 5em → 5 * element's font-size (32px) = 160px
    let (doc, ctx) = flex_lt(r#"
        <div style="font-size:2em; width:5em; height:50px">nested</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 160.0).abs() < 1.0,
        "5em at element's 32px font-size should be 160px, got {}", el.content.width);
}

#[test]
fn resolve_length_em_unit() {
    assert_eq!(
        sizes::resolve_length_ctx(
            Some(&CssValue::Dimension { value: 2.0, unit: CssUnit::Em }),
            0.0,
            &LayoutContext { parent_font_size: 20.0, root_font_size: 16.0,
                viewport_width: 800.0, viewport_height: 600.0,
                containing_width: 800.0, containing_height: 0.0 },
        ),
        Some(40.0)
    );
}

#[test]
fn resolve_length_rem_unit() {
    assert_eq!(
        sizes::resolve_length_ctx(
            Some(&CssValue::Dimension { value: 3.0, unit: CssUnit::Rem }),
            0.0,
            &LayoutContext { parent_font_size: 20.0, root_font_size: 16.0,
                viewport_width: 800.0, viewport_height: 600.0,
                containing_width: 800.0, containing_height: 0.0 },
        ),
        Some(48.0)
    );
}

#[test]
fn resolve_length_pt_unit() {
    let result = sizes::resolve_length(
        Some(&CssValue::Dimension { value: 12.0, unit: CssUnit::Pt }),
        0.0,
    );
    assert!((result.unwrap() - 16.0).abs() < 0.01, "12pt = 16px");
}

#[test]
fn calc_resolved_by_cascade() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:calc(100px + 50px); height:50px">calc</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 150.0).abs() < 1.0,
        "calc(100px + 50px) should be 150px, got {}", el.content.width);
}

#[test]
fn ch_unit_resolved_by_cascade() {
    // 1ch ≈ 0.5em ≈ 8px at default 16px font
    let (doc, ctx) = flex_lt(r#"
        <div style="width:10ch; height:50px">ch-sized</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 80.0).abs() < 1.0,
        "10ch at 16px should be ~80px (0.5em approx), got {}", el.content.width);
}

#[test]
fn ex_unit_resolved_by_cascade() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px; height:4ex">ex-sized</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.height - 32.0).abs() < 1.0,
        "4ex at 16px should be ~32px (0.5em approx), got {}", el.content.height);
}

// ============================================================================
// Viewport units
// ============================================================================

#[test]
fn vw_unit_resolves_against_viewport() {
    // 50vw at 800px viewport = 400px
    let (doc, ctx) = flex_lt(r#"
        <div style="width:50vw; height:50px">half viewport</div>
    "#, 800.0);
    let media = MediaContext { viewport_width: 800.0, viewport_height: 600.0, ..MediaContext::default() };
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 400.0).abs() < 1.0,
        "50vw at 800px viewport should be 400px, got {}", el.content.width);
}

#[test]
fn vh_unit_resolves_against_viewport() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px; height:100vh">full height</div>
    "#, 800.0);
    let media = MediaContext { viewport_width: 800.0, viewport_height: 600.0, ..MediaContext::default() };
    let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.height - 600.0).abs() < 1.0,
        "100vh at 600px viewport should be 600px, got {}", el.content.height);
}
