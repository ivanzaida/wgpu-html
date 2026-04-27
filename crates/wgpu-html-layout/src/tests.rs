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
    let tree = make("<body></body>");
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.margin_rect.x, 0.0);
    assert_eq!(root.margin_rect.y, 0.0);
    assert_eq!(root.margin_rect.w, 800.0);
    // No content + no explicit height → height collapses to 0.
    assert_eq!(root.margin_rect.h, 0.0);
}

#[test]
fn explicit_size_used_verbatim() {
    let tree = make(
        r#"<body style="width: 400px; height: 200px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.w, 400.0);
    assert_eq!(root.border_rect.h, 200.0);
}

#[test]
fn margin_offsets_border_rect() {
    let tree = make(
        r#"<body style="margin: 10px; width: 100px; height: 50px;"></body>"#,
    );
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
    let tree = make(
        r#"<body style="padding: 8px; width: 100px; height: 50px;"></body>"#,
    );
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
        r#"<body>
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
        r#"<body>
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
fn child_inherits_full_inner_width_when_no_width_set() {
    let tree = make(
        r#"<body style="padding: 10px;">
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
        r#"<body style="width: 400px;">
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
        r#"<body style="background-color: red; width: 10px; height: 10px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    let bg = root.background.expect("background should be set");
    // sRGB red → linear: r ≈ 1.0, g = 0, b = 0
    assert!((bg[0] - 1.0).abs() < 1e-6);
    assert_eq!(bg[1], 0.0);
    assert_eq!(bg[2], 0.0);
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
fn nested_body_div_div_stacks_correctly() {
    let tree = make(
        r#"<body>
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
    let tree = make(
        r#"<body style="width: 100px; padding: 10px; height: 50px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.content_rect.w, 100.0);
    assert_eq!(root.border_rect.w, 120.0); // 100 + 10*2 padding
}

#[test]
fn box_sizing_border_box_subtracts_padding() {
    // `width` is the border-box width; padding eats into the content.
    let tree = make(
        r#"<body style="box-sizing: border-box; width: 100px; padding: 10px; height: 50px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.w, 100.0);
    assert_eq!(root.content_rect.w, 80.0); // 100 - 10*2 padding
}

#[test]
fn box_sizing_border_box_subtracts_padding_from_height() {
    let tree = make(
        r#"<body style="box-sizing: border-box; width: 100px; height: 100px; padding: 10px;"></body>"#,
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
        r#"<body style="display: flex; width: 200px; height: 100px;">
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
        r#"<body style="display: flex; justify-content: center; width: 100px; height: 50px;">
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
        r#"<body style="display: flex; justify-content: flex-end; width: 100px; height: 50px;">
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
        r#"<body style="display: flex; justify-content: space-between; width: 120px; height: 50px;">
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
        r#"<body style="display: flex; justify-content: space-evenly; width: 120px; height: 50px;">
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
        r#"<body style="display: flex; gap: 8px; width: 200px; height: 50px;">
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
        r#"<body style="display: flex; align-items: center; width: 200px; height: 100px;">
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
        r#"<body style="display: flex; align-items: flex-end; width: 200px; height: 100px;">
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
        r#"<body style="display: flex; align-items: stretch; width: 200px; height: 80px;">
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
        r#"<body style="display: flex; flex-direction: column; width: 100px; height: 200px;">
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
        r#"<body style="display: flex; flex-direction: row-reverse; width: 200px; height: 50px;">
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
    // The parser wraps <style> + <div id="parent"> in a synthetic <body>;
    // pull #parent out via its known border-box width.
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
    let xs: Vec<f32> = parent
        .children
        .iter()
        .map(|c| c.margin_rect.x)
        .collect();
    assert_eq!(xs, vec![10.0, 45.0, 80.0]);
}

// ---------------------------------------------------------------------------
// box-sizing border-box overflow regression
// ---------------------------------------------------------------------------

#[test]
fn box_sizing_border_box_with_full_width_fits_within_container() {
    // Reproduces the original `width: 100% + padding` overflow bug:
    // with border-box the body now stays inside the viewport.
    let tree = make(
        r#"<body style="box-sizing: border-box; width: 100%; padding: 32px;"></body>"#,
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
    let tree = make(
        r#"<body style="width: 100px; height: 50px; border-width: 4px;"></body>"#,
    );
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
        r#"<body style="box-sizing: border-box;
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
        r#"<body style="width: 50px; height: 50px;
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
        r#"<body style="width: 100px; height: 50px;
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
        r#"<body style="width: 50px; height: 50px;
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
        r#"<body style="width: 50px; height: 50px;
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
        r#"<body style="width: 100px; height: 100px;
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
        r#"<body style="width: 100px; height: 200px;
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
        r#"<body style="width: 100px; height: 200px;
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
        r#"<body style="width: 100px; height: 50px;
                         background-color: red; padding: 10px;
                         border: 4px solid blue;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(r.background_rect, r.border_rect);
}

#[test]
fn background_clip_padding_box_strips_border() {
    let tree = make(
        r#"<body style="width: 100px; height: 50px;
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
        r#"<body style="width: 100px; height: 50px;
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
        r#"<body style="width: 100px; height: 50px;
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
        r#"<body style="width: 50px; height: 50px;
                         border-top-left-radius: -8px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    assert_eq!(r.top_left, Radius::zero());
}

#[test]
fn elliptical_radius_h_v_split() {
    let tree = make(
        r#"<body style="width: 200px; height: 100px;
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
        r#"<body style="width: 200px; height: 100px;
                         border-top-left-radius: 30px 10px;"></body>"#,
    );
    let r = layout(&tree, 800.0, 600.0).unwrap().border_radius;
    assert_eq!(r.top_left.h, 30.0);
    assert_eq!(r.top_left.v, 10.0);
}
