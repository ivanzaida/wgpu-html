use super::*;

fn first_child(b: &LayoutBox) -> &LayoutBox {
    b.children.first().expect("expected a child")
}

fn make(html: &str) -> CascadedTree {
    wgpu_html_style::cascade(&wgpu_html_parser::parse(html))
}

#[test]
fn empty_tree_has_no_layout() {
    let tree = make("");
    assert!(layout(&tree, 800.0, 600.0).is_none());
}

#[test]
fn body_fills_viewport_with_no_explicit_size() {
    let tree = make(r#"<body style="margin: 0;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.margin_rect.x, 0.0);
    assert_eq!(root.margin_rect.y, 0.0);
    assert_eq!(root.margin_rect.w, 800.0);
    // No content + no explicit height → height collapses to 0.
    assert_eq!(root.margin_rect.h, 0.0);
}

#[test]
fn explicit_size_used_verbatim() {
    let tree = make(r#"<body style="margin: 0; width: 400px; height: 200px;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.w, 400.0);
    assert_eq!(root.border_rect.h, 200.0);
}

#[test]
fn opacity_is_carried_to_layout_and_clamped() {
    let tree = make(
        r#"<body style="margin: 0; opacity: 2;">
            <div style="opacity: 0.35; width: 10px; height: 10px;"></div>
            <div style="opacity: -1; width: 10px; height: 10px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.opacity, 1.0);
    assert!((root.children[0].opacity - 0.35).abs() < 0.001);
    assert_eq!(root.children[1].opacity, 0.0);
}

#[test]
fn calc_width_resolves_mixed_percent_and_px() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="width: calc(50% - 20px); height: 10px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let child = first_child(&root);
    assert_eq!(child.border_rect.w, 380.0);
}

#[test]
fn min_max_clamp_lengths_resolve() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="width: min(100%, 300px); height: clamp(10px, 5vw, 80px);"></div>
            <div style="width: max(20px, 10%); height: 10px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.children[0].border_rect.w, 300.0);
    assert_eq!(root.children[0].border_rect.h, 40.0);
    assert_eq!(root.children[1].border_rect.w, 80.0);
}

#[test]
fn calc_numeric_functions_resolve_inside_lengths() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="width: calc(pow(2, 3) * 10px); height: calc(sqrt(16) * 5px);"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let child = first_child(&root);
    assert_eq!(child.border_rect.w, 80.0);
    assert_eq!(child.border_rect.h, 20.0);
}

#[test]
fn margin_offsets_border_rect() {
    let tree =
        make(r#"<body style="margin: 0; margin: 10px; width: 100px; height: 50px;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    // margin_rect outer
    assert_eq!(root.margin_rect.x, 0.0);
    assert_eq!(root.margin_rect.y, 0.0);
    assert_eq!(root.margin_rect.w, 120.0);
    assert_eq!(root.margin_rect.h, 70.0);
    // border_rect inset by margin
    assert_eq!(root.border_rect.x, 10.0);
    assert_eq!(root.border_rect.y, 10.0);
    assert_eq!(root.border_rect.w, 100.0);
    assert_eq!(root.border_rect.h, 50.0);
}

#[test]
fn padding_shrinks_content_rect() {
    let tree =
        make(r#"<body style="margin: 0; padding: 8px; width: 100px; height: 50px;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    // border_rect = content + padding on each side → 100 + 16 = 116, 50 + 16 = 66
    assert_eq!(root.border_rect.w, 116.0);
    assert_eq!(root.border_rect.h, 66.0);
    // content_rect inset by padding
    assert_eq!(root.content_rect.x, 8.0);
    assert_eq!(root.content_rect.y, 8.0);
    assert_eq!(root.content_rect.w, 100.0);
    assert_eq!(root.content_rect.h, 50.0);
}

#[test]
fn children_stack_vertically_with_auto_height() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="height: 30px;"></div>
            <div style="height: 50px;"></div>
            <div style="height: 20px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.children.len(), 3);
    let ys: Vec<f32> = root.children.iter().map(|c| c.margin_rect.y).collect();
    assert_eq!(ys, vec![0.0, 30.0, 80.0]);
    // Body content height fits children: 30 + 50 + 20 = 100
    assert_eq!(root.content_rect.h, 100.0);
}

#[test]
fn child_margin_separates_siblings() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="height: 30px; margin-bottom: 8px;"></div>
            <div style="height: 30px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let kids: Vec<&LayoutBox> = root.children.iter().collect();
    // First child's margin_rect height = 30 + 8 = 38
    assert_eq!(kids[0].margin_rect.h, 38.0);
    // Second child sits below the margin box of the first: y = 38
    assert_eq!(kids[1].margin_rect.y, 38.0);
}

#[test]
fn absolute_positioned_child_does_not_affect_normal_flow() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="position: relative; height: 100px;">
                <div style="position: absolute; top: 0; left: 0; width: 50px; height: 500px;"></div>
            </div>
            <div style="height: 20px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.children[0].border_rect.h, 100.0);
    assert_eq!(root.children[0].children[0].border_rect.h, 500.0);
    assert_eq!(root.children[1].border_rect.y, 100.0);
}

#[test]
fn absolute_inset_fills_positioned_parent_padding_box() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="position: relative; width: 200px; height: 100px; padding: 10px;">
                <div style="position: absolute; inset: 0;"></div>
            </div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let parent = &root.children[0];
    let overlay = &parent.children[0];
    assert_eq!(parent.border_rect.w, 220.0);
    assert_eq!(parent.border_rect.h, 120.0);
    assert_eq!(overlay.border_rect.x, 0.0);
    assert_eq!(overlay.border_rect.y, 0.0);
    assert_eq!(overlay.border_rect.w, 220.0);
    assert_eq!(overlay.border_rect.h, 120.0);
}

