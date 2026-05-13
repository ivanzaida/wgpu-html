use lui_cascade::ComputedStyle;
use lui_core::Rect;
use lui_parse::{HtmlElement, HtmlNode};
use lui_layout::{BoxKind, LayoutBox, LayoutTree, RectEdges};

#[test]
fn layout_box_new_sets_all_fields() {
    let node = HtmlNode::new(HtmlElement::Div);
    let style = ComputedStyle::default();
    let b = LayoutBox::new(BoxKind::Block, &node, &style);

    assert_eq!(b.kind, BoxKind::Block, "kind should be Block");
    assert!(std::ptr::eq(b.node, &node), "node pointer should match");
    assert_eq!(b.margin, RectEdges::default(), "margin should default to zero");
    assert_eq!(b.border, RectEdges::default(), "border should default to zero");
    assert_eq!(b.padding, RectEdges::default(), "padding should default to zero");
    assert_eq!(b.content, Rect::default(), "content should default to zero rect");
    assert!(b.intrinsic.is_none(), "intrinsic should be None");
    assert!(b.children.is_empty(), "children should be empty");
}

#[test]
fn layout_box_outer_width_sums_edges_and_content() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("test");
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    b.margin = RectEdges::new(5.0, 10.0, 3.0, 2.0);   // horizontal = 12
    b.border = RectEdges::new(1.0, 1.0, 1.0, 1.0);     // horizontal = 2
    b.padding = RectEdges::new(4.0, 4.0, 4.0, 4.0);    // horizontal = 8
    b.content.width = 100.0;
    // outer = 12 + 2 + 8 + 100 = 122
    assert_eq!(b.outer_width(), 122.0);
}

#[test]
fn layout_box_outer_height_sums_edges_and_content() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("test");
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    b.margin = RectEdges::new(5.0, 0.0, 10.0, 0.0);    // vertical = 15
    b.border = RectEdges::new(2.0, 0.0, 2.0, 0.0);      // vertical = 4
    b.padding = RectEdges::new(3.0, 0.0, 3.0, 0.0);     // vertical = 6
    b.content.height = 50.0;
    // outer = 15 + 4 + 6 + 50 = 75
    assert_eq!(b.outer_height(), 75.0);
}

#[test]
fn layout_box_border_rect_computes_correctly() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("test");
    let mut b = LayoutBox::new(BoxKind::Block, &node, &style);
    b.content = Rect::new(20.0, 30.0, 100.0, 50.0);
    b.border = RectEdges::new(2.0, 3.0, 2.0, 3.0);
    b.padding = RectEdges::new(8.0, 6.0, 8.0, 6.0);

    let br = b.border_rect();
    // x = 20 - 3 - 6 = 11
    // y = 30 - 2 - 8 = 20
    // width = 100 + (3+3) + (6+6) = 100 + 6 + 12 = 118
    // height = 50 + (2+2) + (8+8) = 50 + 4 + 16 = 70
    assert_eq!(br.x, 11.0, "border_rect x");
    assert_eq!(br.y, 20.0, "border_rect y");
    assert_eq!(br.width, 118.0, "border_rect width");
    assert_eq!(br.height, 70.0, "border_rect height");
}

#[test]
fn layout_tree_find_rect_returns_some_when_rect_exists() {
    let style = ComputedStyle::default();
    let node = HtmlNode::text("find_me");
    let rect = Rect::new(10.0, 20.0, 50.0, 30.0);

    let root_box = LayoutBox::new(BoxKind::Block, &node, &style);
    let tree = LayoutTree {
        root: root_box,
        rects: vec![(&node, rect)],
    };

    let found = tree.find_rect(&node);
    assert_eq!(found, Some(rect), "should find the rect for the matching node");
}

#[test]
fn layout_tree_find_rect_returns_none_when_rect_absent() {
    let style = ComputedStyle::default();
    let node_a = HtmlNode::text("a");
    let node_b = HtmlNode::text("b");
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);

    let root_box = LayoutBox::new(BoxKind::Block, &node_a, &style);
    let tree = LayoutTree {
        root: root_box,
        rects: vec![(&node_a, rect)],
    };

    let found = tree.find_rect(&node_b);
    assert_eq!(found, None, "should not find rect for a different node");
}
