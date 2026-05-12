use lui::{paint::*, renderer::DisplayList, text::TextContext};
use lui_layout_old::LayoutBox;

#[test]
fn paints_single_styled_box() {
  let tree = lui_parser::parse(r#"<body style="width: 100px; height: 50px; background-color: red;"></body>"#);
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  let q = list.quads[0];
  assert_eq!(q.rect.w, 100.0);
  assert_eq!(q.rect.h, 50.0);
}

#[test]
fn opacity_multiplies_background_through_subtree() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0; opacity: 0.5;">
                <div style="opacity: 0.5; width: 100px; height: 50px; background-color: blue;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 1);
  assert!((list.quads[0].color[3] - 0.25).abs() < 0.001);
}

#[test]
fn skips_boxes_without_background() {
  let tree = lui_parser::parse("<div><p>hi</p></div>");
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(list.quads.is_empty());
}

#[test]
fn child_uses_block_flow_position() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <div style="height: 64px; background-color: blue;"></div>
                <div style="height: 30px; background-color: red;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert_eq!(list.quads.len(), 2);
  assert_eq!(list.quads[0].rect.y, 0.0);
  assert_eq!(list.quads[1].rect.y, 64.0);
}

#[test]
fn svg_test_demo_paints_svg_images() {
  let tree = lui_parser::parse(include_str!("../../../../demo-v1/lui-demo/html/svg-test.html"));
  fn count_svg_nodes(node: &lui_tree::Node) -> usize {
    let own = matches!(node.element, lui_tree::Element::Svg(_)) as usize;
    own + node.children.iter().map(count_svg_nodes).sum::<usize>()
  }
  assert_eq!(
    tree.root.as_ref().map(count_svg_nodes).unwrap_or(0),
    8,
    "svg-test.html should parse all inline SVG elements"
  );
  let mut text_ctx = TextContext::new(64);
  let mut image_cache = lui_layout_old::ImageCache::default();
  let layout = lui_layout_old::layout_with_text(
    &lui_style::cascade(&tree),
    &mut text_ctx,
    &mut image_cache,
    1920.0,
    1080.0,
    1.0,
  )
  .expect("layout");
  fn count_layout_images(b: &LayoutBox) -> usize {
    (b.image.is_some() && b.content_rect.w > 0.0 && b.content_rect.h > 0.0) as usize
      + b.children.iter().map(count_layout_images).sum::<usize>()
  }
  fn collect_image_rects(b: &LayoutBox, out: &mut Vec<(f32, f32, f32, f32)>) {
    if b.image.is_some() {
      let r = b.content_rect;
      out.push((r.x, r.y, r.w, r.h));
    }
    for child in &b.children {
      collect_image_rects(child, out);
    }
  }
  let mut image_rects = Vec::new();
  collect_image_rects(&layout, &mut image_rects);
  assert_eq!(
    count_layout_images(&layout),
    8,
    "layout should attach visible rasterized image data to each SVG: {image_rects:?}"
  );
  assert!(
    image_rects
      .iter()
      .all(|(_, _, w, h)| (*w - 160.0).abs() < 0.01 && (*h - 160.0).abs() < 0.01),
    "each SVG should keep its authored 160x160 content box: {image_rects:?}"
  );
  let mut scaled_text_ctx = TextContext::new(64);
  let mut scaled_image_cache = lui_layout_old::ImageCache::default();
  let scaled_layout = lui_layout_old::layout_with_text(
    &lui_style::cascade(&tree),
    &mut scaled_text_ctx,
    &mut scaled_image_cache,
    1920.0,
    1080.0,
    1.5,
  )
  .expect("scaled layout");
  let mut scaled_image_rects = Vec::new();
  collect_image_rects(&scaled_layout, &mut scaled_image_rects);
  assert!(
    scaled_image_rects
      .iter()
      .all(|(_, _, w, h)| (*w - 240.0).abs() < 0.01 && (*h - 240.0).abs() < 0.01),
    "scaled SVG content boxes should apply scale once: {scaled_image_rects:?}"
  );
  let mut direct = DisplayList::new();
  paint_layout(&layout, &mut direct);
  assert_eq!(
    direct.images.len(),
    8,
    "painting the computed layout should emit image quads"
  );
  let list = paint_tree(&tree, 1920.0, 1080.0);

  assert_eq!(
    list.images.len(),
    8,
    "svg-test.html should emit one image quad for each inline SVG"
  );
  for (idx, img) in list.images.iter().enumerate() {
    assert!(
      img.data.chunks_exact(4).any(|px| px[3] > 0),
      "SVG image {idx} must contain at least one visible pixel"
    );
  }
  assert!(
    list
      .commands
      .iter()
      .filter(|cmd| cmd.kind == lui::renderer::DisplayCommandKind::Image)
      .count()
      == 8,
    "every SVG image should have an ordered image command"
  );
}

