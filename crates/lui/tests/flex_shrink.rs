use lui_layout_old::LayoutBox;
use lui_tree::NodeRect;

#[test]
fn text_shrink_demo() {
  let html = include_str!("../../../demo/lui-demo/html/text-shrink.html");
  let mut tree = lui_parser::parse(html);

  // Register fonts so text shaping works.
  tree.register_system_fonts("sans-serif");

  // Cascade + layout.
  let cascaded = lui_style::cascade(&tree);
  let mut text_ctx = lui_text::TextContext::new(64);
  text_ctx.sync_fonts(&tree.fonts);
  let mut image_cache = lui_layout_old::ImageCache::default();
  let root =
    lui_layout_old::layout_with_text(&cascaded, &mut text_ctx, &mut image_cache, 800.0, 600.0, 1.0).expect("layout");

  // Populate Node::rect from LayoutBox by walking both trees.
  populate_rects(&mut tree, &root, &[]);

  let rows = tree.root.as_ref().unwrap().get_elements_by_class_name("tree-row");
  let second_row = rows.get(1).unwrap();

  println!("rows len = {}", rows.len());
  println!("second_row rect = {:?}", second_row.rect);
  for (i, child) in second_row.children.iter().enumerate() {
    println!(
      "  child {i}: border=({:.1},{:.1} {:.1}x{:.1})  edge-to-edge gap={:.1}",
      child.rect.map(|r| r.x).unwrap_or(0.0),
      child.rect.map(|r| r.y).unwrap_or(0.0),
      child.rect.map(|r| r.width).unwrap_or(0.0),
      child.rect.map(|r| r.height).unwrap_or(0.0),
      if i == 0 {
        0.0
      } else {
        let prev_r = second_row.children[i - 1].rect.unwrap();
        let curr_r = child.rect.unwrap();
        curr_r.x - (prev_r.x + prev_r.width)
      }
    );
  }
}

fn populate_rects(tree: &mut lui_tree::Tree, b: &LayoutBox, path: &[usize]) {
  if let Some(node) = tree.root.as_mut().and_then(|r| r.at_path_mut(path)) {
    node.rect = Some(NodeRect {
      x: b.border_rect.x,
      y: b.border_rect.y,
      width: b.border_rect.w,
      height: b.border_rect.h,
    });
  }
  for (i, child) in b.children.iter().enumerate() {
    let mut child_path = path.to_vec();
    child_path.push(i);
    populate_rects(tree, child, &child_path);
  }
}
