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
    // font-size: 2em → 2*16=32px. width: 5em → cascade uses parent 16px → 5*16=80px
    let (doc, ctx) = flex_lt(r#"
        <div style="font-size:2em; width:5em; height:50px">nested</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let el = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!((el.content.width - 80.0).abs() < 1.0,
        "5em at cascade-default 16px should be 80px, got {}", el.content.width);
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