#[test]
fn absolute_inset_from_stylesheet_uses_relative_parent() {
    let tree = make(
        r#"
        <style>
            body { margin: 0; padding: 32px; }
            .stage { position: relative; width: 620px; height: 320px; padding: 24px; border: 2px solid black; }
            .card { position: relative; width: 330px; height: 170px; padding: 18px; border: 2px solid blue; }
            .absolute-fill { position: absolute; inset: 0; border: 3px solid teal; }
        </style>
        <div class="stage">
            <div class="card">
                <div class="absolute-fill"></div>
            </div>
        </div>
        "#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let stage = &root.children[1];
    let card = &stage.children[0];
    let overlay = &card.children[0];
    assert_eq!(stage.border_rect.x, 32.0);
    assert_eq!(stage.border_rect.y, 32.0);
    assert_eq!(card.border_rect.x, 58.0);
    assert_eq!(card.border_rect.y, 58.0);
    assert_eq!(card.border_rect.w, 370.0);
    assert_eq!(card.border_rect.h, 210.0);
    assert_eq!(overlay.border_rect.x, 60.0);
    assert_eq!(overlay.border_rect.y, 60.0);
    assert_eq!(overlay.border_rect.w, 366.0);
    assert_eq!(overlay.border_rect.h, 206.0);
}

#[test]
fn absolute_right_auto_width_shrink_wraps_content() {
    let tree = make(
        r#"
        <style>
            body { margin: 0; }
            .card { position: relative; width: 300px; height: 120px; }
            .badge { position: absolute; top: 10px; right: 12px; padding: 6px 10px; }
        </style>
        <div class="card">
            <div class="badge">absolute</div>
        </div>
        "#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let badge = &root.children[1].children[0];
    assert!(
        badge.border_rect.w < 300.0,
        "auto-width absolute box should shrink-wrap, got {}",
        badge.border_rect.w
    );
    assert_eq!(badge.border_rect.x + badge.border_rect.w, 288.0);
    assert_eq!(badge.border_rect.y, 10.0);
}

#[test]
fn fixed_position_uses_viewport_as_containing_block() {
    let tree = make(
        r#"<body style="margin: 0; padding: 20px;">
            <div style="position: fixed; right: 10px; bottom: 15px; width: 50px; height: 25px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let fixed = &root.children[0];
    assert_eq!(fixed.border_rect.x, 740.0);
    assert_eq!(fixed.border_rect.y, 560.0);
}

#[test]
fn relative_position_offsets_box_but_preserves_flow_slot() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="position: relative; top: 10px; left: 5px; height: 20px;"></div>
            <div style="height: 20px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.children[0].border_rect.x, 5.0);
    assert_eq!(root.children[0].border_rect.y, 10.0);
    assert_eq!(root.children[1].border_rect.y, 20.0);
}

#[test]
fn child_inherits_full_inner_width_when_no_width_set() {
    let tree = make(
        r#"<body style="margin: 0; padding: 10px;">
            <div style="height: 20px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let child = first_child(&root);
    // Body content_rect.w = viewport - padding*2 = 780
    assert_eq!(root.content_rect.w, 780.0);
    // Child fills it
    assert_eq!(child.border_rect.w, 780.0);
    // Child x is inset by body's padding
    assert_eq!(child.border_rect.x, 10.0);
}

#[test]
fn percent_width_is_against_parent_content() {
    let tree = make(
        r#"<body style="margin: 0; width: 400px;">
            <div style="width: 50%; height: 10px;"></div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let child = first_child(&root);
    assert_eq!(child.border_rect.w, 200.0);
}

#[test]
fn background_color_resolves() {
    let tree = make(
        r#"<body style="margin: 0; background-color: red; width: 10px; height: 10px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let bg = root.background.expect("background should be set");
    // sRGB red → linear: r ≈ 1.0, g = 0, b = 0
    assert!((bg[0] - 1.0).abs() < 1e-6);
    assert_eq!(bg[1], 0.0);
    assert_eq!(bg[2], 0.0);
}

#[test]
fn background_shorthand_color_resolves() {
    let tree =
        make(r#"<body style="margin: 0; background: #1b1d22; width: 10px; height: 10px;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let bg = root.background.expect("background should be set");
    assert!((bg[0] - 0.010960094).abs() < 1e-6);
    assert!((bg[1] - 0.012286488).abs() < 1e-6);
    assert!((bg[2] - 0.015996294).abs() < 1e-6);
}

#[test]
fn text_nodes_are_zero_size_placeholders() {
    let tree = make("<p>hi</p>");
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let text = first_child(&root);
    assert_eq!(text.kind, BoxKind::Text);
    assert_eq!(text.margin_rect.w, 0.0);
    assert_eq!(text.margin_rect.h, 0.0);
}

#[test]
fn collapsed_block_text_trims_source_indentation_at_edges() {
    let collapsed = collapse_whitespace("\n  Drag with the left mouse button over text.\n");
    let visible = trim_collapsed_whitespace_edges(&collapsed, true, true);
    assert_eq!(visible, "Drag with the left mouse button over text.");
}

#[test]
fn paragraph_edge_trimming_keeps_internal_inline_spaces() {
    let first = collapse_whitespace("\n  Drag ");
    let middle = collapse_whitespace("with ");
    let last = collapse_whitespace("the mouse button.\n  ");

    assert_eq!(
        trim_collapsed_whitespace_edges(&first, true, false),
        "Drag "
    );
    assert_eq!(
        trim_collapsed_whitespace_edges(&middle, false, false),
        "with "
    );
    assert_eq!(
        trim_collapsed_whitespace_edges(&last, false, true),
        "the mouse button."
    );
}

#[test]
fn whitespace_collapse_carries_across_text_boundaries() {
    let mut style = Style::default();
    style.white_space = Some(WhiteSpace::Normal);
    let mut prev_space = false;
    let first = normalize_text_for_style("Drag ", &style, Some(&mut prev_space));
    let second = normalize_text_for_style(" with", &style, Some(&mut prev_space));
    assert_eq!(format!("{first}{second}"), "Drag with");
}

#[test]
fn pre_line_preserves_newlines_but_collapses_inline_spaces() {
    let mut style = Style::default();
    style.white_space = Some(WhiteSpace::PreLine);
    let mut prev_space = false;
    let out = normalize_text_for_style("a   b\n  c\t\t d", &style, Some(&mut prev_space));
    assert_eq!(out, "a b\n c d");
}

#[test]
fn wrap_policy_respects_white_space_and_text_wrap_mode() {
    let mut style = Style::default();
    style.white_space = Some(WhiteSpace::Nowrap);
    assert!(!style_wraps_text(&style));

    style.white_space = Some(WhiteSpace::PreWrap);
    assert!(style_wraps_text(&style));

    style
        .deferred_longhands
        .insert("text-wrap-mode".to_string(), "nowrap".to_string());
    assert!(!style_wraps_text(&style));

    style
        .deferred_longhands
        .insert("text-wrap-mode".to_string(), "wrap".to_string());
    assert!(style_wraps_text(&style));
}

#[test]
fn collapsed_first_word_split_keeps_head_and_tail() {
    let mut style = Style::default();
    style.white_space = Some(WhiteSpace::Normal);
    let (head, tail) =
        split_collapsed_first_word_prefix_and_tail("   continues after chips", &style)
            .expect("expected split");
    assert_eq!(head, " continues");
    assert_eq!(tail, " after chips");
}

#[test]
fn collapsed_first_word_split_returns_none_without_tail() {
    let mut style = Style::default();
    style.white_space = Some(WhiteSpace::Normal);
    assert!(split_collapsed_first_word_prefix_and_tail("continues", &style).is_none());
}

#[test]
fn img_is_inline_level_inside_text_flow() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div>Animated GIF — `<img>` intrinsic size</div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let label = &root.children[0];
    assert_eq!(label.children.len(), 3);
    assert_eq!(label.children[1].kind, BoxKind::Block);
    assert_eq!(label.children[1].margin_rect.w, 0.0);
    assert_eq!(label.children[1].margin_rect.h, 0.0);
    assert_eq!(
        label.children[2].margin_rect.y,
        label.children[0].margin_rect.y
    );
}

#[test]
fn sized_img_occupies_inline_replaced_space() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div>before <img style="width: 10px; height: 8px;"> after</div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let label = &root.children[0];
    let img = &label.children[1];
    assert_eq!(img.border_rect.w, 10.0);
    assert_eq!(img.border_rect.h, 8.0);
    assert_eq!(label.content_rect.h, 8.0);
    assert_eq!(
        label.children[2].margin_rect.y,
        label.children[0].margin_rect.y
    );
}

#[test]
fn inline_block_preserves_padding_margin_and_radius_geometry() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div>
                <span style="display: inline-block;
                             width: 40px; height: 20px;
                             padding: 4px 8px;
                             margin-right: 6px;
                             background-color: red;
                             border-radius: 999px;">x</span>
            </div>
        </body>"#,
    );
    let root = layout(&tree, 300.0, 200.0).unwrap();
    let line = &root.children[0];
    assert_eq!(line.children.len(), 1);
    let chip = &line.children[0];
    assert_eq!(chip.margin_rect.w, 40.0 + 16.0 + 6.0);
    assert_eq!(chip.border_rect.w, 40.0 + 16.0);
    assert_eq!(chip.border_rect.h, 20.0 + 8.0);
    assert!(chip.background.is_some());
    assert!(chip.border_radius.top_left.h > 0.0);
    assert!(chip.border_radius.top_left.h <= chip.border_rect.w * 0.5 + 0.1);
}

#[test]
fn inline_block_children_wrap_between_atomic_boxes() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div>
                <span style="display: inline-block; width: 70px; height: 20px; margin-right: 10px;
                             background-color: red;">a</span>
                <span style="display: inline-block; width: 70px; height: 20px; margin-right: 10px;
                             background-color: red;">b</span>
                <span style="display: inline-block; width: 70px; height: 20px;
                             background-color: red;">c</span>
            </div>
        </body>"#,
    );
    let root = layout(&tree, 170.0, 200.0).unwrap();
    let line = &root.children[0];
    assert_eq!(line.children.len(), 3);
    assert_eq!(
        line.children[0].margin_rect.y,
        line.children[1].margin_rect.y
    );
    assert!(line.children[2].margin_rect.y > line.children[0].margin_rect.y);
}

