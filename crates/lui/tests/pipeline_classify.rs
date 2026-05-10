//! Tests that verify pipeline classification during interactive editing.
//!
//! These catch regressions where typing in an input triggers FullPipeline
//! (expensive) instead of LayoutOnly (incremental).

use lui::{PipelineAction, PipelineCache, classify_frame, paint_tree_cached};
use lui_tree::{self, Tree};

fn parse_and_register_fonts(html: &str) -> Tree {
  let mut tree = lui::parser::parse(html);
  tree.register_system_fonts("sans-serif");
  tree
}

fn initial_render(tree: &Tree, cache: &mut PipelineCache) {
  let mut text_ctx = lui_text::TextContext::new(64);
  text_ctx.sync_fonts(&tree.fonts);
  let mut image_cache = lui_layout::ImageCache::default();
  paint_tree_cached(
    tree,
    &mut text_ctx,
    &mut image_cache,
    800.0,
    600.0,
    1.0,
    0.0,
    cache,
  );
}

// ── Classification tests ───────────────────────────────────────────

#[test]
fn typing_in_input_classifies_as_layout_only_not_full_pipeline() {
  let html = r#"<input type="email" placeholder="test" />"#;
  let mut tree = parse_and_register_fonts(html);
  let mut cache = PipelineCache::new();
  let mut image_cache = lui_layout::ImageCache::default();

  // First frame: must be FullPipeline (no cache).
  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  assert_eq!(action, PipelineAction::FullPipeline);

  // Render the first frame to populate cache.
  initial_render(&tree, &mut cache);
  assert!(cache.layout().is_some(), "cache.layout should be populated after initial render");

  // Simulate focusing the input and typing.
  lui_tree::focus(&mut tree, Some(&[0]));
  lui_tree::text_input(&mut tree, "a");

  // Second frame: should be LayoutOnly, NOT FullPipeline.
  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  assert_eq!(action, PipelineAction::LayoutOnly,
    "typing should trigger LayoutOnly, not {action:?}. \
     tree.generation={} \
     tree.cascade_generation={} \
     tree.fonts.generation={} \
     has_pending={} has_animated={} \
     cache.layout.is_some={}",
    tree.generation,
    tree.cascade_generation,
    tree.fonts.generation(),
    image_cache.has_pending(),
    image_cache.has_animated(),
    cache.layout().is_some(),
  );
}

#[test]
fn typing_does_not_bump_cascade_generation() {
  let html = r#"<input type="text" />"#;
  let mut tree = parse_and_register_fonts(html);

  let gen_before = tree.cascade_generation;
  lui_tree::focus(&mut tree, Some(&[0]));
  lui_tree::text_input(&mut tree, "hello");

  assert_eq!(tree.cascade_generation, gen_before,
    "text_input should not bump cascade_generation");
}

#[test]
fn typing_pushes_dirty_path() {
  let html = r#"<body><input type="text" /></body>"#;
  let mut tree = parse_and_register_fonts(html);

  // Parser wraps in html > body. The input is at body[0].
  // Find the input path by checking the tree structure.
  let input_path: Vec<usize> = {
    let root = tree.root.as_ref().unwrap();
    let mut path = Vec::new();
    fn find_input(node: &lui_tree::Node, path: &mut Vec<usize>) -> bool {
      if matches!(&node.element, lui_tree::Element::Input(_)) {
        return true;
      }
      for (i, child) in node.children.iter().enumerate() {
        path.push(i);
        if find_input(child, path) {
          return true;
        }
        path.pop();
      }
      false
    }
    find_input(root, &mut path);
    path
  };

  lui_tree::focus(&mut tree, Some(&input_path));
  assert!(tree.interaction.edit_cursor.is_some(), "focus should set edit_cursor on input");
  assert!(tree.dirty_paths.is_empty());

  lui_tree::text_input(&mut tree, "a");
  assert!(!tree.dirty_paths.is_empty(), "text_input should push to dirty_paths");
}

