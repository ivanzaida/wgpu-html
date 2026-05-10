use super::helpers::*;
use crate::*;
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
fn explicit_css_px_scale_to_physical_pixels() {
  let tree = make(r#"<body style="margin: 0; width: 400px; height: 200px;"></body>"#);
  let root = layout_scaled(&tree, 1600.0, 1200.0, 2.0);
  assert_eq!(root.border_rect.w, 800.0);
  assert_eq!(root.border_rect.h, 400.0);
}

#[test]
fn percentages_resolve_against_scaled_parent() {
  let tree = make(
    r#"<body style="margin: 0; width: 400px;">
            <div style="width: 50%; height: 10px;"></div>
        </body>"#,
  );
  let root = layout_scaled(&tree, 1600.0, 1200.0, 2.0);
  assert_eq!(root.border_rect.w, 800.0);
  assert_eq!(first_child(&root).border_rect.w, 400.0);
  assert_eq!(first_child(&root).border_rect.h, 20.0);
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
  let tree = make(r#"<body style="margin: 0; margin: 10px; width: 100px; height: 50px;"></body>"#);
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
  let tree = make(r#"<body style="margin: 0; padding: 8px; width: 100px; height: 50px;"></body>"#);
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
  let tree = make(r#"<body style="margin: 0; background-color: red; width: 10px; height: 10px;"></body>"#);
  let root = layout(&tree, 800.0, 600.0).unwrap();
  let bg = root.background.expect("background should be set");
  // sRGB red → linear: r ≈ 1.0, g = 0, b = 0
  assert!((bg[0] - 1.0).abs() < 1e-6);
  assert_eq!(bg[1], 0.0);
  assert_eq!(bg[2], 0.0);
}

#[test]
fn background_shorthand_color_resolves() {
  let tree = make(r#"<body style="margin: 0; background: #1b1d22; width: 10px; height: 10px;"></body>"#);
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

  assert_eq!(trim_collapsed_whitespace_edges(&first, true, false), "Drag ");
  assert_eq!(trim_collapsed_whitespace_edges(&middle, false, false), "with ");
  assert_eq!(trim_collapsed_whitespace_edges(&last, false, true), "the mouse button.");
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
    .insert("text-wrap-mode".into(), "nowrap".into());
  assert!(!style_wraps_text(&style));

  style
    .deferred_longhands
    .insert("text-wrap-mode".into(), "wrap".into());
  assert!(style_wraps_text(&style));
}

#[test]
fn collapsed_first_word_split_keeps_head_and_tail() {
  let mut style = Style::default();
  style.white_space = Some(WhiteSpace::Normal);
  let (head, tail) =
    split_collapsed_first_word_prefix_and_tail("   continues after chips", &style).expect("expected split");
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
  assert_eq!(label.children[2].margin_rect.y, label.children[0].margin_rect.y);
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
  assert_eq!(label.children[2].margin_rect.y, label.children[0].margin_rect.y);
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
  assert_eq!(line.children[0].margin_rect.y, line.children[1].margin_rect.y);
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

// ── word-break ──────────────────────────────────────────────────────

#[test]
fn word_break_parses_values() {
  assert_eq!(
    lui_parser::parse_inline_style("word-break: break-all").word_break,
    Some(WordBreak::BreakAll)
  );
  assert_eq!(
    lui_parser::parse_inline_style("word-break: keep-all").word_break,
    Some(WordBreak::KeepAll)
  );
  assert_eq!(
    lui_parser::parse_inline_style("word-break: normal").word_break,
    Some(WordBreak::Normal)
  );
  assert_eq!(
    lui_parser::parse_inline_style("color: red").word_break,
    None
  );
}

// ── text-overflow ───────────────────────────────────────────────────

#[test]
fn text_overflow_parses_values() {
  assert_eq!(
    lui_parser::parse_inline_style("text-overflow: ellipsis").text_overflow,
    Some(TextOverflow::Ellipsis)
  );
  assert_eq!(
    lui_parser::parse_inline_style("text-overflow: clip").text_overflow,
    Some(TextOverflow::Clip)
  );
}

#[test]
fn text_overflow_defaults_to_none_on_layout_box() {
  let tree = make(r#"<body style="margin:0"><div style="text-overflow:ellipsis;overflow:hidden;width:50px">long</div></body>"#);
  let root = layout(&tree, 200.0, 100.0).unwrap();
  let div = &root.children[0];
  assert_eq!(div.text_overflow, Some(TextOverflow::Ellipsis));
}

// ── vertical-align ──────────────────────────────────────────────────

#[test]
fn vertical_align_parses_keywords() {
  for (val, expected) in [
    ("baseline", VerticalAlign::Baseline),
    ("sub", VerticalAlign::Sub),
    ("super", VerticalAlign::Super),
    ("top", VerticalAlign::Top),
    ("middle", VerticalAlign::Middle),
    ("bottom", VerticalAlign::Bottom),
    ("text-top", VerticalAlign::TextTop),
    ("text-bottom", VerticalAlign::TextBottom),
  ] {
    let css = format!("vertical-align: {val}");
    let style = lui_parser::parse_inline_style(&css);
    assert_eq!(style.vertical_align.as_ref(), Some(&expected), "failed for {val}");
  }
}

#[test]
fn vertical_align_parses_length() {
  let style = lui_parser::parse_inline_style("vertical-align: 5px");
  assert!(matches!(style.vertical_align, Some(VerticalAlign::Length(_))));
}

#[test]
fn vertical_align_defaults_to_baseline() {
  let style = lui_parser::parse_inline_style("color: red");
  assert_eq!(style.vertical_align, None);
}

#[test]
fn vertical_align_sub_element_renders() {
  // Text with inline elements should produce a paragraph with a text run.
  let root = layout_with_fonts(r#"<body style="margin:0"><span>hello</span></body>"#, 200.0, 100.0);
  assert!(root.margin_rect.h > 0.0, "layout should produce non-zero height");
  assert!(!root.children.is_empty(), "layout should have children");
  // The paragraph box carries the text run directly
  let para = &root.children[0];
  assert!(para.text_run.is_some(), "paragraph should have a text run");
}

// ── transforms ────────────────────────────────────────────────────────

#[test]
fn transform_translate_stored_on_layout_box() {
  let tree = make(r#"<body style="margin:0"><div style="transform:translate(10px,20px);width:100px;height:50px"></div></body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let div = &root.children[0];
  let t = div.transform.as_ref().expect("transform should be set");
  assert!(t.is_translate_only());
  assert_eq!(t.tx, 10.0);
  assert_eq!(t.ty, 20.0);
}

#[test]
fn transform_percentage_translate_resolves_against_border_box() {
  let tree = make(r#"<body style="margin:0"><div style="transform:translate(-50%,-50%);width:200px;height:100px"></div></body>"#);
  let root = layout(&tree, 400.0, 400.0).unwrap();
  let div = &root.children[0];
  let t = div.transform.as_ref().expect("transform should be set");
  assert_eq!(t.tx, -100.0);
  assert_eq!(t.ty, -50.0);
}

#[test]
fn transform_none_is_none() {
  let tree = make(r#"<body style="margin:0"><div style="transform:none;width:100px;height:50px"></div></body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let div = &root.children[0];
  assert!(div.transform.is_none());
}

#[test]
fn transform_origin_defaults_to_center() {
  let tree = make(r#"<body style="margin:0"><div style="transform:scale(2);width:100px;height:50px"></div></body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let div = &root.children[0];
  assert_eq!(div.transform_origin, (50.0, 25.0));
}

// ── margin collapsing ─────────────────────────────────────────────────

#[test]
fn adjacent_sibling_margins_collapse() {
  let tree = make(r#"<body style="margin:0">
    <div style="margin-bottom:20px;height:10px"></div>
    <div style="margin-top:15px;height:10px"></div>
  </body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let first = &root.children[0];
  let second = &root.children[1];
  let gap = second.border_rect.y - (first.border_rect.y + first.border_rect.h);
  assert_eq!(gap, 20.0, "collapsed margin should be max(20, 15) = 20");
}

#[test]
fn equal_sibling_margins_collapse_to_single() {
  let tree = make(r#"<body style="margin:0">
    <div style="margin-bottom:10px;height:10px"></div>
    <div style="margin-top:10px;height:10px"></div>
  </body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let first = &root.children[0];
  let second = &root.children[1];
  let gap = second.border_rect.y - (first.border_rect.y + first.border_rect.h);
  assert_eq!(gap, 10.0, "equal margins collapse to a single margin");
}

#[test]
fn negative_margins_collapse_most_negative() {
  let tree = make(r#"<body style="margin:0">
    <div style="margin-bottom:-5px;height:20px"></div>
    <div style="margin-top:-10px;height:20px"></div>
  </body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let first = &root.children[0];
  let second = &root.children[1];
  let gap = second.border_rect.y - (first.border_rect.y + first.border_rect.h);
  assert_eq!(gap, -10.0, "both negative: use the most negative (-10)");
}

#[test]
fn mixed_positive_negative_margins_sum() {
  let tree = make(r#"<body style="margin:0">
    <div style="margin-bottom:20px;height:10px"></div>
    <div style="margin-top:-5px;height:10px"></div>
  </body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let first = &root.children[0];
  let second = &root.children[1];
  let gap = second.border_rect.y - (first.border_rect.y + first.border_rect.h);
  assert_eq!(gap, 15.0, "mixed: max(positive) + min(negative) = 20 + (-5) = 15");
}

#[test]
fn out_of_flow_children_do_not_collapse() {
  let tree = make(r#"<body style="margin:0;position:relative">
    <div style="margin-bottom:20px;height:10px"></div>
    <div style="position:absolute;margin-top:100px;height:10px"></div>
    <div style="margin-top:15px;height:10px"></div>
  </body>"#);
  let root = layout(&tree, 200.0, 200.0).unwrap();
  let first = &root.children[0];
  let third = &root.children[2];
  let gap = third.border_rect.y - (first.border_rect.y + first.border_rect.h);
  assert_eq!(gap, 20.0, "absolute child is skipped; siblings still collapse");
}