#[test]
fn nested_body_div_div_stacks_correctly() {
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="height: 64px; background-color: blue;"></div>
            <div style="padding: 10px;">
                <div style="height: 30px; background-color: red;"></div>
                <div style="height: 30px; background-color: green;"></div>
            </div>
        </body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.children.len(), 2);

    let header = &root.children[0];
    let card = &root.children[1];
    assert_eq!(header.margin_rect.y, 0.0);
    assert_eq!(header.margin_rect.h, 64.0);

    // Card sits below header.
    assert_eq!(card.margin_rect.y, 64.0);
    // Card auto-height: 10 padding top + 30 + 30 + 10 padding bottom = 80
    assert_eq!(card.border_rect.h, 80.0);

    // Card's children share its inner width = 800 - 20 padding = 780
    let card_child_0 = &card.children[0];
    let card_child_1 = &card.children[1];
    assert_eq!(card_child_0.border_rect.w, 780.0);
    assert_eq!(card_child_1.margin_rect.y, card.content_rect.y + 30.0);
}

// ---------------------------------------------------------------------------
// box-sizing
// ---------------------------------------------------------------------------

#[test]
fn box_sizing_content_box_is_default() {
    // `width` is the content-box width; padding is added on the outside.
    let tree =
        make(r#"<body style="margin: 0; width: 100px; padding: 10px; height: 50px;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.content_rect.w, 100.0);
    assert_eq!(root.border_rect.w, 120.0); // 100 + 10*2 padding
}

#[test]
fn box_sizing_border_box_subtracts_padding() {
    // `width` is the border-box width; padding eats into the content.
    let tree = make(
        r#"<body style="margin: 0; box-sizing: border-box; width: 100px; padding: 10px; height: 50px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.w, 100.0);
    assert_eq!(root.content_rect.w, 80.0); // 100 - 10*2 padding
}

#[test]
fn box_sizing_border_box_subtracts_padding_from_height() {
    let tree = make(
        r#"<body style="margin: 0; box-sizing: border-box; width: 100px; height: 100px; padding: 10px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.h, 100.0);
    assert_eq!(root.content_rect.h, 80.0);
}

// ---------------------------------------------------------------------------
// flex layout
// ---------------------------------------------------------------------------

#[test]
fn flex_row_default_packs_at_start() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 100px;">
            <div style="width: 30px; height: 30px;"></div>
            <div style="width: 40px; height: 30px;"></div>
            <div style="width: 50px; height: 30px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert_eq!(kids.len(), 3);
    assert_eq!(kids[0].margin_rect.x, 0.0);
    assert_eq!(kids[1].margin_rect.x, 30.0);
    assert_eq!(kids[2].margin_rect.x, 70.0);
}

#[test]
fn flex_row_justify_center() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; justify-content: center; width: 100px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    // Total used = 40, free = 60, center → start at 30.
    assert_eq!(body.children[0].margin_rect.x, 30.0);
    assert_eq!(body.children[1].margin_rect.x, 50.0);
}

#[test]
fn flex_row_justify_end() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; justify-content: flex-end; width: 100px; height: 50px;">
            <div style="width: 30px; height: 20px;"></div>
            <div style="width: 30px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.x, 40.0);
    assert_eq!(body.children[1].margin_rect.x, 70.0);
}

#[test]
fn flex_row_justify_space_between() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; justify-content: space-between; width: 120px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![0.0, 50.0, 100.0]);
}

#[test]
fn flex_row_justify_space_evenly() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; justify-content: space-evenly; width: 120px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![15.0, 50.0, 85.0]);
}

#[test]
fn flex_row_gap_separates_children() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; gap: 8px; width: 200px; height: 50px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![0.0, 28.0, 56.0]);
}

#[test]
fn flex_row_align_items_center() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; align-items: center; width: 200px; height: 100px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 60px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 40.0);
    assert_eq!(body.children[1].margin_rect.y, 20.0);
}

#[test]
fn flex_row_align_items_end() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; align-items: flex-end; width: 200px; height: 100px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 60px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 80.0);
    assert_eq!(body.children[1].margin_rect.y, 40.0);
}

#[test]
fn flex_row_align_items_stretch_fills_unspecified_height() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; align-items: stretch; width: 200px; height: 80px;">
            <div style="width: 20px;"></div>
            <div style="width: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].border_rect.h, 80.0);
    assert_eq!(body.children[1].border_rect.h, 80.0);
}

#[test]
fn flex_column_direction_stacks_vertically() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-direction: column; width: 100px; height: 200px;">
            <div style="width: 100%; height: 30px;"></div>
            <div style="width: 100%; height: 50px;"></div>
            <div style="width: 100%; height: 70px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let ys: Vec<f32> = body.children.iter().map(|c| c.margin_rect.y).collect();
    assert_eq!(ys, vec![0.0, 30.0, 80.0]);
}

#[test]
fn flex_row_reverse_orders_right_to_left() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-direction: row-reverse; width: 200px; height: 50px;">
            <div style="width: 30px; height: 30px;"></div>
            <div style="width: 30px; height: 30px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.x, 170.0);
    assert_eq!(body.children[1].margin_rect.x, 140.0);
}

#[test]
fn flex_test_html_layout() {
    // Direct port of crates/wgpu-html-demo/html/flex-test.html minus the
    // text content (which we don't render yet).
    let tree = make(
        r#"
        <style>
            body { margin: 0; }
            #parent {
                width: 100px;
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 10px;
            }
            .child { width: 30px; height: 30px; }
        </style>
        <div id="parent">
            <div class="child"></div>
            <div class="child"></div>
            <div class="child"></div>
        </div>
        "#,
    );
    // The parser wraps <style> + <div id="parent"> in a synthetic
    // <body>; the test's `body { margin: 0 }` overrides the UA's
    // `body { margin: 8px }` so coordinates start at the viewport
    // origin. Pull #parent out via its known border-box width.
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let parent = body
        .children
        .iter()
        .find(|c| c.border_rect.w == 120.0)
        .expect("parent div");
    // content-box: width=100 → border-box = 100 + 10*2 padding = 120.
    assert_eq!(parent.border_rect.w, 120.0);
    // 3 × 30 = 90 main used inside content_w=100, free=10 → space-between
    // gaps of 5, items at content_x=10, 45, 80.
    let xs: Vec<f32> = parent.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![10.0, 45.0, 80.0]);
}

// ---------------------------------------------------------------------------
// flex: grow / shrink / basis / wrap / align-content / align-self / order /
//       auto-margin / min-max / row-gap / column-gap
// ---------------------------------------------------------------------------

#[test]
fn flex_grow_splits_remaining_main_equally() {
    // Two grow=1 items inside a 200px row container → each takes 100px.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="flex-grow: 1; height: 20px;"></div>
            <div style="flex-grow: 1; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert!((kids[0].border_rect.w - 100.0).abs() < 0.01);
    assert!((kids[1].border_rect.w - 100.0).abs() < 0.01);
    assert!((kids[1].margin_rect.x - 100.0).abs() < 0.01);
}

#[test]
fn flex_grow_weighted_by_factor() {
    // grow ratios 1 : 2 split a 300px row as 100 : 200.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 300px; height: 50px;">
            <div style="flex-grow: 1; height: 20px;"></div>
            <div style="flex-grow: 2; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert!((kids[0].border_rect.w - 100.0).abs() < 0.01);
    assert!((kids[1].border_rect.w - 200.0).abs() < 0.01);
}

#[test]
fn flex_basis_overrides_width_for_main_size() {
    // flex-basis takes over from width for main-axis sizing.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 400px; height: 50px;">
            <div style="width: 200px; flex-basis: 100px; height: 20px; flex-grow: 0; flex-shrink: 0;"></div>
            <div style="width: 100px; flex-grow: 0; flex-shrink: 0; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    // flex-basis: 100px → first item 100, second still 100, total 200.
    assert!((body.children[0].border_rect.w - 100.0).abs() < 0.01);
    assert!((body.children[1].margin_rect.x - 100.0).abs() < 0.01);
}

#[test]
fn flex_shorthand_one_value_is_grow_with_zero_basis() {
    // `flex: 1` → grow=1, shrink=1, basis=0. Two items split the
    // entire 200px container even though they have no explicit width.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="flex: 1; height: 20px;"></div>
            <div style="flex: 1; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!((body.children[0].border_rect.w - 100.0).abs() < 0.01);
    assert!((body.children[1].border_rect.w - 100.0).abs() < 0.01);
}