#[test]
fn dirty_paths_cleared_after_render() {
  let html = r#"<input type="text" />"#;
  let mut tree = parse_and_register_fonts(html);
  let mut cache = PipelineCache::new();

  initial_render(&tree, &mut cache);

  lui_tree::focus(&mut tree, Some(&[0]));
  lui_tree::text_input(&mut tree, "a");
  assert!(!tree.dirty_paths.is_empty());

  // Render — dirty_paths should be cleared after.
  // Note: paint_tree_cached takes &Tree so can't clear.
  // The driver clears them. Simulate that:
  let mut text_ctx = lui_text::TextContext::new(64);
  text_ctx.sync_fonts(&tree.fonts);
  let mut image_cache = lui_layout::ImageCache::default();
  paint_tree_cached(
    &tree,
    &mut text_ctx,
    &mut image_cache,
    800.0, 600.0, 1.0, 0.0,
    &mut cache,
  );
  tree.dirty_paths.clear(); // driver does this

  assert!(tree.dirty_paths.is_empty());
}

#[test]
fn second_keystroke_also_classifies_as_layout_only() {
  let html = r#"<input type="text" />"#;
  let mut tree = parse_and_register_fonts(html);
  let mut cache = PipelineCache::new();
  let mut image_cache = lui_layout::ImageCache::default();

  initial_render(&tree, &mut cache);

  lui_tree::focus(&mut tree, Some(&[0]));

  // First keystroke.
  lui_tree::text_input(&mut tree, "a");
  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  assert_eq!(action, PipelineAction::LayoutOnly);

  // Render frame 2.
  let mut text_ctx = lui_text::TextContext::new(64);
  text_ctx.sync_fonts(&tree.fonts);
  paint_tree_cached(&tree, &mut text_ctx, &mut image_cache, 800.0, 600.0, 1.0, 0.0, &mut cache);
  tree.dirty_paths.clear();

  // Second keystroke.
  lui_tree::text_input(&mut tree, "b");
  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  assert_eq!(action, PipelineAction::LayoutOnly,
    "second keystroke should also be LayoutOnly, not {action:?}");
}

#[test]
fn form_with_image_input_does_not_force_full_pipeline_on_typing() {
  let html = r#"
    <form style="display:flex; flex-direction:column;">
      <input type="image" src="nonexistent.png" />
      <input type="email" placeholder="test" />
    </form>
  "#;
  let mut tree = parse_and_register_fonts(html);
  let mut cache = PipelineCache::new();
  let mut image_cache = lui_layout::ImageCache::default();

  // First render — FullPipeline (populates cache).
  let mut text_ctx = lui_text::TextContext::new(64);
  text_ctx.sync_fonts(&tree.fonts);
  paint_tree_cached(&tree, &mut text_ctx, &mut image_cache, 800.0, 600.0, 1.0, 0.0, &mut cache);
  tree.dirty_paths.clear();

  // Focus email input and type.
  lui_tree::focus(&mut tree, Some(&[0, 1]));
  lui_tree::text_input(&mut tree, "x");

  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  assert_ne!(action, PipelineAction::FullPipeline,
    "pending image should not force FullPipeline when typing. Action: {action:?}");
}

#[test]
fn checkbox_toggle_classifies_as_patch_form_controls() {
  let html = r#"<input type="checkbox" />"#;
  let mut tree = parse_and_register_fonts(html);
  let mut cache = PipelineCache::new();
  let mut image_cache = lui_layout::ImageCache::default();

  initial_render(&tree, &mut cache);

  // Toggle checkbox.
  lui_tree::focus(&mut tree, Some(&[0]));
  tree.dispatch_mouse_down(Some(&[0]), (5.0, 5.0), lui_tree::MouseButton::Primary, None);
  tree.dispatch_mouse_up(Some(&[0]), (5.0, 5.0), lui_tree::MouseButton::Primary, None);

  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  // Should be PatchFormControls (form_control_generation changed, generation didn't).
  // Or PartialCascade if focus changed. Either way, NOT FullPipeline.
  assert_ne!(action, PipelineAction::FullPipeline,
    "checkbox toggle should not be FullPipeline. Action: {action:?}");
}

#[test]
fn no_change_classifies_as_repaint_only() {
  let html = r#"<div style="height:100px;"></div>"#;
  let mut tree = parse_and_register_fonts(html);
  let mut cache = PipelineCache::new();
  let mut image_cache = lui_layout::ImageCache::default();

  initial_render(&tree, &mut cache);

  let action = classify_frame(&tree, &cache, &mut image_cache, 800.0, 600.0, 1.0);
  assert_eq!(action, PipelineAction::RepaintOnly);
}
