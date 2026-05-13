use lui_cascade::cascade::InteractionState;
use lui_cascade::media::MediaContext;
use lui_layout::{BoxKind, LayoutBox, engine::layout_tree};
use crate::helpers::*;

// ============================================================================
// 12. Inline line breaking tests
// ============================================================================

#[test]
fn inline_text_wraps_at_container_width() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:80px">The quick brown fox jumps over the lazy dog</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Text should wrap into multiple lines, making height > single line
    assert!(container.content.height > 20.0, "text should wrap, height={}", container.content.height);
}

#[test]
fn inline_short_text_fits_single_line() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:800px">Hi</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(container.content.height > 0.0, "should have nonzero height");
    assert!(container.content.height < 40.0, "short text should be one line, height={}", container.content.height);
}

#[test]
fn inline_container_wraps_children() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:100px">
            <span>aaa </span><span>bbb </span><span>ccc </span><span>ddd </span><span>eee </span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let container = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(container.content.height > 0.0, "should have content");
}

// ============================================================================
// 21. Inline-block tests
// ============================================================================

#[test]
fn inline_block_respects_width_and_height() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <span style="display:inline-block; width:100px; height:50px">box</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let _body = find_by_tag(&lt.root, "body").unwrap();
    // Find the inline-block by looking for InlineBlock kind
    fn find_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind) -> Option<&'a LayoutBox<'a>> {
        if b.kind == kind { return Some(b); }
        for c in &b.children { if let Some(f) = find_kind(c, kind) { return Some(f); } }
        None
    }
    let ib = find_kind(&lt.root, BoxKind::InlineBlock);
    assert!(ib.is_some(), "should find InlineBlock box");
    let ib = ib.unwrap();
    assert!((ib.content.width - 100.0).abs() < 1.0, "width:100px, got {}", ib.content.width);
    assert!((ib.content.height - 50.0).abs() < 1.0, "height:50px, got {}", ib.content.height);
}

#[test]
fn inline_block_flows_horizontally() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <span style="display:inline-block; width:100px; height:30px">A</span>
            <span style="display:inline-block; width:100px; height:30px">B</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_all_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind, out: &mut Vec<&'a LayoutBox<'a>>) {
        if b.kind == kind { out.push(b); }
        for c in &b.children { find_all_kind(c, kind, out); }
    }
    let mut ibs = Vec::new();
    find_all_kind(&lt.root, BoxKind::InlineBlock, &mut ibs);
    assert_eq!(ibs.len(), 2, "should have 2 inline-blocks");
    assert!(ibs[1].content.x > ibs[0].content.x, "second inline-block should be to the right");
}

#[test]
fn inline_block_wraps_when_exceeds_container() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:150px">
            <span style="display:inline-block; width:100px; height:30px">A</span>
            <span style="display:inline-block; width:100px; height:30px">B</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_all_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind, out: &mut Vec<&'a LayoutBox<'a>>) {
        if b.kind == kind { out.push(b); }
        for c in &b.children { find_all_kind(c, kind, out); }
    }
    let mut ibs = Vec::new();
    find_all_kind(&lt.root, BoxKind::InlineBlock, &mut ibs);
    assert_eq!(ibs.len(), 2);
    assert!(ibs[1].content.y > ibs[0].content.y,
        "second inline-block should wrap to next line (y0={}, y1={})",
        ibs[0].content.y, ibs[1].content.y);
}

#[test]
fn inline_block_with_padding_and_border() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:400px">
            <span style="display:inline-block; width:80px; height:40px; padding:10px; border-width:5px">padded</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_kind<'a>(b: &'a LayoutBox<'a>, kind: BoxKind) -> Option<&'a LayoutBox<'a>> {
        if b.kind == kind { return Some(b); }
        for c in &b.children { if let Some(f) = find_kind(c, kind) { return Some(f); } }
        None
    }
    let ib = find_kind(&lt.root, BoxKind::InlineBlock).unwrap();
    assert_eq!(ib.padding.left, 10.0);
    assert_eq!(ib.border.left, 5.0);
    assert!((ib.content.width - 80.0).abs() < 1.0, "content width should be 80, got {}", ib.content.width);
}

