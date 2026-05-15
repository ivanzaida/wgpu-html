use lui_layout::LayoutContext;

#[test]
fn layout_context_new_sets_viewport_and_containing() {
  let ctx = LayoutContext::new(800.0, 600.0);
  assert_eq!(ctx.viewport_width, 800.0);
  assert_eq!(ctx.viewport_height, 600.0);
  assert_eq!(ctx.containing_width, 800.0, "initial containing width = viewport width");
  assert_eq!(
    ctx.containing_height, 600.0,
    "initial containing height = viewport height"
  );
  assert_eq!(ctx.root_font_size, 16.0, "default root font-size is 16px");
  assert_eq!(ctx.parent_font_size, 16.0, "default parent font-size is 16px");
}

#[test]
fn layout_context_new_zero_viewport() {
  let ctx = LayoutContext::new(0.0, 0.0);
  assert_eq!(ctx.viewport_width, 0.0);
  assert_eq!(ctx.viewport_height, 0.0);
  assert_eq!(ctx.containing_width, 0.0);
}
