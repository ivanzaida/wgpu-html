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
    let tree = make(
        r#"<body style="margin: 0; width: 400px; height: 200px;"></body>"#,
    );
    let root = layout(&tree, 800.0, 600.0).unwrap();
    assert_eq!(root.border_rect.w, 400.0);
    assert_eq!(root.border_rect.h, 200.0);
}

#[test]
fn margin_offsets_border_rect() {
    let tree = make(
        r#"<body style="margin: 0; margin: 10px; width: 100px; height: 50px;"></body>"#,
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
        r#"<body style="margin: 0; padding: 8px; width: 100px; height: 50px;"></body>"#,
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
    let tree = make(
        r#"<body style="margin: 0; width: 100px; padding: 10px; height: 50px;"></body>"#,
    );
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
    let xs: Vec<f32> = parent
        .children
        .iter()
        .map(|c| c.margin_rect.x)
        .collect();
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
    let tree = make(
        r#"<body style="margin: 0; width: 100px; height: 50px; border-width: 4px;"></body>"#,
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
    let node = lay.find_element_from_point(&mut tree, (20.0, 20.0)).unwrap();
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
            div.style = Some(
                "width: 123px; height: 40px; margin: 10px 0 0 10px;".to_string(),
            );
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