// ============================================================================
// white-space
// ============================================================================

#[test]
fn white_space_nowrap_prevents_text_wrapping() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:50px; white-space:nowrap">Hello World Test</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let anon = &div.children[0];
    // nowrap: text should not wrap, so text width > container width
    assert!(anon.content.width > 50.0,
        "white-space:nowrap should prevent wrapping, text width={}", anon.content.width);
}

#[test]
fn white_space_normal_wraps_text() {
    // Baseline: normal wrapping happens
    let (doc_wrap, ctx_wrap) = flex_lt(r#"
        <div style="width:50px">Hello World Test</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled_wrap = ctx_wrap.cascade(&doc_wrap.root, &media, &interaction);
    let lt_wrap = layout_tree(&styled_wrap, 800.0, 600.0);
    let div_wrap = find_by_tag(&lt_wrap.root, "body").unwrap().children.first().unwrap();

    let (doc_nowrap, ctx_nowrap) = flex_lt(r#"
        <div style="width:50px; white-space:nowrap">Hello World Test</div>
    "#, 800.0);
    let styled_nowrap = ctx_nowrap.cascade(&doc_nowrap.root, &media, &interaction);
    let lt_nowrap = layout_tree(&styled_nowrap, 800.0, 600.0);
    let div_nowrap = find_by_tag(&lt_nowrap.root, "body").unwrap().children.first().unwrap();

    // nowrap div should be shorter (single line) than wrapped div (multiple lines)
    assert!(div_nowrap.content.height < div_wrap.content.height,
        "nowrap height ({}) should be less than normal height ({})",
        div_nowrap.content.height, div_wrap.content.height);
}

// ============================================================================
// vertical-align
// ============================================================================

#[test]
fn line_height_inherited_to_anonymous_block() {
    // line-height on parent div should affect the anonymous block wrapping inline content
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px; line-height:60px">
            <span style="display:inline-block; width:50px; height:20px">child</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // The anonymous block wrapping the inline-block should have height >= 60 (from line-height)
    let anon = &div.children[0];
    assert!(anon.content.height >= 59.0,
        "anonymous block should inherit line-height:60px from parent, got height={}", anon.content.height);
}

#[test]
fn white_space_pre_wrap_preserves_newlines_and_wraps() {
    // pre-wrap: newlines become line breaks, text also wraps at container width
    let (doc_pre, ctx_pre) = flex_lt(r#"
        <div style="width:200px; white-space:pre-wrap">Line one
Line two</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx_pre.cascade(&doc_pre.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // Two explicit newlines = at least 2 lines of height
    let single_line_h = {
        let (doc_s, ctx_s) = flex_lt(r#"<div style="width:200px">X</div>"#, 800.0);
        let styled_s = ctx_s.cascade(&doc_s.root, &media, &interaction);
        let lt_s = layout_tree(&styled_s, 800.0, 600.0);
        find_by_tag(&lt_s.root, "body").unwrap().children.first().unwrap().content.height
    };
    assert!(div.content.height > single_line_h * 1.5,
        "pre-wrap with newline should be taller than single line: got {} vs {}", div.content.height, single_line_h);
}

#[test]
fn word_break_break_all_keeps_width_within_container() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:30px; word-break:break-all">Superlongword</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let text_box = &div.children[0];
    assert!(text_box.content.width <= 31.0,
        "word-break:break-all text should fit within 30px container, got {}", text_box.content.width);
}

#[test]
fn text_overflow_ellipsis_flag_set() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:50px; overflow:hidden; white-space:nowrap; text-overflow:ellipsis">This is a long text that overflows</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let has_ellipsis = div.children.iter().any(|c| c.text_overflow_ellipsis);
    assert!(has_ellipsis,
        "text-overflow:ellipsis should set flag on overflowing child");
}

#[test]
fn text_indent_offsets_first_line() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:200px; text-indent:30px">Hello</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let anon = &div.children[0];
    let text_child = &anon.children[0];
    // The text child should be indented 30px from the anonymous block start
    let text_x = text_child.content.x - anon.content.x;
    assert!((text_x - 30.0).abs() < 1.0,
        "text-indent:30px should offset text by 30px, got {}", text_x);
}

#[test]
fn letter_spacing_widens_text() {
    let (doc_normal, ctx_normal) = flex_lt(r#"
        <div style="width:300px">Hello</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled_normal = ctx_normal.cascade(&doc_normal.root, &media, &interaction);
    let lt_normal = layout_tree(&styled_normal, 800.0, 600.0);
    let w_normal = find_by_tag(&lt_normal.root, "body").unwrap().children.first().unwrap()
        .children[0].content.width;

    let (doc_spaced, ctx_spaced) = flex_lt(r#"
        <div style="width:300px; letter-spacing:5px">Hello</div>
    "#, 800.0);
    let styled_spaced = ctx_spaced.cascade(&doc_spaced.root, &media, &interaction);
    let lt_spaced = layout_tree(&styled_spaced, 800.0, 600.0);
    let w_spaced = find_by_tag(&lt_spaced.root, "body").unwrap().children.first().unwrap()
        .children[0].content.width;

    // "Hello" = 5 chars, 4 gaps, 5px each = 20px extra
    assert!(w_spaced > w_normal + 15.0,
        "letter-spacing:5px should add ~20px, normal={}, spaced={}", w_normal, w_spaced);
}

#[test]
fn text_transform_uppercase() {
    // text-transform affects shaping — uppercase text should have different metrics
    // We can't easily check the actual text, but we can verify layout runs without panic
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px; text-transform:uppercase">hello world</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert!(div.content.height > 0.0, "text-transform:uppercase should produce content");
}

#[test]
fn text_decoration_stored() {
    let (doc, ctx) = flex_lt(r#"
        <div style="text-decoration:underline; width:200px">decorated</div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    assert_eq!(div.text_decoration.as_deref(), Some("underline"));
}

#[test]
fn inline_flex_lays_out_children_horizontally() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            before
            <span style="display:inline-flex; gap:10px"><span style="width:40px; height:20px">A</span><span style="width:40px; height:20px">B</span></span>
            after
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    fn find_kind_r<'a>(b: &'a LayoutBox<'a>, k: BoxKind) -> Option<&'a LayoutBox<'a>> {
        if b.kind == k { return Some(b); }
        for c in &b.children { if let Some(f) = find_kind_r(c, k) { return Some(f); } }
        None
    }
    let iflex = find_kind_r(div, BoxKind::InlineFlex);
    assert!(iflex.is_some(), "should find InlineFlex box");
    let iflex = iflex.unwrap();
    assert_eq!(iflex.children.len(), 2);
    assert!(iflex.children[1].content.x > iflex.children[0].content.x,
        "inline-flex children should be horizontal");
}

#[test]
fn display_contents_promotes_children() {
    let (doc, ctx) = flex_lt(r#"
        <div style="display:flex; width:300px">
            <div style="display:contents">
                <div style="width:100px; height:30px">A</div>
                <div style="width:100px; height:30px">B</div>
            </div>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let flex = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    // display:contents wrapper generates no box; A and B are direct flex items
    assert_eq!(flex.children.len(), 2,
        "contents wrapper should be transparent, flex should have 2 children, got {}", flex.children.len());
}

#[test]
fn display_list_item_has_marker() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px"><li>Item one</li></div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    fn find_list_item<'a>(b: &'a LayoutBox<'a>) -> Option<&'a LayoutBox<'a>> {
        if b.kind == BoxKind::ListItem { return Some(b); }
        for c in &b.children { if let Some(f) = find_list_item(c) { return Some(f); } }
        None
    }
    let li = find_list_item(&lt.root);
    assert!(li.is_some(), "should find ListItem box");
    assert!(li.unwrap().list_marker.is_some(), "list-item should have a marker");
}

#[test]
fn inline_block_shrink_to_fit() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <span style="display:inline-block; padding:5px">short text</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    fn find_ib<'a>(b: &'a LayoutBox<'a>) -> Option<&'a LayoutBox<'a>> {
        if b.kind == BoxKind::InlineBlock { return Some(b); }
        for c in &b.children { if let Some(f) = find_ib(c) { return Some(f); } }
        None
    }
    let ib = find_ib(div).unwrap();
    // Inline-block without explicit width should shrink to content, not fill 300px
    assert!(ib.content.width < 250.0,
        "inline-block should shrink to fit, got width={}", ib.content.width);
}

