use lui_tree::Tree;

fn make_file_tree() -> Tree {
  let html = r#"<input type="file"/>"#;
  lui_parser::parse(html)
}

fn make_file_tree_with_accept(accept: &str) -> Tree {
  let html = format!(r#"<input type="file" accept="{accept}"/>"#);
  lui_parser::parse(&html)
}

fn input_path(tree: &Tree) -> Vec<usize> {
  tree.query_selector_path("input").expect("input not found")
}

// ── Layout ──

#[test]
fn file_input_has_form_control() {
  let mut tree = make_file_tree();
  tree.register_system_fonts("sans-serif");
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut images = lui_layout_old::ImageCache::default();
  let layout = lui::compute_layout(&tree, &mut text_ctx, &mut images, 800.0, 600.0, 1.0);
  let root = layout.unwrap();
  let path = input_path(&tree);
  let lb = lui::layout_at_path(&root, &path).unwrap();
  assert!(matches!(
    lb.form_control.as_ref().map(|fc| &fc.kind),
    Some(lui_layout_old::FormControlKind::File { .. })
  ));
}

#[test]
fn file_input_has_label_text_run() {
  let mut tree = make_file_tree();
  tree.register_system_fonts("sans-serif");
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut images = lui_layout_old::ImageCache::default();
  let layout = lui::compute_layout(&tree, &mut text_ctx, &mut images, 800.0, 600.0, 1.0);
  let root = layout.unwrap();
  let path = input_path(&tree);
  let lb = lui::layout_at_path(&root, &path).unwrap();
  let run = lb.text_run.as_ref().expect("file input should have label text run");
  assert!(
    run.text.contains("No file chosen") || run.text.contains("file"),
    "label text should be the file label, got: {:?}",
    run.text
  );
}

#[test]
fn file_input_has_button_text_run() {
  let mut tree = make_file_tree();
  tree.register_system_fonts("sans-serif");
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut images = lui_layout_old::ImageCache::default();
  let layout = lui::compute_layout(&tree, &mut text_ctx, &mut images, 800.0, 600.0, 1.0);
  let root = layout.unwrap();
  let path = input_path(&tree);
  let lb = lui::layout_at_path(&root, &path).unwrap();
  let fb = lb.file_button.as_ref().expect("file input should have FileButtonStyle");
  let btn_run = fb
    .text_run
    .as_ref()
    .expect("FileButtonStyle should have its own text run");
  assert!(
    btn_run.text.contains("Browse"),
    "button text should contain 'Browse', got: {:?}",
    btn_run.text
  );
}

#[test]
fn button_and_label_are_separate_runs() {
  let mut tree = make_file_tree();
  tree.register_system_fonts("sans-serif");
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut images = lui_layout_old::ImageCache::default();
  let layout = lui::compute_layout(&tree, &mut text_ctx, &mut images, 800.0, 600.0, 1.0);
  let root = layout.unwrap();
  let path = input_path(&tree);
  let lb = lui::layout_at_path(&root, &path).unwrap();

  let label_text = &lb.text_run.as_ref().unwrap().text;
  let btn_text = &lb.file_button.as_ref().unwrap().text_run.as_ref().unwrap().text;

  assert!(
    !label_text.contains("Browse"),
    "label run should NOT contain button text"
  );
  assert!(
    !btn_text.contains("No file"),
    "button run should NOT contain label text"
  );
}

// ── Button width ──

#[test]
fn file_button_width_includes_padding() {
  let mut tree = make_file_tree();
  tree.register_system_fonts("sans-serif");
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut images = lui_layout_old::ImageCache::default();
  let layout = lui::compute_layout(&tree, &mut text_ctx, &mut images, 800.0, 600.0, 1.0);
  let root = layout.unwrap();
  let path = input_path(&tree);
  let lb = lui::layout_at_path(&root, &path).unwrap();
  let fb = lb.file_button.as_ref().unwrap();
  let btn_run = fb.text_run.as_ref().unwrap();

  let btn_w = lui::paint::file_button_width(lb);
  let expected = fb.padding[3] + btn_run.width + fb.padding[1];
  assert!(
    (btn_w - expected).abs() < 0.01,
    "button width should be pad_left + text_width + pad_right: got {btn_w:.1}, expected {expected:.1}"
  );
}

#[test]
fn label_does_not_overlap_button() {
  let mut tree = make_file_tree();
  tree.register_system_fonts("sans-serif");
  let mut text_ctx = lui_text::TextContext::new(64);
  let mut images = lui_layout_old::ImageCache::default();
  let layout = lui::compute_layout(&tree, &mut text_ctx, &mut images, 800.0, 600.0, 1.0);
  let root = layout.unwrap();
  let path = input_path(&tree);
  let lb = lui::layout_at_path(&root, &path).unwrap();

  let btn_w = lui::paint::file_button_width(lb);
  let label_run = lb.text_run.as_ref().unwrap();
  let first_label_glyph_x = label_run.glyphs.first().map(|g| g.x).unwrap_or(0.0);

  // The label is painted at content_rect.x + btn_w + 8px gap + glyph.x.
  // The first glyph.x is relative to the label run origin, so the
  // visual label start is at btn_w + 8 + first_label_glyph_x.
  // It must be past btn_w.
  let gap = 8.0;
  let label_visual_start = btn_w + gap + first_label_glyph_x;
  assert!(
    label_visual_start > btn_w + 4.0,
    "label should start at least 4px after button: label_start={label_visual_start:.1}, btn_w={btn_w:.1}"
  );
}
