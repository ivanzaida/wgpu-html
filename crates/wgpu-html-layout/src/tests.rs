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