#[test]
fn flex_shrink_reduces_overflowing_items() {
    // Three items 100px each in a 200px container with shrink=1
    // (default). Total would be 300; each item shrinks by 100/300 of
    // its base size → final 200/3 ≈ 66.67 each.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="width: 100px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let expected = 200.0_f32 / 3.0;
    for child in &body.children {
        assert!(
            (child.border_rect.w - expected).abs() < 0.1,
            "got {} expected {}",
            child.border_rect.w,
            expected
        );
    }
}

#[test]
fn flex_min_width_floors_shrunk_item() {
    // Without min-width, three 100px items would all shrink to 66.67 in
    // a 200px container. `min-width: 80px` on the first floors it.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="width: 100px; min-width: 80px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
            <div style="width: 100px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!((body.children[0].border_rect.w - 80.0).abs() < 0.05);
    // Remaining 120 split between two items → 60 each.
    assert!((body.children[1].border_rect.w - 60.0).abs() < 0.5);
    assert!((body.children[2].border_rect.w - 60.0).abs() < 0.5);
}

#[test]
fn flex_max_width_caps_grown_item() {
    // grow=1 on both, but `max-width: 80px` caps the first.
    // Remaining free space goes to the second.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="flex-grow: 1; max-width: 80px; height: 20px;"></div>
            <div style="flex-grow: 1; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!((body.children[0].border_rect.w - 80.0).abs() < 0.5);
    assert!((body.children[1].border_rect.w - 120.0).abs() < 0.5);
}

#[test]
fn flex_wrap_breaks_to_new_line() {
    // Three 80px items in a 200px wrap row → two lines: [80, 80] and [80].
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap; width: 200px;">
            <div style="width: 80px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 80px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 80px; height: 30px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    // First two on line 1 (y=0).
    assert_eq!(kids[0].margin_rect.y, 0.0);
    assert_eq!(kids[1].margin_rect.y, 0.0);
    // Third wraps to line 2 (y=30).
    assert_eq!(kids[2].margin_rect.y, 30.0);
    // Body content height = both lines' max cross sizes = 60.
    assert_eq!(body.content_rect.h, 60.0);
}

#[test]
fn flex_column_wrap_with_indefinite_height_stays_single_line() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-direction: column; flex-wrap: wrap; width: 120px;">
            <div style="width: 20px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 20px; height: 30px; flex-shrink: 0;"></div>
            <div style="width: 20px; height: 30px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 0.0);
    assert_eq!(body.children[1].margin_rect.y, 30.0);
    assert_eq!(body.children[2].margin_rect.y, 60.0);
    assert_eq!(body.children[0].margin_rect.x, 0.0);
    assert_eq!(body.children[1].margin_rect.x, 0.0);
    assert_eq!(body.children[2].margin_rect.x, 0.0);
    assert_eq!(body.content_rect.h, 90.0);
}