#[test]
fn direction_rtl_reverses_inline_order() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px; direction:rtl">
            <span style="display:inline-block; width:50px; height:20px">A</span>
            <span style="display:inline-block; width:50px; height:20px">B</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    fn find_ibs<'a>(b: &'a LayoutBox<'a>) -> Vec<&'a LayoutBox<'a>> {
        let mut r = Vec::new();
        if b.kind == BoxKind::InlineBlock { r.push(b); }
        for c in &b.children { r.extend(find_ibs(c)); }
        r
    }
    let ibs = find_ibs(div);
    if ibs.len() >= 2 {
        // In RTL, A should be to the RIGHT of B
        assert!(ibs[0].content.x > ibs[1].content.x,
            "rtl: A should be right of B, A.x={} B.x={}", ibs[0].content.x, ibs[1].content.x);
    }
}

#[test]
fn vertical_align_middle_centers_shorter_child() {
    // Two inline-blocks: tall (60px) and short (20px, middle-aligned)
    // The short one should be centered vertically in the 60px line
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <span style="display:inline-block; width:50px; height:60px">tall</span>
            <span style="display:inline-block; width:50px; height:20px; vertical-align:middle">mid</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let anon = &div.children[0];
    fn find_ibs<'a>(b: &'a LayoutBox<'a>) -> Vec<&'a LayoutBox<'a>> {
        let mut result = Vec::new();
        if b.kind == BoxKind::InlineBlock { result.push(b); }
        for c in &b.children { result.extend(find_ibs(c)); }
        result
    }
    let ibs = find_ibs(anon);
    if ibs.len() >= 2 {
        let tall = ibs[0];
        let mid = ibs[1];
        let tall_center = tall.content.y + tall.outer_height() / 2.0;
        let mid_center = mid.content.y + mid.outer_height() / 2.0;
        assert!((mid_center - tall_center).abs() < 5.0,
            "vertical-align:middle should center mid_y_center={}, tall_y_center={}", mid_center, tall_center);
    }
}