#[test]
fn template_contents_do_not_paint() {
  let tree = lui_parser::parse(
    r#"<body style="margin: 0;">
                <template>
                    <div style="width: 100px; height: 50px; background-color: red;"></div>
                </template>
                <div style="width: 100px; height: 50px; background-color: blue;"></div>
            </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  assert!(!list.quads.iter().any(|q| q.color == [1.0, 0.0, 0.0, 1.0]));
  assert!(list.quads.iter().any(|q| q.color == [0.0, 0.0, 1.0, 1.0]));
}

#[test]
fn z_index_sorts_positioned_children_in_paint_order() {
  let tree = lui_parser::parse(
    r#"<body style="margin:0; width:200px; height:100px;">
          <div style="position:absolute; z-index:10; width:50px; height:50px;
                      background-color:#00f; left:0; top:0;"></div>
          <div style="position:absolute; z-index:5; width:50px; height:50px;
                      background-color:#f00; left:10px; top:10px;"></div>
          <div style="position:absolute; z-index:auto; width:50px; height:50px;
                      background-color:#0f0; left:20px; top:20px;"></div>
        </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let colors: Vec<_> = list.quads.iter().map(|q| q.color).collect();
  assert_eq!(colors[0], [0.0, 1.0, 0.0, 1.0]);
  assert_eq!(colors[1], [1.0, 0.0, 0.0, 1.0]);
  assert_eq!(colors[2], [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn negative_z_index_paints_behind_non_positioned_siblings() {
  let tree = lui_parser::parse(
    r#"<body style="margin:0; width:200px; height:100px;">
          <div style="position:absolute; z-index:-1; width:80px; height:80px;
                      background-color:#f00; left:0; top:0;"></div>
          <div style="position:relative; width:50px; height:50px;
                      background-color:#00f;"></div>
        </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let colors: Vec<_> = list.quads.iter().map(|q| q.color).collect();
  assert_eq!(colors.len(), 2);
  assert_eq!(colors[0], [1.0, 0.0, 0.0, 1.0]);
  assert_eq!(colors[1], [0.0, 0.0, 1.0, 1.0]);
}

#[test]
fn absolute_z_neg1_child_paints_behind_normal_flow_sibling_with_margins() {
  let tree = lui_parser::parse(
    r#"<body style="margin:0;">
          <div style="position:relative; width:200px; height:80px;
                      background-color:#222;">
            <div style="position:absolute; z-index:-1; left:10px; top:10px;
                        width:80px; height:60px; background-color:#f00;"></div>
            <div style="width:100px; height:30px; margin-top:25px; margin-left:50px;
                        background-color:#00f;"></div>
          </div>
        </body>"#,
  );
  let list = paint_tree(&tree, 800.0, 600.0);
  let colors: Vec<_> = list.quads.iter().map(|q| q.color).collect();
  assert!(!colors.is_empty(), "no quads emitted");
  let red_idx = colors.iter().position(|c| *c == [1.0, 0.0, 0.0, 1.0]);
  let blue_idx = colors.iter().position(|c| *c == [0.0, 0.0, 1.0, 1.0]);
  assert!(red_idx.is_some(), "red quad not found; colors: {colors:?}");
  assert!(blue_idx.is_some(), "blue quad not found; colors: {colors:?}");
  assert!(red_idx < blue_idx, "red (z=-1) must paint before blue");
}