#[test]
fn flex_percent_cross_size_with_indefinite_cross_disables_stretch_but_lays_out_as_auto() {
    let tree = make(
        r#"<body style="margin: 0; display: flex; align-items: stretch; width: 200px;">
            <div style="width: 20px; height: 50%; padding: 5px;"></div>
            <div style="width: 20px; height: 40px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].border_rect.h, 10.0);
    assert_eq!(body.children[1].border_rect.h, 40.0);
}

#[test]
fn flex_align_self_overrides_align_items() {
    // align-items: center; one item overrides with align-self: flex-end.
    let tree = make(
        r#"<body style="margin: 0; display: flex; align-items: center;
                          width: 200px; height: 100px;">
            <div style="width: 20px; height: 20px;"></div>
            <div style="width: 20px; height: 20px; align-self: flex-end;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 40.0); // centered
    assert_eq!(body.children[1].margin_rect.y, 80.0); // pinned to bottom
}

#[test]
fn flex_align_content_center_with_two_lines() {
    // Two lines of 30px each in a 100px container → free 40, center
    // → first line at y=20, second at y=50.
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center;
                          width: 100px; height: 100px;">
            <div style="width: 100px; height: 30px;"></div>
            <div style="width: 100px; height: 30px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 20.0);
    assert_eq!(body.children[1].margin_rect.y, 50.0);
}

#[test]
fn flex_auto_margin_main_axis_pushes_to_end() {
    // `margin-left: auto` on the second item shoves it to the right.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 200px; height: 50px;">
            <div style="width: 30px; height: 20px;"></div>
            <div style="width: 30px; height: 20px; margin-left: auto;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.x, 0.0);
    // First item used 30, free = 200 - 60 = 140 → second at 30+140 = 170.
    assert_eq!(body.children[1].margin_rect.x, 170.0);
}

#[test]
fn flex_order_reorders_visual_layout() {
    // order: -1 puts the third item in front of the first two.
    let tree = make(
        r#"<body style="margin: 0; display: flex; width: 300px; height: 50px;">
            <div style="width: 50px; height: 20px;"></div>
            <div style="width: 50px; height: 20px;"></div>
            <div style="width: 50px; height: 20px; order: -1;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    // Source order is preserved on the layout tree (so hit-testing
    // stays consistent), but visual main-axis positions reflect the
    // re-ordered placement.
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    // The third source-order child (order=-1) is laid out first → x=0.
    assert_eq!(xs[2], 0.0);
    assert_eq!(xs[0], 50.0);
    assert_eq!(xs[1], 100.0);
}

#[test]
fn flex_row_gap_and_column_gap_independent() {
    // row-gap (cross axis) and column-gap (main axis) act on
    // different axes when wrap is on.
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          row-gap: 10px; column-gap: 5px;
                          width: 100px;">
            <div style="width: 40px; height: 20px; flex-shrink: 0;"></div>
            <div style="width: 40px; height: 20px; flex-shrink: 0;"></div>
            <div style="width: 40px; height: 20px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    // Line 1: items at x=0, x=45 (40 + 5 column-gap), but third
    // wraps because 40 + 5 + 40 + 5 + 40 = 130 > 100 → line 2.
    assert_eq!(kids[0].margin_rect.x, 0.0);
    assert_eq!(kids[1].margin_rect.x, 45.0);
    // Third is on line 2 with row-gap 10 below 20px line 1 → y=30.
    assert_eq!(kids[2].margin_rect.y, 30.0);
}

#[test]
fn min_width_clamps_block_size() {
    // Outside flex: a div with explicit width below `min-width` is
    // clamped up.
    let tree = make(
        r#"<body style="margin: 0;">
            <div style="width: 50px; min-width: 200px; height: 20px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].border_rect.w, 200.0);
}

#[test]
fn max_height_clamps_auto_height() {
    // No explicit height — children sum to 200, but `max-height: 80`
    // caps the block.
    let tree = make(
        r#"<body style="margin: 0; max-height: 80px;">
            <div style="height: 100px;"></div>
            <div style="height: 100px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.border_rect.h, 80.0);
}

#[test]
fn auto_horizontal_margins_center_block() {
    // Classic `margin: 0 auto` centering for a fixed-width child.
    let tree = make(
        r#"<body style="margin: 0; width: 400px;">
            <div style="width: 100px; height: 20px;
                         margin-left: auto; margin-right: auto;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let child = &body.children[0];
    // Free = 400 - 100 = 300; auto/auto → 150 left, 150 right.
    assert_eq!(child.border_rect.x, 150.0);
}

// ---------------------------------------------------------------------------
// box-sizing border-box overflow regression
// ---------------------------------------------------------------------------

#[test]
fn box_sizing_border_box_with_full_width_fits_within_container() {
    // Reproduces the original `width: 100% + padding` overflow bug:
    // with border-box the body now stays inside the viewport.
    let tree = make(
        r#"<body style="margin: 0; box-sizing: border-box; width: 100%; padding: 32px;"></body>"#,
    );
    let root = layout(&tree, 1024.0, 768.0).unwrap();
    assert_eq!(root.border_rect.w, 1024.0);
    assert_eq!(root.content_rect.x, 32.0);
    assert_eq!(root.content_rect.w, 960.0); // 1024 - 32*2
}

// ---------------------------------------------------------------------------
// border
// ---------------------------------------------------------------------------

#[test]
fn border_width_pushes_content_inward() {
    // content-box: width=100 + border-width=4 (each side) → border_rect=108.
    let tree =
        make(r#"<body style="margin: 0; width: 100px; height: 50px; border-width: 4px;"></body>"#);
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border.top, 4.0);
    assert_eq!(root.border.left, 4.0);
    assert_eq!(root.border_rect.w, 108.0);
    assert_eq!(root.border_rect.h, 58.0);
    assert_eq!(root.content_rect.x, 4.0);
    assert_eq!(root.content_rect.y, 4.0);
    assert_eq!(root.content_rect.w, 100.0);
}

#[test]
fn border_box_subtracts_border_too() {
    // box-sizing: border-box → 100px = border + padding + content.
    let tree = make(
        r#"<body style="margin: 0; box-sizing: border-box;
                         width: 100px; height: 100px;
                         border-width: 4px; padding: 8px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.w, 100.0);
    // 100 - border*2 (8) - padding*2 (16) = 76
    assert_eq!(root.content_rect.w, 76.0);
}

#[test]
fn border_color_resolves_for_paint() {
    let tree = make(
        r#"<body style="margin: 0; width: 50px; height: 50px;
                         border: 2px solid red;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    // The shorthand fans red to all four sides.
    let c = root.border_colors.top.expect("top");
    assert!((c[0] - 1.0).abs() < 1e-6);
    assert_eq!(c[1], 0.0);
    assert_eq!(c[2], 0.0);
    assert!(root.border_colors.left.is_some());
    assert!(root.border_colors.right.is_some());
    assert!(root.border_colors.bottom.is_some());
}

#[test]
fn per_side_border_widths_become_per_side_insets() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 50px;
                         border-width: 1px 2px 3px 4px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border.top, 1.0);
    assert_eq!(root.border.right, 2.0);
    assert_eq!(root.border.bottom, 3.0);
    assert_eq!(root.border.left, 4.0);
    // border_rect = content + horizontal/vertical border.
    assert_eq!(root.border_rect.w, 100.0 + 2.0 + 4.0);
    assert_eq!(root.border_rect.h, 50.0 + 1.0 + 3.0);
    // content offset by left/top borders.
    assert_eq!(root.content_rect.x, 4.0);
    assert_eq!(root.content_rect.y, 1.0);
}

#[test]
fn per_side_border_colors_make_their_way_to_layout() {
    let tree = make(
        r#"<body style="margin: 0; width: 50px; height: 50px;
                         border-width: 2px;
                         border-color: red green blue orange;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert!(root.border_colors.top.is_some());
    assert!(root.border_colors.right.is_some());
    assert!(root.border_colors.bottom.is_some());
    assert!(root.border_colors.left.is_some());
    // Different sides should resolve to different values.
    assert_ne!(root.border_colors.top, root.border_colors.right);
}

#[test]
fn border_radius_per_corner_lays_out() {
    let tree = make(
        r#"<body style="margin: 0; width: 50px; height: 50px;
                         border-radius: 1px 2px 3px 4px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_radius.top_left, Radius::circle(1.0));
    assert_eq!(root.border_radius.top_right, Radius::circle(2.0));
    assert_eq!(root.border_radius.bottom_right, Radius::circle(3.0));
    assert_eq!(root.border_radius.bottom_left, Radius::circle(4.0));
}

#[test]
fn radii_no_overflow_left_unchanged() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 100px;
                         border-radius: 10px 20px 30px 40px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    // 10+20=30 ≤ 100 (top), 30+40=70 ≤ 100 (bottom), 10+40=50 ≤ 100 (left), 20+30=50 ≤ 100 (right)
    assert_eq!(r.top_left, Radius::circle(10.0));
    assert_eq!(r.top_right, Radius::circle(20.0));
    assert_eq!(r.bottom_right, Radius::circle(30.0));
    assert_eq!(r.bottom_left, Radius::circle(40.0));
}

#[test]
fn radii_horizontal_overflow_scales_all_corners() {
    // Top side sum = 60 + 80 = 140 > 100 → scale = 100 / 140.
    // Final radii: each multiplied by 100/140 ≈ 0.7142857.
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 200px;
                         border-radius: 60px 80px 60px 80px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    let s = 100.0_f32 / 140.0;
    // Both axes scale uniformly for a circular input radius.
    assert!((r.top_left.h - 60.0 * s).abs() < 1e-3);
    assert!((r.top_right.h - 80.0 * s).abs() < 1e-3);
    assert!((r.bottom_right.h - 60.0 * s).abs() < 1e-3);
    assert!((r.bottom_left.h - 80.0 * s).abs() < 1e-3);
    assert!((r.top_left.h + r.top_right.h - 100.0).abs() < 1e-3);
}

#[test]
fn radii_smallest_factor_wins_when_multiple_sides_overflow() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 200px;
                         border-radius: 80px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    let s = 100.0_f32 / 160.0;
    assert!((r.top_left.h - 80.0 * s).abs() < 1e-3);
    assert!((r.bottom_right.h - 80.0 * s).abs() < 1e-3);
}

#[test]
fn background_clip_default_is_border_box() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(r.background_rect, r.border_rect);
}

#[test]
fn background_clip_padding_box_strips_border() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;
                         background-clip: padding-box;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(r.background_rect.x, r.border_rect.x + 4.0);
    assert_eq!(r.background_rect.y, r.border_rect.y + 4.0);
    assert_eq!(r.background_rect.w, r.border_rect.w - 8.0);
    assert_eq!(r.background_rect.h, r.border_rect.h - 8.0);
}

#[test]
fn background_clip_content_box_strips_border_and_padding() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;
                         background-clip: content-box;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(r.background_rect, r.content_rect);
}

#[test]
fn background_clip_padding_box_shrinks_radii() {
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 50px;
                         background-color: red;
                         border: 4px solid blue;
                         border-radius: 12px;
                         background-clip: padding-box;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(r.background_radii.top_left, Radius::circle(8.0));
    assert_eq!(r.background_radii.bottom_right, Radius::circle(8.0));
}

#[test]
fn radii_negative_input_clamped_to_zero() {
    // Negative px in the source → resolved to 0 by `.max(0.0)`.
    let tree = make(
        r#"<body style="margin: 0; width: 50px; height: 50px;
                         border-top-left-radius: -8px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    assert_eq!(r.top_left, Radius::zero());
}

#[test]
fn elliptical_radius_h_v_split() {
    let tree = make(
        r#"<body style="margin: 0; width: 200px; height: 100px;
                         border-radius: 20px / 10px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    assert_eq!(r.top_left.h, 20.0);
    assert_eq!(r.top_left.v, 10.0);
    assert_eq!(r.bottom_right.h, 20.0);
    assert_eq!(r.bottom_right.v, 10.0);
}

#[test]
fn per_corner_h_v_in_longhand() {
    let tree = make(
        r#"<body style="margin: 0; width: 200px; height: 100px;
                         border-top-left-radius: 30px 10px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    assert_eq!(r.top_left.h, 30.0);
    assert_eq!(r.top_left.v, 10.0);
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

const HIT_HTML: &str = r#"<body style="margin: 0; width: 800px; height: 600px;">
         <div style="width: 200px; height: 100px;">
           <div style="width: 50px; height: 40px;
                        margin: 10px 0 0 10px;"></div>
         </div>
       </body>"#;

fn hit_setup() -> (wgpu_html_tree::Tree, LayoutBox) {
    let tree = wgpu_html_parser::parse(HIT_HTML);
    let cascaded = wgpu_html_style::cascade(&tree);
    let lay = layout(&cascaded, 800.0, 600.0).unwrap();
    (tree, lay)
}

fn element_kind(n: &wgpu_html_tree::Node) -> &'static str {
    use wgpu_html_tree::Element;
    match &n.element {
        Element::Body(_) => "body",
        Element::Div(_) => "div",
        Element::Text(_) => "text",
        _ => "other",
    }
}

#[test]
fn hit_path_outside_is_none() {
    let (_tree, lay) = hit_setup();
    assert!(lay.hit_path((10_000.0, 10_000.0)).is_none());
}

#[test]
fn hit_path_drills_to_inner_div() {
    let (_tree, lay) = hit_setup();
    // (20, 20) lives inside the inner div: outer (idx 0) → inner (idx 0).
    let path = lay.hit_path((20.0, 20.0)).unwrap();
    assert_eq!(path, vec![0, 0]);
}

#[test]
fn find_element_outside_returns_none() {
    let (mut tree, lay) = hit_setup();
    assert!(
        lay.find_element_from_point(&mut tree, (10_000.0, 10_000.0))
            .is_none()
    );
}

#[test]
fn find_element_returns_deepest_node() {
    let (mut tree, lay) = hit_setup();
    let node = lay
        .find_element_from_point(&mut tree, (20.0, 20.0))
        .unwrap();
    assert_eq!(element_kind(node), "div");
    assert!(node.children.is_empty()); // it's the inner div
}

#[test]
fn find_element_lets_caller_mutate_style() {
    let (mut tree, lay) = hit_setup();
    {
        let node = lay
            .find_element_from_point(&mut tree, (20.0, 20.0))
            .unwrap();
        // The whole point of returning &mut Node: mutate the source
        // element's style attribute, then re-cascade and re-layout.
        if let wgpu_html_tree::Element::Div(div) = &mut node.element {
            div.style = Some("width: 123px; height: 40px; margin: 10px 0 0 10px;".to_string());
        } else {
            panic!("expected a Div at the hit point");
        }
    }
    let cascaded = wgpu_html_style::cascade(&tree);
    let lay2 = layout(&cascaded, 800.0, 600.0).unwrap();
    let inner = &lay2.children[0].children[0];
    assert_eq!(inner.border_rect.w, 123.0);
}

#[test]
fn find_element_falls_back_to_root_when_no_descendant_hit() {
    let (mut tree, lay) = hit_setup();
    // (300, 50) is inside body but past the outer div (only 200 wide).
    let node = lay
        .find_element_from_point(&mut tree, (300.0, 50.0))
        .unwrap();
    assert_eq!(element_kind(node), "body");
}

#[test]
fn find_elements_orders_child_to_parent() {
    let (mut tree, lay) = hit_setup();
    let chain = lay.find_elements_from_point(&mut tree, (20.0, 20.0));
    assert_eq!(chain.len(), 3);
    // Deepest first: inner div, outer div, body.
    assert_eq!(element_kind(chain[0]), "div");
    assert!(chain[0].children.is_empty());
    assert_eq!(element_kind(chain[1]), "div");
    assert_eq!(chain[1].children.len(), 1);
    assert_eq!(element_kind(chain[2]), "body");
}

#[test]
fn find_elements_outside_is_empty() {
    let (mut tree, lay) = hit_setup();
    assert!(
        lay.find_elements_from_point(&mut tree, (-1.0, -1.0))
            .is_empty()
    );
}

fn synthetic_text_layout() -> LayoutBox {
    let r = Rect::new(10.0, 20.0, 100.0, 24.0);
    LayoutBox {
        margin_rect: r,
        border_rect: r,
        content_rect: r,
        background: None,
        background_rect: r,
        background_radii: CornerRadii::zero(),
        border: Insets::zero(),
        border_colors: BorderColors::default(),
        border_styles: BorderStyles::default(),
        border_radius: CornerRadii::zero(),
        kind: BoxKind::Text,
        text_run: Some(ShapedRun {
            glyphs: vec![
                PositionedGlyph {
                    x: 0.0,
                    y: 0.0,
                    w: 10.0,
                    h: 20.0,
                    uv_min: [0.0, 0.0],
                    uv_max: [1.0, 1.0],
                    color: [0.0, 0.0, 0.0, 1.0],
                },
                PositionedGlyph {
                    x: 10.0,
                    y: 0.0,
                    w: 10.0,
                    h: 20.0,
                    uv_min: [0.0, 0.0],
                    uv_max: [1.0, 1.0],
                    color: [0.0, 0.0, 0.0, 1.0],
                },
                PositionedGlyph {
                    x: 20.0,
                    y: 0.0,
                    w: 10.0,
                    h: 20.0,
                    uv_min: [0.0, 0.0],
                    uv_max: [1.0, 1.0],
                    color: [0.0, 0.0, 0.0, 1.0],
                },
            ],
            lines: vec![wgpu_html_text::ShapedLine {
                top: 0.0,
                height: 20.0,
                glyph_range: (0, 3),
            }],
            text: "abc".to_string(),
            byte_boundaries: wgpu_html_text::utf8_boundaries("abc"),
            width: 30.0,
            height: 20.0,
            ascent: 15.0,
        }),
        text_color: Some([0.0, 0.0, 0.0, 1.0]),
        text_decorations: Vec::new(),
        overflow: OverflowAxes::visible(),
        opacity: 1.0,
        image: None,
        background_image: None,
        children: Vec::new(),
    }
}

#[test]
fn hit_text_cursor_maps_point_to_glyph_boundary() {
    let lay = synthetic_text_layout();
    let c0 = lay.hit_text_cursor((11.0, 24.0)).expect("cursor");
    let c1 = lay.hit_text_cursor((26.0, 24.0)).expect("cursor");
    let c2 = lay.hit_text_cursor((39.0, 24.0)).expect("cursor");
    assert_eq!(c0.glyph_index, 0);
    assert_eq!(c1.glyph_index, 2);
    assert_eq!(c2.glyph_index, 3);
}

#[test]
fn hit_text_cursor_outside_returns_none() {
    let lay = synthetic_text_layout();
    assert!(lay.hit_text_cursor((200.0, 24.0)).is_none());
}

// ---------------------------------------------------------------------------
// overflow propagation
// ---------------------------------------------------------------------------

#[test]
fn overflow_field_propagates_from_style() {
    let tree = make(r#"<body style="overflow: hidden; width: 100px; height: 50px;"></body>"#);
    let body = layout(&tree, 800.0, 600.0).unwrap();
    use wgpu_html_models::common::css_enums::Overflow;
    assert_eq!(body.overflow.x, Overflow::Hidden);
    assert_eq!(body.overflow.y, Overflow::Hidden);
}

#[test]
fn overflow_visible_is_default() {
    let tree = make(r#"<body style="width: 100px; height: 50px;"></body>"#);
    let body = layout(&tree, 800.0, 600.0).unwrap();
    use wgpu_html_models::common::css_enums::Overflow;
    assert_eq!(body.overflow.x, Overflow::Visible);
    assert_eq!(body.overflow.y, Overflow::Visible);
}

#[test]
fn overflow_axis_longhand_wins_over_shorthand() {
    let tree = make(
        r#"<body style="overflow: scroll; overflow-y: clip; width: 100px; height: 50px;"></body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    use wgpu_html_models::common::css_enums::Overflow;
    assert_eq!(body.overflow.x, Overflow::Scroll);
    assert_eq!(body.overflow.y, Overflow::Hidden);
}

#[test]
fn overflow_shorthand_two_values_sets_axes() {
    let tree = make(r#"<body style="overflow: clip visible; width: 100px; height: 50px;"></body>"#);
    let body = layout(&tree, 800.0, 600.0).unwrap();
    use wgpu_html_models::common::css_enums::Overflow;
    assert_eq!(body.overflow.x, Overflow::Clip);
    assert_eq!(body.overflow.y, Overflow::Visible);
}

#[test]
fn overflow_visible_computes_to_auto_against_scrollable_axis() {
    let tree =
        make(r#"<body style="overflow: hidden visible; width: 100px; height: 50px;"></body>"#);
    let body = layout(&tree, 800.0, 600.0).unwrap();
    use wgpu_html_models::common::css_enums::Overflow;
    assert_eq!(body.overflow.x, Overflow::Hidden);
    assert_eq!(body.overflow.y, Overflow::Auto);
}

#[test]
fn overflow_hidden_blocks_child_hit_outside_padding_box() {
    let tree = make(
        r#"<body style="margin: 0; width: 300px; height: 300px;">
            <div style="width: 100px; height: 100px; overflow: hidden;">
                <div style="width: 200px; height: 200px;"></div>
            </div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.hit_path((120.0, 20.0)).unwrap(), Vec::<usize>::new());
}

#[test]
fn overflow_visible_allows_child_hit_outside_parent() {
    let tree = make(
        r#"<body style="margin: 0; width: 300px; height: 300px;">
            <div style="width: 100px; height: 100px; overflow: visible;">
                <div style="width: 200px; height: 200px;"></div>
            </div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.hit_path((120.0, 20.0)).unwrap(), vec![0, 0]);
}

// ---------------------------------------------------------------------------
// CSS Grid: tracks, placement, alignment, gaps, distribution
// ---------------------------------------------------------------------------

#[test]
fn grid_two_by_two_fixed_columns() {
    // 2×2 fixed grid: columns 100px / 100px, rows 50px / 50px.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px;
                          grid-template-rows: 50px 50px;">
            <div></div>
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert_eq!(kids[0].margin_rect.x, 0.0);
    assert_eq!(kids[0].margin_rect.y, 0.0);
    assert_eq!(kids[1].margin_rect.x, 100.0);
    assert_eq!(kids[1].margin_rect.y, 0.0);
    assert_eq!(kids[2].margin_rect.x, 0.0);
    assert_eq!(kids[2].margin_rect.y, 50.0);
    assert_eq!(kids[3].margin_rect.x, 100.0);
    assert_eq!(kids[3].margin_rect.y, 50.0);
}

#[test]
fn grid_fr_distributes_remaining_width() {
    // 1fr 2fr inside a 300px container → 100 / 200.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 1fr 2fr;
                          width: 300px;">
            <div style="height: 50px;"></div>
            <div style="height: 50px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert!((kids[0].margin_rect.w - 100.0).abs() < 0.05);
    assert!((kids[1].margin_rect.w - 200.0).abs() < 0.05);
    assert!((kids[1].margin_rect.x - 100.0).abs() < 0.05);
}

#[test]
fn grid_repeat_expands_track_list() {
    // `repeat(3, 80px)` → three identical 80px columns.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: repeat(3, 80px);">
            <div style="height: 40px;"></div>
            <div style="height: 40px;"></div>
            <div style="height: 40px;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![0.0, 80.0, 160.0]);
    for c in &body.children {
        assert_eq!(c.margin_rect.w, 80.0);
    }
}

#[test]
fn grid_explicit_placement_via_grid_column() {
    // `grid-column: 2 / 4` covers cols 2 and 3 (sum = 200px).
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px 100px;
                          grid-template-rows: 60px;">
            <div style="grid-column: 2 / 4;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let item = &body.children[0];
    assert_eq!(item.margin_rect.x, 100.0);
    assert!((item.margin_rect.w - 200.0).abs() < 0.05);
}

#[test]
fn grid_span_shorthand() {
    // `grid-column: span 2` from auto-placement at col 0 covers cols
    // 0 and 1.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px 100px;
                          grid-template-rows: 60px;">
            <div style="grid-column: span 2;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let item = &body.children[0];
    assert_eq!(item.margin_rect.x, 0.0);
    assert!((item.margin_rect.w - 200.0).abs() < 0.05);
}

#[test]
fn grid_auto_flow_row_packs_in_source_order() {
    // Three items on a 3-column track → all on row 1.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 80px 80px 80px;
                          grid-auto-rows: 50px;">
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let ys: Vec<f32> = body.children.iter().map(|c| c.margin_rect.y).collect();
    assert_eq!(ys, vec![0.0, 0.0, 0.0]);
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![0.0, 80.0, 160.0]);
}

#[test]
fn grid_auto_flow_column_packs_vertically() {
    // Three items on a 3-row track in column-major flow → all in col 1.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-auto-flow: column;
                          grid-template-rows: 50px 50px 50px;
                          grid-auto-columns: 80px;">
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let xs: Vec<f32> = body.children.iter().map(|c| c.margin_rect.x).collect();
    assert_eq!(xs, vec![0.0, 0.0, 0.0]);
    let ys: Vec<f32> = body.children.iter().map(|c| c.margin_rect.y).collect();
    assert_eq!(ys, vec![0.0, 50.0, 100.0]);
}

#[test]
fn grid_implicit_rows_use_grid_auto_rows() {
    // Two items on a single explicit row → second wraps to an
    // implicit row sized at `grid-auto-rows: 70px`.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 50px;
                          grid-auto-rows: 70px;">
            <div></div>
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 0.0);
    // Second item lands in the implicit row at y = first row height (50).
    assert_eq!(body.children[1].margin_rect.y, 50.0);
}

#[test]
fn grid_row_gap_and_column_gap_separate_cells() {
    // `column-gap: 10px; row-gap: 5px;` with 100px tracks → second
    // column at x=110, second row at y=55.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px 100px;
                          grid-template-rows: 50px 50px;
                          column-gap: 10px;
                          row-gap: 5px;">
            <div></div>
            <div></div>
            <div></div>
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert_eq!(kids[1].margin_rect.x, 110.0);
    assert_eq!(kids[2].margin_rect.y, 55.0);
}

#[test]
fn grid_align_self_end_anchors_item_to_cell_bottom() {
    // Cell is 50px tall, item is 20px → align-self: end pushes the
    // item to y=30.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 50px;">
            <div style="height: 20px; align-self: end;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.y, 30.0);
}

#[test]
fn grid_justify_self_center_horizontally() {
    // Cell 100px, item 40px → centered at x=30.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 50px;">
            <div style="width: 40px; height: 20px; justify-self: center;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.x, 30.0);
}

#[test]
fn grid_align_items_stretch_default_fills_cell_vertically() {
    // No explicit height → default `align-items: stretch` makes the
    // child fill the cell's row height.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 100px;
                          grid-template-rows: 80px;">
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].border_rect.h, 80.0);
}

#[test]
fn grid_justify_content_center_centers_track_block() {
    // Two 80px columns in a 400px container with `justify-content:
    // center` → blocks start at (400 − 160) / 2 = 120.
    let tree = make(
        r#"<body style="margin: 0;
                          display: grid;
                          grid-template-columns: 80px 80px;
                          grid-template-rows: 40px;
                          justify-content: center;
                          width: 400px;">
            <div></div>
            <div></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(body.children[0].margin_rect.x, 120.0);
    assert_eq!(body.children[1].margin_rect.x, 200.0);
}

// --------------------------------------------------------------------------
// Cross-axis alignment regressions: a `flex-wrap: wrap` container whose
// items happen to fit on one line is *single-line* per CSS-Flex-1 §9.4
// step 15 — `align-content` has no effect there. These tests pin that
// behaviour so the `flex-grow.html` `.row-wrap` case (6×120 px items
// in a wide viewport, single line, `align-content: center`) doesn't
// silently start centering.
// --------------------------------------------------------------------------

#[test]
fn flex_wrap_single_line_ignores_align_content_per_spec() {
    // `flex-wrap: wrap` is set, but the items fit in one line, so the
    // container is single-line. `align-content: center` must be a
    // no-op: items stay at the top of the 160px-tall line (default
    // align-items: stretch + explicit item height ≡ flex-start).
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center;
                          width: 800px; height: 160px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    // All three items fit on one line → align-content is dropped.
    for c in &body.children {
        assert_eq!(c.margin_rect.y, 0.0, "single-line items pin to start");
        assert_eq!(c.margin_rect.h, 40.0, "explicit height is preserved");
    }
}

#[test]
fn flex_wrap_single_line_align_items_center_does_center() {
    // The fix for the flex-grow demo's 3rd row: pair the wrap
    // container with `align-items: center` so each item centers
    // within its (single) line. Same items as above; line cross size
    // = container height = 160; items 40px tall → centered y = 60.
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-items: center;
                          width: 800px; height: 160px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    for c in &body.children {
        assert_eq!(
            c.margin_rect.y, 60.0,
            "align-items: center centers each item in the single line"
        );
    }
}

#[test]
fn flex_wrap_actually_wraps_when_container_too_narrow() {
    // Drop the container width so 6 items of 120px each can't all
    // fit on one line. Now the container is multi-line and
    // `align-content: center` distributes free cross space.
    //
    // Container 400px wide, 8px gap → 3 items per line
    // (3*120 + 2*8 = 376 ≤ 400). 6 items → 2 lines.
    // total_lines_cross = 2 * 40 (line cross) + 1 * 8 (cross-gap) = 88.
    // Container cross = 200 → free = 112 → center start offset = 56.
    // Line 1 at y=56, line 2 at y = 56 + 40 + 8 = 104.
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center; gap: 8px;
                          width: 400px; height: 200px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    for c in &kids[0..3] {
        assert_eq!(c.margin_rect.y, 56.0);
    }
    for c in &kids[3..6] {
        assert_eq!(c.margin_rect.y, 104.0);
    }
}

#[test]
fn flex_align_self_center_overrides_default_alignment_on_single_line() {
    // Even on a single-line container with `align-content: center`
    // (which is ignored), an item with `align-self: center` still
    // centers itself per `align-self`. Documents that the per-item
    // override route works regardless of `align-content`.
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center;
                          width: 800px; height: 160px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;
                          align-self: center;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let kids = &body.children;
    assert_eq!(kids[0].margin_rect.y, 0.0, "default keeps top alignment");
    assert_eq!(kids[1].margin_rect.y, 60.0, "align-self: center wins");
    assert_eq!(kids[2].margin_rect.y, 0.0, "third stays at top");
}

// ── placeholder rendering ─────────────────────────────────────────────────
//
// The layout-test pipeline doesn't register a font, so
// `shape_text_run` short-circuits to `(None, …)` — `text_run` is
// `None` even when the placeholder code path fires. We therefore
// gate placeholder presence on `text_color`, which the helper
// always sets when it decides to emit a placeholder run.

fn any_box_with_placeholder(b: &LayoutBox) -> bool {
    b.text_color.is_some() || b.children.iter().any(any_box_with_placeholder)
}

fn first_box_with_placeholder(b: &LayoutBox) -> Option<&LayoutBox> {
    if b.text_color.is_some() {
        return Some(b);
    }
    for c in &b.children {
        if let Some(found) = first_box_with_placeholder(c) {
            return Some(found);
        }
    }
    None
}

#[test]
fn input_with_placeholder_attaches_placeholder_run() {
    // An empty `<input>` with a `placeholder` should drive the
    // layout box through the placeholder code path (visible via
    // `text_color` being set even when no font is registered).
    //
    // The UA stylesheet sets `color: fieldtext` (black) on inputs,
    // so the placeholder helper halves the alpha → `[0, 0, 0, 0.5]`.
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="text" placeholder="Type here">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let input = first_box_with_placeholder(&body)
        .expect("input box should carry placeholder color");
    let color = input.text_color.unwrap();
    assert_eq!(color, [0.0, 0.0, 0.0, 0.5]);
}

#[test]
fn input_with_value_suppresses_placeholder() {
    // A non-empty `value` overrides the placeholder; no
    // placeholder color should be set.
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="text" placeholder="Type here" value="actual content">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!(
        !any_box_with_placeholder(&body),
        "value=\"…\" should suppress placeholder rendering"
    );
}

#[test]
fn input_without_placeholder_has_no_placeholder_run() {
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="text">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!(!any_box_with_placeholder(&body));
}

#[test]
fn input_type_hidden_does_not_render_placeholder() {
    // `type="hidden"` is `display: none` per UA — even with a
    // placeholder, no placeholder run should be emitted.
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="hidden" placeholder="invisible">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!(!any_box_with_placeholder(&body));
}

#[test]
fn input_with_empty_placeholder_attribute_has_no_placeholder_run() {
    // `placeholder=""` is empty — no glyphs to shape, no run.
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="text" placeholder="">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    assert!(!any_box_with_placeholder(&body));
}

#[test]
fn textarea_with_placeholder_attaches_placeholder_run() {
    // Empty `<textarea>` with a `placeholder` attribute drives
    // the placeholder code path the same way.
    let tree = make(
        r#"<body style="margin: 0;">
            <textarea placeholder="A few words..."></textarea>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let ta = first_box_with_placeholder(&body).expect("textarea placeholder");
    assert!(ta.text_color.is_some());
}

#[test]
fn placeholder_respects_user_padding_shorthand() {
    // User CSS `padding: 8px 10px` should override the UA's
    // `padding-block: 1px; padding-inline: 2px`, so the input's
    // content_rect is inset by 8/10/8/10. The placeholder shaping
    // uses that content_rect — the box's content edges should
    // therefore match the user-specified padding.
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="text" placeholder="x"
                   style="padding: 8px 10px; border: 0; width: 100px;">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let input =
        first_box_with_placeholder(&body).expect("input box should carry a placeholder");
    let cr = input.content_rect;
    let br = input.border_rect;
    // border = 0, padding = 8 vertical / 10 horizontal.
    assert!(
        (cr.x - (br.x + 10.0)).abs() < 0.01,
        "left padding 10px not applied: cr.x={} br.x={}",
        cr.x, br.x
    );
    assert!(
        (cr.y - (br.y + 8.0)).abs() < 0.01,
        "top padding 8px not applied: cr.y={} br.y={}",
        cr.y, br.y
    );
    assert!(
        (cr.w - (br.w - 20.0)).abs() < 0.01,
        "horizontal padding 20px (10+10) not applied: cr.w={} br.w={}",
        cr.w, br.w
    );
}

#[test]
fn placeholder_color_uses_cascaded_color_with_half_alpha() {
    // When the cascaded `color` is set, placeholder colour =
    // `color` × alpha 0.5. With CSS `color: red`, alpha is 0.5
    // and the channels track red's linearised RGB.
    let tree = make(
        r#"<body style="margin: 0;">
            <input type="text" placeholder="hint" style="color: red;">
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    let input = first_box_with_placeholder(&body)
        .expect("input box should carry placeholder color");
    let color = input.text_color.unwrap();
    assert!(
        (color[3] - 0.5).abs() < 1e-4,
        "alpha should be halved (got {})",
        color[3]
    );
    // Linearised red ≈ 1.0; green/blue stay at 0.
    assert!(color[0] > 0.9, "red channel ≈ 1 (got {})", color[0]);
    assert_eq!(color[1], 0.0);
    assert_eq!(color[2], 0.0);
}

#[test]
fn textarea_in_flex_row_does_not_inflate_height() {
    // Regression guard for the "no text after textarea" symptom.
    // A flex row containing `<label>` + `<textarea height: 64px>`
    // should advance the body's block flow by ~64-72px (textarea
    // content + UA padding/border), not by hundreds of pixels —
    // which would push following siblings off-screen.
    let tree = make(
        r#"<body style="margin: 0; padding: 0;">
            <h2 style="font-size: 11px; margin: 0;">First</h2>
            <div style="display: flex; gap: 0;">
                <label>Bio</label>
                <textarea style="min-width: 320px; height: 64px;"></textarea>
            </div>
            <h2 style="font-size: 11px; margin: 0;">Second</h2>
        </body>"#,
    );
    let body = layout(&tree, 1280.0, 720.0).unwrap();
    let kids = &body.children;
    assert_eq!(kids.len(), 3, "body kids: {}", kids.len());
    let row = &kids[1];
    let after = &kids[2];
    assert!(
        after.margin_rect.y < row.margin_rect.y + 200.0,
        "h2 after textarea row sits at y={}, row starts at y={} (delta {} > 200) — \
         textarea row is sized way larger than its 64px height",
        after.margin_rect.y,
        row.margin_rect.y,
        after.margin_rect.y - row.margin_rect.y
    );
}

#[test]
fn flex_wrap_no_height_does_not_apply_align_content() {
    // Multi-line container with *no* explicit cross size: `align-content`
    // has nothing to distribute. Lines pack to start regardless of
    // the value, per CSS-Flex-1 §9.6 ("only meaningful with definite
    // cross size").
    let tree = make(
        r#"<body style="margin: 0; display: flex; flex-wrap: wrap;
                          align-content: center; gap: 0px;
                          width: 200px;">
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
            <div style="width: 120px; height: 40px; flex-shrink: 0;"></div>
        </body>"#,
    );
    let body = layout(&tree, 800.0, 600.0).unwrap();
    // Two lines (one item each), no extra cross space → both pack
    // to start: y=0 and y=40.
    assert_eq!(body.children[0].margin_rect.y, 0.0);
    assert_eq!(body.children[1].margin_rect.y, 40.0);
}