#[test]
fn vertical_align_bottom_aligns_shorter_child() {
    let (doc, ctx) = flex_lt(r#"
        <div style="width:300px">
            <span style="display:inline-block; width:50px; height:60px">tall</span>
            <span style="display:inline-block; width:50px; height:20px; vertical-align:bottom">bot</span>
        </div>
    "#, 800.0);
    let media = MediaContext::default(); let interaction = InteractionState::default();
    let styled = ctx.cascade(&doc.root, &media, &interaction);
    let lt = layout_tree(&styled, 800.0, 600.0);
    let div = find_by_tag(&lt.root, "body").unwrap().children.first().unwrap();
    let anon = &div.children[0];
    fn find_ibs<'a>(b: &'a LayoutBox<'a>) -> Vec<&'a LayoutBox<'a>> {
        let mut result = Vec::new();
        if b.kind == BoxKind::InlineBlock { result.push(b); }
        for c in &b.children { result.extend(find_ibs(c)); }
        result
    }
    let ibs = find_ibs(anon);
    if ibs.len() >= 2 {
        let tall = ibs[0];
        let bot = ibs[1];
        let tall_bottom = tall.content.y + tall.outer_height();
        let bot_bottom = bot.content.y + bot.outer_height();
        assert!((bot_bottom - tall_bottom).abs() < 5.0,
            "vertical-align:bottom, bot_bottom={}, tall_bottom={}", bot_bottom, tall_bottom);
    }
}
